use serde::{Deserialize, Serialize};

/// Корневой объект ответа API `/v1/description-category/tree`
#[derive(Debug, Serialize, Deserialize)]
pub struct CategoryTreeResponse {
    /// Список категорий верхнего уровня
    pub result: Vec<CategoryNode>,
}

/// Узел дерева категорий или типов
#[derive(Debug, Serialize, Deserialize)]
pub struct CategoryNode {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    /// Идентификатор категории (присутствует для категорий)
    pub description_category_id: Option<i64>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    /// Название категории (присутствует для категорий)
    pub category_name: Option<String>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    /// Идентификатор типа товара (присутствует для типов)
    pub type_id: Option<i64>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    /// Название типа товара (присутствует для типов)
    pub type_name: Option<String>,

    /// true, если в категории/типе нельзя создавать товары
    pub disabled: bool,

    /// Подкатегории или вложенные типы (рекурсивная структура)
    pub children: Vec<CategoryNode>,
}

/// Корневой объект ответа API `/v1/description-category/attribute`
#[derive(Debug, Serialize, Deserialize)]
pub struct CategoryAttributesResponse {
    /// Результаты запроса — список характеристик
    pub result: Vec<CategoryAttribute>,
}

/// Характеристика категории/типа товара
#[derive(Debug, Serialize, Deserialize)]
pub struct CategoryAttribute {
    /// Идентификатор характеристики
    pub id: i64,

    /// Идентификатор комплексного атрибута
    pub attribute_complex_id: i64,

    /// Название характеристики
    pub name: String,

    /// Описание характеристики
    pub description: String,

    /// Тип характеристики (например: string, number и т.д.)
    pub r#type: String,

    /// Признак, что характеристика может содержать несколько значений
    pub is_collection: bool,

    /// Признак обязательной характеристики
    pub is_required: bool,

    /// Признак аспектного атрибута (например: цвет, размер)
    pub is_aspect: bool,

    /// Максимальное количество значений для атрибута
    pub max_value_count: i64,

    /// Название группы характеристик
    pub group_name: String,

    /// Идентификатор группы характеристик
    pub group_id: i64,

    /// Идентификатор словаря (0, если нет словаря)
    pub dictionary_id: i64,

    /// Признак, что значения словаря зависят от категории
    pub category_dependent: bool,

    /// Признак, что комплексная характеристика может содержать несколько значений
    pub complex_is_collection: bool,
}
