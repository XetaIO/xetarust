use async_trait::async_trait;

use xetaravel_kernel::DomainResult;

use super::{Category, CategoryId, Slug};

/// Persistence port of the [`Category`] aggregate.
#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait CategoryRepository: Send + Sync {
    /// Finds a category by id.
    async fn find_by_id(&self, id: CategoryId) -> DomainResult<Option<Category>>;

    /// Lists every category, ordered by name.
    async fn list_all(&self) -> DomainResult<Vec<Category>>;

    /// Tells whether another category (other than `excluding`) already uses this slug.
    async fn slug_exists(&self, slug: &Slug, excluding: Option<CategoryId>) -> DomainResult<bool>;

    /// Inserts a new category.
    async fn create(&self, category: &Category) -> DomainResult<()>;

    /// Persists the changes of an existing category.
    async fn update(&self, category: &Category) -> DomainResult<()>;

    /// Deletes a category. Returns `false` when it did not exist.
    async fn delete(&self, id: CategoryId) -> DomainResult<bool>;
}
