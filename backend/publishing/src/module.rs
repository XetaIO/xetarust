//! Assembly of the Publishing context: builds its adapters once and injects
//! them into every use case.

use std::sync::Arc;

use sea_orm::DatabaseConnection;
use xetaravel_kernel::Clock;

use crate::application::contract::PublishedArticles;
use crate::application::ports::AuthorDirectory;
use crate::application::use_cases::{
    CreateArticle, CreateCategory, DeleteArticle, DeleteCategory, FindPublishedArticle, GetArticle,
    GetPublishedArticle, ListArticles, ListCategories, ListPublishedArticles, UpdateArticle,
    UpdateCategory,
};
use crate::domain::{ArticleRepository, CategoryRepository};
use crate::infrastructure::persistence::{SeaOrmArticleRepository, SeaOrmCategoryRepository};

/// Every use case of the Publishing context.
pub struct PublishingModule {
    pub list_published_articles: ListPublishedArticles,
    pub get_published_article: GetPublishedArticle,
    pub list_categories: ListCategories,
    pub list_articles: ListArticles,
    pub get_article: GetArticle,
    pub create_article: CreateArticle,
    pub update_article: UpdateArticle,
    pub delete_article: DeleteArticle,
    pub create_category: CreateCategory,
    pub update_category: UpdateCategory,
    pub delete_category: DeleteCategory,
    find_published_article: Arc<FindPublishedArticle>,
}

impl PublishingModule {
    /// Wires the PostgreSQL repositories and the given outgoing ports into
    /// the use cases. `authors` is provided by the composition root (ACL).
    pub fn new(
        db: DatabaseConnection,
        clock: Arc<dyn Clock>,
        authors: Arc<dyn AuthorDirectory>,
    ) -> Self {
        let articles: Arc<dyn ArticleRepository> =
            Arc::new(SeaOrmArticleRepository::new(db.clone()));
        let categories: Arc<dyn CategoryRepository> = Arc::new(SeaOrmCategoryRepository::new(db));

        Self {
            list_published_articles: ListPublishedArticles::new(articles.clone(), authors.clone()),
            get_published_article: GetPublishedArticle::new(articles.clone(), authors.clone()),
            list_categories: ListCategories::new(categories.clone()),
            list_articles: ListArticles::new(articles.clone(), authors.clone()),
            get_article: GetArticle::new(articles.clone(), authors.clone()),
            create_article: CreateArticle::new(
                articles.clone(),
                categories.clone(),
                authors.clone(),
                clock.clone(),
            ),
            update_article: UpdateArticle::new(
                articles.clone(),
                categories.clone(),
                authors,
                clock.clone(),
            ),
            delete_article: DeleteArticle::new(articles.clone()),
            create_category: CreateCategory::new(categories.clone(), clock.clone()),
            update_category: UpdateCategory::new(categories.clone(), clock),
            delete_category: DeleteCategory::new(categories, articles.clone()),
            find_published_article: Arc::new(FindPublishedArticle::new(articles)),
        }
    }

    /// Returns the public catalog other contexts use to find published articles.
    pub fn published_articles(&self) -> Arc<dyn PublishedArticles> {
        self.find_published_article.clone()
    }
}
