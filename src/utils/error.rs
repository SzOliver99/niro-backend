use actix_web::{HttpResponse, ResponseError};
use serde::Serialize;
use std::fmt::{Display, Formatter, Result as FmtResult};

#[derive(Debug)]
pub enum ApiError {
    Validation(String),
    NotFound(String),
    Unauthorized(String),
    Forbidden(String),
    Conflict(String),
    Internal,
}

impl Display for ApiError {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match self {
            ApiError::Validation(msg) => write!(f, "validation error: {}", msg),
            ApiError::NotFound(msg) => write!(f, "not found: {}", msg),
            ApiError::Unauthorized(msg) => write!(f, "unauthorized: {}", msg),
            ApiError::Forbidden(msg) => write!(f, "forbidden: {}", msg),
            ApiError::Conflict(msg) => write!(f, "conflict: {}", msg),
            ApiError::Internal => write!(f, "internal server error"),
        }
    }
}

#[derive(Serialize)]
struct ErrorBody {
    error: String,
}

impl ResponseError for ApiError {
    fn error_response(&self) -> HttpResponse {
        match self {
            ApiError::Validation(msg) => {
                HttpResponse::BadRequest().json(ErrorBody { error: msg.clone() })
            }
            ApiError::NotFound(msg) => {
                HttpResponse::NotFound().json(ErrorBody { error: msg.clone() })
            }
            ApiError::Unauthorized(msg) => {
                HttpResponse::Unauthorized().json(ErrorBody { error: msg.clone() })
            }
            ApiError::Forbidden(msg) => {
                HttpResponse::Forbidden().json(ErrorBody { error: msg.clone() })
            }
            ApiError::Conflict(msg) => {
                HttpResponse::Conflict().json(ErrorBody { error: msg.clone() })
            }
            ApiError::Internal => HttpResponse::InternalServerError().json(ErrorBody {
                error: "internal server error".to_string(),
            }),
        }
    }
}

impl From<anyhow::Error> for ApiError {
    fn from(_err: anyhow::Error) -> Self {
        ApiError::Internal
    }
}
