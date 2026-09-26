//! PostgreSQL persistence of the Discussion context through SeaORM.

mod comment_repository;
mod entity;

pub use comment_repository::SeaOrmCommentRepository;
