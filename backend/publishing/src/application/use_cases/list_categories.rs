use std::sync::Arc;

use xetaravel_kernel::AppResult;

use crate::application::dto::CategoryDto;
use crate::domain::CategoryRepository;

/// Lists every blog category.
pub struct ListCategories {
    categories: Arc<dyn CategoryRepository>,
}

impl ListCategories {
    /// Builds the use case with its dependencies.
    pub fn new(categories: Arc<dyn CategoryRepository>) -> Self {
        Self { categories }
    }

    /// Returns all categories ordered by name.
    pub async fn execute(&self) -> AppResult<Vec<CategoryDto>> {
        let categories = self.categories.list_all().await?;
        Ok(categories.iter().map(CategoryDto::from).collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::application::test_support::category;
    use crate::domain::MockCategoryRepository;

    #[tokio::test]
    async fn returns_every_category() {
        let mut categories = MockCategoryRepository::new();
        categories
            .expect_list_all()
            .returning(|| Ok(vec![category("Rust"), category("Web")]));

        let result = ListCategories::new(Arc::new(categories))
            .execute()
            .await
            .unwrap();

        let slugs: Vec<_> = result.iter().map(|c| c.slug.as_str()).collect();
        assert_eq!(slugs, ["rust", "web"]);
    }
}
