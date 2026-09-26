//! Business rules of the Discussion context: the [`Comment`] aggregate, its
//! identifiers and its persistence port.
//!
//! Framework-free: no SeaORM, no Axum, no serde.

mod comment;
mod comment_repository;
mod ids;

pub use comment::Comment;
pub use comment_repository::CommentRepository;
pub use ids::{ArticleId, AuthorId, CommentId};

#[cfg(test)]
pub use comment_repository::MockCommentRepository;
