use super::QuestionStatus;
use serde::Serialize;

#[derive(Debug, Clone, Copy, Default, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
/// Статусы отзывов при запросе.
pub enum ReviewStatus {
    #[default]
    All,
    Unprocessed,
    Processed,
}

#[derive(Debug, Clone, Default, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Language {
    #[default]
    Default,
    En,
    Ru,
    /// турецкий
    Tr,
    /// китайский
    ZhHanz,
}

#[derive(Debug, Clone, Default, Serialize)]
pub enum SortDir {
    #[default]
    /// по возрастанию
    ASC,

    /// по убыванию
    DESC,
}

/// Универсальный фильтр.
#[derive(Debug, Clone, Default, Serialize)]
pub struct Filter<'a> {
    #[serde(skip_serializing_if = "Option::is_none")]
    /// Фильтр по параметру offer_id. Вы можете передавать список значений.
    pub offer_id: Option<&'a [&'a str]>,

    #[serde(skip_serializing_if = "Option::is_none")]
    /// Фильтр по параметру product_id. Вы можете передавать список значений.
    pub product_id: Option<&'a [i64]>,

    #[serde(skip_serializing_if = "Option::is_none")]
    /// Идентификатор товара в системе Ozon — SKU.
    pub sku: Option<&'a [i64]>,

    #[serde(skip_serializing_if = "Option::is_none")]
    /// Фильтр по видимости товара
    pub visibility: Option<Visibility>,
}

/// Фильтр по видимости товара в Ozon
#[derive(Debug, Clone, Default, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Visibility {
    #[default]
    /// Все товары
    All,

    /// Товары, которые видны покупателям
    Visible,

    /// Товары, которые не видны покупателям
    Invisible,

    /// Товары, у которых не указано наличие
    EmptyStock,

    /// Товары, которые не прошли модерацию
    NotModerated,

    /// Товары, которые прошли модерацию
    Moderated,

    /// Товары видны покупателям, но недоступны к покупке
    Disabled,

    /// Товары, создание которых завершилось ошибкой
    StateFailed,

    /// Товары, готовые к поставке
    ReadyToSupply,

    /// Товары проходят проверку валидатором на премодерации
    ValidationStatePending,

    /// Товары, которые не прошли проверку валидатором на премодерации
    ValidationStateFail,

    /// Товары, которые прошли проверку валидатором на премодерации
    ValidationStateSuccess,

    /// Товары, готовые к продаже
    ToSupply,

    /// Товары в продаже
    InSale,

    /// Товары, скрытые от покупателей
    RemovedFromSale,

    /// Товары с завышенной ценой
    Overpriced,

    /// Товары со слишком завышенной ценой
    CriticallyOverpriced,

    /// Товары без штрихкода
    EmptyBarcode,

    /// Товары со штрихкодом
    BarcodeExists,

    /// Товары на карантине после изменения цены более чем на 50%
    Quarantine,

    /// Товары в архиве
    Archived,

    /// Товары в продаже со стоимостью выше, чем у конкурентов
    OverpricedWithStock,

    /// Товары в продаже с пустым или неполным описанием
    PartialApproved,
}

/// Статус вопроса
#[derive(Debug, Default, Clone, Copy, Serialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum SetQuestionStatus {
    #[default]
    New,
    Viewed,
    Processed,
}

#[derive(Debug, Clone, Copy, Default, Serialize)]
pub struct QuestionListFilter<'a> {
    pub status: QuestionStatus,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub date_from: Option<&'a str>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub date_to: Option<&'a str>,
}

/// Минимальное количество отзывов в ответе
pub const REVIEW_MIN_LIMIT: usize = 20;

/// Максимальное количество отзывов в ответе
pub const REVIEW_MAX_LIMIT: usize = 100;
