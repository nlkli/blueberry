use serde::{Deserialize, Serialize};

/// Ответ на запрос списка ответов на вопрос `/v1/question/answer/list`
#[derive(Debug, Serialize, Deserialize)]
pub struct QuestionAnswerListResponse {
    /// Список ответов на вопрос
    pub answers: Vec<QuestionAnswer>,

    /// Идентификатор последнего значения на странице.
    /// Чтобы получить следующие значения, передайте это значение в параметре `last_id` следующего запроса.
    pub last_id: String,
}

/// Ответ на вопрос
#[derive(Debug, Serialize, Deserialize)]
pub struct QuestionAnswer {
    /// Автор ответа
    pub author_name: String,

    /// Идентификатор ответа
    pub id: String,

    /// Дата публикации ответа (формат RFC 3339, например 2024-08-14T11:44:35.352Z)
    pub published_at: String,

    /// Идентификатор вопроса
    pub question_id: String,

    /// Идентификатор товара (SKU в системе Ozon)
    pub sku: i64,

    /// Текст ответа
    pub text: String,
}

/// Статус вопроса
#[derive(Debug, Default, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum QuestionStatus {
    New,
    #[default]
    All,
    Viewed,
    Processed,
    Unprocessed,
}

/// Ответ на запрос информации по вопросу `/v1/question/info`
#[derive(Debug, Serialize, Deserialize)]
pub struct QuestionInfoResponse {
    /// Количество ответов на вопрос
    pub answers_count: i64,

    /// Автор вопроса
    pub author_name: String,

    /// Идентификатор вопроса
    pub id: String,

    /// Ссылка на товар
    pub product_url: String,

    /// Дата публикации вопроса (формат RFC 3339)
    pub published_at: String,

    /// Ссылка на вопрос
    pub question_link: String,

    /// Идентификатор товара (SKU в системе Ozon)
    pub sku: i64,

    /// Статус вопроса
    pub status: QuestionStatus,

    /// Текст вопроса
    pub text: String,
}

/// Ответ на запрос списка вопросов `/v1/question/list`
#[derive(Debug, Serialize, Deserialize)]
pub struct QuestionListResponse {
    /// Список вопросов
    pub questions: Vec<Question>,

    /// Идентификатор последнего значения на странице
    pub last_id: String,
}

/// Вопрос покупателя
#[derive(Debug, Serialize, Deserialize)]
pub struct Question {
    /// Количество ответов на вопрос
    pub answers_count: i64,

    /// Имя автора вопроса
    pub author_name: String,

    /// Идентификатор вопроса
    pub id: String,

    /// Ссылка на товар
    pub product_url: String,

    /// Дата публикации вопроса (формат RFC 3339)
    pub published_at: String,

    /// Ссылка на вопрос
    pub question_link: String,

    /// Идентификатор товара (SKU в системе Ozon)
    pub sku: i64,

    /// Статус вопроса
    pub status: QuestionStatus,

    /// Текст вопроса
    pub text: String,
}

/// Ответ на создание ответа на вопрос `/v1/question/answer/create`
#[derive(Debug, Serialize, Deserialize)]
pub struct QuestionAnswerCreateResponse {
    /// Идентификатор созданного ответа на вопрос
    pub answer_id: String,
}

/// Ответ на запрос количества вопросов по статусам `/v1/question/count`
#[derive(Debug, Serialize, Deserialize)]
pub struct QuestionsCountResponse {
    /// Всего вопросов
    pub all: u32,

    /// Новые вопросы
    #[serde(rename = "new")]
    pub new: u32,

    /// Обработанные вопросы
    pub processed: u32,

    /// Необработанные вопросы
    pub unprocessed: u32,

    /// Просмотренные вопросы
    pub viewed: u32,
}
