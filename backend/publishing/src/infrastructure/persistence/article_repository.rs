use std::collections::HashMap;

use async_trait::async_trait;
use sea_orm::sea_query::NullOrdering;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, Order, PaginatorTrait,
    QueryFilter, QueryOrder,
};
use uuid::Uuid;
use xetaravel_kernel::pagination::{Page, PageRequest};
use xetaravel_kernel::persistence::db_error;
use xetaravel_kernel::{DomainError, DomainResult};

use super::entities::{article, category};
use super::mappers::{from_article, to_article, to_category};
use crate::domain::{
    Article, ArticleFilter, ArticleId, ArticleRepository, CategorizedArticle, Category, CategoryId,
    Slug,
};

/// PostgreSQL implementation of [`ArticleRepository`].
#[derive(Clone)]
pub struct SeaOrmArticleRepository {
    db: DatabaseConnection,
}

impl SeaOrmArticleRepository {
    /// Builds the repository on top of a connection pool.
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }

    /// Attaches categories to article rows with one batched query.
    async fn categorize(
        &self,
        models: Vec<article::Model>,
    ) -> DomainResult<Vec<CategorizedArticle>> {
        let categories = self
            .load_categories(models.iter().map(|m| m.category_id))
            .await?;

        models
            .into_iter()
            .map(|model| {
                let category = categories
                    .get(&model.category_id)
                    .cloned()
                    .ok_or_else(|| DomainError::Repository("article category is missing".into()))?;
                Ok(CategorizedArticle {
                    article: to_article(model)?,
                    category,
                })
            })
            .collect()
    }

    /// Loads the categories of the given ids in a single query.
    async fn load_categories(
        &self,
        ids: impl IntoIterator<Item = Uuid>,
    ) -> DomainResult<HashMap<Uuid, Category>> {
        let mut ids: Vec<_> = ids.into_iter().collect();
        ids.sort_unstable();
        ids.dedup();
        if ids.is_empty() {
            return Ok(HashMap::new());
        }

        category::Entity::find()
            .filter(category::Column::Id.is_in(ids))
            .all(&self.db)
            .await
            .map_err(db_error)?
            .into_iter()
            .map(|model| Ok((model.id, to_category(model)?)))
            .collect()
    }

    /// Converts an optional row into an optional categorized article.
    async fn categorize_one(
        &self,
        model: Option<article::Model>,
    ) -> DomainResult<Option<CategorizedArticle>> {
        match model {
            Some(model) => Ok(self.categorize(vec![model]).await?.pop()),
            None => Ok(None),
        }
    }
}

#[async_trait]
impl ArticleRepository for SeaOrmArticleRepository {
    /// Finds an article by id.
    async fn find_by_id(&self, id: ArticleId) -> DomainResult<Option<Article>> {
        article::Entity::find_by_id(id.as_uuid())
            .one(&self.db)
            .await
            .map_err(db_error)?
            .map(to_article)
            .transpose()
    }

    /// Finds an article with its category by id.
    async fn find_categorized_by_id(
        &self,
        id: ArticleId,
    ) -> DomainResult<Option<CategorizedArticle>> {
        let model = article::Entity::find_by_id(id.as_uuid())
            .one(&self.db)
            .await
            .map_err(db_error)?;
        self.categorize_one(model).await
    }

    /// Finds an article with its category by slug.
    async fn find_categorized_by_slug(
        &self,
        slug: &Slug,
    ) -> DomainResult<Option<CategorizedArticle>> {
        let model = article::Entity::find()
            .filter(article::Column::Slug.eq(slug.as_str()))
            .one(&self.db)
            .await
            .map_err(db_error)?;
        self.categorize_one(model).await
    }

    /// Lists articles with their category matching the filter, newest first.
    async fn list_categorized(
        &self,
        filter: ArticleFilter,
        request: PageRequest,
    ) -> DomainResult<Page<CategorizedArticle>> {
        let mut query = article::Entity::find();

        if filter.published_only {
            query = query.filter(article::Column::PublishedAt.is_not_null());
        }
        if let Some(slug) = &filter.category {
            let category = category::Entity::find()
                .filter(category::Column::Slug.eq(slug.as_str()))
                .one(&self.db)
                .await
                .map_err(db_error)?;
            let Some(category) = category else {
                return Ok(Page {
                    items: vec![],
                    total: 0,
                    request,
                });
            };
            query = query.filter(article::Column::CategoryId.eq(category.id));
        }

        let paginator = query
            .order_by_with_nulls(
                article::Column::PublishedAt,
                Order::Desc,
                NullOrdering::First,
            )
            .order_by_desc(article::Column::CreatedAt)
            .order_by_desc(article::Column::Id)
            .paginate(&self.db, request.per_page());

        let total = paginator.num_items().await.map_err(db_error)?;
        let models = paginator
            .fetch_page(request.page() - 1)
            .await
            .map_err(db_error)?;

        Ok(Page {
            items: self.categorize(models).await?,
            total,
            request,
        })
    }

    /// Tells whether another article already uses this slug.
    async fn slug_exists(&self, slug: &Slug, excluding: Option<ArticleId>) -> DomainResult<bool> {
        let mut query = article::Entity::find().filter(article::Column::Slug.eq(slug.as_str()));
        if let Some(id) = excluding {
            query = query.filter(article::Column::Id.ne(id.as_uuid()));
        }
        Ok(query.count(&self.db).await.map_err(db_error)? > 0)
    }

    /// Counts the articles attached to a category.
    async fn count_by_category(&self, category_id: CategoryId) -> DomainResult<u64> {
        article::Entity::find()
            .filter(article::Column::CategoryId.eq(category_id.as_uuid()))
            .count(&self.db)
            .await
            .map_err(db_error)
    }

    /// Inserts a new article.
    async fn create(&self, article: &Article) -> DomainResult<()> {
        from_article(article)
            .insert(&self.db)
            .await
            .map_err(db_error)?;
        Ok(())
    }

    /// Persists the changes of an existing article.
    async fn update(&self, article: &Article) -> DomainResult<()> {
        from_article(article)
            .update(&self.db)
            .await
            .map_err(db_error)?;
        Ok(())
    }

    /// Deletes an article (comments cascade in the database); returns `false`
    /// when it did not exist.
    async fn delete(&self, id: ArticleId) -> DomainResult<bool> {
        let result = article::Entity::delete_by_id(id.as_uuid())
            .exec(&self.db)
            .await
            .map_err(db_error)?;
        Ok(result.rows_affected > 0)
    }
}
