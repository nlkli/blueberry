use serde::{Deserialize, Serialize};

pub const DEFAULT_AUTHOR_NAME: &str = "User";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewQuestion {
    pub id: String,
    pub product_id: String,
    pub author_name: String,
    pub text: String,
    pub published_at: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewReview {
    pub id: String,
    pub product_id: String,
    pub author_name: String,
    pub content: String,
    pub score: f32,
    pub photos_amount: u16,
    pub videos_amount: u16,
    pub published_at: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NewFeedback {
    Review(NewReview),
    Question(NewQuestion),
}

pub const WB_PLACE_SYMBOL: &str = "wb";
pub const OZON_PLACE_SYMBOL: &str = "oz";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Product {
    pub id: String,
    pub place_symbol: &'static str,
    pub name: String,
}
