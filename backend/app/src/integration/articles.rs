//! Commentable articles for Discussion, served by Publishing.

use std::sync::Arc;

use async_trait::async_trait;
use xetaravel_discussion::domain::ArticleId;
use xetaravel_kernel::AppResult;
use xetaravel_publishing::PublishedArticles;

/// Implements the Discussion `ArticleCatalog` port on top of the Publishing
/// public contract: only published articles can be commented.
pub struct PublishingArticleCatalog {
    articles: Arc<dyn PublishedArticles>,
}

impl PublishingArticleCatalog {
    /// Wraps the Publishing public catalog.
    pub fn new(articles: Arc<dyn PublishedArticles>) -> Self {
        Self { articles }
    }
}

#[async_trait]
impl xetaravel_discussion::ArticleCatalog for PublishingArticleCatalog {
    /// Translates the published article reference into a Discussion id.
    async fn published_article_id(&self, slug: &str) -> AppResult<Option<ArticleId>> {
        Ok(self
            .articles
            .find_published(slug)
            .await?
            .map(|article| ArticleId::from(article.id)))
    }
}

#[cfg(test)]
mod tests {
    use uuid::Uuid;
    use xetaravel_discussion::ArticleCatalog;
    use xetaravel_publishing::PublishedArticleRef;

    use super::*;

    /// Publishing catalog knowing one published slug.
    struct FakeArticles(Uuid);

    #[async_trait]
    impl PublishedArticles for FakeArticles {
        /// Only "hello-rust" is published.
        async fn find_published(&self, slug: &str) -> AppResult<Option<PublishedArticleRef>> {
            Ok((slug == "hello-rust").then_some(PublishedArticleRef { id: self.0 }))
        }
    }

    #[tokio::test]
    async fn translates_published_articles() {
        let id = Uuid::now_v7();
        let catalog = PublishingArticleCatalog::new(Arc::new(FakeArticles(id)));

        assert_eq!(
            catalog.published_article_id("hello-rust").await.unwrap(),
            Some(ArticleId::from(id))
        );
        assert_eq!(catalog.published_article_id("draft").await.unwrap(), None);
    }
}
