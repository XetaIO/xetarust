use sea_orm_migration::{prelude::*, schema::*};

use super::m20260927_publishing_000002_create_articles::Articles;

/// Adds the `comments_enabled` flag to `articles`. Existing articles keep
/// their comments open (`DEFAULT true`).
#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    /// Applies the migration.
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(Articles::Table)
                    .add_column(boolean(ArticleComments::CommentsEnabled).default(true))
                    .to_owned(),
            )
            .await
    }

    /// Reverts the migration.
    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(Articles::Table)
                    .drop_column(ArticleComments::CommentsEnabled)
                    .to_owned(),
            )
            .await
    }
}

/// Column added to the `articles` table.
#[derive(DeriveIden)]
enum ArticleComments {
    CommentsEnabled,
}
