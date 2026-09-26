use async_trait::async_trait;
use sea_orm::sea_query::{Expr, ExprTrait, Func};
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, PaginatorTrait, QueryFilter,
    QueryOrder,
};
use xetaravel_kernel::DomainResult;
use xetaravel_kernel::pagination::{Page, PageRequest};
use xetaravel_kernel::persistence::db_error;

use super::entity as user;
use super::mappers::{from_user, to_user};
use crate::domain::{Email, User, UserId, UserRepository, Username};

/// PostgreSQL implementation of [`UserRepository`].
#[derive(Clone)]
pub struct SeaOrmUserRepository {
    db: DatabaseConnection,
}

impl SeaOrmUserRepository {
    /// Builds the repository on top of a connection pool.
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }
}

#[async_trait]
impl UserRepository for SeaOrmUserRepository {
    /// Finds a user by id.
    async fn find_by_id(&self, id: UserId) -> DomainResult<Option<User>> {
        user::Entity::find_by_id(id.as_uuid())
            .one(&self.db)
            .await
            .map_err(db_error)?
            .map(to_user)
            .transpose()
    }

    /// Finds the users of the given ids in a single query.
    async fn find_by_ids(&self, ids: &[UserId]) -> DomainResult<Vec<User>> {
        if ids.is_empty() {
            return Ok(Vec::new());
        }
        user::Entity::find()
            .filter(user::Column::Id.is_in(ids.iter().map(UserId::as_uuid)))
            .all(&self.db)
            .await
            .map_err(db_error)?
            .into_iter()
            .map(to_user)
            .collect()
    }

    /// Finds a user by email (emails are stored lower-cased).
    async fn find_by_email(&self, email: &Email) -> DomainResult<Option<User>> {
        user::Entity::find()
            .filter(user::Column::Email.eq(email.as_str()))
            .one(&self.db)
            .await
            .map_err(db_error)?
            .map(to_user)
            .transpose()
    }

    /// Tells whether an account already uses this email.
    async fn email_exists(&self, email: &Email) -> DomainResult<bool> {
        let count = user::Entity::find()
            .filter(user::Column::Email.eq(email.as_str()))
            .count(&self.db)
            .await
            .map_err(db_error)?;
        Ok(count > 0)
    }

    /// Tells whether an account already uses this username, ignoring case.
    async fn username_exists(&self, username: &Username) -> DomainResult<bool> {
        let count = user::Entity::find()
            .filter(ExprTrait::eq(
                Func::lower(Expr::col(user::Column::Username)),
                username.as_str().to_lowercase(),
            ))
            .count(&self.db)
            .await
            .map_err(db_error)?;
        Ok(count > 0)
    }

    /// Lists users, newest first.
    async fn list(&self, request: PageRequest) -> DomainResult<Page<User>> {
        let paginator = user::Entity::find()
            .order_by_desc(user::Column::CreatedAt)
            .order_by_desc(user::Column::Id)
            .paginate(&self.db, request.per_page());

        let total = paginator.num_items().await.map_err(db_error)?;
        let items = paginator
            .fetch_page(request.page() - 1)
            .await
            .map_err(db_error)?
            .into_iter()
            .map(to_user)
            .collect::<DomainResult<_>>()?;

        Ok(Page {
            items,
            total,
            request,
        })
    }

    /// Inserts a new user.
    async fn create(&self, user: &User) -> DomainResult<()> {
        from_user(user).insert(&self.db).await.map_err(db_error)?;
        Ok(())
    }

    /// Persists the changes of an existing user.
    async fn update(&self, user: &User) -> DomainResult<()> {
        from_user(user).update(&self.db).await.map_err(db_error)?;
        Ok(())
    }
}
