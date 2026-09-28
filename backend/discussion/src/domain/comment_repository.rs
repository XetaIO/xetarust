use async_trait::async_trait;
use xetaravel_kernel::DomainResult;

use super::{ArticleId, AuthorId, Comment, CommentId};

/// Persistence port of the [`Comment`] aggregate.
#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait CommentRepository: Send + Sync {
    /// Finds a comment by id.
    async fn find_by_id(&self, id: CommentId) -> DomainResult<Option<Comment>>;

    /// Lists the comments of an article, oldest first.
    async fn list_by_article(&self, article_id: ArticleId) -> DomainResult<Vec<Comment>>;

    /// Inserts a new comment.
    async fn create(&self, comment: &Comment) -> DomainResult<()>;

    /// Deletes a comment. Returns `false` when it did not exist.
    async fn delete(&self, id: CommentId) -> DomainResult<bool>;

    /// Deletes every comment written by `author_id`. Returns how many were deleted.
    async fn delete_by_author(&self, author_id: AuthorId) -> DomainResult<u64>;
}
