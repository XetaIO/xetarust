//! PostgreSQL schema of Xetaravel.
//!
//! Every bounded context owns its tables and migrations; this crate only
//! aggregates them in dependency order: Identity (`users`) → Publishing
//! (`categories`, `articles`) → Discussion (`comments`).
//!
//! Run them with `cargo run -p migration -- up` (see `-- --help` for all
//! commands). The application also applies pending migrations when it starts.

pub use sea_orm_migration::prelude::*;

#[cfg(feature = "test-support")]
pub mod testing;

/// Lists the migrations of every context in dependency order.
pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    /// Returns the migrations to apply: each context's list, oldest first.
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        let mut migrations = xetaravel_identity::migrations();
        migrations.extend(xetaravel_publishing::migrations());
        migrations.extend(xetaravel_discussion::migrations());
        migrations
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn contexts_are_migrated_in_dependency_order() {
        let names: Vec<String> = Migrator::migrations()
            .iter()
            .map(|m| m.name().to_owned())
            .collect();
        assert_eq!(
            names,
            [
                "m20260927_identity_000001_create_users",
                "m20260927_publishing_000001_create_categories",
                "m20260927_publishing_000002_create_articles",
                "m20260927_discussion_000001_create_comments",
            ]
        );
    }
}
