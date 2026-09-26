use sea_orm_migration::{prelude::*, schema::*};

/// Creates the `user_role` enum and the `users` table.
#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    /// Applies the migration.
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .get_connection()
            .execute_unprepared("CREATE TYPE user_role AS ENUM ('member', 'admin')")
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(Users::Table)
                    .col(uuid(Users::Id).primary_key())
                    .col(string_len(Users::Username, 30))
                    .col(string_len(Users::Email, 254))
                    .col(string(Users::PasswordHash))
                    .col(
                        ColumnDef::new(Users::Role)
                            .custom(Alias::new("user_role"))
                            .not_null()
                            .default(Expr::cust("'member'")),
                    )
                    .col(timestamp_with_time_zone(Users::CreatedAt))
                    .col(timestamp_with_time_zone(Users::UpdatedAt))
                    .to_owned(),
            )
            .await?;

        // Case-insensitive uniqueness for usernames and emails.
        manager
            .get_connection()
            .execute_unprepared(
                "CREATE UNIQUE INDEX users_username_unique ON users (LOWER(username));
                 CREATE UNIQUE INDEX users_email_unique ON users (LOWER(email));",
            )
            .await?;

        Ok(())
    }

    /// Reverts the migration.
    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Users::Table).to_owned())
            .await?;
        manager
            .get_connection()
            .execute_unprepared("DROP TYPE user_role")
            .await?;
        Ok(())
    }
}

/// Identifiers of the `users` table.
#[derive(DeriveIden)]
pub enum Users {
    Table,
    Id,
    Username,
    Email,
    PasswordHash,
    Role,
    CreatedAt,
    UpdatedAt,
}
