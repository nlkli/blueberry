use serde::{Deserialize, Serialize};

/// Ответ API `/v3/product/info/list`
#[derive(Debug, Serialize, Deserialize)]
pub struct ProductInfoListResponse {
    /// Список товаров
    pub items: Vec<ProductInfo>,
}

/// Информация о товаре из ProductInfoListResponse
#[derive(Debug, Serialize, Deserialize)]
pub struct ProductInfo {
    /// Идентификатор товара в системе продавца (product_id)
    pub id: i64,

    /// SKU товара в системе Ozon
    pub sku: i64,

    /// Артикул (offer_id)
    pub offer_id: String,

    /// Название товара
    pub name: String,

    /// Дата и время создания товара
    pub created_at: String,

    /// Дата последнего обновления товара
    pub updated_at: String,

    /// Валюта (например, "RUB")
    pub currency_code: String,

    /// Идентификатор категории описания
    pub description_category_id: i64,

    /// Остатки уценённого товара на складе Ozon
    pub discounted_fbo_stocks: i32,

    /// Признак, что у товара есть уценённые аналоги
    pub has_discounted_fbo_item: bool,

    /// Признак архивирования вручную
    pub is_archived: bool,

    /// Признак автоархивирования
    pub is_autoarchived: bool,

    /// Признак, что товар уценён
    pub is_discounted: bool,

    /// Признак крупногабаритного товара
    pub is_kgt: bool,

    /// Возможность предоплаты (deprecated)
    pub is_prepayment_allowed: bool,

    /// Признак "супер-товара"
    pub is_super: bool,

    /// Цена на товар с учётом акций
    pub marketing_price: String,

    /// Минимальная цена после применения акций
    pub min_price: String,

    /// Цена до учёта скидок (отображается зачёркнутой)
    pub old_price: String,

    /// Текущая цена товара
    pub price: String,

    /// Тип товара
    pub type_id: i64,

    /// Ставка НДС
    pub vat: String,

    /// Объёмный вес
    pub volume_weight: f64,

    /// Штрихкоды товара
    pub barcodes: Vec<String>,

    /// Изображения цвета товара
    pub color_image: Vec<String>,

    /// Основные изображения
    pub images: Vec<String>,

    /// 360° изображения
    pub images360: Vec<String>,

    /// Главное изображение
    pub primary_image: Vec<String>,

    /// Информация о комиссиях
    pub commissions: Vec<Commission>,

    /// Ошибки при создании/валидации товара
    pub errors: Vec<ProductError>,

    /// Информация о модели
    pub model_info: ModelInfo,

    /// Ценовые индексы
    pub price_indexes: PriceIndexes,

    /// Акции
    pub promotions: Vec<Promotion>,

    /// Источники товара
    pub sources: Vec<ProductSource>,

    /// Статусы товара
    pub statuses: ProductStatuses,

    /// Остатки товара
    pub stocks: ProductStocks,

    /// Настройки видимости
    pub visibility_details: VisibilityDetails,
}

/// Комиссия
#[derive(Debug, Serialize, Deserialize)]
pub struct Commission {
    #[serde(default)]
    /// Стоимость доставки
    pub delivery_amount: f64,

    #[serde(default)]
    /// Процент комиссии
    pub percent: f64,

    #[serde(default)]
    /// Стоимость возврата
    pub return_amount: f64,

    #[serde(default)]
    /// Схема продажи
    pub sale_schema: String,

    #[serde(default)]
    /// Сумма комиссии
    pub value: f64,
}

/// Ошибка при создании или валидации товара
#[derive(Debug, Serialize, Deserialize)]
pub struct ProductError {
    /// Идентификатор характеристики
    pub attribute_id: i64,

    /// Код ошибки
    pub code: String,

    /// Поле, в котором найдена ошибка
    pub field: String,

    /// Уровень ошибки (ERROR_LEVEL_*)
    pub level: String,

    /// Статус товара, в котором произошла ошибка
    pub state: String,

    /// Описание ошибки
    pub texts: ErrorTexts,
}

/// Детали ошибки
#[derive(Debug, Serialize, Deserialize)]
pub struct ErrorTexts {
    /// Название атрибута
    pub attribute_name: String,

    /// Описание ошибки
    pub description: String,

    /// Код подсказки
    pub hint_code: String,

    /// Сообщение об ошибке
    pub message: String,

    /// Параметры ошибки
    pub params: Vec<ErrorParam>,

    /// Краткое описание
    pub short_description: String,
}

/// Параметр ошибки
#[derive(Debug, Serialize, Deserialize)]
pub struct ErrorParam {
    /// Название параметра
    pub name: String,

    /// Значение параметра
    pub value: String,
}

/// Информация о модели
#[derive(Debug, Serialize, Deserialize)]
pub struct ModelInfo {
    /// Количество товаров
    pub count: i64,

    /// Идентификатор модели
    pub model_id: i64,
}

/// Данные об индексах цен
#[derive(Debug, Serialize, Deserialize)]
pub struct PriceIndexes {
    /// Тип индекса цены (например, COLOR_INDEX_GREEN)
    pub color_index: String,

    /// Цена товара у конкурентов на других площадках
    #[serde(default)]
    pub external_index_data: Option<IndexData>,

    /// Цена товара у конкурентов на Ozon
    #[serde(default)]
    pub ozon_index_data: Option<IndexData>,

    /// Цена вашего товара на других площадках
    #[serde(default)]
    pub self_marketplaces_index_data: Option<IndexData>,
}

/// Универсальная структура для данных о ценах
#[derive(Debug, Serialize, Deserialize)]
pub struct IndexData {
    /// Минимальная цена
    pub minimal_price: String,

    /// Валюта минимальной цены
    pub minimal_price_currency: String,

    /// Значение индекса цены (может быть дробным)
    pub price_index_value: f64,
}

/// Акция
#[derive(Debug, Serialize, Deserialize)]
pub struct Promotion {
    /// Признак включённой акции
    pub is_enabled: bool,

    /// Тип акции
    pub r#type: String,
}

/// Источник товара
#[derive(Debug, Serialize, Deserialize)]
pub struct ProductSource {
    /// Дата создания
    pub created_at: String,

    /// Код кванта
    pub quant_code: String,

    /// Тип упаковки
    pub shipment_type: String,

    /// SKU товара
    pub sku: i64,

    /// Схема продажи (FBO/FBS)
    pub source: String,
}

/// Статусы товара
#[derive(Debug, Serialize, Deserialize)]
pub struct ProductStatuses {
    /// Признак, что товар создан
    pub is_created: bool,

    /// Статус модерации
    pub moderate_status: String,

    /// Общий статус товара
    pub status: String,

    /// Описание статуса
    pub status_description: String,

    /// Статус, в котором возникла ошибка
    pub status_failed: String,

    /// Название статуса
    pub status_name: String,

    /// Подсказка по статусу
    pub status_tooltip: String,

    /// Дата последнего обновления статуса
    pub status_updated_at: String,

    /// Статус валидации
    pub validation_status: String,
}

/// Информация об остатках
#[derive(Debug, Serialize, Deserialize)]
pub struct ProductStocks {
    /// Признак, что есть остатки
    pub has_stock: bool,

    /// Список складских остатков
    pub stocks: Vec<StockItem>,
}

/// Остатки на складе
#[derive(Debug, Serialize, Deserialize)]
pub struct StockItem {
    /// Доступное количество
    pub present: i64,

    /// Зарезервированное количество
    pub reserved: i64,

    /// SKU товара
    pub sku: i64,

    /// Источник
    pub source: String,
}

/// Видимость товара
#[derive(Debug, Serialize, Deserialize)]
pub struct VisibilityDetails {
    /// Есть цена
    pub has_price: bool,

    /// Есть остаток
    pub has_stock: bool,
}

/// Корневой объект ответа API `/v3/product/list`
#[derive(Debug, Serialize, Deserialize)]
pub struct ProductListResponse {
    /// Результат запроса
    pub result: ProductListResult,
}

/// Результат запроса со списком товаров
#[derive(Debug, Serialize, Deserialize)]
pub struct ProductListResult {
    /// Список товаров
    pub items: Vec<ProductListItem>,

    /// Общее количество товаров
    pub total: u32,

    /// Идентификатор последнего значения на странице (для пагинации)
    pub last_id: String,
}

/// Информация о товаре в списке
#[derive(Debug, Serialize, Deserialize)]
pub struct ProductListItem {
    /// Товар в архиве
    pub archived: bool,

    /// Есть остатки на складах FBO
    pub has_fbo_stocks: bool,

    /// Есть остатки на складах FBS
    pub has_fbs_stocks: bool,

    /// Уценённый товар
    pub is_discounted: bool,

    /// Идентификатор товара в системе продавца — артикул
    pub offer_id: String,

    /// Идентификатор товара в системе продавца — product_id
    pub product_id: i64,

    /// Остатки по квантам
    pub quants: Vec<Quant>,
}

/// Информация о кванте товара (остатки)
#[derive(Debug, Serialize, Deserialize)]
pub struct Quant {
    /// Код кванта
    pub quant_code: String,

    /// Размер кванта
    pub quant_size: i64,
}

/// Корневой объект ответа API `/v1/product/info/description`
#[derive(Debug, Serialize, Deserialize)]
pub struct ProductDescriptionResponse {
    /// Результат — описание товара
    pub result: ProductDescription,
}

/// Информация об описании товара
#[derive(Debug, Serialize, Deserialize)]
pub struct ProductDescription {
    /// Идентификатор товара в системе Ozon
    pub id: i64,

    /// Идентификатор товара в системе продавца — артикул
    pub offer_id: String,

    /// Название товара
    pub name: String,

    /// Подробное описание товара (может содержать HTML-разметку)
    pub description: String,
}

/// Корневой объект ответа API
#[derive(Debug, Serialize, Deserialize)]
pub struct ProductAttributesResponse {
    /// Результаты запроса — массив товаров с характеристиками
    pub result: Vec<ProductAttributes>,

    /// Общее количество товаров в списке
    pub total: i64,

    /// Идентификатор последнего элемента на странице для пагинации
    pub last_id: String,
}

/// Информация о товаре из ProductAttributesResponse
#[derive(Debug, Serialize, Deserialize)]
pub struct ProductAttributes {
    /// Идентификатор товара в системе продавца
    pub id: i64,

    /// Основной штрихкод
    pub barcode: String,

    /// Все штрихкоды товара
    pub barcodes: Vec<String>,

    /// Название товара
    pub name: String,

    /// Идентификатор товара (артикул) в системе продавца
    pub offer_id: String,

    /// Идентификатор типа товара
    pub type_id: i64,

    /// Высота упаковки
    pub height: i64,

    /// Глубина упаковки
    pub depth: i64,

    /// Ширина упаковки
    pub width: i64,

    /// Единица измерения габаритов (mm, cm, in)
    pub dimension_unit: String,

    /// Вес товара в упаковке
    pub weight: i64,

    /// Единица измерения веса (g, kg)
    pub weight_unit: String,

    /// Ссылка на главное изображение товара
    pub primary_image: String,

    /// SKU товара в системе Ozon
    pub sku: i64,

    /// Информация о модели товара
    pub model_info: ModelInfo,

    /// Ссылки на все изображения товара
    pub images: Vec<String>,

    /// Список PDF-файлов, связанных с товаром
    pub pdf_list: Vec<PdfFile>,

    /// Список характеристик товара
    pub attributes: Vec<Attribute>,

    /// Список идентификаторов характеристик со значением по умолчанию
    pub attributes_with_defaults: Vec<i64>,

    /// Сложные вложенные характеристики
    pub complex_attributes: Vec<Attribute>,

    /// Маркетинговый цвет товара
    pub color_image: String,

    /// Идентификатор категории для описания товара
    pub description_category_id: i64,
}

/// PDF-файл, связанный с товаром
#[derive(Debug, Serialize, Deserialize)]
pub struct PdfFile {
    /// Путь к PDF-файлу
    pub file_name: String,

    /// Название PDF-файла
    pub name: String,
}

/// Характеристика товара
#[derive(Debug, Serialize, Deserialize)]
pub struct Attribute {
    /// Идентификатор характеристики
    pub id: i64,

    /// Идентификатор сложной характеристики (если есть вложенные свойства)
    pub complex_id: i64,

    /// Массив значений характеристики
    pub values: Vec<AttributeValue>,
}

/// Значение характеристики товара
#[derive(Debug, Serialize, Deserialize)]
pub struct AttributeValue {
    /// Идентификатор значения в словаре (если есть)
    pub dictionary_value_id: i64,

    /// Значение характеристики
    pub value: String,
}
