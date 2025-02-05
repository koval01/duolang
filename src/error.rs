use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    extract::rejection::QueryRejection,
    Json,
};
use axum::extract::rejection::PathRejection;
use bb8::RunError;
use prisma_client_rust::QueryError;
use redis::RedisError;

use tracing::debug;
use crate::response::ApiResponse;

#[allow(dead_code)]
#[derive(Debug)]
pub enum ApiError {
    BadRequest,
    Unauthorized,
    Forbidden,
    NotFound(String),
    Conflict(String),
    Timeout,
    Database(QueryError),
    Redis(RunError<RedisError>),
    InternalServerError,
    Custom(StatusCode, String),
}

impl ApiError {
    pub fn status_code(&self) -> StatusCode {
        match self {
            ApiError::BadRequest => StatusCode::BAD_REQUEST,
            ApiError::Unauthorized => StatusCode::UNAUTHORIZED,
            ApiError::Forbidden => StatusCode::FORBIDDEN,
            ApiError::NotFound(_) => StatusCode::NOT_FOUND,
            ApiError::Conflict(_) => StatusCode::CONFLICT,
            ApiError::Timeout => StatusCode::GATEWAY_TIMEOUT,
            ApiError::Database(_) => StatusCode::INTERNAL_SERVER_ERROR,
            ApiError::Redis(_) => StatusCode::INTERNAL_SERVER_ERROR,
            ApiError::InternalServerError => StatusCode::INTERNAL_SERVER_ERROR,
            ApiError::Custom(code, _) => *code,
        }
    }

    pub fn message(&self) -> String {
        match self {
            ApiError::BadRequest => "bad request".to_string(),
            ApiError::Unauthorized => "unauthorised".to_string(),
            ApiError::Forbidden => "forbidden".to_string(),
            ApiError::NotFound(error) => if error.is_empty() { "not found".to_string() } else { error.clone() },
            ApiError::Conflict(error) => if error.is_empty() { "conflict".to_string() } else { error.clone() },
            ApiError::Timeout => "request timed out".to_string(),
            ApiError::Database(error) => format!("database error: {}", error),
            ApiError::Redis(error) => format!("redis error: {}", error),
            ApiError::InternalServerError => "internal error".to_string(),
            ApiError::Custom(_, message) => message.clone(),
        }
    }
}

impl From<QueryError> for ApiError {
    fn from(error: QueryError) -> Self {
        debug!("{:#?}", error);
        ApiError::Database(error)
    }
}

impl From<RedisError> for ApiError {
    fn from(error: RedisError) -> Self {
        debug!("{:#?}", error);
        ApiError::Redis(RunError::User(error))
    }
}

impl From<serde_json::Error> for ApiError {
    fn from(error: serde_json::Error) -> Self {
        debug!("{:#?}", error);
        ApiError::InternalServerError
    }
}

impl From<QueryRejection> for ApiError {
    fn from(error: QueryRejection) -> Self {
        debug!("{:#?}", error);
        ApiError::Custom(StatusCode::BAD_REQUEST, error.body_text()) 
    }
}

impl From<PathRejection> for ApiError {
    fn from(error: PathRejection) -> Self {
        debug!("{:#?}", error);
        ApiError::Custom(StatusCode::BAD_REQUEST, error.body_text())
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let status = self.status_code();
        let message = self.message();
        let response = ApiResponse::<()>::error(&message, status);
        (status, Json(response)).into_response()
    }
}
