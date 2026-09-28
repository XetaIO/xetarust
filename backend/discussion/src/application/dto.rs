//! Data Transfer Objects of the Discussion context (public API contract).
//!
//! TypeScript bindings are generated in `frontend/src/types/api/discussion/`.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use ts_rs::TS;
use uuid::Uuid;
use validator::Validate;
use xetaravel_kernel::dto::AuthorDto;

use crate::domain::Comment;

/// A comment with its author.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(export, export_to = "discussion/")]
pub struct CommentDto {
    pub id: Uuid,
    pub article_id: Uuid,
    pub author: AuthorDto,
    pub content: String,
    pub created_at: DateTime<Utc>,
}

impl CommentDto {
    /// Builds the DTO from a domain comment and the public name of its author.
    pub fn new(comment: &Comment, author_name: String) -> Self {
        Self {
            id: comment.id.as_uuid(),
            article_id: comment.article_id.as_uuid(),
            author: AuthorDto {
                id: comment.author_id.as_uuid(),
                username: author_name,
            },
            content: comment.content.clone(),
            created_at: comment.created_at,
        }
    }
}

/// Body of `POST /api/articles/{slug}/comments`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Validate, TS)]
#[ts(export, export_to = "discussion/")]
pub struct CreateCommentRequest {
    #[validate(length(
        min = 2,
        max = 5000,
        message = "must contain between 2 and 5000 characters"
    ))]
    pub content: String,
}

/// Returned by `DELETE /api/admin/users/{id}/comments`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(export, export_to = "discussion/")]
pub struct DeletedCommentsDto {
    /// Number of comments deleted.
    pub deleted: u64,
}
