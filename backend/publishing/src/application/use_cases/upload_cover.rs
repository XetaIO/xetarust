use std::sync::Arc;

use uuid::Uuid;
use xetaravel_kernel::{AppResult, Clock, Principal};

use super::{article_not_found, discard_cover, load_article};
use crate::application::dto::ArticleDto;
use crate::application::ports::{AuthorDirectory, CoverStorage};
use crate::domain::{ArticleId, ArticleRepository, CoverImage, ImageFormat};

/// Sets (or replaces) the cover image of an article.
pub struct UploadCover {
    articles: Arc<dyn ArticleRepository>,
    authors: Arc<dyn AuthorDirectory>,
    covers: Arc<dyn CoverStorage>,
    clock: Arc<dyn Clock>,
}

impl UploadCover {
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

    /// Validates the image, stores it under a fresh name, attaches it to the
    /// article and deletes the file of the previous cover.
    pub async fn execute(
        &self,
        principal: Principal,
        id: Uuid,
        bytes: &[u8],
    ) -> AppResult<ArticleDto> {
        principal.require_admin()?;
        let format = ImageFormat::sniff(bytes)?;
        let mut article = self
            .articles
            .find_by_id(ArticleId::from(id))
            .await?
            .ok_or_else(article_not_found)?;

        let cover = CoverImage::new(format);
        self.covers.store(&cover, bytes).await?;
        let previous = article.replace_cover(cover.clone(), self.clock.now());
        if let Err(error) = self.articles.update(&article).await {
            discard_cover(self.covers.as_ref(), &cover).await;
            return Err(error.into());
        }
        if let Some(previous) = previous {
            discard_cover(self.covers.as_ref(), &previous).await;
        }

        load_article(self.articles.as_ref(), self.authors.as_ref(), article.id).await
    }
}

#[cfg(test)]
mod tests {
    use xetaravel_kernel::{AppError, DomainError};

    use super::*;
    use crate::application::ports::{MockAuthorDirectory, MockCoverStorage};
    use crate::application::test_support::{
        admin_principal, authors, categorized, member_principal, now,
    };
    use crate::domain::{Article, MockArticleRepository};

    /// Header of a PNG file.
    const PNG: &[u8] = &[0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A, 0];

    /// Builds the use case under test.
    fn use_case(
        articles: MockArticleRepository,
        authors: MockAuthorDirectory,
        covers: MockCoverStorage,
    ) -> UploadCover {
        UploadCover::new(
            Arc::new(articles),
            Arc::new(authors),
            Arc::new(covers),
            crate::application::test_support::clock(),
        )
    }

    /// Returns a repository mock holding `existing` and accepting updates.
    fn repository_with(existing: Article) -> MockArticleRepository {
        let mut articles = MockArticleRepository::new();
        let found = existing.clone();
        articles
            .expect_find_by_id()
            .returning(move |_| Ok(Some(found.clone())));
        articles
            .expect_update()
            .withf(|a| {
                a.updated_at == now()
                    && a.cover
                        .as_ref()
                        .is_some_and(|c| c.as_str().ends_with(".png"))
            })
            .times(1)
            .returning(|_| Ok(()));
        articles.expect_find_categorized_by_id().returning(|_| {
            let mut entry = categorized(true);
            entry
                .article
                .replace_cover(CoverImage::new(ImageFormat::Png), now());
            Ok(Some(entry))
        });
        articles
    }

    #[tokio::test]
    async fn stores_the_image_and_attaches_it() {
        let existing = categorized(true).article;
        let id = existing.id.as_uuid();
        let mut covers = MockCoverStorage::new();
        covers
            .expect_store()
            .withf(|cover, bytes| cover.format() == ImageFormat::Png && bytes == PNG)
            .times(1)
            .returning(|_, _| Ok(()));
        covers.expect_delete().never();

        let article = use_case(repository_with(existing), authors(), covers)
            .execute(admin_principal(), id, PNG)
            .await
            .unwrap();

        assert!(article.summary.cover_image.is_some());
    }

    #[tokio::test]
    async fn deletes_the_previous_file() {
        let mut existing = categorized(true).article;
        let previous = CoverImage::new(ImageFormat::Jpeg);
        existing.replace_cover(previous.clone(), now());
        let mut covers = MockCoverStorage::new();
        covers.expect_store().returning(|_, _| Ok(()));
        covers
            .expect_delete()
            .withf(move |cover| *cover == previous)
            .times(1)
            .returning(|_| Ok(()));

        use_case(repository_with(existing), authors(), covers)
            .execute(admin_principal(), Uuid::now_v7(), PNG)
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn a_failed_cleanup_does_not_fail_the_upload() {
        let mut existing = categorized(true).article;
        existing.replace_cover(CoverImage::new(ImageFormat::Jpeg), now());
        let mut covers = MockCoverStorage::new();
        covers.expect_store().returning(|_, _| Ok(()));
        covers
            .expect_delete()
            .returning(|_| Err(AppError::Internal("disk".into())));

        assert!(
            use_case(repository_with(existing), authors(), covers)
                .execute(admin_principal(), Uuid::now_v7(), PNG)
                .await
                .is_ok()
        );
    }

    #[tokio::test]
    async fn removes_the_new_file_when_saving_fails() {
        let existing = categorized(true).article;
        let mut articles = MockArticleRepository::new();
        articles
            .expect_find_by_id()
            .returning(move |_| Ok(Some(existing.clone())));
        articles
            .expect_update()
            .returning(|_| Err(DomainError::Repository("down".into())));
        let mut covers = MockCoverStorage::new();
        covers.expect_store().returning(|_, _| Ok(()));
        covers
            .expect_delete()
            .withf(|cover| cover.format() == ImageFormat::Png)
            .times(1)
            .returning(|_| Ok(()));

        let error = use_case(articles, MockAuthorDirectory::new(), covers)
            .execute(admin_principal(), Uuid::now_v7(), PNG)
            .await
            .unwrap_err();

        assert!(matches!(error, AppError::Internal(_)));
    }

    #[tokio::test]
    async fn rejects_files_that_are_not_images() {
        let error = use_case(
            MockArticleRepository::new(),
            MockAuthorDirectory::new(),
            MockCoverStorage::new(),
        )
        .execute(admin_principal(), Uuid::now_v7(), b"plain text")
        .await
        .unwrap_err();

        assert_eq!(
            error,
            AppError::field("cover", "must be a JPEG, PNG or WebP image")
        );
    }

    #[tokio::test]
    async fn missing_article_is_not_found() {
        let mut articles = MockArticleRepository::new();
        articles.expect_find_by_id().returning(|_| Ok(None));
        let mut covers = MockCoverStorage::new();
        covers.expect_store().never();

        let error = use_case(articles, MockAuthorDirectory::new(), covers)
            .execute(admin_principal(), Uuid::now_v7(), PNG)
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
        .execute(member_principal(), Uuid::now_v7(), PNG)
        .await
        .unwrap_err();

        assert!(matches!(error, AppError::Forbidden(_)));
    }
}
