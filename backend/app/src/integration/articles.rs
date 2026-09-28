//! Commentable articles for Discussion, served by Publishing.

use std::sync::Arc;

use async_trait::async_trait;
use xetaravel_discussion::domain::{ArticleId, CommentableArticle};
use xetaravel_kernel::AppResult;
use xetaravel_publishing::PublishedArticles;

/// Implements the Discussion `ArticleCatalog` port on top of the Publishing
/// public contract: only published articles can be commented, and only
/// while their comments are enabled.
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
    /// Translates the published article reference into a Discussion
    /// commentable article.
    async fn published_article(&self, slug: &str) -> AppResult<Option<CommentableArticle>> {
        Ok(self
            .articles
            .find_published(slug)
            .await?
            .map(|article| CommentableArticle {
                id: ArticleId::from(article.id),
                comments_open: article.comments_enabled,
            }))
    }
}

#[cfg(test)]
mod tests {
    use uuid::Uuid;
    use xetaravel_discussion::ArticleCatalog;
    use xetaravel_publishing::PublishedArticleRef;

    use super::*;

    /// Publishing catalog knowing two published slugs: "hello-rust" (open to
    /// comments) and "closed" (comments disabled).
    struct FakeArticles(Uuid);

    #[async_trait]
    impl PublishedArticles for FakeArticles {
        /// Only "hello-rust" and "closed" are published.
        async fn find_published(&self, slug: &str) -> AppResult<Option<PublishedArticleRef>> {
            Ok(match slug {
                "hello-rust" | "closed" => Some(PublishedArticleRef {
                    id: self.0,
                    comments_enabled: slug == "hello-rust",
                }),
                _ => None,
            })
        }
    }

    #[tokio::test]
    async fn translates_published_articles() {
        let id = Uuid::now_v7();
        let catalog = PublishingArticleCatalog::new(Arc::new(FakeArticles(id)));

        assert_eq!(
            catalog.published_article("hello-rust").await.unwrap(),
            Some(CommentableArticle {
                id: ArticleId::from(id),
                comments_open: true
            })
        );
        assert_eq!(catalog.published_article("draft").await.unwrap(), None);
    }

    #[tokio::test]
    async fn translates_closed_articles() {
        let id = Uuid::now_v7();
        let catalog = PublishingArticleCatalog::new(Arc::new(FakeArticles(id)));

        assert_eq!(
            catalog.published_article("closed").await.unwrap(),
            Some(CommentableArticle {
                id: ArticleId::from(id),
                comments_open: false
            })
        );
    }
}
