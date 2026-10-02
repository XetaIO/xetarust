//! SeaORM helpers shared by the persistence adapters of every context.

use sea_orm::{ConnectOptions, Database, DatabaseConnection, DbErr, RuntimeErr, SqlErr};

use crate::error::DomainError;

/// SQLSTATE `restrict_violation`: since PostgreSQL 18, deleting a row still
/// referenced through an `ON DELETE RESTRICT` foreign key raises it instead of
/// `foreign_key_violation` (unknown to [`SqlErr`]).
const RESTRICT_VIOLATION: &str = "23001";

/// Opens a connection pool to PostgreSQL.
pub async fn connect(database_url: &str) -> Result<DatabaseConnection, DbErr> {
    let mut options = ConnectOptions::new(database_url);
    options.max_connections(20).sqlx_logging(false);
    Database::connect(options).await
}

/// Translates a SeaORM error into a domain error.
pub fn db_error(error: DbErr) -> DomainError {
    match error.sql_err() {
        Some(SqlErr::UniqueConstraintViolation(_)) => {
            DomainError::Conflict("this resource already exists".into())
        }
        Some(SqlErr::ForeignKeyConstraintViolation(_)) => referenced(),
        _ if is_restrict_violation(&error) => referenced(),
        _ => match error {
            DbErr::RecordNotUpdated | DbErr::RecordNotFound(_) => DomainError::NotFound("record"),
            other => DomainError::Repository(other.to_string()),
        },
    }
}

/// Conflict raised when a write would break a foreign key.
fn referenced() -> DomainError {
    DomainError::Conflict("this resource is referenced by another one".into())
}

/// Tells whether `error` is a PostgreSQL [`RESTRICT_VIOLATION`].
fn is_restrict_violation(error: &DbErr) -> bool {
    match error {
        DbErr::Exec(RuntimeErr::SqlxError(error)) | DbErr::Query(RuntimeErr::SqlxError(error)) => {
            error
                .as_database_error()
                .and_then(|error| error.code())
                .is_some_and(|code| code == RESTRICT_VIOLATION)
        }
        _ => false,
    }
}

/// Wraps a validation failure on stored data into a repository error: a stored
/// row that no longer respects the business rules means the data is corrupted.
pub fn corrupted(table: &str, error: DomainError) -> DomainError {
    DomainError::Repository(format!("corrupted row in {table}: {error}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn maps_missing_records_and_generic_failures() {
        assert_eq!(
            db_error(DbErr::RecordNotUpdated),
            DomainError::NotFound("record")
        );
        assert!(matches!(
            db_error(DbErr::Custom("boom".into())),
            DomainError::Repository(_)
        ));
    }

    #[test]
    fn corrupted_rows_are_repository_errors() {
        let error = corrupted("users", DomainError::validation("email", "bad"));
        assert!(matches!(error, DomainError::Repository(m) if m.contains("users")));
    }
}
