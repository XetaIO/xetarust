use std::sync::Arc;

use validator::Validate;
use xetaravel_kernel::{AppResult, Clock, Principal};

use super::{parse_optional_slug, slug_taken};
use crate::application::dto::{CategoryDto, UpsertCategoryRequest};
use crate::domain::{Category, CategoryRepository};

/// Creates a blog category.
pub struct CreateCategory {
    categories: Arc<dyn CategoryRepository>,
    clock: Arc<dyn Clock>,
}

impl CreateCategory {
    /// Builds the use case with its dependencies.
    pub fn new(categories: Arc<dyn CategoryRepository>, clock: Arc<dyn Clock>) -> Self {
        Self { categories, clock }
    }

    /// Validates the form, ensures the slug is unique and stores the category.
    pub async fn execute(
        &self,
        principal: Principal,
        input: UpsertCategoryRequest,
    ) -> AppResult<CategoryDto> {
        principal.require_admin()?;
        input.validate()?;

        let category = Category::create(
            &input.name,
            parse_optional_slug(input.slug.as_deref())?,
            input.description.as_deref(),
            self.clock.now(),
        )?;
        if self.categories.slug_exists(&category.slug, None).await? {
            return Err(slug_taken());
        }
        self.categories.create(&category).await?;

        Ok((&category).into())
    }
}

#[cfg(test)]
mod tests {
    use xetaravel_kernel::AppError;

    use super::*;
    use crate::application::test_support::{admin_principal, clock, member_principal};
    use crate::domain::MockCategoryRepository;

    /// Returns a valid category form.
    fn request() -> UpsertCategoryRequest {
        UpsertCategoryRequest {
            name: "Web Development".into(),
            slug: None,
            description: Some("All about the web".into()),
        }
    }

    #[tokio::test]
    async fn creates_a_category_with_derived_slug() {
        let mut categories = MockCategoryRepository::new();
        categories.expect_slug_exists().returning(|_, _| Ok(false));
        categories.expect_create().times(1).returning(|_| Ok(()));

        let dto = CreateCategory::new(Arc::new(categories), clock())
            .execute(admin_principal(), request())
            .await
            .unwrap();

        assert_eq!(dto.slug, "web-development");
    }

    #[tokio::test]
    async fn rejects_duplicated_slug() {
        let mut categories = MockCategoryRepository::new();
        categories.expect_slug_exists().returning(|_, _| Ok(true));

        let error = CreateCategory::new(Arc::new(categories), clock())
            .execute(admin_principal(), request())
            .await
            .unwrap_err();

        assert_eq!(error, AppError::field("slug", "is already taken"));
    }

    #[tokio::test]
    async fn is_reserved_to_admins() {
        let error = CreateCategory::new(Arc::new(MockCategoryRepository::new()), clock())
            .execute(member_principal(), request())
            .await
            .unwrap_err();

        assert!(matches!(error, AppError::Forbidden(_)));
    }
}
