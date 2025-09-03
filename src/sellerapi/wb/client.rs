use super::models;
use crate::error::Result;
use reqwest::{Method, header::HeaderMap};
use serde::{Deserialize, de::DeserializeOwned};
use serde_json::{Value, json};
use std::{any::TypeId, fmt::Write, time::Duration};
use thiserror::Error as ThisError;
use tokio::sync::{/*Mutex,*/ Semaphore};

const DEFAULT_TIMEOUT: Duration = Duration::from_secs(5);
const MAX_RATE_LIMIT: usize = 4;

/// Универсальная ошибка обёртка для WB Seller API.
#[derive(Debug, Deserialize, ThisError)]
#[error("WbSellerApiError: status_code: {status_code}, detail: {detail}.")]
pub struct WbSellerApiError {
    pub status_code: u16,
    pub rate_limit_retry: Option<Duration>,
    pub detail: String,
}

/// Хелпер-макрос для однотипных ответов вида { error, errorText, additionalErrors, data }
macro_rules! unwrap_data_or_api_err {
    ($res:expr) => {{
        let res = $res;
        if res.error || res.data.is_none() {
            return Err(WbSellerApiError {
                status_code: 200,
                rate_limit_retry: None,
                detail: format!(
                    "{}, additional_errors: {:?}",
                    res.error_text, res.additional_errors
                ),
            }
            .into());
        }
        Ok(res.data.unwrap())
    }};
}

/// Клиент для WB Seller API.
#[derive(Debug)]
pub struct WbSellerClient {
    token: String,
    sem: Semaphore,
    // blocker: Arc<Mutex<()>>,
}

impl WbSellerClient {
    pub fn new(token: String) -> Self {
        Self {
            token,
            sem: Semaphore::const_new(MAX_RATE_LIMIT),
            // blocker: Arc::new(Mutex::new(())),
        }
    }

    /// Создает экземпляр клиента WbSellerClient из переменных окружения.
    ///
    /// Переменные окружения:
    /// - `WB_SELLER_API_TOKEN` — токен клиента Wb.
    ///
    /// # Паника
    /// Метод вызовет `panic!`, если одна из переменных окружения не установлена.
    pub fn from_env() -> Self {
        let _ = dotenv::dotenv().ok();
        let token = std::env::var("WB_SELLER_API_TOKEN")
            .expect("Переменная окружения WB_SELLER_API_TOKEN не установлена");

        Self::new(token)
    }

    // /// Блокирует выполнение всех запросов.
    // ///
    // /// Пока возвращённый `OwnedMutexGuard` не будет дропнут,
    // /// вызовы `call_api` будут ждать освобождения.
    // /// Может использоватся для ожидания в случае ошибки 429.
    // pub async fn block(&self) -> tokio::sync::OwnedMutexGuard<()> {
    //     self.blocker.clone().lock_owned().await
    // }

    #[inline]
    async fn call_api<T: DeserializeOwned + 'static>(
        &self,
        method: Method,
        url: &str,
        payload: Option<Vec<u8>>,
    ) -> Result<T> {
        // let block_guard = self.blocker.lock().await;
        // drop(block_guard);

        let _permit = self.sem.acquire().await.unwrap();
        if self.sem.available_permits() <= 1 {
            tokio::time::sleep(Duration::from_millis(500)).await;
        }

        let mut headers = HeaderMap::new();
        headers.insert("Authorization", self.token.parse().unwrap());

        let mut reqwest_builder = reqwest::Client::default()
            .request(method, url)
            .timeout(DEFAULT_TIMEOUT);

        if let Some(body) = payload {
            headers.insert("Content-Type", "application/json".parse().unwrap());
            reqwest_builder = reqwest_builder.body(body);
        }

        let response = reqwest_builder.headers(headers).send().await?;

        match response.error_for_status_ref() {
            Ok(_) => {
                if TypeId::of::<T>() == TypeId::of::<()>() {
                    Ok(unsafe { std::mem::zeroed() })
                } else {
                    Ok(response.json::<T>().await?)
                }
            }
            Err(e) => {
                let status_code = e.status().unwrap_or_default().as_u16();
                let rate_limit_retry = if status_code == 429 {
                    response
                        .headers()
                        .get("X-Ratelimit-Retry")
                        .and_then(|v| v.to_str().ok())
                        .and_then(|v| v.parse::<f64>().ok())
                        .map(Duration::from_secs_f64)
                } else {
                    None
                };

                Err(WbSellerApiError {
                    status_code,
                    rate_limit_retry,
                    detail: response.text().await.unwrap_or_default(),
                }
                .into())
            }
        }
    }

    /// [Список карточек товаров](https://dev.wildberries.ru/openapi/work-with-products/#tag/Kartochki-tovarov/paths/~1content~1v2~1get~1cards~1list/post)
    pub async fn get_cards_list(
        &self,
        filter: Option<&models::params::Filter<'_>>,
        cursor: &models::params::CardListCursor,
    ) -> Result<models::CardsListResponse> {
        let payload = serde_json::to_vec(&json!({
            "settings": {
                "cursor": cursor,
                "filter": filter.unwrap_or(&models::params::Filter {
                    with_photo: Some(-1),
                    ..Default::default()
                })
            }
        }))
        .unwrap();

        const URL: &str = "https://content-api.wildberries.ru/content/v2/get/cards/list";

        self.call_api(Method::POST, URL, Some(payload)).await
    }

    /// Внешний метод получения Rich контента. Не использует авторизации.
    pub async fn get_product_rich_content(basket_urlpath: &str, version: i32) -> Result<String> {
        let url = format!("{}/info/ru/rich_v{}.json", basket_urlpath, version);

        let responce = reqwest::get(&url).await?;

        match responce.error_for_status() {
            Ok(res) => Ok(res.text().await?),
            Err(e) => Err(e.into()),
        }
    }

    /// [Получить товары с ценами](https://dev.wildberries.ru/openapi/work-with-products/#tag/Ceny-i-skidki/paths/~1api~1v2~1list~1goods~1filter/get)
    pub async fn get_products_price(
        &self,
        limit: u32,
        offset: Option<u32>,
        filter_nmid: Option<i64>,
    ) -> Result<models::GoodsListResponse> {
        const URL: &str = "https://discounts-prices-api.wildberries.ru/api/v2/list/goods/filter";

        let mut url_with_query = format!("{}?limit={}", URL, limit);

        if let Some(val) = offset {
            let _ = write!(&mut url_with_query, "&offset={}", val);
        }
        if let Some(val) = filter_nmid {
            let _ = write!(&mut url_with_query, "&filterNmID={}", val);
        }

        self.call_api(Method::GET, &url_with_query, None).await
    }

    /// [Работа с вопросами](https://dev.wildberries.ru/openapi/user-communication/#tag/Voprosy/paths/~1api~1v1~1questions/patch)
    /// Обновляет состояние вопроса:
    /// - ответить или отредактировать ответ,
    /// - отклонить вопрос,
    /// - отметить вопрос как просмотренный.
    pub async fn update_question(
        &self,
        p: &models::params::UpdateQuestionParams<'_>,
    ) -> Result<Value> {
        let value = if let Some(ref text) = p.answer_text {
            json!({
                "id": p.id,
                "answer": {
                    "text": text
                },
                "state": p.state.unwrap_or(models::params::ANSWER_OR_EDIT_STATE)
            })
        } else {
            json!({
                "id": p.id,
                "wasViewed": p.was_viewed.unwrap_or_default()
            })
        };

        let payload = serde_json::to_vec(&value).unwrap();

        const URL: &str = "https://feedbacks-api.wildberries.ru/api/v1/questions";

        match self
            .call_api::<models::UpdateQuestionResponse>(Method::PATCH, URL, Some(payload))
            .await
        {
            Ok(res) => unwrap_data_or_api_err!(res),
            Err(e) => Err(e),
        }
    }

    /// [Список вопросов](https://dev.wildberries.ru/openapi/user-communication/#tag/Voprosy/paths/~1api~1v1~1questions/get)
    /// Метод предоставляет список вопросов по заданным фильтрам.
    pub async fn get_question_list(
        &self,
        filter: &models::params::QuestionsAndReviewsFilter,
    ) -> Result<models::QuestionListData> {
        const URL: &str = "https://feedbacks-api.wildberries.ru/api/v1/questions";

        let url_with_query = format!("{}?{}", URL, filter.as_query_params());

        match self
            .call_api::<models::QuestionListResponse>(Method::GET, &url_with_query, None)
            .await
        {
            Ok(res) => unwrap_data_or_api_err!(res),
            Err(e) => Err(e),
        }
    }

    /// [Количество вопросов](https://dev.wildberries.ru/openapi/user-communication/#tag/Voprosy/paths/~1api~1v1~1questions~1count/get)
    /// Метод предоставляет количество обработанных или необработанных вопросов за заданный период.
    pub async fn get_question_count(
        &self,
        is_aswered: Option<bool>,
        date_from: Option<u64>,
        date_to: Option<u64>,
    ) -> Result<u32> {
        const URL: &str = "https://feedbacks-api.wildberries.ru/api/v1/questions/count";

        let mut query = String::new();

        if let Some(val) = is_aswered {
            let _ = write!(&mut query, "isAnswered={}&", val);
        }
        if let Some(val) = date_from {
            let _ = write!(&mut query, "dateFrom={}&", val);
        }
        if let Some(val) = date_to {
            let _ = write!(&mut query, "dateTo={}&", val);
        }

        let url_with_query = if !query.is_empty() {
            Some(format!("{}?{}", URL, query.trim_end_matches("&")))
        } else {
            None
        };

        let full_url = url_with_query.as_ref().map_or(URL, |v| v);

        match self
            .call_api::<models::ReviewsCountResponse>(Method::GET, &full_url, None)
            .await
        {
            Ok(res) => unwrap_data_or_api_err!(res),
            Err(e) => Err(e),
        }
    }

    /// [Непросмотренные отзывы и вопросы](https://feedbacks-api.wildberries.ru/api/v1/new-feedbacks-questions)
    pub async fn get_new_feedbacks(&self) -> Result<models::NewFeedbacksQuestionsData> {
        const URL: &str = "https://feedbacks-api.wildberries.ru/api/v1/new-feedbacks-questions";

        match self
            .call_api::<models::NewFeedbacksQuestionsResponse>(Method::GET, URL, None)
            .await
        {
            Ok(res) => unwrap_data_or_api_err!(res),
            Err(e) => Err(e),
        }
    }

    /// [Ответить на отзыв](https://dev.wildberries.ru/openapi/user-communication/#tag/Otzyvy/paths/~1api~1v1~1feedbacks~1answer/post)
    pub async fn reply_to_review(&self, id: &str, text: &str) -> Result<()> {
        let payload = serde_json::to_vec(&json!({
            "id": id,
            "text": text,
        }))
        .unwrap();

        const URL: &str = "https://feedbacks-api.wildberries.ru/api/v1/feedbacks/answer";

        self.call_api(Method::POST, URL, Some(payload)).await
    }

    /// [Список отзывов](https://dev.wildberries.ru/openapi/user-communication/#tag/Otzyvy/paths/~1api~1v1~1feedbacks/get)
    /// Метод предоставляет список отзывов по заданным фильтрам.
    pub async fn get_review_list(
        &self,
        filter: &models::params::QuestionsAndReviewsFilter,
    ) -> Result<models::ReviewListData> {
        const URL: &str = "https://feedbacks-api.wildberries.ru/api/v1/feedbacks";

        let url_with_query = format!("{}?{}", URL, filter.as_query_params());

        match self
            .call_api::<models::ReviewListResponse>(Method::GET, &url_with_query, None)
            .await
        {
            Ok(res) => unwrap_data_or_api_err!(res),
            Err(e) => Err(e),
        }
    }

    /// [Количество отзывов](https://dev.wildberries.ru/openapi/user-communication/#tag/Otzyvy/paths/~1api~1v1~1feedbacks~1count/get)
    /// Метод предоставляет количество обработанных или необработанных отзывов за заданный период.
    pub async fn get_review_count(
        &self,
        is_aswered: Option<bool>,
        date_from: Option<u64>,
        date_to: Option<u64>,
    ) -> Result<u32> {
        const URL: &str = "https://feedbacks-api.wildberries.ru/api/v1/feedbacks/count";

        let mut query = String::new();

        if let Some(val) = is_aswered {
            let _ = write!(&mut query, "isAnswered={}&", val);
        }
        if let Some(val) = date_from {
            let _ = write!(&mut query, "dateFrom={}&", val);
        }
        if let Some(val) = date_to {
            let _ = write!(&mut query, "dateTo={}&", val);
        }

        let url_with_query = if !query.is_empty() {
            Some(format!("{}?{}", URL, query.trim_end_matches("&")))
        } else {
            None
        };

        let full_url = url_with_query.as_ref().map_or(URL, |v| v);

        match self
            .call_api::<models::ReviewsCountResponse>(Method::GET, &full_url, None)
            .await
        {
            Ok(res) => unwrap_data_or_api_err!(res),
            Err(e) => Err(e),
        }
    }

    /// [Необработанные отзывы](https://dev.wildberries.ru/openapi/user-communication/#tag/Otzyvy/paths/~1api~1v1~1feedbacks~1count-unanswered/get)
    pub async fn get_review_count_unanswered(&self) -> Result<models::ReviewsCountUnansweredData> {
        const URL: &str = "https://feedbacks-api.wildberries.ru/api/v1/feedbacks/count-unanswered";

        match self
            .call_api::<models::ReviewsCountUnansweredResponse>(Method::GET, URL, None)
            .await
        {
            Ok(res) => unwrap_data_or_api_err!(res),
            Err(e) => Err(e),
        }
    }
}
