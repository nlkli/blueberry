use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Serialize, Deserialize)]
/// Успешный ответ на метод работы с вопромами `/api/v1/questions`.
pub struct UpdateQuestionResponse {
    #[serde(default)]
    pub data: Option<Value>,

    /// Есть ли ошибка.
    pub error: bool,

    #[serde(default, rename = "errorText")]
    /// Описание ошибки.
    pub error_text: String,

    #[serde(default, rename = "additionalErrors")]
    /// Дополнительные ошибки.
    pub additional_errors: Option<Vec<String>>,
}

/// Ответ на запрос списка вопросов `/api/v1/questions`
#[derive(Debug, Serialize, Deserialize)]
pub struct QuestionListResponse {
    #[serde(default)]
    /// Данные о вопросах
    pub data: Option<QuestionListData>,

    /// Признак наличия ошибки
    pub error: bool,

    /// Описание ошибки (если есть)
    #[serde(rename = "errorText")]
    pub error_text: String,

    /// Дополнительные ошибки (если есть)
    #[serde(rename = "additionalErrors")]
    pub additional_errors: Option<Vec<String>>,
}

/// Данные о вопросах
#[derive(Debug, Serialize, Deserialize)]
pub struct QuestionListData {
    /// Количество необработанных вопросов
    #[serde(rename = "countUnanswered")]
    pub count_unanswered: u32,

    /// Количество обработанных вопросов
    #[serde(rename = "countArchive")]
    pub count_archive: u32,

    /// Список вопросов
    pub questions: Vec<Question>,
}

/// Вопрос покупателя
#[derive(Debug, Serialize, Deserialize)]
pub struct Question {
    /// Идентификатор вопроса
    pub id: String,

    /// Текст вопроса
    pub text: String,

    /// Дата и время создания вопроса (RFC 3339)
    #[serde(rename = "createdDate")]
    pub created_date: String,

    /// Статус вопроса:
    /// - `none` — вопрос отклонён продавцом
    /// - `wbRu` — ответ предоставлен, отображается на сайте покупателей
    /// - `suppliersPortalSynch` — новый вопрос
    pub state: String,

    /// Ответ на вопрос (если есть)
    pub answer: Option<Answer>,

    /// Детали товара
    #[serde(rename = "productDetails")]
    pub product_details: ProductDetails,

    /// Просмотрен ли вопрос
    #[serde(rename = "wasViewed")]
    pub was_viewed: bool,

    /// Признак подозрительного вопроса
    #[serde(rename = "isWarned")]
    pub is_warned: bool,
}

/// Ответ продавца на вопрос
#[derive(Debug, Serialize, Deserialize)]
pub struct Answer {
    /// Текст ответа
    pub text: String,

    /// Можно ли отредактировать ответ
    pub editable: bool,

    /// Дата и время создания ответа (RFC 3339)
    #[serde(rename = "createDate")]
    pub create_date: String,
}

/// Детали товара, к которому относится вопрос
#[derive(Debug, Serialize, Deserialize)]
pub struct ProductDetails {
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
    pub supplier_article: String,

    /// Имя продавца
    #[serde(rename = "supplierName")]
    pub supplier_name: String,

    /// Название бренда
    #[serde(rename = "brandName")]
    pub brand_name: String,
}

/// Ответ на запрос наличия непросмотренных отзывов и вопросов `/api/v1/new-feedbacks-questions`
#[derive(Debug, Serialize, Deserialize)]
pub struct NewFeedbacksQuestionsResponse {
    #[serde(default)]
    /// Данные о наличии непросмотренных вопросов и отзывов
    pub data: Option<NewFeedbacksQuestionsData>,

    /// Признак наличия ошибки
    pub error: bool,

    /// Описание ошибки (если есть)
    #[serde(rename = "errorText")]
    pub error_text: String,

    /// Дополнительные ошибки (если есть)
    #[serde(rename = "additionalErrors")]
    pub additional_errors: Option<Vec<String>>,
}

/// Данные о непросмотренных вопросах и отзывах
#[derive(Debug, Serialize, Deserialize)]
pub struct NewFeedbacksQuestionsData {
    /// Есть ли непросмотренные вопросы (true — есть, false — нет)
    #[serde(rename = "hasNewQuestions")]
    pub has_new_questions: bool,

    /// Есть ли непросмотренные отзывы (true — есть, false — нет)
    #[serde(rename = "hasNewFeedbacks")]
    pub has_new_feedbacks: bool,
}
