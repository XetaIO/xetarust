//! Driven adapters of the Publishing context: SeaORM repositories, the
//! schema migrations of the `categories` and `articles` tables and the
//! cover image storage.

pub mod migrations;
pub mod persistence;
pub mod storage;
