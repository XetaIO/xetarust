//! Schema migrations owned by the Publishing context (`categories` and
//! `articles` tables). They must run after the Identity migrations:
//! `articles.author_id` references `users` (a deliberate cross-context
//! foreign key guaranteeing integrity in the single database).

use sea_orm_migration::MigrationTrait;

mod m20260927_publishing_000001_create_categories;
mod m20260927_publishing_000002_create_articles;
mod m20260927_publishing_000003_add_article_cover;
mod m20260928_publishing_000004_add_article_comments_enabled;

/// Returns the migrations of the Publishing context, oldest first.
pub fn migrations() -> Vec<Box<dyn MigrationTrait>> {
    vec![
        Box::new(m20260927_publishing_000001_create_categories::Migration),
        Box::new(m20260927_publishing_000002_create_articles::Migration),
        Box::new(m20260927_publishing_000003_add_article_cover::Migration),
        Box::new(m20260928_publishing_000004_add_article_comments_enabled::Migration),
    ]
}
