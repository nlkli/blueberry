use std::collections::BTreeMap;

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
    pub text: String,
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Product {
    pub id: String,
    pub name: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ProductFormatInfo {
    pub id: String,
    pub name: String,
    pub price: String,
    pub desc: String,
    pub attrs: BTreeMap<String, String>,
    pub weight: String,
    pub r#box: String,
}
