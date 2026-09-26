//! PostgreSQL persistence of the Publishing context through SeaORM.

mod article_repository;
mod category_repository;
mod entities;
mod mappers;

pub use article_repository::SeaOrmArticleRepository;
pub use category_repository::SeaOrmCategoryRepository;
