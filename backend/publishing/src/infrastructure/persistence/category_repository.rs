use async_trait::async_trait;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, PaginatorTrait, QueryFilter,
    QueryOrder,
};
use xetaravel_kernel::DomainResult;
use xetaravel_kernel::persistence::db_error;

use super::entities::category;
use super::mappers::{from_category, to_category};
use crate::domain::{Category, CategoryId, CategoryRepository, Slug};

/// PostgreSQL implementation of [`CategoryRepository`].
#[derive(Clone)]
pub struct SeaOrmCategoryRepository {
    db: DatabaseConnection,
}

impl SeaOrmCategoryRepository {
    /// Builds the repository on top of a connection pool.
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }
}

#[async_trait]
impl CategoryRepository for SeaOrmCategoryRepository {
    /// Finds a category by id.
    async fn find_by_id(&self, id: CategoryId) -> DomainResult<Option<Category>> {
        category::Entity::find_by_id(id.as_uuid())
            .one(&self.db)
            .await
            .map_err(db_error)?
            .map(to_category)
            .transpose()
    }

    /// Lists every category ordered by name.
    async fn list_all(&self) -> DomainResult<Vec<Category>> {
        category::Entity::find()
            .order_by_asc(category::Column::Name)
            .all(&self.db)
            .await
            .map_err(db_error)?
            .into_iter()
            .map(to_category)
            .collect()
    }

    /// Tells whether another category already uses this slug.
    async fn slug_exists(&self, slug: &Slug, excluding: Option<CategoryId>) -> DomainResult<bool> {
        let mut query = category::Entity::find().filter(category::Column::Slug.eq(slug.as_str()));
        if let Some(id) = excluding {
            query = query.filter(category::Column::Id.ne(id.as_uuid()));
        }
        Ok(query.count(&self.db).await.map_err(db_error)? > 0)
    }

    /// Inserts a new category.
    async fn create(&self, category: &Category) -> DomainResult<()> {
        from_category(category)
            .insert(&self.db)
            .await
            .map_err(db_error)?;
        Ok(())
    }

    /// Persists the changes of an existing category.
    async fn update(&self, category: &Category) -> DomainResult<()> {
        from_category(category)
            .update(&self.db)
            .await
            .map_err(db_error)?;
        Ok(())
    }

    /// Deletes a category; returns `false` when it did not exist.
    async fn delete(&self, id: CategoryId) -> DomainResult<bool> {
        let result = category::Entity::delete_by_id(id.as_uuid())
            .exec(&self.db)
            .await
            .map_err(db_error)?;
        Ok(result.rows_affected > 0)
    }
}
