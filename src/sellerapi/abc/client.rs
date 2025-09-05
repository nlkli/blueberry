use super::models::{DEFAULT_AUTHOR_NAME, NewQuestion, NewReview};
use crate::error::{Error, Result};
use crate::sellerapi::abcmodels::{NewFeedback, Product, ProductFormatInfo};
use crate::sellerapi::ozmodels::params::PRODUCT_LIST_MAX_LIMIT;
use crate::sellerapi::{OzonSellerClient, WbSellerClient, ozmodels, wbmodels};
use std::collections::BTreeMap;
use std::fmt::Write;
use std::sync::atomic::{AtomicBool, Ordering};
use std::usize;
use std::{sync::Arc, time::Duration};
use tokio::sync::mpsc::{self, UnboundedReceiver, UnboundedSender};

pub const OZON_PLACE_FULL_SYMBOL: &str = "Ozon";
pub const WB_PLACE_FULL_SYMBOL: &str = "Wildberries";
pub const OZON_PLACE_SYMBOL: &str = "oz";
pub const WB_PLACE_SYMBOL: &str = "wb";

/// Клиент для взаимодействия с конкретным маркетплейсом (Ozon или Wildberries).
/// Хранит подключение к соответствующему клиенту в Arc.
#[derive(Clone)]
pub enum SellerClient {
    Ozon(Arc<OzonSellerClient>),
    Wb(Arc<WbSellerClient>),
}

impl SellerClient {
    #[inline]
    pub fn str_symbol(&self) -> &'static str {
        match self {
            Self::Ozon(_) => OZON_PLACE_SYMBOL,
            Self::Wb(_) => WB_PLACE_SYMBOL,
        }
    }

    #[inline]
    pub fn str_full_symbol(&self) -> &'static str {
        match self {
            Self::Ozon(_) => OZON_PLACE_FULL_SYMBOL,
            Self::Wb(_) => WB_PLACE_FULL_SYMBOL,
        }
    }

    pub async fn get_product_format_info(&self, product_id: &str) -> Result<ProductFormatInfo> {
        match self {
            Self::Ozon(cli) => {
                let tmp = [product_id];
                let filter = ozmodels::params::Filter {
                    sku: Some(&tmp[..]),
                    ..Default::default()
                };

                let mut product = cli
                    .get_product_info_list(&filter)
                    .await
                    .map_err(|e| {
                        Error::ProductCtxData(format!("product info request failed: {e}"))
                    })?
                    .items
                    .into_iter()
                    .next()
                    .ok_or_else(|| Error::MissingRequiredField("product_info".into()))?;

                let product_id = product.id;
                let name = std::mem::take(&mut product.name);
                let price = format!("{} {}", product.marketing_price, product.currency_code);
                drop(product);

                let description_task = {
                    let cli = cli.clone();
                    tokio::spawn(async move { cli.get_product_info_description(product_id).await })
                };

                let attrs_v4 = cli
                    .get_product_attributes_v4(Some(&filter), 1, None)
                    .await
                    .map_err(|e| {
                        Error::ProductCtxData(format!("product attributes request failed: {e}"))
                    })?
                    .result
                    .into_iter()
                    .next()
                    .ok_or_else(|| Error::MissingRequiredField("product_attributes".into()))?;

                let weight = format!("{}{}", attrs_v4.weight, attrs_v4.weight_unit);

                let unit = attrs_v4.dimension_unit.as_str();
                let r#box = format!(
                    "height: {}{}, width: {}{}, depth: {}{}",
                    attrs_v4.height, unit, attrs_v4.width, unit, attrs_v4.depth, unit
                );

                let desc_category_id = attrs_v4.description_category_id;
                let type_id = attrs_v4.type_id;

                let category_attrs = cli
                    .get_attributes(desc_category_id, None, type_id)
                    .await
                    .map_err(|e| {
                        Error::ProductCtxData(format!("product attributes request failed: {e}"))
                    })?
                    .result;

                let desc = description_task
                    .await
                    .map_err(|e| {
                        Error::ProductCtxData(format!("join description task failed: {e}"))
                    })?? // двойной ? — из JoinHandle и из Result внутри
                    .result
                    .description;

                let product_attrs = attrs_v4.attributes;
                let mut attrs = BTreeMap::new();

                let (mut skipped_desc, mut processed_rich) = (false, false);

                for attr in &product_attrs {
                    if let Some(cat) = category_attrs.iter().find(|v| v.id == attr.id) {
                        let mut value = attr
                            .values
                            .iter()
                            .map(|v| v.value.as_str().trim())
                            .collect::<Vec<_>>()
                            .join(", ");

                        if !skipped_desc && value == desc {
                            skipped_desc = true;
                            continue;
                        }

                        if !processed_rich
                            && cat.name.starts_with("Rich-")
                            && cat.name.ends_with(" JSON")
                        {
                            value = match serde_json::from_str::<serde_json::Value>(&value) {
                                Ok(mut v) => {
                                    sanitize_ozon_rich_content_json(&mut v);
                                    serde_json::to_string(&v).unwrap_or_default()
                                }
                                Err(_) => "Empty".to_string(),
                            };
                            processed_rich = true;
                        }

                        attrs.insert(cat.name.clone(), value);
                    }
                }

                Ok(ProductFormatInfo {
                    id: product_id.to_string(),
                    name,
                    price,
                    desc,
                    attrs,
                    weight,
                    r#box,
                })
            }
            Self::Wb(cli) => {
                const UNKNOWN: &str = "Unknown";

                let filter = wbmodels::params::Filter {
                    with_photo: Some(-1),
                    text_search: Some(product_id),
                    ..Default::default()
                };

                let cards_response = cli
                    .get_cards_list(
                        Some(&filter),
                        &wbmodels::params::CardListCursor {
                            limit: Some(1),
                            ..Default::default()
                        },
                    )
                    .await
                    .map_err(|e| Error::ProductCtxData(format!("cards request failed: {e}")))?;

                let mut card = cards_response.cards.into_iter().next().ok_or_else(|| {
                    Error::ProductCtxData(format!("not found product by nmid {product_id}"))
                })?;

                let name = std::mem::take(&mut card.title);
                let desc = std::mem::take(&mut card.description);

                let mut attrs = BTreeMap::from([
                    ("Бренд".to_owned(), std::mem::take(&mut card.brand)),
                    (
                        "Категория".to_owned(),
                        std::mem::take(&mut card.subject_name),
                    ),
                ]);

                std::mem::take(&mut card.characteristics)
                    .into_iter()
                    .for_each(|c| {
                        // Безопасно сериализуем значение характеристики.
                        let val = serde_json::to_string(&c.value).unwrap_or_default();
                        attrs.insert(c.name, val);
                    });

                if let Some(first_photo) = card.photos.get(0) {
                    if let Some((bucket_path, _)) = first_photo.big.split_once("/images/") {
                        let rich_content = WbSellerClient::get_product_rich_content(bucket_path, 1)
                            .await
                            .ok()
                            .and_then(|raw| serde_json::from_str::<serde_json::Value>(&raw).ok())
                            .and_then(|mut v| {
                                sanitize_wb_rich_content_json(&mut v);
                                serde_json::to_string(&v).ok()
                            })
                            .unwrap_or_else(|| "Empty".to_string());
                        attrs.insert("Rich-контент JSON".to_owned(), rich_content);
                    }
                }

                let (weight, r#box) = card
                    .dimensions
                    .map(|dims| {
                        (
                            dims.weight_brutto.to_string(),
                            format!(
                                "height: {}, width: {}, length: {}",
                                dims.height, dims.width, dims.length
                            ),
                        )
                    })
                    .unwrap_or_else(|| (UNKNOWN.to_string(), UNKNOWN.to_string()));

                let price = cli
                    .get_products_price(1, None, product_id.parse().ok())
                    .await
                    .ok()
                    .and_then(|r| r.data.list_goods.into_iter().next())
                    .map(|g| {
                        (
                            g.sizes
                                .into_iter()
                                .next()
                                .map(|sp| sp.discounted_price.to_string())
                                .unwrap_or_else(|| UNKNOWN.to_string()),
                            g.currency_iso_code4217,
                        )
                    })
                    .map(|(p, c)| format!("{p} {c}"))
                    .unwrap_or_else(|| UNKNOWN.to_string());

                Ok(ProductFormatInfo {
                    id: product_id.to_string(),
                    name,
                    price,
                    desc,
                    attrs,
                    weight,
                    r#box,
                })
            }
        }
    }

    /// Возвращает канал с товарами.
    pub fn all_products_stream(&self) -> UnboundedReceiver<Result<Product>> {
        let seller = self.clone();

        let (tx, rx) = mpsc::unbounded_channel();

        tokio::spawn(async move {
            match seller {
                Self::Ozon(cli) => {
                    let limit = 100;
                    let mut last_id: Option<String> = None;

                    loop {
                        let res = cli
                            .get_product_list(
                                Some(&ozmodels::params::Filter {
                                    visibility: Some(ozmodels::params::Visibility::All),
                                    ..Default::default()
                                }),
                                limit,
                                last_id.as_deref(),
                            )
                            .await;

                        if let Err(e) = res {
                            let _ = tx.send(Err(e));
                            return;
                        }

                        let mut res = unsafe { res.unwrap_unchecked() };

                        if !res.result.last_id.is_empty() {
                            last_id.insert(std::mem::take(&mut res.result.last_id));
                        }

                        let product_id_list = res
                            .result
                            .items
                            .iter()
                            .map(|i| i.product_id.to_string())
                            .collect::<Vec<_>>();

                        match cli
                            .get_product_info_list(&ozmodels::params::Filter {
                                product_id: Some(
                                    &product_id_list
                                        .iter()
                                        .map(|s| s.as_str())
                                        .collect::<Vec<_>>(),
                                ),
                                ..Default::default()
                            })
                            .await
                        {
                            Ok(res) => {
                                for i in res.items {
                                    let product = Product {
                                        id: i.sku.to_string(),
                                        name: i.name,
                                    };

                                    if tx.send(Ok(product)).is_err() {
                                        return;
                                    }
                                }
                            }
                            Err(e) => {
                                let _ = tx.send(Err(e));
                                return;
                            }
                        }

                        if product_id_list.len() < limit as usize {
                            return;
                        }
                    }
                }
                Self::Wb(cli) => {
                    let mut limit = 100;

                    let mut cursor = wbmodels::params::CardListCursor {
                        limit: Some(limit),
                        updated_at: None,
                        nm_id: None,
                    };

                    loop {
                        let res = cli
                            .get_cards_list(
                                Some(&wbmodels::params::Filter {
                                    with_photo: Some(-1),
                                    ..Default::default()
                                }),
                                &cursor,
                            )
                            .await;

                        if let Err(e) = res {
                            let _ = tx.send(Err(e));
                            return;
                        }

                        let mut res = unsafe { res.unwrap_unchecked() };

                        for (n, i) in res.cards.into_iter().enumerate() {
                            if n as u32 == limit - 1 {
                                break;
                            }
                            let product = Product {
                                id: i.nm_id.to_string(),
                                name: i.title,
                            };
                            if tx.send(Ok(product)).is_err() {
                                return;
                            }
                        }

                        cursor.nm_id = Some(res.cursor.nm_id);
                        cursor.updated_at = res.cursor.updated_at;

                        if (res.cursor.total as u32) < limit {
                            return;
                        }
                    }
                }
            };
        });

        rx
    }

    /// Ответить на вопрос. После ответа вопрос будет помечен как обработанный.
    /// Параметр product_id обязательный только для Ozon.
    pub async fn answer_question(
        &self,
        id: &str,
        text: &str,
        product_id: Option<&str>,
    ) -> Result<()> {
        match self {
            Self::Ozon(cli) => cli
                .create_question_answer(
                    id,
                    product_id.and_then(|v| v.parse().ok()).unwrap_or(0),
                    text,
                )
                .await
                .map(|_| ()),
            Self::Wb(cli) => cli
                .update_question(&wbmodels::params::UpdateQuestionParams {
                    id,
                    answer_text: Some(text),
                    state: Some(wbmodels::params::ANSWER_OR_EDIT_STATE),
                    ..Default::default()
                })
                .await
                .map(|_| ()),
        }
    }

    /// Ответить на отзыв. После ответа отзыв будет помечен как обработанный.
    pub async fn answer_review(&self, id: &str, text: &str) -> Result<()> {
        match self {
            Self::Ozon(cli) => cli
                .create_review_comment(id, text, None, true)
                .await
                .map(|_| ()),
            Self::Wb(cli) => cli.reply_to_review(id, text).await,
        }
    }

    // /// Возвращает текущее количество вопросов на стороне маркетплейса.
    // pub async fn get_question_count(&self) -> Result<u32> {
    //     match self {
    //         Self::Ozon(cli) => Ok(cli.get_question_count().await?.all),
    //         Self::Wb(cli) => cli.get_question_count(None, None, None).await,
    //     }
    // }

    // /// Возвращает текущее количество отзывов на стороне маркетплейса.
    // pub async fn get_review_count(&self) -> Result<u32> {
    //     match self {
    //         Self::Ozon(cli) => Ok(cli.get_review_count().await?.total),
    //         Self::Wb(cli) => cli.get_review_count(None, None, None).await,
    //     }
    // }

    /// Вспомогательная функция: сортирует по `published_at` (по убыванию),
    /// фильтрует записи старше `date_from` и возвращает до `limit` элементов.
    fn process_questions(
        mut qs: Vec<NewQuestion>,
        limit: usize,
        date_from: u64,
    ) -> Vec<NewQuestion> {
        qs.sort_by(|a, b| b.published_at.cmp(&a.published_at));
        qs.into_iter()
            .filter(|q| q.published_at >= date_from)
            .take(limit)
            .collect()
    }

    /// Вспомогательная функция аналогично для отзывов.
    fn process_reviews(mut rs: Vec<NewReview>, limit: usize, date_from: u64) -> Vec<NewReview> {
        rs.sort_by(|a, b| b.published_at.cmp(&a.published_at));
        rs.into_iter()
            .filter(|r| r.published_at >= date_from)
            .take(limit)
            .collect()
    }

    /// Получить последние новые вопросы (с учётом `limit` и `date_from`).
    pub async fn get_last_new_questions(
        &self,
        limit: u32,
        date_from: u64,
    ) -> Result<Vec<NewQuestion>> {
        match self {
            Self::Ozon(cli) => cli
                .get_question_list(
                    Some(&ozmodels::params::QuestionListFilter {
                        status: ozmodels::QuestionStatus::Unprocessed,
                        date_from: Some(&unix_timestamp_to_rfc3339_format(date_from)),
                        ..Default::default()
                    }),
                    None,
                )
                .await
                .map(|res| {
                    let mapped = res
                        .questions
                        .into_iter()
                        .map(|q| NewQuestion {
                            id: q.id,
                            product_id: q.sku.to_string(),
                            author_name: q.author_name,
                            text: q.text,
                            published_at: format_rfc3339_to_unix_timestamp(&q.published_at),
                        })
                        .collect::<Vec<_>>();

                    Self::process_questions(mapped, limit as usize, date_from)
                }),
            Self::Wb(cli) => {
                let take = limit
                    .max(1)
                    .min(wbmodels::params::QUESTION_MAX_LIMIT as u32);

                cli.get_question_list(&wbmodels::params::QuestionsAndReviewsFilter {
                    is_answered: false,
                    take,
                    skip: 0,
                    date_from: Some(date_from),
                    ..Default::default()
                })
                .await
                .map(|res| {
                    let mapped = res
                        .questions
                        .into_iter()
                        .map(|q| NewQuestion {
                            id: q.id,
                            product_id: q.product_details.nm_id.to_string(),
                            author_name: DEFAULT_AUTHOR_NAME.to_string(),
                            text: q.text,
                            published_at: format_rfc3339_to_unix_timestamp(&q.created_date),
                        })
                        .collect::<Vec<_>>();

                    Self::process_questions(mapped, limit as usize, date_from)
                })
            }
        }
    }

    /// Получить последние новые отзывы (с учётом `limit` и `date_from`).
    pub async fn get_last_new_reviews(&self, limit: u32, date_from: u64) -> Result<Vec<NewReview>> {
        match self {
            Self::Ozon(cli) => {
                let take = limit
                    .max(ozmodels::params::REVIEW_MIN_LIMIT as u32)
                    .min(ozmodels::params::REVIEW_MAX_LIMIT as u32);

                cli.get_review_list(
                    take,
                    Some(&ozmodels::params::ReviewStatus::Unprocessed),
                    None,
                    None,
                )
                .await
                .map(|res| {
                    let mapped = res
                        .reviews
                        .into_iter()
                        .map(|r| NewReview {
                            id: r.id,
                            product_id: r.sku.to_string(),
                            author_name: DEFAULT_AUTHOR_NAME.to_string(),
                            text: r.text,
                            score: r.rating as f32,
                            photos_amount: r.photos_amount as u16,
                            videos_amount: r.videos_amount as u16,
                            published_at: format_rfc3339_to_unix_timestamp(&r.published_at),
                        })
                        .collect::<Vec<_>>();

                    Self::process_reviews(mapped, limit as usize, date_from)
                })
            }
            Self::Wb(cli) => {
                let take = limit.max(1).min(wbmodels::params::REVIEW_MAX_LIMIT as u32);

                cli.get_review_list(&wbmodels::params::QuestionsAndReviewsFilter {
                    is_answered: false,
                    take,
                    skip: 0,
                    date_from: Some(date_from),
                    ..Default::default()
                })
                .await
                .map(|res| {
                    let mapped = res
                        .reviews
                        .into_iter()
                        .map(|r| {
                            let mut text = "".to_string();

                            if !r.pros.is_empty() {
                                let _ = write!(&mut text, "Достоинства: {}\n", r.pros);
                            }
                            if !r.cons.is_empty() {
                                let _ = write!(&mut text, "Недостатки: {}\n", r.cons);
                            }
                            if !r.text.is_empty() {
                                let _ = write!(&mut text, "Комментарий: {}\n", r.text);
                            }

                            let text = text.trim().to_string();

                            NewReview {
                                id: r.id,
                                product_id: r.product_details.nm_id.to_string(),
                                author_name: r.user_name,
                                text: text,
                                score: r.product_valuation as f32,
                                photos_amount: r.photo_links.map(|v| v.len()).unwrap_or(0) as u16,
                                videos_amount: r.video.map(|_| 1).unwrap_or(0) as u16,
                                published_at: format_rfc3339_to_unix_timestamp(&r.created_date),
                            }
                        })
                        .collect::<Vec<_>>();

                    Self::process_reviews(mapped, limit as usize, date_from)
                })
            }
        }
    }

    // /// Запускает наблюдатель, который шлёт разницу количества вопросов в канал.
    // /// Первый обнаруженный счётчик игнорируется (флаг `once`), далее — при изменении шлёт diff.
    // pub fn spawn_has_new_question_observer(
    //     &self,
    //     chan: UnboundedSender<Result<u32>>,
    //     interval: Duration,
    // ) {
    //     let seller = self.clone();
    //     tokio::spawn(async move {
    //         let mut question_count = 0u32;
    //         let mut once = true;
    //         loop {
    //             match seller.get_question_count().await {
    //                 Ok(count) => {
    //                     if count != question_count {
    //                         let diff = (count as i32 - question_count as i32).max(0) as u32;
    //                         if diff == 0 {
    //                             continue;
    //                         }
    //                         question_count = count;
    //                         if once {
    //                             once = !once;
    //                         } else {
    //                             if chan.send(Ok(diff)).is_err() {
    //                                 return;
    //                             }
    //                         }
    //                     }
    //                 }
    //                 Err(e) => {
    //                     let _ = chan.send(Err(e));
    //                     return;
    //                 }
    //             }
    //             tokio::time::sleep(interval).await;
    //         }
    //     });
    // }

    // /// Запускает наблюдатель, который шлёт разницу количества отзывов в канал.
    // /// Первый обнаруженный счётчик игнорируется (флаг `once`), далее — при изменении шлёт diff.
    // pub fn spawn_has_new_review_observer(
    //     &self,
    //     chan: UnboundedSender<Result<u32>>,
    //     interval: Duration,
    // ) {
    //     let seller = self.clone();
    //     tokio::spawn(async move {
    //         let mut review_count = 0u32;
    //         let mut once = true;
    //         loop {
    //             match seller.get_review_count().await {
    //                 Ok(count) => {
    //                     if count != review_count {
    //                         let diff = (count as i32 - review_count as i32).max(0) as u32;
    //                         if diff == 0 {
    //                             continue;
    //                         }
    //                         review_count = count;
    //                         if once {
    //                             once = !once;
    //                         } else {
    //                             if chan.send(Ok(diff)).is_err() {
    //                                 return;
    //                             }
    //                         }
    //                     }
    //                 }
    //                 Err(e) => {
    //                     let _ = chan.send(Err(e));
    //                     return;
    //                 }
    //             }
    //             tokio::time::sleep(interval).await;
    //         }
    //     });
    // }

    #[inline]
    /// Вспомогательная функция: берёт первые 8 байт id (если есть) и конвертирует в u64,
    /// возвращает кортеж (unique_id, published_at).
    fn compute_unique_key(id: &str, timestamp: u64) -> (u64, u64) {
        let uid = id
            .as_bytes()
            .get(..8)
            .and_then(|b| b.try_into().ok())
            .map(|arr: [u8; 8]| u64::from_be_bytes(arr))
            .unwrap_or_default();
        (uid, timestamp)
    }

    /// Запускает наблюдатель, который при появлении новых вопросов шлёт `NewQuestion` в канал.
    pub fn spawn_new_question_observer(
        &self,
        interval: Duration,
    ) -> UnboundedReceiver<Result<NewQuestion>> {
        let seller = self.clone();

        let (tx, rx) = mpsc::unbounded_channel();

        tokio::spawn(async move {
            let mut unique_set = CircularArr::<_, 16>::new();
            let mut date_from = 0u64;
            let mut once = true;

            loop {
                match seller.get_last_new_questions(20, date_from).await {
                    Ok(list) => {
                        if once {
                            date_from = list.first().map_or(0, |q| q.published_at + 1);
                            once = !once;
                            continue;
                        }

                        for new_question in list {
                            let unique = Self::compute_unique_key(
                                &new_question.id,
                                new_question.published_at,
                            );

                            if unique_set.as_slise().contains(&unique) {
                                continue;
                            }

                            date_from = new_question.published_at - 1;
                            if tx.send(Ok(new_question)).is_err() {
                                return;
                            }

                            unique_set.add(unique);
                        }
                    }
                    Err(e) => {
                        let _ = tx.send(Err(e));
                        return;
                    }
                }
                tokio::time::sleep(interval).await;
            }
        });

        rx

        //     while let Some(res) = rx.recv().await {
        //         if let Err(e) = res {
        //             let _ = chan.send(Err(e));
        //             return;
        //         }
        //         let diff = res.unwrap().max(1);
        //         match seller
        //             .get_last_new_questions(
        //                 diff,
        //                 (time::UtcDateTime::now().unix_timestamp() as u64) - interval.as_secs() * 2
        //                     + 1,
        //             )
        //             .await
        //         {
        //             Ok(list) => {
        //                 dbg!(&list);
        //                 for new_question in list {
        //                     dbg!(&new_question);
        //                     let unique = Self::compute_unique_key(
        //                         &new_question.id,
        //                         new_question.published_at,
        //                     );
        //                     dbg!(unique);
        //                     if unique_set.as_slise().contains(&unique) {
        //                         continue;
        //                     }
        //                     dbg!("try send");
        //                     if chan.send(Ok(new_question)).is_err() {
        //                         return;
        //                     }
        //                     unique_set.add(unique);
        //                 }
        //             }
        //             Err(e) => {
        //                 let _ = chan.send(Err(e));
        //                 return;
        //             }
        //         }
        //     }
        // });
    }

    /// Запускает наблюдатель, который при появлении новых отзывов шлёт `NewReview` в канал.
    pub fn spawn_new_review_observer(
        &self,
        interval: Duration,
    ) -> UnboundedReceiver<Result<NewReview>> {
        let seller = self.clone();

        let (tx, rx) = mpsc::unbounded_channel();

        tokio::spawn(async move {
            let mut unique_set = CircularArr::<_, 16>::new();
            let mut date_from = 0u64;
            let mut once = true;

            loop {
                match seller.get_last_new_reviews(20, date_from).await {
                    Ok(list) => {
                        if once {
                            once = !once;
                            date_from = list.first().map_or(0, |q| q.published_at + 1);
                            continue;
                        }

                        for new_reviews in list {
                            let unique =
                                Self::compute_unique_key(&new_reviews.id, new_reviews.published_at);

                            if unique_set.as_slise().contains(&unique) {
                                continue;
                            }

                            date_from = new_reviews.published_at - 1;
                            if tx.send(Ok(new_reviews)).is_err() {
                                return;
                            }

                            unique_set.add(unique);
                        }
                    }
                    Err(e) => {
                        let _ = tx.send(Err(e));
                        return;
                    }
                }
                tokio::time::sleep(interval).await;
            }
        });

        rx

        // tokio::spawn(async move {
        //     let (tx, mut rx) = mpsc::unbounded_channel::<Result<_>>();
        //     seller.spawn_has_new_review_observer(tx, interval);
        //     let mut unique_set = CircularArr::<_, 16>::new();
        //     while let Some(res) = rx.recv().await {
        //         if let Err(e) = res {
        //             let _ = chan.send(Err(e));
        //             return;
        //         }
        //         let diff = res.unwrap().max(1);
        //         let seller_0 = seller.clone();
        //         match seller
        //             .get_last_new_reviews(
        //                 diff,
        //                 (time::UtcDateTime::now().unix_timestamp() as u64) - 3600,
        //             )
        //             .await
        //         {
        //             Ok(list) => {
        //                 for new_review in list {
        //                     let unique =
        //                         Self::compute_unique_key(&new_review.id, new_review.published_at);
        //                     if unique_set.as_slise().contains(&unique) {
        //                         continue;
        //                     }
        //                     if chan.send(Ok(new_review)).is_err() {
        //                         return;
        //                     }
        //                     unique_set.add(unique);
        //                 }
        //             }
        //             Err(e) => {
        //                 let _ = chan.send(Err(e));
        //                 return;
        //             }
        //         }
        //     }
        // });
    }

    /// Запускает наблюдатель, который при появлении новых вопросов или отзывов
    /// шлёт `NewFeedback` в канал. Если один поток завершился, другой тоже остановится.
    pub fn spawn_new_feedback_observer(
        &self,
        question_interval: Duration,
        review_interval: Duration,
    ) -> UnboundedReceiver<Result<NewFeedback>> {
        let (tx, rx) = mpsc::unbounded_channel();

        let stop_flag = Arc::new(AtomicBool::new(false));

        {
            let seller = self.clone();
            let tx = tx.clone();
            let stop_flag = stop_flag.clone();

            let mut rx = seller.spawn_new_question_observer(question_interval);

            tokio::spawn(async move {
                while !stop_flag.load(Ordering::Relaxed) {
                    if let Some(res) = rx.recv().await {
                        match res {
                            Ok(q) => {
                                if tx.send(Ok(NewFeedback::Question(q))).is_err() {
                                    break;
                                }
                            }
                            Err(e) => {
                                let _ = tx.send(Err(e));
                                break;
                            }
                        }
                    } else {
                        break;
                    }
                }
                stop_flag.store(true, Ordering::Relaxed);
            });
        }

        {
            let seller = self.clone();
            let tx = tx.clone();
            let stop_flag = stop_flag.clone();

            let mut rx = seller.spawn_new_review_observer(review_interval);

            tokio::spawn(async move {
                while !stop_flag.load(Ordering::Relaxed) {
                    if let Some(res) = rx.recv().await {
                        match res {
                            Ok(r) => {
                                if tx.send(Ok(NewFeedback::Review(r))).is_err() {
                                    break;
                                }
                            }
                            Err(e) => {
                                let _ = tx.send(Err(e));
                                break;
                            }
                        }
                    } else {
                        break;
                    }
                }
                stop_flag.store(true, Ordering::Relaxed);
            });
        }

        rx
    }
}

struct CircularArr<T: Copy + PartialEq, const CAP: usize> {
    arr: [T; CAP],
    cursor: usize,
}

impl<T: Copy + PartialEq, const CAP: usize> CircularArr<T, CAP> {
    fn new() -> Self {
        Self {
            arr: [unsafe { std::mem::zeroed() }; CAP],
            cursor: 0,
        }
    }

    fn add(&mut self, value: T) {
        self.arr[self.cursor] = value;
        self.cursor += 1;
        if self.cursor >= CAP {
            self.cursor = 0;
        }
    }

    fn as_slise(&self) -> &[T] {
        self.arr.as_slice()
    }
}

use time::format_description::well_known::Rfc3339;

fn format_rfc3339_to_unix_timestamp(s: &str) -> u64 {
    time::OffsetDateTime::parse(s, &Rfc3339)
        .map(|v| v.unix_timestamp())
        .unwrap_or_default() as u64
}

fn unix_timestamp_to_rfc3339_format(ts: u64) -> String {
    time::UtcDateTime::from_unix_timestamp(ts as i64)
        .map(|v| v.format(&Rfc3339).unwrap_or_default())
        .unwrap_or_default()
}

/// Чёрный список ключей для Rich-контента Ozon.
const OZON_RICH_CONTENT_BLACKLIST_KEYS: [&str; 14] = [
    "img",
    "imgLink",
    "size",
    "width",
    "height",
    "align",
    "contentAlign",
    "color",
    "src",
    "sources",
    "srcMobile",
    "reverse",
    "version",
    "id",
];

/// Чёрный список ключей для Rich-контента Wildberries.
const WB_RICH_CONTENT_BLACKLIST_KEYS: [&str; 5] = ["style", "image", "src", "preview", "version"];

/// Рекурсивно удаляет лишние поля и мусор из JSON Rich-контента.
fn sanitize_rich_content_json(value: &mut serde_json::Value, blacklist: &[&str]) {
    match value {
        serde_json::Value::Array(arr) => {
            // Удаляем пустые/короткие строковые элементы-мусор.
            arr.retain(|i| !i.as_str().map(|s| s.len() <= 1).unwrap_or(false));
            for item in arr {
                sanitize_rich_content_json(item, blacklist);
            }
        }
        serde_json::Value::Object(obj) => {
            for key in blacklist {
                obj.remove(*key);
            }
            for v in obj.values_mut() {
                sanitize_rich_content_json(v, blacklist);
            }
        }
        _ => {}
    }
}

/// Очищает JSON Rich-контента Ozon.
fn sanitize_ozon_rich_content_json(value: &mut serde_json::Value) {
    sanitize_rich_content_json(value, &OZON_RICH_CONTENT_BLACKLIST_KEYS);
}

/// Очищает JSON Rich-контента Wildberries.
fn sanitize_wb_rich_content_json(value: &mut serde_json::Value) {
    sanitize_rich_content_json(value, &WB_RICH_CONTENT_BLACKLIST_KEYS);
}
