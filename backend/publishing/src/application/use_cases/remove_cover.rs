use std::sync::Arc;

use uuid::Uuid;
use xetaravel_kernel::{AppResult, Clock, Principal};

use super::{article_not_found, discard_cover, load_article};
use crate::application::dto::ArticleDto;
use crate::application::ports::{AuthorDirectory, CoverStorage};
use crate::domain::{ArticleId, ArticleRepository};

/// Removes the cover image of an article.
pub struct RemoveCover {
    articles: Arc<dyn ArticleRepository>,
    authors: Arc<dyn AuthorDirectory>,
    covers: Arc<dyn CoverStorage>,
    clock: Arc<dyn Clock>,
}

impl RemoveCover {
    /// Builds the use case with its dependencies.
    pub fn new(
        articles: Arc<dyn ArticleRepository>,
        authors: Arc<dyn AuthorDirectory>,
        covers: Arc<dyn CoverStorage>,
        clock: Arc<dyn Clock>,
    ) -> Self {
        Self {
            articles,
            authors,
            covers,
            clock,
        }
    }

    /// Detaches the cover from the article and deletes its file. Removing
    /// the cover of an article without one is a no-op.
    pub async fn execute(&self, principal: Principal, id: Uuid) -> AppResult<ArticleDto> {
        principal.require_admin()?;
        let mut article = self
            .articles
            .find_by_id(ArticleId::from(id))
            .await?
            .ok_or_else(article_not_found)?;

        if let Some(previous) = article.remove_cover(self.clock.now()) {
            self.articles.update(&article).await?;
            discard_cover(self.covers.as_ref(), &previous).await;
        }

        load_article(self.articles.as_ref(), self.authors.as_ref(), article.id).await
    }
}

#[cfg(test)]
mod tests {
    use xetaravel_kernel::AppError;

    use super::*;
    use crate::application::ports::{MockAuthorDirectory, MockCoverStorage};
    use crate::application::test_support::{
        admin_principal, authors, categorized, clock, member_principal, now,
    };
    use crate::domain::{Article, CoverImage, ImageFormat, MockArticleRepository};

    /// Builds the use case under test.
    fn use_case(
        articles: MockArticleRepository,
        authors: MockAuthorDirectory,
        covers: MockCoverStorage,
    ) -> RemoveCover {
        RemoveCover::new(
            Arc::new(articles),
            Arc::new(authors),
            Arc::new(covers),
            clock(),
        )
    }

    /// Returns a repository mock holding `existing`, expecting `updates` saves.
    fn repository_with(existing: Article, updates: usize) -> MockArticleRepository {
        let mut articles = MockArticleRepository::new();
        articles
            .expect_find_by_id()
            .returning(move |_| Ok(Some(existing.clone())));
        articles
            .expect_update()
            .withf(|a| a.cover.is_none())
            .times(updates)
            .returning(|_| Ok(()));
        articles
            .expect_find_categorized_by_id()
            .returning(|_| Ok(Some(categorized(true))));
        articles
    }

    #[tokio::test]
    async fn detaches_the_cover_and_deletes_its_file() {
        let mut existing = categorized(true).article;
        let cover = CoverImage::new(ImageFormat::Webp);
        existing.replace_cover(cover.clone(), now());
        let mut covers = MockCoverStorage::new();
        covers
            .expect_delete()
            .withf(move |c| *c == cover)
            .times(1)
            .returning(|_| Ok(()));

        let article = use_case(repository_with(existing, 1), authors(), covers)
            .execute(admin_principal(), Uuid::now_v7())
            .await
            .unwrap();

        assert_eq!(article.summary.cover_image, None);
    }

    #[tokio::test]
    async fn is_a_no_op_without_cover() {
        let mut covers = MockCoverStorage::new();
        covers.expect_delete().never();

        use_case(
            repository_with(categorized(true).article, 0),
            authors(),
            covers,
        )
        .execute(admin_principal(), Uuid::now_v7())
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn missing_article_is_not_found() {
        let mut articles = MockArticleRepository::new();
        articles.expect_find_by_id().returning(|_| Ok(None));

        let error = use_case(
            articles,
            MockAuthorDirectory::new(),
            MockCoverStorage::new(),
        )
        .execute(admin_principal(), Uuid::now_v7())
        .await
        .unwrap_err();

        assert!(matches!(error, AppError::NotFound(_)));
    }

    #[tokio::test]
    async fn is_reserved_to_admins() {
        let error = use_case(
            MockArticleRepository::new(),
            MockAuthorDirectory::new(),
            MockCoverStorage::new(),
        )
        .execute(member_principal(), Uuid::now_v7())
        .await
        .unwrap_err();

        assert!(matches!(error, AppError::Forbidden(_)));
    }
}
