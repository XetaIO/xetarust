use std::sync::Arc;

use uuid::Uuid;
use xetaravel_kernel::{AppResult, Principal};

use super::article_not_found;
use crate::domain::{ArticleId, ArticleRepository};

/// Deletes an article. Its comments (Discussion context) are removed by the
/// database cascade.
pub struct DeleteArticle {
    articles: Arc<dyn ArticleRepository>,
}

impl DeleteArticle {
    /// Builds the use case with its dependencies.
    pub fn new(articles: Arc<dyn ArticleRepository>) -> Self {
        Self { articles }
    }

    /// Deletes the article or fails with `NotFound`.
    pub async fn execute(&self, principal: Principal, id: Uuid) -> AppResult<()> {
        principal.require_admin()?;
        if self.articles.delete(ArticleId::from(id)).await? {
            Ok(())
        } else {
            Err(article_not_found())
        }
    }
}

#[cfg(test)]
mod tests {
    use xetaravel_kernel::AppError;

    use super::*;
    use crate::application::test_support::{admin_principal, member_principal};
    use crate::domain::MockArticleRepository;

    /// Builds the use case with a repository answering `deleted`.
    fn use_case(deleted: bool) -> DeleteArticle {
        let mut articles = MockArticleRepository::new();
        articles.expect_delete().returning(move |_| Ok(deleted));
        DeleteArticle::new(Arc::new(articles))
    }

    #[tokio::test]
    async fn deletes_existing_article() {
        assert!(
            use_case(true)
                .execute(admin_principal(), Uuid::now_v7())
                .await
                .is_ok()
        );
    }

    #[tokio::test]
    async fn reports_missing_article() {
        assert!(matches!(
            use_case(false)
                .execute(admin_principal(), Uuid::now_v7())
                .await,
            Err(AppError::NotFound(_))
        ));
    }

    #[tokio::test]
    async fn is_reserved_to_admins() {
        assert!(matches!(
            use_case(true)
                .execute(member_principal(), Uuid::now_v7())
                .await,
            Err(AppError::Forbidden(_))
        ));
    }
}
