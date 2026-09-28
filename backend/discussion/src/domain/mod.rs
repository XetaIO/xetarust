//! Business rules of the Discussion context: the [`Comment`] aggregate, its
//! identifiers, its persistence port, the [`CommentThrottle`] anti-flood policy
//! and the [`CommentableArticle`] read model.
//!
//! Framework-free: no SeaORM, no Axum, no serde.

mod comment;
mod comment_repository;
mod comment_throttle;
mod commentable_article;
mod ids;

pub use comment::Comment;
pub use comment_repository::CommentRepository;
pub use comment_throttle::CommentThrottle;
pub use commentable_article::CommentableArticle;
pub use ids::{ArticleId, AuthorId, CommentId};

#[cfg(test)]
pub use comment_repository::MockCommentRepository;
