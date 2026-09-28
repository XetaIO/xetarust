//! SeaORM entity of the `articles` table.

use sea_orm::entity::prelude::*;

/// Row of the `articles` table.
#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "articles")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub author_id: Uuid,
    pub category_id: Uuid,
    pub title: String,
    #[sea_orm(unique)]
    pub slug: String,
    pub excerpt: Option<String>,
    #[sea_orm(column_type = "Text")]
    pub content: String,
    pub cover_image: Option<String>,
    pub published_at: Option<DateTimeUtc>,
    pub comments_enabled: bool,
    pub created_at: DateTimeUtc,
    pub updated_at: DateTimeUtc,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
