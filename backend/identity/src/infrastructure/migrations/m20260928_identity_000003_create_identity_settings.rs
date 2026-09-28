use sea_orm_migration::{prelude::*, schema::*};

/// Creates the single-row `identity_settings` table (registrations open).
#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    /// Applies the migration.
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(IdentitySettings::Table)
                    .col(small_integer(IdentitySettings::Id).primary_key())
                    .col(boolean(IdentitySettings::RegistrationEnabled).default(true))
                    .col(timestamp_with_time_zone_null(IdentitySettings::UpdatedAt))
                    .check(Expr::col(IdentitySettings::Id).eq(1))
                    .to_owned(),
            )
            .await?;

        manager
            .exec_stmt(
                Query::insert()
                    .into_table(IdentitySettings::Table)
                    .columns([IdentitySettings::Id])
                    .values_panic([1.into()])
                    .to_owned(),
            )
            .await
    }

    /// Reverts the migration.
    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(IdentitySettings::Table).to_owned())
            .await
    }
}

/// Identifiers of the `identity_settings` table.
#[derive(DeriveIden)]
enum IdentitySettings {
    Table,
    Id,
    RegistrationEnabled,
    UpdatedAt,
}
