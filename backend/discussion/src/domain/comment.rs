use chrono::{DateTime, Utc};
use xetaravel_kernel::DomainResult;
use xetaravel_kernel::text::validate_text;

use super::{ArticleId, AuthorId, CommentId};

/// Minimum length of a comment.
const CONTENT_MIN: usize = 2;
/// Maximum length of a comment.
const CONTENT_MAX: usize = 5_000;

/// A reader comment attached to an article.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Comment {
    pub id: CommentId,
    pub article_id: ArticleId,
    pub author_id: AuthorId,
    pub content: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Comment {
    /// Posts a new comment on an article.
    pub fn post(
        article_id: ArticleId,
        author_id: AuthorId,
        content: &str,
        now: DateTime<Utc>,
    ) -> DomainResult<Self> {
        Ok(Self {
            id: CommentId::generate(),
            article_id,
            author_id,
            content: validate_text("content", content, CONTENT_MIN, CONTENT_MAX)?,
            created_at: now,
            updated_at: now,
        })
    }

    /// Only the author of a comment or an admin may delete it.
    pub fn can_be_deleted_by(&self, actor_id: AuthorId, actor_is_admin: bool) -> bool {
        actor_is_admin || self.author_id == actor_id
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn post_trims_content() {
        let comment = Comment::post(
            ArticleId::generate(),
            AuthorId::generate(),
            "  Nice article!  ",
            Utc::now(),
        )
        .unwrap();
        assert_eq!(comment.content, "Nice article!");
    }

    #[test]
    fn post_rejects_too_short_content() {
        assert!(
            Comment::post(
                ArticleId::generate(),
                AuthorId::generate(),
                " a ",
                Utc::now()
            )
            .is_err()
        );
    }

    #[test]
    fn deletion_rights() {
        let author = AuthorId::generate();
        let comment = Comment::post(ArticleId::generate(), author, "Hello", Utc::now()).unwrap();

        assert!(comment.can_be_deleted_by(author, false));
        assert!(comment.can_be_deleted_by(AuthorId::generate(), true));
        assert!(!comment.can_be_deleted_by(AuthorId::generate(), false));
    }
}
