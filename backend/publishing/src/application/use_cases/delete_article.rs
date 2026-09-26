use std::sync::Arc;

use uuid::Uuid;
use xetaravel_kernel::{AppResult, Principal};

use super::{article_not_found, discard_cover};
use crate::application::ports::CoverStorage;
use crate::domain::{ArticleId, ArticleRepository};

/// Deletes an article and its cover image file. Its comments (Discussion
/// context) are removed by the database cascade.
pub struct DeleteArticle {
    articles: Arc<dyn ArticleRepository>,
    covers: Arc<dyn CoverStorage>,
}

impl DeleteArticle {
    /// Builds the use case with its dependencies.
    pub fn new(articles: Arc<dyn ArticleRepository>, covers: Arc<dyn CoverStorage>) -> Self {
        Self { articles, covers }
    }

    /// Deletes the article or fails with `NotFound`.
    pub async fn execute(&self, principal: Principal, id: Uuid) -> AppResult<()> {
        principal.require_admin()?;
        let id = ArticleId::from(id);
        let article = self
            .articles
            .find_by_id(id)
            .await?
            .ok_or_else(article_not_found)?;

        if !self.articles.delete(id).await? {
            return Err(article_not_found());
        }
        if let Some(cover) = &article.cover {
            discard_cover(self.covers.as_ref(), cover).await;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use xetaravel_kernel::AppError;

    use super::*;
    use crate::application::ports::MockCoverStorage;
    use crate::application::test_support::{admin_principal, categorized, member_principal, now};
    use crate::domain::{Article, CoverImage, ImageFormat, MockArticleRepository};

    /// Builds the use case over a repository holding `existing`.
    fn use_case(existing: Option<Article>, covers: MockCoverStorage) -> DeleteArticle {
        let mut articles = MockArticleRepository::new();
        let deleted = existing.is_some();
        articles
            .expect_find_by_id()
            .returning(move |_| Ok(existing.clone()));
        articles.expect_delete().returning(move |_| Ok(deleted));
        DeleteArticle::new(Arc::new(articles), Arc::new(covers))
    }

    #[tokio::test]
    async fn deletes_existing_article() {
        let mut covers = MockCoverStorage::new();
        covers.expect_delete().never();

        assert!(
            use_case(Some(categorized(true).article), covers)
                .execute(admin_principal(), Uuid::now_v7())
                .await
                .is_ok()
        );
    }

    #[tokio::test]
    async fn deletes_the_cover_file() {
        let mut article = categorized(true).article;
        let cover = CoverImage::new(ImageFormat::Png);
        article.replace_cover(cover.clone(), now());
        let mut covers = MockCoverStorage::new();
        covers
            .expect_delete()
            .withf(move |c| *c == cover)
            .times(1)
            .returning(|_| Ok(()));

        use_case(Some(article), covers)
            .execute(admin_principal(), Uuid::now_v7())
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn reports_missing_article() {
        assert!(matches!(
            use_case(None, MockCoverStorage::new())
                .execute(admin_principal(), Uuid::now_v7())
                .await,
            Err(AppError::NotFound(_))
        ));
    }

    #[tokio::test]
    async fn is_reserved_to_admins() {
        assert!(matches!(
            use_case(None, MockCoverStorage::new())
                .execute(member_principal(), Uuid::now_v7())
                .await,
            Err(AppError::Forbidden(_))
        ));
    }
}
