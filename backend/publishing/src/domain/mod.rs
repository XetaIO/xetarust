//! Business rules of the Publishing context: the [`Article`] and
//! [`Category`] aggregates, their value objects and persistence ports.
//!
//! Framework-free: no SeaORM, no Axum, no serde.

mod article;
mod article_repository;
mod category;
mod category_repository;
mod ids;
mod read_models;
mod slug;

pub use article::{Article, ArticleDraft};
pub use article_repository::ArticleRepository;
pub use category::Category;
pub use category_repository::CategoryRepository;
pub use ids::{ArticleId, AuthorId, CategoryId};
pub use read_models::{ArticleFilter, CategorizedArticle};
pub use slug::Slug;

#[cfg(test)]
pub use article_repository::MockArticleRepository;
#[cfg(test)]
pub use category_repository::MockCategoryRepository;
