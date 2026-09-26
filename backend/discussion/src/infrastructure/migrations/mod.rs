//! Schema migrations owned by the Discussion context (`comments` table).
//! They must run after the Identity and Publishing migrations: comments
//! reference `articles` (cascading deletion) and `users` by foreign key, a
//! deliberate cross-context compromise guaranteeing integrity in the single
//! database.

use sea_orm_migration::MigrationTrait;

mod m20260927_discussion_000001_create_comments;

/// Returns the migrations of the Discussion context, oldest first.
pub fn migrations() -> Vec<Box<dyn MigrationTrait>> {
    vec![Box::new(
        m20260927_discussion_000001_create_comments::Migration,
    )]
}
