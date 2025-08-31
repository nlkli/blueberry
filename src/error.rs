use crate::sellerapi::{OzonSellerApiError, WbSellerApiError};
use reqwest::Error as ReqwestError;
use std::io::Error as StdIoError;
use std::result::Result as StdResult;
use thiserror::Error as ThisError;

pub type Result<T> = StdResult<T, Error>;

#[derive(ThisError, Debug)]
pub enum Error {
    #[error(transparent)]
    Io(#[from] StdIoError),

    #[error(transparent)]
    Reqwest(#[from] ReqwestError),

    #[error(transparent)]
    OzonSellerApi(#[from] OzonSellerApiError),

    #[error(transparent)]
    WbSellerApi(#[from] WbSellerApiError),

    #[error("ProductCtxDataError: {0}.")]
    ProductCtxData(String),

    #[error("Missing required field: {0}.")]
    MissingRequiredField(String),
}
