use std::sync::Arc;

use xetaravel_kernel::AppResult;

use super::{commentable_article, to_dtos};
use crate::application::dto::CommentDto;
use crate::application::ports::{ArticleCatalog, AuthorDirectory};
use crate::domain::CommentRepository;

/// Lists the comments of a published article.
pub struct ListComments {
    comments: Arc<dyn CommentRepository>,
    articles: Arc<dyn ArticleCatalog>,
    authors: Arc<dyn AuthorDirectory>,
}

impl ListComments {
    /// Builds the use case with its dependencies.
    pub fn new(
        comments: Arc<dyn CommentRepository>,
        articles: Arc<dyn ArticleCatalog>,
        authors: Arc<dyn AuthorDirectory>,
    ) -> Self {
        Self {
            comments,
            articles,
            authors,
        }
    }

    /// Returns the comments of the article identified by `slug`, oldest first.
    pub async fn execute(&self, slug: &str) -> AppResult<Vec<CommentDto>> {
        let article_id = commentable_article(self.articles.as_ref(), slug).await?;
        let comments = self.comments.list_by_article(article_id).await?;
        to_dtos(self.authors.as_ref(), &comments).await
    }
}

#[cfg(test)]
mod tests {
    use xetaravel_kernel::AppError;

    use super::*;
    use crate::application::ports::MockAuthorDirectory;
    use crate::application::test_support::{authors, catalog, comment_by, member_principal};
    use crate::domain::MockCommentRepository;

    #[tokio::test]
    async fn lists_comments_of_a_published_article() {
        let comment = comment_by(member_principal());
        let mut comments = MockCommentRepository::new();
        comments
            .expect_list_by_article()
            .returning(move |_| Ok(vec![comment.clone()]));

        let result = ListComments::new(
            Arc::new(comments),
            Arc::new(catalog(true)),
            Arc::new(authors()),
        )
        .execute("hello-rust")
        .await
        .unwrap();

        assert_eq!(result.len(), 1);
        assert_eq!(result[0].author.username, "john");
    }

    #[tokio::test]
    async fn hides_comments_of_drafts() {
        let error = ListComments::new(
            Arc::new(MockCommentRepository::new()),
            Arc::new(catalog(false)),
            Arc::new(MockAuthorDirectory::new()),
        )
        .execute("hello-rust")
        .await
        .unwrap_err();

        assert!(matches!(error, AppError::NotFound(_)));
    }
}
