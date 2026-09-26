use std::sync::Arc;

use xetaravel_kernel::AppResult;

use super::{article_not_found, find_published};
use crate::application::dto::ArticleDto;
use crate::application::ports::AuthorDirectory;
use crate::application::views::with_author;
use crate::domain::ArticleRepository;

/// Returns a published article by its slug.
pub struct GetPublishedArticle {
    articles: Arc<dyn ArticleRepository>,
    authors: Arc<dyn AuthorDirectory>,
}

impl GetPublishedArticle {
    /// Builds the use case with its dependencies.
    pub fn new(articles: Arc<dyn ArticleRepository>, authors: Arc<dyn AuthorDirectory>) -> Self {
        Self { articles, authors }
    }

    /// Loads the article; drafts and malformed slugs are reported as not found.
    pub async fn execute(&self, slug: &str) -> AppResult<ArticleDto> {
        let entry = find_published(self.articles.as_ref(), slug)
            .await?
            .ok_or_else(article_not_found)?;
        let view = with_author(self.authors.as_ref(), entry).await?;
        Ok(ArticleDto::from(&view))
    }
}

#[cfg(test)]
mod tests {
    use xetaravel_kernel::AppError;

    use super::*;
    use crate::application::ports::MockAuthorDirectory;
    use crate::application::test_support::{authors, categorized};
    use crate::domain::MockArticleRepository;

    /// Builds the use case with a repository returning `published` (or nothing).
    fn use_case(published: Option<bool>, authors: MockAuthorDirectory) -> GetPublishedArticle {
        let mut articles = MockArticleRepository::new();
        articles
            .expect_find_categorized_by_slug()
            .returning(move |_| Ok(published.map(categorized)));
        GetPublishedArticle::new(Arc::new(articles), Arc::new(authors))
    }

    #[tokio::test]
    async fn returns_published_article_with_content_and_author() {
        let article = use_case(Some(true), authors())
            .execute("hello-rust")
            .await
            .unwrap();
        assert_eq!(article.summary.title, "Hello Rust");
        assert_eq!(article.summary.author.username, "xety");
        assert_eq!(article.content, "Some content");
    }

    #[tokio::test]
    async fn hides_drafts() {
        let error = use_case(Some(false), MockAuthorDirectory::new())
            .execute("hello-rust")
            .await
            .unwrap_err();
        assert!(matches!(error, AppError::NotFound(_)));
    }

    #[tokio::test]
    async fn reports_missing_and_malformed_slugs_as_not_found() {
        assert!(matches!(
            use_case(None, MockAuthorDirectory::new())
                .execute("missing")
                .await,
            Err(AppError::NotFound(_))
        ));
        assert!(matches!(
            use_case(None, MockAuthorDirectory::new())
                .execute("Bad Slug")
                .await,
            Err(AppError::NotFound(_))
        ));
    }
}
