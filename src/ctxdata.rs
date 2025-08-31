use crate::{
    error::{Error, Result},
    sellerapi::{OzonSellerClient, WbSellerClient, ozmodels, wbmodels},
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{collections::BTreeMap, sync::Arc};
use tera::Context;

/// Данные товара, подготовленные для вставки в контекст шаблона.
#[derive(Debug, Serialize, Deserialize)]
pub struct ProductCtxData {
    /// Идентификатор товара (sku для Ozon).
    pub id: String,

    /// Название товара.
    pub name: String,

    /// Цена товара (строкой).
    pub price: String,

    /// Описание товара.
    pub desc: String,

    /// Информация на странице товара (текстом).
    pub info: String,

    /// Информация на странице товара (словарь).
    pub attrs: BTreeMap<String, String>,

    /// Вес упаковки.
    pub weight: String,

    /// Размер упаковки.
    pub r#box: String,
}

impl ProductCtxData {
    /// Формирует контекст товара по `nmid` из Wildberries.
    pub async fn from_wb_product_nmid(cli: Arc<WbSellerClient>, nmid: i64) -> Result<Self> {
        const UNKNOWN: &str = "Unknown";

        let text_search = nmid.to_string();
        let filter = wbmodels::params::Filter {
            with_photo: Some(-1),
            text_search: Some(&text_search),
            ..Default::default()
        };

        let cards_response = cli
            .get_cards_list(Some(&filter), 1)
            .await
            .map_err(|e| Error::ProductCtxData(format!("cards request failed: {e}")))?;

        let mut card =
            cards_response.cards.into_iter().next().ok_or_else(|| {
                Error::ProductCtxData(format!("not found product by nmid {nmid}"))
            })?;

        let name = std::mem::take(&mut card.title);
        let desc = std::mem::take(&mut card.description);

        let mut attrs = BTreeMap::from([
            ("Бренд".to_owned(), std::mem::take(&mut card.brand)),
            (
                "Категория".to_owned(),
                std::mem::take(&mut card.subject_name),
            ),
        ]);

        std::mem::take(&mut card.characteristics)
            .into_iter()
            .for_each(|c| {
                // Безопасно сериализуем значение характеристики.
                let val = serde_json::to_string(&c.value).unwrap_or_default();
                attrs.insert(c.name, val);
            });

        // Пытаемся получить rich-контент по первой фотографии.
        if let Some(first_photo) = card.photos.get(0) {
            if let Some((bucket_path, _)) = first_photo.big.split_once("/images/") {
                let rich_content = WbSellerClient::get_product_rich_content(bucket_path, 1)
                    .await
                    .ok()
                    .and_then(|raw| serde_json::from_str::<Value>(&raw).ok())
                    .and_then(|mut v| {
                        sanitize_wb_rich_content_json(&mut v);
                        serde_json::to_string(&v).ok()
                    })
                    .unwrap_or_else(|| "Empty".to_string());
                attrs.insert("Rich-контент JSON".to_owned(), rich_content);
            }
        }

        let (weight, r#box) = card
            .dimensions
            .map(|dims| {
                (
                    dims.weight_brutto.to_string(),
                    format!(
                        "height: {}, width: {}, length: {}",
                        dims.height, dims.width, dims.length
                    ),
                )
            })
            .unwrap_or_else(|| (UNKNOWN.to_string(), UNKNOWN.to_string()));

        let price = cli
            .get_products_price(1, None, Some(nmid))
            .await
            .ok()
            .and_then(|r| r.data.list_goods.into_iter().next())
            .and_then(|g| g.sizes.into_iter().next())
            .map(|sp| sp.price.to_string())
            .unwrap_or_else(|| UNKNOWN.to_string());

        let info = join_attrs_as_info(&attrs);

        Ok(Self {
            id: text_search,
            name,
            price,
            desc,
            info,
            attrs,
            weight,
            r#box,
        })
    }

    /// Формирует контекст товара по `sku` из Ozon.
    pub async fn from_ozon_product_sku(cli: Arc<OzonSellerClient>, sku: i64) -> Result<Self> {
        let tmp = [sku];
        let filter = ozmodels::params::Filter {
            sku: Some(&tmp[..]),
            ..Default::default()
        };

        let mut product = cli
            .get_product_info_list(&filter)
            .await
            .map_err(|e| Error::ProductCtxData(format!("product info request failed: {e}")))?
            .items
            .into_iter()
            .next()
            .ok_or_else(|| Error::MissingRequiredField("product_info".into()))?;

        let product_id = product.id;
        let name = std::mem::take(&mut product.name);
        let price = format!("{} {}", product.marketing_price, product.currency_code);
        drop(product);

        // Запускаем запрос описания параллельно.
        let description_task = {
            let cli = cli.clone();
            tokio::spawn(async move { cli.get_product_info_description(product_id).await })
        };

        let attrs_v4 = cli
            .get_product_attributes_v4(Some(&filter), 1, None)
            .await
            .map_err(|e| Error::ProductCtxData(format!("product attributes request failed: {e}")))?
            .result
            .into_iter()
            .next()
            .ok_or_else(|| Error::MissingRequiredField("product_attributes".into()))?;

        let weight = format!("{}{}", attrs_v4.weight, attrs_v4.weight_unit);

        let unit = attrs_v4.dimension_unit.as_str();
        let r#box = format!(
            "height: {}{}, width: {}{}, depth: {}{}",
            attrs_v4.height, unit, attrs_v4.width, unit, attrs_v4.depth, unit
        );

        let desc_category_id = attrs_v4.description_category_id;
        let type_id = attrs_v4.type_id;

        let category_attrs = cli
            .get_attributes(desc_category_id, None, type_id)
            .await
            .map_err(|e| Error::ProductCtxData(format!("product attributes request failed: {e}")))?
            .result;

        let desc = description_task
            .await
            .map_err(|e| Error::ProductCtxData(format!("join description task failed: {e}")))?? // двойной ? — из JoinHandle и из Result внутри
            .result
            .description;

        let product_attrs = attrs_v4.attributes;
        let mut attrs = BTreeMap::new();

        // Флаги для единоразового пропуска дублей.
        let (mut skipped_desc, mut processed_rich) = (false, false);

        for attr in &product_attrs {
            if let Some(cat) = category_attrs.iter().find(|v| v.id == attr.id) {
                let mut value = attr
                    .values
                    .iter()
                    .map(|v| v.value.as_str().trim())
                    .collect::<Vec<_>>()
                    .join(", ");

                // Пропускаем дублирующееся текстовое описание.
                if !skipped_desc && value == desc {
                    skipped_desc = true;
                    continue;
                }

                // Нормализуем Rich-* JSON при первом попадании.
                if !processed_rich && cat.name.starts_with("Rich-") && cat.name.ends_with(" JSON") {
                    value = match serde_json::from_str::<Value>(&value) {
                        Ok(mut v) => {
                            sanitize_ozon_rich_content_json(&mut v);
                            serde_json::to_string(&v).unwrap_or_default()
                        }
                        Err(_) => "Empty".to_string(),
                    };
                    processed_rich = true;
                }

                attrs.insert(cat.name.clone(), value);
            }
        }

        let info = join_attrs_as_info(&attrs);

        Ok(Self {
            id: sku.to_string(),
            name,
            price,
            desc,
            info,
            attrs,
            weight,
            r#box,
        })
    }

    /// Вставляет данные `product` в контекст шаблона Tera.
    #[inline]
    pub fn insert_to_ctx(&self, ctx: &mut Context) {
        ctx.insert("product", self);
    }
}

/// Чёрный список ключей для Rich-контента Ozon.
const OZON_RICH_CONTENT_BLACKLIST_KEYS: [&str; 14] = [
    "img",
    "imgLink",
    "size",
    "width",
    "height",
    "align",
    "contentAlign",
    "color",
    "src",
    "sources",
    "srcMobile",
    "reverse",
    "version",
    "id",
];

/// Чёрный список ключей для Rich-контента Wildberries.
const WB_RICH_CONTENT_BLACKLIST_KEYS: [&str; 5] = ["style", "image", "src", "preview", "version"];

/// Рекурсивно удаляет лишние поля и мусор из JSON Rich-контента.
fn sanitize_rich_content_json(value: &mut Value, blacklist: &[&str]) {
    match value {
        Value::Array(arr) => {
            // Удаляем пустые/короткие строковые элементы-мусор.
            arr.retain(|i| !i.as_str().map(|s| s.len() <= 1).unwrap_or(false));
            for item in arr {
                sanitize_rich_content_json(item, blacklist);
            }
        }
        Value::Object(obj) => {
            for key in blacklist {
                obj.remove(*key);
            }
            for v in obj.values_mut() {
                sanitize_rich_content_json(v, blacklist);
            }
        }
        _ => {}
    }
}

/// Очищает JSON Rich-контента Ozon.
fn sanitize_ozon_rich_content_json(value: &mut Value) {
    sanitize_rich_content_json(value, &OZON_RICH_CONTENT_BLACKLIST_KEYS);
}

/// Очищает JSON Rich-контента Wildberries.
fn sanitize_wb_rich_content_json(value: &mut Value) {
    sanitize_rich_content_json(value, &WB_RICH_CONTENT_BLACKLIST_KEYS);
}

/// Преобразует словарь атрибутов в удобочитаемую строку для `info`.
fn join_attrs_as_info(attrs: &BTreeMap<String, String>) -> String {
    attrs
        .iter()
        .map(|(k, v)| format!("{}: {};", k, v.trim()))
        .collect::<Vec<_>>()
        .join("\n")
}
