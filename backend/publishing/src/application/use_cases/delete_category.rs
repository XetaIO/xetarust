use std::sync::Arc;

use uuid::Uuid;
use xetaravel_kernel::{AppError, AppResult, Principal};

use crate::domain::{ArticleRepository, CategoryId, CategoryRepository};

/// Deletes an empty blog category.
pub struct DeleteCategory {
    categories: Arc<dyn CategoryRepository>,
    articles: Arc<dyn ArticleRepository>,
}

impl DeleteCategory {
    /// Builds the use case with its dependencies.
    pub fn new(
        categories: Arc<dyn CategoryRepository>,
        articles: Arc<dyn ArticleRepository>,
    ) -> Self {
        Self {
            categories,
            articles,
        }
    }

    /// Deletes the category; refuses when articles still use it.
    pub async fn execute(&self, principal: Principal, id: Uuid) -> AppResult<()> {
        principal.require_admin()?;
        let id = CategoryId::from(id);

        let count = self.articles.count_by_category(id).await?;
        if count > 0 {
            return Err(AppError::Conflict(format!(
                "this category still contains {count} article(s)"
            )));
        }

        if self.categories.delete(id).await? {
            Ok(())
        } else {
            Err(AppError::NotFound("category not found".into()))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::application::test_support::admin_principal;
    use crate::domain::{MockArticleRepository, MockCategoryRepository};

    /// Builds the use case: the category holds `count` articles and deletion returns `deleted`.
    fn use_case(count: u64, deleted: bool) -> DeleteCategory {
        let mut articles = MockArticleRepository::new();
        articles
            .expect_count_by_category()
            .returning(move |_| Ok(count));
        let mut categories = MockCategoryRepository::new();
        categories.expect_delete().returning(move |_| Ok(deleted));
        DeleteCategory::new(Arc::new(categories), Arc::new(articles))
    }

    #[tokio::test]
    async fn deletes_empty_category() {
        assert!(
            use_case(0, true)
                .execute(admin_principal(), Uuid::now_v7())
                .await
                .is_ok()
        );
    }

    #[tokio::test]
    async fn refuses_non_empty_category() {
        assert!(matches!(
            use_case(2, true)
                .execute(admin_principal(), Uuid::now_v7())
                .await,
            Err(AppError::Conflict(_))
        ));
    }

    #[tokio::test]
    async fn reports_missing_category() {
        assert!(matches!(
            use_case(0, false)
                .execute(admin_principal(), Uuid::now_v7())
                .await,
            Err(AppError::NotFound(_))
        ));
    }
}
