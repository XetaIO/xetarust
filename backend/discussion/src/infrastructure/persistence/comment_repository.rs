use async_trait::async_trait;
use sea_orm::ActiveValue::Set;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, QueryOrder,
};
use xetaravel_kernel::DomainResult;
use xetaravel_kernel::persistence::db_error;

use super::entity as comment;
use crate::domain::{ArticleId, AuthorId, Comment, CommentId, CommentRepository};

/// PostgreSQL implementation of [`CommentRepository`].
#[derive(Clone)]
pub struct SeaOrmCommentRepository {
    db: DatabaseConnection,
}

impl SeaOrmCommentRepository {
    /// Builds the repository on top of a connection pool.
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }
}

/// Converts a `comments` row into a domain comment.
fn to_comment(model: comment::Model) -> Comment {
    Comment {
        id: model.id.into(),
        article_id: model.article_id.into(),
        author_id: model.author_id.into(),
        content: model.content,
        created_at: model.created_at,
        updated_at: model.updated_at,
    }
}

/// Converts a domain comment into a fully set active model.
fn from_comment(comment: &Comment) -> comment::ActiveModel {
    comment::ActiveModel {
        id: Set(comment.id.as_uuid()),
        article_id: Set(comment.article_id.as_uuid()),
        author_id: Set(comment.author_id.as_uuid()),
        content: Set(comment.content.clone()),
        created_at: Set(comment.created_at),
        updated_at: Set(comment.updated_at),
    }
}

#[async_trait]
impl CommentRepository for SeaOrmCommentRepository {
    /// Finds a comment by id.
    async fn find_by_id(&self, id: CommentId) -> DomainResult<Option<Comment>> {
        Ok(comment::Entity::find_by_id(id.as_uuid())
            .one(&self.db)
            .await
            .map_err(db_error)?
            .map(to_comment))
    }

    /// Lists the comments of an article, oldest first.
    async fn list_by_article(&self, article_id: ArticleId) -> DomainResult<Vec<Comment>> {
        Ok(comment::Entity::find()
            .filter(comment::Column::ArticleId.eq(article_id.as_uuid()))
            .order_by_asc(comment::Column::CreatedAt)
            .order_by_asc(comment::Column::Id)
            .all(&self.db)
            .await
            .map_err(db_error)?
            .into_iter()
            .map(to_comment)
            .collect())
    }

    /// Inserts a new comment.
    async fn create(&self, comment: &Comment) -> DomainResult<()> {
        from_comment(comment)
            .insert(&self.db)
            .await
            .map_err(db_error)?;
        Ok(())
    }

    /// Deletes a comment; returns `false` when it did not exist.
    async fn delete(&self, id: CommentId) -> DomainResult<bool> {
        let result = comment::Entity::delete_by_id(id.as_uuid())
            .exec(&self.db)
            .await
            .map_err(db_error)?;
        Ok(result.rows_affected > 0)
    }

    /// Deletes every comment written by `author_id`; returns how many were deleted.
    async fn delete_by_author(&self, author_id: AuthorId) -> DomainResult<u64> {
        let result = comment::Entity::delete_many()
            .filter(comment::Column::AuthorId.eq(author_id.as_uuid()))
            .exec(&self.db)
            .await
            .map_err(db_error)?;
        Ok(result.rows_affected)
    }
}
