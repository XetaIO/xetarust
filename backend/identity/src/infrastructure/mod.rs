//! Driven adapters of the Identity context: SeaORM repository, Argon2
//! hashing, JWT tokens and the schema migrations of the `users` table.

pub mod migrations;
pub mod persistence;
pub mod security;
