use crate::sellerapi::OzonSellerClient;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, sync::Mutex, time::Instant};
use tera::Context;

// #[derive(Debug, Default, Serialize, Deserialize)]
// pub struct Config {
//     pub seller_client: OzonSellerClient,
// }

#[derive(Debug)]
pub struct AppState {
    // pub cfg: Mutex<Config>,
    pub ctx_cache: Mutex<HashMap<String, (Context, Instant)>>,
}
