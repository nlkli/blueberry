use std::fmt::Write;

use serde::Serialize;

/// Универсальный фильтр.
#[derive(Debug, Clone, Default, Serialize)]
pub struct Filter<'a> {
    #[serde(skip_serializing_if = "Option::is_none", rename = "withPhoto")]
    /// Фильтр по фото:
    /// - 0 — только карточки без фото
    /// - 1 — только карточки с фото
    /// - -1 — все карточки товара
    pub with_photo: Option<i8>,

    #[serde(skip_serializing_if = "Option::is_none", rename = "textSearch")]
    /// Поиск по артикулу продавца, артикулу WB, баркоду.
    pub text_search: Option<&'a str>,

    #[serde(skip_serializing_if = "Option::is_none", rename = "objectIDs")]
    pub object_ids: Option<&'a [i64]>,

    #[serde(skip_serializing_if = "Option::is_none", rename = "imtID")]
    pub imt_id: Option<i64>,
}

/// Для QuestionParams.state: none - вопрос отклонён продавцом (такой вопрос не отображается на портале покупателей)
pub const REJECT_QUESTION_STATE: &str = "none";

/// Для QuestionParams.state: wbRu - ответ предоставлен, вопрос отображается на сайте покупателей.
pub const ANSWER_OR_EDIT_STATE: &str = "wbRu";

/// В зависимости от тела запроса, метод позволяет:
/// отметить вопрос как просмотренный (id, was_viewed)
/// отклонить вопрос (id, text, state=none)
/// ответить на вопрос или отредактировать ответ (id, text, state=wbRu)
#[derive(Debug, Clone, Default)]
pub struct UpdateQuestionParams<'a> {
    /// Id вопроса
    pub id: &'a str,

    /// Текст ответа
    pub answer_text: Option<&'a str>,

    /// Статус вопроса: [`REJECT_QUESTION_STATE`] или [`ANSWER_OR_EDIT_STATE`].
    /// - none - вопрос отклонён продавцом (такой вопрос не отображается на портале покупателей)
    /// - wbRu - ответ предоставлен, вопрос отображается на сайте покупателей.
    pub state: Option<&'static str>,

    /// Просмотреть вопрос. rename = "wasViewed"
    /// Просмотрен (true), не просмотрен (false)
    pub was_viewed: Option<bool>,
}

pub const ORDER_DATE_ASC: &str = "dateAsc";

pub const ORDER_DATE_DESC: &str = "dateDesc";

/// Фильтр параметров для получения списка вопросов или отзывов [`/api/v1/questions`, `/api/v1/feedbacks`].
#[derive(Debug, Clone, Default)]
pub struct QuestionsAndReviewsFilter {
    // #[serde(rename = "isAnswered")]
    /// Отвеченные вопросы (true) или неотвеченные вопросы (false).
    pub is_answered: bool,

    // #[serde(skip_serializing_if = "Option::is_none", rename = "nmId")]
    /// Артикул WB (nmId).
    pub nm_id: Option<i64>,

    /// Количество запрашиваемых вопросов (максимум 10_000).
    /// Количество запрашиваемых отзывов (максимум 5_000).
    pub take: u32,

    /// Количество вопросов для пропуска (максимум 10_000).
    /// /// Количество вопросов для пропуска (максимум 199_990).
    pub skip: u32,

    // #[serde(skip_serializing_if = "Option::is_none")]
    /// Сортировка вопросов по дате: [`ORDER_DATE_ASC`] или [`ORDER_DATE_DESC`].
    pub order: Option<&'static str>,

    // #[serde(skip_serializing_if = "Option::is_none", rename = "dateFrom")]
    /// Дата начала периода (Unix timestamp).
    pub date_from: Option<u64>,

    // #[serde(skip_serializing_if = "Option::is_none", rename = "dateTo")]
    /// Дата конца периода (Unix timestamp).
    pub date_to: Option<u64>,
}

impl QuestionsAndReviewsFilter {
    /// Параметры запроса ссылки
    pub fn as_query_params(&self) -> String {
        let mut query = format!(
            "isAnswered={}&take={}&skip={}",
            self.is_answered, self.take, self.skip
        );

        if let Some(nm_id) = self.nm_id {
            let _ = write!(&mut query, "&nmId={}", nm_id);
        }

        if let Some(order) = self.order {
            let _ = write!(&mut query, "&order={}", order);
        }

        if let Some(date_from) = self.date_from {
            let _ = write!(&mut query, "&dateFrom={}", date_from);
        }

        if let Some(date_to) = self.date_to {
            let _ = write!(&mut query, "&dateTo={}", date_to);
        }

        query
    }
}

/// Максимальное количество отзывов в ответе
pub const REVIEW_MAX_LIMIT: usize = 5000;

/// Максимальное количество вопросов в ответе
pub const QUESTION_MAX_LIMIT: usize = 10000;
