//! Error types shared by every bounded context.
//!
//! - [`DomainError`] is raised by entities, value objects and repositories.
//! - [`AppError`] is returned by the use cases; adapters (HTTP, CLI...) map it
//!   to their own protocol.

use std::collections::BTreeMap;

use thiserror::Error;
use validator::ValidationErrors;

/// Result alias used across the domain layers.
pub type DomainResult<T> = Result<T, DomainError>;

/// Every failure a domain can express.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum DomainError {
    /// A business invariant was violated by the given field.
    #[error("invalid {field}: {message}")]
    Validation {
        field: &'static str,
        message: String,
    },

    /// The requested resource does not exist.
    #[error("{0} not found")]
    NotFound(&'static str),

    /// The operation would break a uniqueness rule.
    #[error("{0}")]
    Conflict(String),

    /// The actor is not allowed to perform the operation.
    #[error("{0}")]
    Forbidden(String),

    /// A storage adapter failed (connection lost, unexpected SQL error...).
    #[error("repository failure: {0}")]
    Repository(String),
}

impl DomainError {
    /// Builds a [`DomainError::Validation`] for the given field.
    pub fn validation(field: &'static str, message: impl Into<String>) -> Self {
        Self::Validation {
            field,
            message: message.into(),
        }
    }
}

/// Result alias used by every use case.
pub type AppResult<T> = Result<T, AppError>;

/// Validation messages grouped by field name.
pub type FieldErrors = BTreeMap<String, Vec<String>>;

/// Errors returned by the use cases.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum AppError {
    /// The input is invalid; carries the messages of every faulty field.
    #[error("the given data is invalid")]
    Validation(FieldErrors),

    /// The requested resource does not exist.
    #[error("{0}")]
    NotFound(String),

    /// The caller is not authenticated (or the credentials are wrong).
    #[error("{0}")]
    Unauthorized(String),

    /// The caller is authenticated but not allowed to perform the operation.
    #[error("{0}")]
    Forbidden(String),

    /// The operation conflicts with the current state.
    #[error("{0}")]
    Conflict(String),

    /// An unexpected technical failure.
    #[error("internal error: {0}")]
    Internal(String),
}

impl AppError {
    /// Builds a validation error holding a single field message.
    pub fn field(field: impl Into<String>, message: impl Into<String>) -> Self {
        Self::Validation(BTreeMap::from([(field.into(), vec![message.into()])]))
    }
}

impl From<DomainError> for AppError {
    /// Translates a domain failure into its application counterpart.
    fn from(error: DomainError) -> Self {
        match error {
            DomainError::Validation { field, message } => Self::field(field, message),
            DomainError::NotFound(resource) => Self::NotFound(format!("{resource} not found")),
            DomainError::Conflict(message) => Self::Conflict(message),
            DomainError::Forbidden(message) => Self::Forbidden(message),
            DomainError::Repository(message) => Self::Internal(message),
        }
    }
}

impl From<ValidationErrors> for AppError {
    /// Flattens `validator` errors into a field -> messages map.
    fn from(errors: ValidationErrors) -> Self {
        let fields = errors
            .field_errors()
            .into_iter()
            .map(|(field, errors)| {
                let messages = errors
                    .iter()
                    .map(|e| e.message.as_ref().unwrap_or(&e.code).to_string())
                    .collect();
                (field.to_string(), messages)
            })
            .collect();

        Self::Validation(fields)
    }
}

#[cfg(test)]
mod tests {
    use validator::Validate;

    use super::*;

    #[derive(Validate)]
    struct Input {
        #[validate(length(min = 3, message = "is too short"))]
        name: String,
        #[validate(email)]
        email: String,
    }

    #[test]
    fn converts_validator_errors_per_field() {
        let input = Input {
            name: "a".into(),
            email: "nope".into(),
        };
        let AppError::Validation(fields) = AppError::from(input.validate().unwrap_err()) else {
            panic!("expected a validation error");
        };
        assert_eq!(fields["name"], vec!["is too short"]);
        assert_eq!(fields["email"], vec!["email"]);
    }

    #[test]
    fn converts_domain_errors() {
        assert_eq!(
            AppError::from(DomainError::validation("slug", "bad")),
            AppError::field("slug", "bad")
        );
        assert_eq!(
            AppError::from(DomainError::NotFound("article")),
            AppError::NotFound("article not found".into())
        );
        assert!(matches!(
            AppError::from(DomainError::Repository("db down".into())),
            AppError::Internal(_)
        ));
    }
}
