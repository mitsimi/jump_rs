use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};
use thiserror::Error;

use crate::api::ApiResult;
use crate::error::ApiError;

#[derive(Debug, Error)]
pub enum WebError {
    #[error(transparent)]
    Api(#[from] ApiError),

    #[error("{0}")]
    Form(String),
}

impl WebError {
    pub const fn status_code(&self) -> StatusCode {
        match self {
            Self::Api(err) => err.status_code(),
            Self::Form(_) => StatusCode::BAD_REQUEST,
        }
    }

    pub fn message(&self) -> String {
        self.to_string()
    }
}

impl IntoResponse for WebError {
    fn into_response(self) -> Response {
        let status = self.status_code();
        let body = crate::web::views::error_page(status, &self.message());
        (status, body).into_response()
    }
}

pub type WebResult<T> = Result<T, WebError>;

pub fn form_result<T>(result: Result<T, String>) -> WebResult<T> {
    result.map_err(WebError::Form)
}

pub fn api_result<T>(result: ApiResult<T>) -> WebResult<T> {
    result.map_err(WebError::Api)
}
