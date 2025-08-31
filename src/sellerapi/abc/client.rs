use super::models::{DEFAULT_AUTHOR_NAME, NewQuestion, NewReview};
use super::util::{format_rfc3339_to_unix_timestamp, unix_timestamp_to_rfc3339_format};
use crate::error::Result;
use crate::sellerapi::abcmodels::NewFeedback;
use crate::sellerapi::{OzonSellerClient, WbSellerClient, ozmodels, wbmodels};
use std::sync::atomic::{AtomicBool, Ordering};
use std::usize;
use std::{sync::Arc, time::Duration};
use tokio::sync::mpsc::{self, UnboundedSender};

// #[derive(Debug)]
// pub enum Seller {
//     Ozon,
//     Wb,
// }

// impl fmt::Display for Seller {
//     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
//         match self {
//             Seller::Ozon => write!(f, "Ozon"),
//             Seller::Wb => write!(f, "Wb"),
//         }
//     }
// }

/// Клиент для взаимодействия с конкретным маркетплейсом (Ozon или Wildberries).
/// Хранит подключение к соответствующему клиенту в Arc.
#[derive(Clone)]
pub enum SellerClient {
    Ozon(Arc<OzonSellerClient>),
    Wb(Arc<WbSellerClient>),
}

impl SellerClient {
    /// Возвращает цены и скидки на товар.
    // pub async fn get_product_prices(&self) -> Result<u32> {
    // }

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

    /// Возвращает текущее количество вопросов на стороне маркетплейса.
    pub async fn get_question_count(&self) -> Result<u32> {
        match self {
            Self::Ozon(cli) => Ok(cli.get_question_count().await?.all),
            Self::Wb(cli) => cli.get_question_count(None, None, None).await,
        }
    }

    /// Возвращает текущее количество отзывов на стороне маркетплейса.
    pub async fn get_review_count(&self) -> Result<u32> {
        match self {
            Self::Ozon(cli) => Ok(cli.get_review_count().await?.total),
            Self::Wb(cli) => cli.get_review_count(None, None, None).await,
        }
    }

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

                    dbg!(&mapped);

                    Self::process_reviews(mapped, limit as usize, date_from)
                })
            }
            Self::Wb(cli) => {
                let take = limit.max(1).min(wbmodels::params::REVIEW_MAX_LIMIT as u32);

                dbg!(take);

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
                        .map(|r| NewReview {
                            id: r.id,
                            product_id: r.product_details.nm_id.to_string(),
                            author_name: r.user_name,
                            text: r.text,
                            score: r.product_valuation as f32,
                            photos_amount: r.photo_links.map(|v| v.len()).unwrap_or(0) as u16,
                            videos_amount: r.video.map(|_| 1).unwrap_or(0) as u16,
                            published_at: format_rfc3339_to_unix_timestamp(&r.created_date),
                        })
                        .collect::<Vec<_>>();

                    dbg!(&mapped);

                    Self::process_reviews(mapped, limit as usize, date_from)
                })
            }
        }
    }

    /// Запускает наблюдатель, который шлёт разницу количества вопросов в канал.
    /// Первый обнаруженный счётчик игнорируется (флаг `once`), далее — при изменении шлёт diff.
    pub fn spawn_has_new_question_observer(
        &self,
        chan: UnboundedSender<Result<u32>>,
        interval: Duration,
    ) {
        let seller = self.clone();
        tokio::spawn(async move {
            let mut question_count = 0u32;
            let mut once = true;
            loop {
                match seller.get_question_count().await {
                    Ok(count) => {
                        if count != question_count {
                            let diff = (count as i32 - question_count as i32).max(0) as u32;
                            if diff == 0 {
                                continue;
                            }

                            dbg!(diff);

                            question_count = count;
                            dbg!(question_count);

                            if once {
                                once = !once;
                            } else {
                                println!("have new question");
                                if chan.send(Ok(diff)).is_err() {
                                    return;
                                }
                            }
                        }
                    }
                    Err(e) => {
                        let _ = chan.send(Err(e));
                        return;
                    }
                }
                tokio::time::sleep(interval).await;
            }
        });
    }

    /// Запускает наблюдатель, который шлёт разницу количества отзывов в канал.
    /// Первый обнаруженный счётчик игнорируется (флаг `once`), далее — при изменении шлёт diff.
    pub fn spawn_has_new_review_observer(
        &self,
        chan: UnboundedSender<Result<u32>>,
        interval: Duration,
    ) {
        let seller = self.clone();
        tokio::spawn(async move {
            let mut review_count = 0u32;
            let mut once = true;
            loop {
                match seller.get_review_count().await {
                    Ok(count) => {
                        if count != review_count {
                            let diff = (count as i32 - review_count as i32).max(0) as u32;
                            if diff == 0 {
                                continue;
                            }

                            dbg!(diff);

                            review_count = count;
                            dbg!(review_count);

                            if once {
                                once = !once;
                            } else {
                                println!("have new review");
                                if chan.send(Ok(diff)).is_err() {
                                    return;
                                }
                            }
                        }
                    }
                    Err(e) => {
                        let _ = chan.send(Err(e));
                        return;
                    }
                }
                tokio::time::sleep(interval).await;
            }
        });
    }

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
        chan: UnboundedSender<Result<NewQuestion>>,
        interval: Duration,
    ) {
        let seller = self.clone();

        tokio::spawn(async move {
            let (tx, mut rx) = mpsc::unbounded_channel::<Result<_>>();

            seller.spawn_has_new_question_observer(tx, interval);

            let mut unique_set = CircularArr::<_, 16>::new();

            while let Some(res) = rx.recv().await {
                if let Err(e) = res {
                    let _ = chan.send(Err(e));
                    return;
                }

                let diff = res.unwrap().max(1);

                match seller
                    .get_last_new_questions(
                        diff,
                        (time::UtcDateTime::now().unix_timestamp() as u64) - interval.as_secs() * 2
                            + 1,
                    )
                    .await
                {
                    Ok(list) => {
                        dbg!(&list);

                        for new_question in list {
                            dbg!(&new_question);

                            let unique = Self::compute_unique_key(
                                &new_question.id,
                                new_question.published_at,
                            );

                            dbg!(unique);

                            if unique_set.as_slise().contains(&unique) {
                                continue;
                            }

                            dbg!("try send");

                            if chan.send(Ok(new_question)).is_err() {
                                return;
                            }

                            unique_set.add(unique);
                        }
                    }
                    Err(e) => {
                        let _ = chan.send(Err(e));
                        return;
                    }
                }
            }
        });
    }

    /// Запускает наблюдатель, который при появлении новых отзывов шлёт `NewReview` в канал.
    pub fn spawn_new_review_observer(
        &self,
        chan: UnboundedSender<Result<NewReview>>,
        interval: Duration,
    ) {
        let seller = self.clone();

        tokio::spawn(async move {
            let (tx, mut rx) = mpsc::unbounded_channel::<Result<_>>();

            seller.spawn_has_new_review_observer(tx, interval);

            let mut unique_set = CircularArr::<_, 16>::new();

            while let Some(res) = rx.recv().await {
                if let Err(e) = res {
                    let _ = chan.send(Err(e));
                    return;
                }

                let diff = res.unwrap().max(1);

                let seller_0 = seller.clone();

                // Отложенный запрос
                tokio::spawn(async move {
                    let mut n = 0;
                    loop {
                        if let Ok(list) = seller_0
                            .get_last_new_reviews(
                                diff,
                                (time::UtcDateTime::now().unix_timestamp() as u64) - 3600,
                            )
                            .await
                        {
                            println!("---- atempt: {}, data: {:?}", n, list);
                            break;
                        }
                        println!("---- atempt: {}", n);
                        tokio::time::sleep(Duration::from_secs(1)).await;
                        if n == 60 {
                            return;
                        }
                        n += 1;
                    }
                });

                match seller
                    .get_last_new_reviews(
                        diff,
                        (time::UtcDateTime::now().unix_timestamp() as u64) - 3600,
                    )
                    .await
                {
                    Ok(list) => {
                        dbg!(&list);

                        for new_review in list {
                            dbg!(&new_review);

                            let unique =
                                Self::compute_unique_key(&new_review.id, new_review.published_at);

                            dbg!(unique);

                            if unique_set.as_slise().contains(&unique) {
                                continue;
                            }

                            dbg!("try send");

                            if chan.send(Ok(new_review)).is_err() {
                                return;
                            }

                            unique_set.add(unique);
                        }
                    }
                    Err(e) => {
                        let _ = chan.send(Err(e));
                        return;
                    }
                }
            }
        });
    }

    /// Запускает наблюдатель, который при появлении новых вопросов или отзывов
    /// шлёт `NewFeedback` в канал. Если один поток завершился, другой тоже остановится.
    pub fn spawn_new_feedback_observer(
        &self,
        chan: UnboundedSender<Result<NewFeedback>>,
        question_interval: Duration,
        review_interval: Duration,
    ) {
        let stop_flag = Arc::new(AtomicBool::new(false));

        {
            let seller = self.clone();
            let chan = chan.clone();
            let stop_flag = stop_flag.clone();

            let (tx, mut rx) = mpsc::unbounded_channel::<Result<NewQuestion>>();
            seller.spawn_new_question_observer(tx, question_interval);

            tokio::spawn(async move {
                while !stop_flag.load(Ordering::Relaxed) {
                    if let Some(res) = rx.recv().await {
                        match res {
                            Ok(q) => {
                                if chan.send(Ok(NewFeedback::Question(q))).is_err() {
                                    break;
                                }
                            }
                            Err(e) => {
                                let _ = chan.send(Err(e));
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
            let chan = chan.clone();
            let stop_flag = stop_flag.clone();

            let (tx, mut rx) = mpsc::unbounded_channel::<Result<NewReview>>();
            seller.spawn_new_review_observer(tx, review_interval);

            tokio::spawn(async move {
                while !stop_flag.load(Ordering::Relaxed) {
                    if let Some(res) = rx.recv().await {
                        match res {
                            Ok(r) => {
                                if chan.send(Ok(NewFeedback::Review(r))).is_err() {
                                    break;
                                }
                            }
                            Err(e) => {
                                let _ = chan.send(Err(e));
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
