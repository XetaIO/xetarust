use sea_orm_migration::{prelude::*, schema::*};

/// Table owned by the Identity context, referenced by id only.
const USERS: &str = "users";
/// Table owned by the Publishing context, referenced by id only.
const ARTICLES: &str = "articles";

/// Creates the `comments` table.
#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    /// Applies the migration.
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Comments::Table)
                    .col(uuid(Comments::Id).primary_key())
                    .col(uuid(Comments::ArticleId))
                    .col(uuid(Comments::AuthorId))
                    .col(text(Comments::Content))
                    .col(timestamp_with_time_zone(Comments::CreatedAt))
                    .col(timestamp_with_time_zone(Comments::UpdatedAt))
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_comments_article")
                            .from(Comments::Table, Comments::ArticleId)
                            .to(Alias::new(ARTICLES), Alias::new("id"))
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_comments_author")
                            .from(Comments::Table, Comments::AuthorId)
                            .to(Alias::new(USERS), Alias::new("id"))
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_comments_article_id")
                    .table(Comments::Table)
                    .col(Comments::ArticleId)
                    .to_owned(),
            )
            .await
    }

    /// Reverts the migration.
    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Comments::Table).to_owned())
            .await
    }
}

/// Identifiers of the `comments` table.
#[derive(DeriveIden)]
pub enum Comments {
    Table,
    Id,
    ArticleId,
    AuthorId,
    Content,
    CreatedAt,
    UpdatedAt,
}
