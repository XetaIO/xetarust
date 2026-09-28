//! Read model of an article as seen by Discussion: something readers may
//! comment on, owned by another context.

use xetaravel_kernel::{DomainError, DomainResult};

use super::ArticleId;

/// A published article that comments can be attached to.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CommentableArticle {
    pub id: ArticleId,
    /// `false` when the article no longer accepts new comments.
    pub comments_open: bool,
}

impl CommentableArticle {
    /// Fails with [`DomainError::Forbidden`] when new comments are refused.
    /// Existing comments stay readable either way.
    pub fn ensure_open(&self) -> DomainResult<()> {
        if self.comments_open {
            Ok(())
        } else {
            Err(DomainError::Forbidden(
                "comments are closed on this article".into(),
            ))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Builds an article whose comments are open (or not).
    fn article(comments_open: bool) -> CommentableArticle {
        CommentableArticle {
            id: ArticleId::generate(),
            comments_open,
        }
    }

    #[test]
    fn open_articles_accept_comments() {
        assert_eq!(article(true).ensure_open(), Ok(()));
    }

    #[test]
    fn closed_articles_refuse_comments() {
        assert_eq!(
            article(false).ensure_open(),
            Err(DomainError::Forbidden(
                "comments are closed on this article".into()
            ))
        );
    }
}
