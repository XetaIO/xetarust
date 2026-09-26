//! Driven ports of the Publishing context towards the outside world.
//! The composition root provides the implementations (anti-corruption layer).

use std::collections::HashMap;

use async_trait::async_trait;
use xetaravel_kernel::AppResult;

use crate::domain::AuthorId;

/// Gives the public names of article authors (owned by another context).
#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait AuthorDirectory: Send + Sync {
    /// Returns the public name of every known author among `ids`.
    /// Unknown authors are simply absent from the map.
    async fn names(&self, ids: &[AuthorId]) -> AppResult<HashMap<AuthorId, String>>;
}
