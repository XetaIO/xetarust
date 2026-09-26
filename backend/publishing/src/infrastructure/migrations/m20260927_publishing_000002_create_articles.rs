use sea_orm_migration::{prelude::*, schema::*};

use super::m20260927_publishing_000001_create_categories::Categories;

/// Table owned by the Identity context, referenced by id only.
const USERS: &str = "users";

/// Creates the `articles` table.
#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    /// Applies the migration.
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Articles::Table)
                    .col(uuid(Articles::Id).primary_key())
                    .col(uuid(Articles::AuthorId))
                    .col(uuid(Articles::CategoryId))
                    .col(string_len(Articles::Title, 200))
                    .col(string_len_uniq(Articles::Slug, 200))
                    .col(string_len_null(Articles::Excerpt, 500))
                    .col(text(Articles::Content))
                    .col(timestamp_with_time_zone_null(Articles::PublishedAt))
                    .col(timestamp_with_time_zone(Articles::CreatedAt))
                    .col(timestamp_with_time_zone(Articles::UpdatedAt))
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_articles_author")
                            .from(Articles::Table, Articles::AuthorId)
                            .to(Alias::new(USERS), Alias::new("id"))
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_articles_category")
                            .from(Articles::Table, Articles::CategoryId)
                            .to(Categories::Table, Categories::Id)
                            .on_delete(ForeignKeyAction::Restrict),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_articles_published_at")
                    .table(Articles::Table)
                    .col(Articles::PublishedAt)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_articles_category_id")
                    .table(Articles::Table)
                    .col(Articles::CategoryId)
                    .to_owned(),
            )
            .await
    }

    /// Reverts the migration.
    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Articles::Table).to_owned())
            .await
    }
}

/// Identifiers of the `articles` table.
#[derive(DeriveIden)]
pub enum Articles {
    Table,
    Id,
    AuthorId,
    CategoryId,
    Title,
    Slug,
    Excerpt,
    Content,
    PublishedAt,
    CreatedAt,
    UpdatedAt,
}
