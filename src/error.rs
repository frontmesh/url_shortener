use serde_json::json;
use thiserror::Error;
use worker::kv::KvError;
use worker::Response;
use worker::wasm_bindgen::JsValue;
use crate::error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("invalid request: {0}")]
    InvalidRequest(String),
    #[error("forbidden")]
    Forbidden,
    #[error("not found: {0}")]
    NotFound(String),
    #[error("internal error: worker")]
    WorkerError(#[from] worker::Error),
    #[error("internal error: kv")]
    KvError(#[from] worker::kv::KvError),
    #[error("internal error: serde_json")]
    SerdeJsonError(#[from] serde_json::Error),
    #[error("internal error")]
    InternalError(String),

}

impl Error {
    fn status_code(&self) -> u16 {
        match self {
            Error::InvalidRequest(_) => 400,
            Error::Forbidden => 403,
            Error::NotFound(_) => 404,
            Error::WorkerError(_) => 500,
            Error::KvError(_) => 500,
            Error::SerdeJsonError(_) => 500,
            Error::InternalError(_) => 500,
        }
    }

    fn json(&self) -> serde_json::Value {
        json!({
            "error": self.to_string(),
            "status_code": self.status_code(),
        })
    }
}

impl From<Error> for Response {
    fn from(err: Error) -> Self {
        let error_code = err.status_code();
        let resp = Response::from_json(&err.json())
            .unwrap()
            .with_status(error_code);
        resp
    }
}


impl From<error::Error> for KvError {
    fn from(error: error::Error) -> Self {
        KvError::from(JsValue::from_str(&error.to_string()))
    }
}

pub type Result<T> = std::result::Result<T, Error>;