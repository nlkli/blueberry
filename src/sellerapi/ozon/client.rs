use super::models;
use crate::error::Result;
use reqwest::{IntoUrl, Method, header::HeaderMap};
use serde::{Deserialize, de::DeserializeOwned};
use serde_json::{Value, json};
use std::{
    any::TypeId,
    sync::{
        LazyLock,
        atomic::{AtomicU32, AtomicU64, Ordering},
    },
    time::{Duration, Instant},
};
use thiserror::Error as ThisError;

// const BASE_URL: &str = "https://api-seller.ozon.ru";
const DEFAULT_TIMEOUT: Duration = Duration::from_secs(5);
const LIMIT_OF_REQUESTS_PER_SECOND: u32 = 42;

#[derive(Debug, Deserialize, ThisError)]
#[error("OzonSellerApiError: code: {code}, message: {message}, details: {details:?}.")]
pub struct OzonSellerApiError {
    pub code: i32,
    pub details: Vec<OzonSellerApiErrorDetail>,
    pub message: String,
}

#[derive(Debug, Deserialize)]
pub struct OzonSellerApiErrorDetail {
    #[serde(rename = "typeUrl")]
    pub type_url: String,
    pub value: String,
}

#[derive(Debug)]
pub struct OzonSellerClient {
    client_id: String,
    api_key: String,
    last_request_time: AtomicU64,
    requests_per_sec: AtomicU32,
}

impl OzonSellerClient {
    pub fn new(client_id: String, api_key: String) -> Self {
        Self {
            client_id,
            api_key,
            last_request_time: AtomicU64::new(Self::now_millis()),
            requests_per_sec: AtomicU32::new(0),
        }
    }

    /// Создает экземпляр клиента OzonSellerClient из переменных окружения.
    ///
    /// Переменные окружения:
    /// - `OZON_SELLER_CLIENT_ID` — идентификатор клиента Ozon.
    /// - `OZON_SELLER_API_KEY` — API ключ для доступа к API Ozon.
    ///
    /// # Паника
    /// Метод вызовет `panic!`, если одна из переменных окружения не установлена.
    pub fn from_env() -> Self {
        let _ = dotenv::dotenv().ok();
        let client_id = std::env::var("OZON_SELLER_CLIENT_ID")
            .expect("Переменная окружения OZON_SELLER_CLIENT_ID не установлена");
        let api_key = std::env::var("OZON_SELLER_API_KEY")
            .expect("Переменная окружения OZON_SELLER_API_KEY не установлена");

        Self::new(client_id, api_key)
    }

    #[inline]
    fn now_millis() -> u64 {
        static START: LazyLock<Instant> = LazyLock::new(Instant::now);
        START.elapsed().as_millis() as u64
    }

    async fn throttle(&self) {
        let now = Self::now_millis();
        let last = self.last_request_time.load(Ordering::Relaxed);

        if now - last >= 1000 {
            self.last_request_time.store(now, Ordering::Relaxed);
            self.requests_per_sec.store(0, Ordering::Relaxed);
        }

        let count = self.requests_per_sec.fetch_add(1, Ordering::SeqCst);
        if count >= LIMIT_OF_REQUESTS_PER_SECOND {
            let sleep_ms = 1000u64.saturating_sub(now - last);
            if sleep_ms > 0 {
                tokio::time::sleep(std::time::Duration::from_millis(sleep_ms)).await;
            }

            self.last_request_time
                .store(Self::now_millis(), Ordering::Relaxed);
            self.requests_per_sec.store(0, Ordering::Relaxed);
        }
    }

    #[inline]
    async fn call_api<T: DeserializeOwned + 'static>(
        &self,
        method: Method,
        url: &str,
        payload: Option<Vec<u8>>,
    ) -> Result<T> {
        self.throttle().await;

        let mut headers = HeaderMap::new();

        headers.insert("Client-Id", self.client_id.parse().unwrap());
        headers.insert("Api-Key", self.api_key.parse().unwrap());

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
            Err(e) => match response.json::<OzonSellerApiError>().await {
                Ok(e) => Err(e.into()),
                Err(_) => Err(e.into()),
            },
        }
    }

    /// Список товаров
    pub async fn get_product_list(
        &self,
        filter: Option<&models::params::Filter<'_>>,
        limit: u32,
        last_id: Option<&str>,
    ) -> Result<models::ProductListResponse> {
        let payload = serde_json::to_vec(&json!({
            "filter": filter.unwrap_or(&models::params::Filter::default()),
            "limit": limit,
            "last_id": last_id.unwrap_or(""),
        }))
        .unwrap();

        const URL: &str = "https://api-seller.ozon.ru/v3/product/list";

        self.call_api(Method::POST, URL, Some(payload)).await
    }

    /// Получить информацию о товарах по идентификаторам
    pub async fn get_product_info_list(
        &self,
        filter: &models::params::Filter<'_>,
    ) -> Result<models::ProductInfoListResponse> {
        let payload = serde_json::to_vec(&json!({
            "offer_id": filter.offer_id.unwrap_or(&[]),
            "product_id": filter.product_id.unwrap_or(&[]),
            "sku": filter.sku.unwrap_or(&[]),
        }))
        .unwrap();

        const URL: &str = "https://api-seller.ozon.ru/v3/product/info/list";

        self.call_api(Method::POST, URL, Some(payload)).await
    }

    /// Получить описание товара
    pub async fn get_product_info_description(
        &self,
        product_id: i64,
    ) -> Result<models::ProductDescriptionResponse> {
        let payload = serde_json::to_vec(&json!({
            "product_id": product_id,
        }))
        .unwrap();

        const URL: &str = "https://api-seller.ozon.ru/v1/product/info/description";

        self.call_api(Method::POST, URL, Some(payload)).await
    }

    /// Получить описание характеристик товара
    pub async fn get_product_attributes_v4(
        &self,
        filter: Option<&models::params::Filter<'_>>,
        limit: u32,
        last_id: Option<&str>,
    ) -> Result<models::ProductAttributesResponse> {
        let payload = serde_json::to_vec(&json!({
            "filter": filter.unwrap_or(&models::params::Filter::default()),
            "limit": limit,
            "last_id": last_id.unwrap_or(""),
        }))
        .unwrap();

        const URL: &str = "https://api-seller.ozon.ru/v4/product/info/attributes";

        self.call_api(Method::POST, URL, Some(payload)).await
    }

    /// Дерево категорий и типов товаров
    pub async fn get_category_tree(
        &self,
        language: Option<&models::params::Language>,
    ) -> Result<models::CategoryTreeResponse> {
        let payload = serde_json::to_vec(&json!({
            "language": language.unwrap_or(&models::params::Language::default()),
        }))
        .unwrap();

        const URL: &str = "https://api-seller.ozon.ru/v1/description-category/tree";

        self.call_api(Method::POST, URL, Some(payload)).await
    }

    /// Список характеристик категории
    pub async fn get_attributes(
        &self,
        description_category_id: i64,
        language: Option<&models::params::Language>,
        type_id: i64,
    ) -> Result<models::CategoryAttributesResponse> {
        let payload = serde_json::to_vec(&json!({
            "description_category_id": description_category_id,
            "language": language.unwrap_or(&models::params::Language::default()),
            "type_id": type_id,
        }))
        .unwrap();

        const URL: &str = "https://api-seller.ozon.ru/v1/description-category/attribute";

        self.call_api(Method::POST, URL, Some(payload)).await
    }

    /// Справочник значений характеристики
    pub async fn get_attribute_values(
        &self,
        attribute_id: i64,
        description_category_id: i64,
        language: Option<&models::params::Language>,
        last_value_id: Option<i64>,
        limit: u32,
        type_id: i64,
    ) -> Result<Value> {
        let payload = serde_json::to_vec(&json!({
            "attribute_id": attribute_id,
            "description_category_id": description_category_id,
            "language": language.unwrap_or(&models::params::Language::default()),
            "last_value_id": last_value_id.unwrap_or_default(),
            "limit": limit,
            "type_id": type_id,
        }))
        .unwrap();

        const URL: &str = "https://api-seller.ozon.ru/v1/description-category/attribute/values";

        self.call_api(Method::POST, URL, Some(payload)).await
    }

    /// Получить список отзывов
    pub async fn get_review_list(
        &self,
        limit: u32,
        status: Option<&models::params::ReviewStatus>,
        sort_dir: Option<&models::params::SortDir>,
        last_id: Option<&str>,
    ) -> Result<models::ReviewListResponse> {
        let payload = serde_json::to_vec(&json!({
            "limit": limit,
            "status": status.unwrap_or(&models::params::ReviewStatus::default()),
            "sort_dir": sort_dir.unwrap_or(&models::params::SortDir::default()),
            "last_id": last_id.unwrap_or(""),
        }))
        .unwrap();

        const URL: &str = "https://api-seller.ozon.ru/v1/review/list";

        self.call_api(Method::POST, URL, Some(payload)).await
    }

    /// Получить информацию об отзыве
    pub async fn get_review_info(&self, review_id: &str) -> Result<models::ReviewInfoResponse> {
        let payload = serde_json::to_vec(&json!({
            "review_id": review_id,
        }))
        .unwrap();

        const URL: &str = "https://api-seller.ozon.ru/v1/review/info";

        self.call_api(Method::POST, URL, Some(payload)).await
    }

    /// Список комментариев на отзыв
    pub async fn get_review_comments(
        &self,
        review_id: &str,
        limit: u32,
        offset: Option<u32>,
    ) -> Result<models::ReviewCommentListResponse> {
        let payload = serde_json::to_vec(&json!({
            "review_id": review_id,
            "limit": limit,
            "offset": offset.unwrap_or(0),
        }))
        .unwrap();

        const URL: &str = "https://api-seller.ozon.ru/v1/review/comment/list";

        self.call_api(Method::POST, URL, Some(payload)).await
    }

    /// Ответить на отзыв
    pub async fn create_review_comment(
        &self,
        review_id: &str,
        text: &str,
        parent_comment_id: Option<&str>,
        mark_review_as_processed: bool,
    ) -> Result<models::ReviewCommentCreateResponse> {
        let payload = serde_json::to_vec(&json!({
            "review_id": review_id,
            "text": text,
            "parent_comment_id": parent_comment_id.unwrap_or(""),
            "mark_review_as_processed": mark_review_as_processed,
        }))
        .unwrap();

        const URL: &str = "https://api-seller.ozon.ru/v1/review/comment/create";

        self.call_api(Method::POST, URL, Some(payload)).await
    }

    /// Количество отзывов по статусам
    pub async fn get_review_count(&self) -> Result<models::ReviewsCountResponse> {
        let payload = "{}".as_bytes().to_vec();

        const URL: &str = "https://api-seller.ozon.ru/v1/review/count";

        self.call_api(Method::POST, URL, Some(payload)).await
    }

    /// Создать ответ на вопрос
    pub async fn create_question_answer(
        &self,
        question_id: &str,
        sku: i64,
        text: &str,
    ) -> Result<models::QuestionAnswerCreateResponse> {
        let payload = serde_json::to_vec(&json!({
            "question_id": question_id,
            "sku": sku,
            "text": text,
        }))
        .unwrap();

        const URL: &str = "https://api-seller.ozon.ru/v1/question/answer/create";

        self.call_api(Method::POST, URL, Some(payload)).await
    }

    /// Получить список ответов на вопрос
    pub async fn get_question_answers(
        &self,
        question_id: &str,
        sku: i64,
        last_id: Option<&str>,
    ) -> Result<models::QuestionAnswerListResponse> {
        let payload = serde_json::to_vec(&json!({
            "question_id": question_id,
            "sku": sku,
            "last_id": last_id.unwrap_or(""),
        }))
        .unwrap();

        const URL: &str = "https://api-seller.ozon.ru/v1/question/answer/list";

        self.call_api(Method::POST, URL, Some(payload)).await
    }

    /// Получить информацию по вопросу
    pub async fn get_question_info(
        &self,
        question_id: &str,
    ) -> Result<models::QuestionInfoResponse> {
        let payload = serde_json::to_vec(&json!({
            "question_id": question_id,
        }))
        .unwrap();

        const URL: &str = "https://api-seller.ozon.ru/v1/question/info";

        self.call_api(Method::POST, URL, Some(payload)).await
    }

    /// Получить список вопросов
    pub async fn get_question_list(
        &self,
        filter: Option<&models::params::QuestionListFilter<'_>>,
        last_id: Option<&str>,
    ) -> Result<models::QuestionListResponse> {
        let payload = serde_json::to_vec(&json!({
            "filter": filter.unwrap_or(&models::params::QuestionListFilter::default()),
            "last_id": last_id.unwrap_or(""),
        }))
        .unwrap();

        const URL: &str = "https://api-seller.ozon.ru/v1/question/list";

        self.call_api(Method::POST, URL, Some(payload)).await
    }

    /// Количество вопросов по статусам
    pub async fn get_question_count(&self) -> Result<models::QuestionsCountResponse> {
        let payload = "{}".as_bytes().to_vec();

        const URL: &str = "https://api-seller.ozon.ru/v1/question/count";

        self.call_api(Method::POST, URL, Some(payload)).await
    }

    /// Изменить статус вопросов
    pub async fn change_question_status(
        &self,
        id: &str,
        status: &models::params::SetQuestionStatus,
    ) -> Result<()> {
        let payload = serde_json::to_vec(&json!({
            "question_ids": id,
            "status": status,
        }))
        .unwrap();

        const URL: &str = "https://api-seller.ozon.ru/v1/question/change-status";

        self.call_api(Method::POST, URL, Some(payload)).await
    }

    /// Получить информацию о текущих рейтингах продавца
    pub async fn seller_rating_summary(
        &self,
        id: &str,
        status: &models::params::SetQuestionStatus,
    ) -> Result<()> {
        let payload = "{}".as_bytes().to_vec();

        const URL: &str = "https://api-seller.ozon.ru/v1/rating/summary";

        self.call_api(Method::POST, URL, Some(payload)).await
    }
}
