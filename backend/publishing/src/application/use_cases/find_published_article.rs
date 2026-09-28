use std::sync::Arc;

use async_trait::async_trait;
use xetaravel_kernel::AppResult;

use super::find_published;
use crate::application::contract::{PublishedArticleRef, PublishedArticles};
use crate::domain::ArticleRepository;

/// Tells other contexts whether a slug designates a published article.
/// This use case implements the [`PublishedArticles`] contract.
pub struct FindPublishedArticle {
    articles: Arc<dyn ArticleRepository>,
}

impl FindPublishedArticle {
    /// Builds the use case with its dependencies.
    pub fn new(articles: Arc<dyn ArticleRepository>) -> Self {
        Self { articles }
    }

    /// Returns a reference to the published article using `slug`, if any.
    pub async fn execute(&self, slug: &str) -> AppResult<Option<PublishedArticleRef>> {
        Ok(find_published(self.articles.as_ref(), slug)
            .await?
            .map(|entry| PublishedArticleRef {
                id: entry.article.id.as_uuid(),
                comments_enabled: entry.article.comments_enabled,
            }))
    }
}

#[async_trait]
impl PublishedArticles for FindPublishedArticle {
    /// Delegates to [`FindPublishedArticle::execute`].
    async fn find_published(&self, slug: &str) -> AppResult<Option<PublishedArticleRef>> {
        self.execute(slug).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::application::test_support::categorized;
    use crate::domain::MockArticleRepository;

    /// Builds the use case with a repository returning `published` (or nothing).
    fn use_case(published: Option<bool>) -> FindPublishedArticle {
        let mut articles = MockArticleRepository::new();
        articles
            .expect_find_categorized_by_slug()
            .returning(move |_| Ok(published.map(categorized)));
        FindPublishedArticle::new(Arc::new(articles))
    }

    #[tokio::test]
    async fn exposes_the_comments_setting() {
        let mut entry = categorized(true);
        entry.article.comments_enabled = false;
        let id = entry.article.id.as_uuid();
        let mut articles = MockArticleRepository::new();
        articles
            .expect_find_categorized_by_slug()
            .returning(move |_| Ok(Some(entry.clone())));

        let found = FindPublishedArticle::new(Arc::new(articles))
            .execute("hello-rust")
            .await
            .unwrap();

        assert_eq!(
            found,
            Some(PublishedArticleRef {
                id,
                comments_enabled: false
            })
        );
    }

    #[tokio::test]
    async fn returns_published_articles() {
        let found = use_case(Some(true))
            .find_published("hello-rust")
            .await
            .unwrap();
        assert!(found.is_some());
    }

    #[tokio::test]
    async fn hides_drafts_missing_and_malformed_slugs() {
        assert_eq!(use_case(Some(false)).execute("hello-rust").await, Ok(None));
        assert_eq!(use_case(None).execute("missing").await, Ok(None));
        assert_eq!(use_case(None).execute("Bad Slug").await, Ok(None));
    }
}
