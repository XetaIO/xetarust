use std::sync::Arc;

use uuid::Uuid;
use validator::Validate;
use xetaravel_kernel::{AppError, AppResult, Clock, Principal};

use super::{parse_optional_slug, slug_taken};
use crate::application::dto::{CategoryDto, UpsertCategoryRequest};
use crate::domain::{CategoryId, CategoryRepository};

/// Edits a blog category.
pub struct UpdateCategory {
    categories: Arc<dyn CategoryRepository>,
    clock: Arc<dyn Clock>,
}

impl UpdateCategory {
    /// Builds the use case with its dependencies.
    pub fn new(categories: Arc<dyn CategoryRepository>, clock: Arc<dyn Clock>) -> Self {
        Self { categories, clock }
    }

    /// Applies the form to the category, keeping its slug unique.
    pub async fn execute(
        &self,
        principal: Principal,
        id: Uuid,
        input: UpsertCategoryRequest,
    ) -> AppResult<CategoryDto> {
        principal.require_admin()?;
        input.validate()?;

        let mut category = self
            .categories
            .find_by_id(CategoryId::from(id))
            .await?
            .ok_or_else(|| AppError::NotFound("category not found".into()))?;
        category.update(
            &input.name,
            parse_optional_slug(input.slug.as_deref())?,
            input.description.as_deref(),
            self.clock.now(),
        )?;

        if self
            .categories
            .slug_exists(&category.slug, Some(category.id))
            .await?
        {
            return Err(slug_taken());
        }
        self.categories.update(&category).await?;

        Ok((&category).into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::application::test_support::{admin_principal, category, clock};
    use crate::domain::MockCategoryRepository;

    /// Returns a category form renaming the category.
    fn request() -> UpsertCategoryRequest {
        UpsertCategoryRequest {
            name: "Rust lang".into(),
            slug: Some("rust-lang".into()),
            description: None,
        }
    }

    #[tokio::test]
    async fn updates_the_category() {
        let mut categories = MockCategoryRepository::new();
        categories
            .expect_find_by_id()
            .returning(|_| Ok(Some(category("Rust"))));
        categories.expect_slug_exists().returning(|_, _| Ok(false));
        categories
            .expect_update()
            .withf(|c| c.name == "Rust lang")
            .times(1)
            .returning(|_| Ok(()));

        let dto = UpdateCategory::new(Arc::new(categories), clock())
            .execute(admin_principal(), Uuid::now_v7(), request())
            .await
            .unwrap();

        assert_eq!(dto.slug, "rust-lang");
    }

    #[tokio::test]
    async fn missing_category_is_not_found() {
        let mut categories = MockCategoryRepository::new();
        categories.expect_find_by_id().returning(|_| Ok(None));

        let error = UpdateCategory::new(Arc::new(categories), clock())
            .execute(admin_principal(), Uuid::now_v7(), request())
            .await
            .unwrap_err();

        assert!(matches!(error, AppError::NotFound(_)));
    }
}
