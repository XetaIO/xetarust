use async_trait::async_trait;

use xetaravel_kernel::DomainResult;
use xetaravel_kernel::pagination::{Page, PageRequest};

use super::{Article, ArticleFilter, ArticleId, CategorizedArticle, CategoryId, Slug};

/// Persistence port of the [`Article`] aggregate.
#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait ArticleRepository: Send + Sync {
    /// Finds an article by id.
    async fn find_by_id(&self, id: ArticleId) -> DomainResult<Option<Article>>;

    /// Finds an article with its category by id.
    async fn find_categorized_by_id(
        &self,
        id: ArticleId,
    ) -> DomainResult<Option<CategorizedArticle>>;

    /// Finds an article with its category by slug.
    async fn find_categorized_by_slug(
        &self,
        slug: &Slug,
    ) -> DomainResult<Option<CategorizedArticle>>;

    /// Lists articles (with their category) matching `filter`. Published
    /// articles are ordered by publication date, then drafts by creation
    /// date (newest first).
    async fn list_categorized(
        &self,
        filter: ArticleFilter,
        page: PageRequest,
    ) -> DomainResult<Page<CategorizedArticle>>;

    /// Tells whether another article (other than `excluding`) already uses this slug.
    async fn slug_exists(&self, slug: &Slug, excluding: Option<ArticleId>) -> DomainResult<bool>;

    /// Counts the articles (drafts included) attached to a category.
    async fn count_by_category(&self, category_id: CategoryId) -> DomainResult<u64>;

    /// Inserts a new article.
    async fn create(&self, article: &Article) -> DomainResult<()>;

    /// Persists the changes of an existing article.
    async fn update(&self, article: &Article) -> DomainResult<()>;

    /// Deletes an article. Returns `false` when it did not exist.
    async fn delete(&self, id: ArticleId) -> DomainResult<bool>;
}
