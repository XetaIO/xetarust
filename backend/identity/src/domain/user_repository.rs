use async_trait::async_trait;

use xetaravel_kernel::DomainResult;
use xetaravel_kernel::pagination::{Page, PageRequest};

use super::{Email, User, UserId, Username};

/// Persistence port of the [`User`] aggregate.
#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait UserRepository: Send + Sync {
    /// Finds a user by id.
    async fn find_by_id(&self, id: UserId) -> DomainResult<Option<User>>;

    /// Finds the users matching the given ids (unknown ids are ignored).
    async fn find_by_ids(&self, ids: &[UserId]) -> DomainResult<Vec<User>>;

    /// Finds a user by (normalized) email.
    async fn find_by_email(&self, email: &Email) -> DomainResult<Option<User>>;

    /// Tells whether an account already uses this email.
    async fn email_exists(&self, email: &Email) -> DomainResult<bool>;

    /// Tells whether an account already uses this username (case-insensitive).
    async fn username_exists(&self, username: &Username) -> DomainResult<bool>;

    /// Lists users, newest first.
    async fn list(&self, page: PageRequest) -> DomainResult<Page<User>>;

    /// Inserts a new user.
    async fn create(&self, user: &User) -> DomainResult<()>;

    /// Persists the changes of an existing user.
    async fn update(&self, user: &User) -> DomainResult<()>;
}
