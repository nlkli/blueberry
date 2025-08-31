use serde::{Deserialize, Serialize};

/// Ответ на запрос информации о рейтингах продавца `/v1/rating/summary`
#[derive(Debug, Serialize, Deserialize)]
pub struct RatingSummaryResponse {
    /// Список групп рейтингов
    pub groups: Vec<RatingGroup>,

    /// Данные по индексу локализации (если за 14 дней были продажи)
    pub localization_index: Vec<LocalizationIndex>,

    /// Признак превышения баланса штрафных баллов
    pub penalty_score_exceeded: bool,

    /// Признак наличия подписки Premium
    pub premium: bool,

    /// Признак наличия подписки Premium Plus
    pub premium_plus: bool,
}

/// Группа рейтингов
#[derive(Debug, Serialize, Deserialize)]
pub struct RatingGroup {
    /// Название группы рейтингов
    pub group_name: String,

    /// Список рейтингов в группе
    pub items: Vec<RatingItem>,
}

/// Отдельный показатель рейтинга
#[derive(Debug, Serialize, Deserialize)]
pub struct RatingItem {
    /// Изменение показателя
    pub change: RatingChange,

    /// Текущее значение
    pub current_value: i32,

    /// Название показателя
    pub name: String,

    /// Значение в прошлом периоде
    pub past_value: i32,

    /// Уровень рейтинга (например, "A", "B")
    pub rating: String,

    /// Направление изменения рейтинга
    pub rating_direction: String,

    /// Статус показателя
    pub status: String,

    /// Тип значения (например, "percent", "score")
    pub value_type: String,
}

/// Данные об изменении рейтинга
#[derive(Debug, Serialize, Deserialize)]
pub struct RatingChange {
    /// Направление изменения (например, "up", "down")
    pub direction: String,

    /// Значение/смысл изменения
    pub meaning: String,
}

/// Индекс локализации
#[derive(Debug, Serialize, Deserialize)]
pub struct LocalizationIndex {
    /// Дата расчёта индекса
    pub calculation_date: String,

    /// Значение индекса локализации
    pub localization_percentage: i32,
}
