use sea_orm_migration::{prelude::*, schema::*};

/// Adds the ban columns (`banned_at`, `ban_reason`) to the `users` table.
#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    /// Applies the migration.
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(Users::Table)
                    .add_column(timestamp_with_time_zone_null(Users::BannedAt))
                    .add_column(string_len_null(Users::BanReason, 255))
                    .to_owned(),
            )
            .await
    }

    /// Reverts the migration.
    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(Users::Table)
                    .drop_column(Users::BanReason)
                    .drop_column(Users::BannedAt)
                    .to_owned(),
            )
            .await
    }
}

/// Identifiers of the `users` table touched by this migration.
#[derive(DeriveIden)]
enum Users {
    Table,
    BannedAt,
    BanReason,
}
