//! Public contract of the Publishing context for the other contexts.

use async_trait::async_trait;
use uuid::Uuid;
use xetaravel_kernel::AppResult;

/// Minimal public reference to a published article.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PublishedArticleRef {
    pub id: Uuid,
    /// Whether readers can post new comments on it.
    pub comments_enabled: bool,
}

/// Read-only access to the publicly visible articles.
#[async_trait]
pub trait PublishedArticles: Send + Sync {
    /// Returns the published article using `slug`, or `None` when it does
    /// not exist, is a draft or the slug is malformed.
    async fn find_published(&self, slug: &str) -> AppResult<Option<PublishedArticleRef>>;
}
