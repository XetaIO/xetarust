//! # Xetaravel shared kernel
//!
//! The deliberately small set of concepts every bounded context (Identity,
//! Publishing, Discussion) agrees on: error types, pagination, text
//! validation, the clock port, the authenticated [`Principal`] and a few DTOs.
//!
//! Optional features add the technical plumbing shared by the adapters:
//! - `http`: `ApiError`, JSON-aware extractors and principal extractors (Axum);
//! - `persistence`: SeaORM connection and error translation.
//!
//! The kernel never depends on a bounded context.

pub mod clock;
pub mod dto;
pub mod error;
mod ids;
pub mod pagination;
pub mod principal;
pub mod text;

#[cfg(feature = "http")]
pub mod http;
#[cfg(feature = "persistence")]
pub mod persistence;

pub use clock::{Clock, FixedClock, SystemClock};
pub use error::{AppError, AppResult, DomainError, DomainResult, FieldErrors};
pub use principal::{Principal, PrincipalResolver};

/// Re-exported so [`define_id!`] works without extra imports at the call site.
#[doc(hidden)]
pub use uuid;
