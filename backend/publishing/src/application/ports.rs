//! Driven ports of the Publishing context towards the outside world.
//! The composition root provides the implementations (anti-corruption layer).

use std::collections::HashMap;

use async_trait::async_trait;
use xetaravel_kernel::AppResult;

use crate::domain::{AuthorId, CoverImage};

/// Gives the public names of article authors (owned by another context).
#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait AuthorDirectory: Send + Sync {
    /// Returns the public name of every known author among `ids`.
    /// Unknown authors are simply absent from the map.
    async fn names(&self, ids: &[AuthorId]) -> AppResult<HashMap<AuthorId, String>>;
}

/// Stores the binary files of the article cover images.
#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait CoverStorage: Send + Sync {
    /// Writes the bytes of `cover`, replacing any file with the same name.
    async fn store(&self, cover: &CoverImage, bytes: &[u8]) -> AppResult<()>;

    /// Reads the bytes of `cover`; `None` when the file does not exist.
    async fn load(&self, cover: &CoverImage) -> AppResult<Option<Vec<u8>>>;

    /// Deletes the file of `cover`. Deleting a missing file is not an error.
    async fn delete(&self, cover: &CoverImage) -> AppResult<()>;
}
