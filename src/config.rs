use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct OzonSellerCredentials {
    pub client_id: String,
    pub api_key: String,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct LLmConfig {
    pub base_url: String,
    pub api_key: String,
    pub model: String,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Config {
    pub ozon_seller_credentials: OzonSellerCredentials,
    pub llm_config: LLmConfig,
}
