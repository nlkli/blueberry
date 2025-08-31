use std::sync::Arc;

use crate::sellerapi::{OzonSellerClient, WbSellerClient};

pub enum SellerClient {
    Ozon(Arc<OzonSellerClient>),
    Wb(Arc<WbSellerClient>),
}
