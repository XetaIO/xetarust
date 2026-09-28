//! Driven ports of the Discussion context towards the other contexts.
//! The composition root provides the implementations (anti-corruption layer).

use std::collections::HashMap;

use async_trait::async_trait;
use xetaravel_kernel::AppResult;

use crate::domain::{AuthorId, CommentableArticle};

/// Tells which articles can be commented (owned by another context).
#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait ArticleCatalog: Send + Sync {
    /// Returns the published article using `slug`, or `None` when it does
    /// not exist, is a draft or the slug is malformed.
    async fn published_article(&self, slug: &str) -> AppResult<Option<CommentableArticle>>;
}

/// Gives the public names of comment authors (owned by another context).
#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait AuthorDirectory: Send + Sync {
    /// Returns the public name of every known author among `ids`.
    /// Unknown authors are simply absent from the map.
    async fn names(&self, ids: &[AuthorId]) -> AppResult<HashMap<AuthorId, String>>;
}
