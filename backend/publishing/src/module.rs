//! Assembly of the Publishing context: builds its adapters once and injects
//! them into every use case.

use std::path::PathBuf;
use std::sync::Arc;

use sea_orm::DatabaseConnection;
use xetaravel_kernel::Clock;

use crate::application::contract::PublishedArticles;
use crate::application::ports::{AuthorDirectory, CoverStorage};
use crate::application::use_cases::{
    CreateArticle, CreateCategory, DeleteArticle, DeleteCategory, FindPublishedArticle, GetArticle,
    GetCover, GetPublishedArticle, ListArticles, ListCategories, ListPublishedArticles,
    RemoveCover, UpdateArticle, UpdateCategory, UploadCover,
};
use crate::domain::{ArticleRepository, CategoryRepository};
use crate::infrastructure::persistence::{SeaOrmArticleRepository, SeaOrmCategoryRepository};
use crate::infrastructure::storage::FsCoverStorage;

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
    pub upload_cover: UploadCover,
    pub remove_cover: RemoveCover,
    pub get_cover: GetCover,
    pub create_category: CreateCategory,
    pub update_category: UpdateCategory,
    pub delete_category: DeleteCategory,
    find_published_article: Arc<FindPublishedArticle>,
}

impl PublishingModule {
    /// Wires the PostgreSQL repositories, the cover image storage (files kept
    /// in `covers_dir`) and the given outgoing ports into the use cases.
    /// `authors` is provided by the composition root (ACL).
    pub fn new(
        db: DatabaseConnection,
        clock: Arc<dyn Clock>,
        authors: Arc<dyn AuthorDirectory>,
        covers_dir: PathBuf,
    ) -> Self {
        let articles: Arc<dyn ArticleRepository> =
            Arc::new(SeaOrmArticleRepository::new(db.clone()));
        let categories: Arc<dyn CategoryRepository> = Arc::new(SeaOrmCategoryRepository::new(db));
        let covers: Arc<dyn CoverStorage> = Arc::new(FsCoverStorage::new(covers_dir));

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
                authors.clone(),
                clock.clone(),
            ),
            delete_article: DeleteArticle::new(articles.clone(), covers.clone()),
            upload_cover: UploadCover::new(
                articles.clone(),
                authors.clone(),
                covers.clone(),
                clock.clone(),
            ),
            remove_cover: RemoveCover::new(
                articles.clone(),
                authors,
                covers.clone(),
                clock.clone(),
            ),
            get_cover: GetCover::new(covers),
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
