use sea_orm_migration::{prelude::*, schema::*};

/// Creates the `categories` table.
#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    /// Applies the migration.
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Categories::Table)
                    .col(uuid(Categories::Id).primary_key())
                    .col(string_len(Categories::Name, 100))
                    .col(string_len_uniq(Categories::Slug, 200))
                    .col(string_len_null(Categories::Description, 500))
                    .col(timestamp_with_time_zone(Categories::CreatedAt))
                    .col(timestamp_with_time_zone(Categories::UpdatedAt))
                    .to_owned(),
            )
            .await
    }

    /// Reverts the migration.
    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Categories::Table).to_owned())
            .await
    }
}

/// Identifiers of the `categories` table.
#[derive(DeriveIden)]
pub enum Categories {
    Table,
    Id,
    Name,
    Slug,
    Description,
    CreatedAt,
    UpdatedAt,
}
