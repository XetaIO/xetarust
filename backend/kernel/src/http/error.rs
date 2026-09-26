//! JSON error responses shared by every HTTP adapter.

use crate::error::{AppError, FieldErrors};
use axum::Json;
use axum::extract::rejection::{JsonRejection, PathRejection, QueryRejection};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde::Serialize;
use ts_rs::TS;

/// Stable machine-readable error codes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export, export_to = "shared/")]
pub enum ErrorCode {
    ValidationError,
    NotFound,
    Unauthorized,
    Forbidden,
    Conflict,
    InternalError,
}

/// JSON body of every error response.
#[derive(Debug, Serialize, TS)]
#[ts(export, export_to = "shared/")]
pub struct ErrorBody {
    pub error: ErrorCode,
    /// Human-readable message.
    pub message: String,
    /// Per-field messages, only for validation errors.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub fields: Option<FieldErrors>,
}

/// HTTP view of an [`AppError`].
#[derive(Debug)]
pub struct ApiError(pub AppError);

impl From<AppError> for ApiError {
    /// Wraps an application error.
    fn from(error: AppError) -> Self {
        Self(error)
    }
}

impl From<JsonRejection> for ApiError {
    /// Turns a malformed JSON body into a validation error.
    fn from(rejection: JsonRejection) -> Self {
        Self(AppError::Validation(FieldErrors::from([(
            "body".to_owned(),
            vec![rejection.body_text()],
        )])))
    }
}

impl From<PathRejection> for ApiError {
    /// Turns an unparsable path parameter (e.g. a bad UUID) into a 404.
    fn from(_: PathRejection) -> Self {
        Self(AppError::NotFound("resource not found".into()))
    }
}

impl From<QueryRejection> for ApiError {
    /// Turns an invalid query string into a validation error.
    fn from(rejection: QueryRejection) -> Self {
        Self(AppError::Validation(FieldErrors::from([(
            "query".to_owned(),
            vec![rejection.body_text()],
        )])))
    }
}

impl ApiError {
    /// Returns the HTTP status and the stable code of the error.
    fn status_and_code(&self) -> (StatusCode, ErrorCode) {
        match &self.0 {
            AppError::Validation(_) => {
                (StatusCode::UNPROCESSABLE_ENTITY, ErrorCode::ValidationError)
            }
            AppError::NotFound(_) => (StatusCode::NOT_FOUND, ErrorCode::NotFound),
            AppError::Unauthorized(_) => (StatusCode::UNAUTHORIZED, ErrorCode::Unauthorized),
            AppError::Forbidden(_) => (StatusCode::FORBIDDEN, ErrorCode::Forbidden),
            AppError::Conflict(_) => (StatusCode::CONFLICT, ErrorCode::Conflict),
            AppError::Internal(_) => (StatusCode::INTERNAL_SERVER_ERROR, ErrorCode::InternalError),
        }
    }
}

impl IntoResponse for ApiError {
    /// Serializes the error as `{ error, message, fields? }`. Internal details
    /// are logged, never sent to the client.
    fn into_response(self) -> Response {
        let (status, code) = self.status_and_code();
        let (message, fields) = match self.0 {
            AppError::Validation(fields) => ("the given data is invalid".to_owned(), Some(fields)),
            AppError::Internal(details) => {
                tracing::error!(%details, "internal error");
                ("an unexpected error occurred".to_owned(), None)
            }
            other => (other.to_string(), None),
        };

        (
            status,
            Json(ErrorBody {
                error: code,
                message,
                fields,
            }),
        )
            .into_response()
    }
}

/// Result type returned by every handler.
pub type ApiResult<T> = Result<T, ApiError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn maps_every_error_to_its_status() {
        let cases = [
            (AppError::field("a", "b"), StatusCode::UNPROCESSABLE_ENTITY),
            (AppError::NotFound("x".into()), StatusCode::NOT_FOUND),
            (AppError::Unauthorized("x".into()), StatusCode::UNAUTHORIZED),
            (AppError::Forbidden("x".into()), StatusCode::FORBIDDEN),
            (AppError::Conflict("x".into()), StatusCode::CONFLICT),
            (
                AppError::Internal("x".into()),
                StatusCode::INTERNAL_SERVER_ERROR,
            ),
        ];
        for (error, status) in cases {
            assert_eq!(ApiError(error).into_response().status(), status);
        }
    }
}
