use serde::{Deserialize, Serialize};

/// Ответ на запрос списка карточек товаров `/content/v2/get/cards/list`
#[derive(Debug, Serialize, Deserialize)]
pub struct CardsListResponse {
    /// Список карточек товаров
    pub cards: Vec<Card>,

    /// Пагинатор для запроса следующей страницы
    pub cursor: Cursor,
}

/// Карточка товара
#[derive(Debug, Serialize, Deserialize)]
pub struct Card {
    /// Артикул WB
    #[serde(rename = "nmID")]
    pub nm_id: i64,

    /// ID объединённой карточки товара
    #[serde(rename = "imtID")]
    pub imt_id: i64,

    /// Внутренний технический ID карточки товара (UUID)
    #[serde(rename = "nmUUID")]
    pub nm_uuid: String,

    /// ID предмета
    #[serde(rename = "subjectID")]
    pub subject_id: i64,

    /// Название предмета
    #[serde(rename = "subjectName")]
    pub subject_name: String,

    /// Артикул продавца
    #[serde(rename = "vendorCode")]
    pub vendor_code: String,

    /// Бренд
    pub brand: String,

    /// Наименование товара
    pub title: String,

    /// Описание товара
    pub description: String,

    /// Требуется ли код маркировки
    #[serde(rename = "needKiz")]
    pub need_kiz: bool,

    /// Список фотографий
    #[serde(default)]
    pub photos: Vec<Photo>,

    /// Ссылка на видео
    #[serde(default)]
    pub video: Option<String>,

    /// Данные об оптовой продаже
    #[serde(default)]
    pub wholesale: Option<Wholesale>,

    /// Габариты и вес товара
    #[serde(default)]
    pub dimensions: Option<ProductDim>,

    /// Характеристики товара
    #[serde(default)]
    pub characteristics: Vec<Characteristic>,

    /// Размеры товара
    #[serde(default)]
    pub sizes: Vec<ProductSize>,

    /// Ярлыки карточки
    #[serde(default)]
    pub tags: Vec<CardTag>,

    /// Дата создания карточки (RFC 3339)
    #[serde(rename = "createdAt")]
    pub created_at: String,

    /// Дата обновления карточки (RFC 3339)
    #[serde(default, rename = "updatedAt")]
    pub updated_at: String,
}

/// Фотографии товара
#[derive(Debug, Serialize, Deserialize)]
pub struct Photo {
    /// URL фото 900x1200
    pub big: String,

    /// URL фото 248x328
    pub c246x328: String,

    /// URL фото 516x688
    pub c516x688: String,

    /// URL фото 600x600
    pub square: String,

    /// URL фото 75x100
    pub tm: String,
}

/// Оптовая продажа
#[derive(Debug, Serialize, Deserialize)]
pub struct Wholesale {
    /// Предназначена ли карточка для оптовой продажи
    pub enabled: bool,

    /// Количество единиц товара в упаковке
    pub quantum: u64,
}

/// Габариты товара
#[derive(Debug, Serialize, Deserialize)]
pub struct ProductDim {
    /// Длина, см
    pub length: i32,

    /// Ширина, см
    pub width: i32,

    /// Высота, см
    pub height: i32,

    /// Вес брутто, кг
    #[serde(rename = "weightBrutto")]
    pub weight_brutto: f64,

    /// Признак потенциальной корректности габаритов
    #[serde(rename = "isValid")]
    pub is_valid: bool,
}

/// Характеристика товара
#[derive(Debug, Serialize, Deserialize)]
pub struct Characteristic {
    /// ID характеристики
    pub id: i64,

    /// Название характеристики
    pub name: String,

    /// Значение характеристики (может быть разного типа, поэтому `serde_json::Value`)
    pub value: serde_json::Value,
}

/// Размер товара
#[derive(Debug, Serialize, Deserialize)]
pub struct ProductSize {
    /// Числовой ID размера для артикула WB
    #[serde(rename = "chrtID")]
    pub chrt_id: i64,

    /// Технический размер (например: "0", "XXL", "57")
    #[serde(rename = "techSize")]
    pub tech_size: String,

    /// Российский размер (может отсутствовать)
    #[serde(default, rename = "wbSize")]
    pub wb_size: Option<String>,

    /// Список баркодов товара
    pub skus: Vec<String>,
}

/// Ярлык карточки
#[derive(Debug, Serialize, Deserialize)]
pub struct CardTag {
    /// ID ярлыка
    pub id: i64,

    /// Название ярлыка
    pub name: String,

    /// Цвет ярлыка (например: `D1CFD7` — серый)
    pub color: String,
}

/// Пагинатор списка карточек
#[derive(Debug, Serialize, Deserialize)]
pub struct Cursor {
    /// Дата и время, с которых надо запрашивать следующий список карточек
    #[serde(default, skip_serializing_if = "Option::is_none", rename = "updatedAt")]
    pub updated_at: Option<String>,

    /// Артикул WB, с которого надо запрашивать следующий список
    #[serde(rename = "nmID")]
    pub nm_id: i64,

    /// Количество возвращённых карточек
    pub total: i32,
}

/// Ответ на запрос списка товаров с ценами `/api/v2/list/goods/filter`
#[derive(Debug, Serialize, Deserialize)]
pub struct GoodsListResponse {
    /// Обёртка с данными
    pub data: GoodsData,
}

/// Данные о товарах
#[derive(Debug, Serialize, Deserialize)]
pub struct GoodsData {
    /// Список товаров
    #[serde(default, rename = "listGoods")]
    pub list_goods: Vec<Goods>,
}

/// Информация о товаре
#[derive(Debug, Serialize, Deserialize)]
pub struct Goods {
    /// Артикул WB
    #[serde(rename = "nmID")]
    pub nm_id: i64,

    /// Артикул продавца
    #[serde(rename = "vendorCode")]
    pub vendor_code: String,

    /// Размеры и цены товара
    pub sizes: Vec<SizePrice>,

    /// Валюта (ISO 4217, например: RUB)
    #[serde(rename = "currencyIsoCode4217")]
    pub currency_iso_code4217: String,

    /// Общая скидка, %
    pub discount: i32,

    /// Скидка WB Клуба, %
    #[serde(rename = "clubDiscount")]
    pub club_discount: i32,

    /// Можно ли устанавливать цены отдельно для разных размеров
    #[serde(rename = "editableSizePrice")]
    pub editable_size_price: bool,
}

/// Цена для конкретного размера товара
#[derive(Debug, Serialize, Deserialize)]
pub struct SizePrice {
    /// ID размера (в методах контента это поле chrtID)
    #[serde(rename = "sizeID")]
    pub size_id: i64,

    /// Цена без скидки
    pub price: i64,

    /// Цена со скидкой
    #[serde(rename = "discountedPrice")]
    pub discounted_price: f64,

    /// Цена со скидкой, включая скидку WB Клуба
    #[serde(rename = "clubDiscountedPrice")]
    pub club_discounted_price: f64,

    /// Размер товара (например: "42")
    #[serde(rename = "techSizeName")]
    pub tech_size_name: String,
}
