//! HTTP plumbing shared by the Axum adapters of every context.

mod error;
mod extractors;

pub use error::{ApiError, ApiResult, ErrorBody, ErrorCode};
pub use extractors::{
    AdminPrincipal, ClientIp, CurrentPrincipal, JsonBody, PathParam, QueryParams,
};
