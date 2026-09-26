//! Use cases of the Discussion context. Each use case is a struct holding its
//! dependencies as `Arc<dyn Port>` and exposing a single `execute` method.

mod delete_comment;
mod list_comments;
mod post_comment;

pub use delete_comment::DeleteComment;
pub use list_comments::ListComments;
pub use post_comment::PostComment;

use xetaravel_kernel::{AppError, AppResult};

use crate::application::dto::CommentDto;
use crate::application::ports::{ArticleCatalog, AuthorDirectory};
use crate::domain::{ArticleId, AuthorId, Comment};

/// Name displayed when the author is unknown to the directory.
pub const UNKNOWN_AUTHOR: &str = "unknown";

/// Resolves a slug into a commentable (published) article, or fails with
/// `NotFound`. Drafts and malformed slugs are reported as missing.
async fn commentable_article(catalog: &dyn ArticleCatalog, slug: &str) -> AppResult<ArticleId> {
    catalog
        .published_article_id(slug)
        .await?
        .ok_or_else(|| AppError::NotFound("article not found".into()))
}

/// Turns comments into DTOs, naming their authors with a single lookup.
async fn to_dtos(
    authors: &dyn AuthorDirectory,
    comments: &[Comment],
) -> AppResult<Vec<CommentDto>> {
    let mut ids: Vec<AuthorId> = comments.iter().map(|c| c.author_id).collect();
    ids.sort_unstable();
    ids.dedup();
    let names = if ids.is_empty() {
        Default::default()
    } else {
        authors.names(&ids).await?
    };

    Ok(comments
        .iter()
        .map(|comment| {
            let name = names
                .get(&comment.author_id)
                .cloned()
                .unwrap_or_else(|| UNKNOWN_AUTHOR.to_owned());
            CommentDto::new(comment, name)
        })
        .collect())
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::*;
    use crate::application::ports::MockAuthorDirectory;
    use crate::application::test_support::{catalog, comment_by, member_principal};

    #[tokio::test]
    async fn unknown_authors_get_a_placeholder_name() {
        let mut authors = MockAuthorDirectory::new();
        authors.expect_names().returning(|_| Ok(HashMap::new()));

        let dtos = to_dtos(&authors, &[comment_by(member_principal())])
            .await
            .unwrap();

        assert_eq!(dtos[0].author.username, UNKNOWN_AUTHOR);
    }

    #[tokio::test]
    async fn only_published_articles_are_commentable() {
        assert!(commentable_article(&catalog(true), "hello").await.is_ok());
        assert!(matches!(
            commentable_article(&catalog(false), "hello").await,
            Err(AppError::NotFound(_))
        ));
    }
}
