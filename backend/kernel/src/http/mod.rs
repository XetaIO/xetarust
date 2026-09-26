//! HTTP plumbing shared by the Axum adapters of every context.

mod error;
mod extractors;

pub use error::{ApiError, ApiResult, ErrorBody, ErrorCode};
pub use extractors::{AdminPrincipal, CurrentPrincipal, JsonBody, PathParam, QueryParams};
