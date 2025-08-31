use std::default;

use serde::{Deserialize, Serialize};

/// Ответ на запрос списка отзывов `/api/v1/feedbacks`
#[derive(Debug, Serialize, Deserialize)]
pub struct ReviewListResponse {
    /// Данные о списке отзывов
    #[serde(default)]
    pub data: Option<ReviewListData>,

    /// Признак наличия ошибки
    pub error: bool,

    /// Описание ошибки (если есть)
    #[serde(rename = "errorText")]
    pub error_text: String,

    /// Дополнительные ошибки (если есть)
    #[serde(rename = "additionalErrors")]
    pub additional_errors: Option<Vec<String>>,
}

/// Данные о списке отзывов
#[derive(Debug, Serialize, Deserialize)]
pub struct ReviewListData {
    /// Количество необработанных отзывов
    #[serde(rename = "countUnanswered")]
    pub count_unanswered: i32,

    /// Количество обработанных отзывов
    #[serde(rename = "countArchive")]
    pub count_archive: i32,

    /// Список отзывов
    #[serde(rename = "feedbacks")]
    pub reviews: Vec<Review>,
}

/// Отзыв покупателя
#[derive(Debug, Serialize, Deserialize)]
pub struct Review {
    /// Идентификатор отзыва
    pub id: String,

    /// Текст отзыва
    pub text: String,

    /// Достоинства товара
    pub pros: String,

    /// Недостатки товара
    pub cons: String,

    /// Оценка товара
    #[serde(rename = "productValuation")]
    pub product_valuation: i32,

    /// Дата и время создания отзыва (RFC 3339)
    #[serde(rename = "createdDate")]
    pub created_date: String,

    /// Ответ на отзыв (если есть)
    pub answer: Option<ReviewAnswer>,

    /// Статус отзыва:
    /// - `none` — не обработан
    /// - `wbRu` — обработан
    pub state: String,

    /// Детали товара
    #[serde(rename = "productDetails")]
    pub product_details: ReviewProductDetails,

    /// Видео (если есть)
    pub video: Option<ReviewVideo>,

    /// Просмотрен ли отзыв
    #[serde(rename = "wasViewed")]
    pub was_viewed: bool,

    /// Фотографии отзыва
    #[serde(rename = "photoLinks")]
    pub photo_links: Option<Vec<PhotoLink>>,

    /// Имя автора
    #[serde(rename = "userName")]
    pub user_name: String,

    /// Соответствие заявленного размера реальному
    #[serde(rename = "matchingSize")]
    pub matching_size: String,

    /// Доступна ли жалоба на отзыв
    #[serde(rename = "isAbleSupplierFeedbackValuation")]
    pub is_able_supplier_review_valuation: bool,

    /// Ключ причины жалобы на отзыв
    #[serde(rename = "supplierFeedbackValuation")]
    pub supplier_review_valuation: i32,

    /// Доступна ли возможность сообщить о проблеме с товаром
    #[serde(rename = "isAbleSupplierProductValuation")]
    pub is_able_supplier_product_valuation: bool,

    /// Ключ проблемы с товаром
    #[serde(rename = "supplierProductValuation")]
    pub supplier_product_valuation: i32,

    /// Доступна ли опция возврата
    #[serde(rename = "isAbleReturnProductOrders")]
    pub is_able_return_product_orders: bool,

    /// Дата получения ответа на возврат
    #[serde(rename = "returnProductOrdersDate")]
    pub return_product_orders_date: Option<String>,

    /// Список тегов покупателя
    pub bables: Option<Vec<String>>,

    /// Штрихкод последнего заказа
    #[serde(rename = "lastOrderShkId")]
    pub last_order_shk_id: i64,

    /// Дата последнего заказа
    #[serde(rename = "lastOrderCreatedAt")]
    pub last_order_created_at: String,

    /// Цвет товара
    pub color: String,

    /// ID предмета
    #[serde(rename = "subjectId")]
    pub subject_id: i64,

    /// Название предмета
    #[serde(rename = "subjectName")]
    pub subject_name: String,

    /// ID начального отзыва (если есть)
    #[serde(rename = "parentFeedbackId")]
    pub parent_review_id: Option<String>,

    /// ID дополненного отзыва (если есть)
    #[serde(rename = "childFeedbackId")]
    pub child_review_id: Option<String>,
}

/// Ответ на отзыв
#[derive(Debug, Serialize, Deserialize)]
pub struct ReviewAnswer {
    /// Текст ответа
    pub text: String,

    /// Статус ответа
    pub state: String,

    /// Можно ли редактировать ответ
    pub editable: bool,
}

/// Детали товара в отзыве
#[derive(Debug, Serialize, Deserialize)]
pub struct ReviewProductDetails {
    /// ID карточки товара
    #[serde(rename = "imtId")]
    pub imt_id: i64,

    /// Артикул WB
    #[serde(rename = "nmId")]
    pub nm_id: i64,

    /// Название товара
    #[serde(rename = "productName")]
    pub product_name: String,

    /// Артикул продавца
    #[serde(rename = "supplierArticle")]
    pub supplier_article: Option<String>,

    /// Имя продавца
    #[serde(rename = "supplierName")]
    pub supplier_name: Option<String>,

    /// Бренд товара
    #[serde(rename = "brandName")]
    pub brand_name: Option<String>,

    /// Размер товара (techSize)
    pub size: String,
}

/// Фото отзыва
#[derive(Debug, Serialize, Deserialize)]
pub struct PhotoLink {
    /// Ссылка на фото полного размера
    #[serde(rename = "fullSize")]
    pub full_size: String,

    /// Ссылка на мини-фото
    #[serde(rename = "miniSize")]
    pub mini_size: String,
}

/// Видео отзыва
#[derive(Debug, Serialize, Deserialize)]
pub struct ReviewVideo {
    /// Превью видео
    #[serde(rename = "previewImage")]
    pub preview_image: String,

    /// Ссылка на файл плейлиста HLS
    pub link: String,

    /// Продолжительность видео в секундах
    #[serde(rename = "durationSec")]
    pub duration_sec: i32,
}

/// Ответ на запрос количества отзывов `/api/v1/feedbacks/count`
#[derive(Debug, Serialize, Deserialize)]
pub struct ReviewsCountResponse {
    /// Количество отзывов
    #[serde(default)]
    pub data: Option<u32>,

    /// Признак наличия ошибки
    pub error: bool,

    /// Описание ошибки (если есть)
    #[serde(rename = "errorText")]
    pub error_text: String,

    /// Дополнительные ошибки (если есть)
    #[serde(rename = "additionalErrors")]
    pub additional_errors: Option<Vec<String>>,
}

/// Ответ на запрос количества yеобработанныx отзывов `/api/v1/feedbacks/count-unanswered`
#[derive(Debug, Serialize, Deserialize)]
pub struct ReviewsCountUnansweredResponse {
    /// Количество отзывов
    #[serde(default)]
    pub data: Option<ReviewsCountUnansweredData>,

    /// Признак наличия ошибки
    pub error: bool,

    /// Описание ошибки (если есть)
    #[serde(rename = "errorText")]
    pub error_text: String,

    /// Дополнительные ошибки (если есть)
    #[serde(rename = "additionalErrors")]
    pub additional_errors: Option<Vec<String>>,
}

/// Количество необработанных отзывов за сегодня и за всё время.
#[derive(Debug, Serialize, Deserialize)]
pub struct ReviewsCountUnansweredData {
    /// Количество необработанных отзывов
    #[serde(rename = "countUnanswered")]
    pub count_unanswered: u32,

    /// Количество необработанных отзывов за сегодня
    #[serde(rename = "countUnansweredToday")]
    pub count_unanswered_today: u32,

    /// Средняя оценка всех отзывов
    pub valuation: String,
}
