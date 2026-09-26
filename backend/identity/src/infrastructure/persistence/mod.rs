//! PostgreSQL persistence of the Identity context through SeaORM.

mod entity;
mod mappers;
mod user_repository;

pub use user_repository::SeaOrmUserRepository;
