use sea_orm_migration::{prelude::*, schema::*};

use super::m20260927_publishing_000002_create_articles::Articles;

/// Adds the nullable `cover_image` column (stored file name) to `articles`.
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
                    .add_column(string_len_null(ArticleCover::CoverImage, 64))
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
                    .drop_column(ArticleCover::CoverImage)
                    .to_owned(),
            )
            .await
    }
}

/// Column added to the `articles` table.
#[derive(DeriveIden)]
enum ArticleCover {
    CoverImage,
}
