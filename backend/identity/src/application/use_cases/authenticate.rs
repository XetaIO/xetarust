use std::sync::Arc;

use async_trait::async_trait;
use xetaravel_kernel::{AppError, AppResult, Principal, PrincipalResolver};

use super::principal_of;
use crate::application::ports::TokenService;
use crate::domain::UserRepository;

/// Resolves a bearer token into the [`Principal`] performing a request.
///
/// The user is reloaded from storage so a role change, a ban or a deleted
/// account takes effect immediately, even if the token is still valid. This use case
/// is the Identity implementation of the kernel [`PrincipalResolver`] port.
pub struct Authenticate {
    users: Arc<dyn UserRepository>,
    tokens: Arc<dyn TokenService>,
}

impl Authenticate {
    /// Builds the use case with its dependencies.
    pub fn new(users: Arc<dyn UserRepository>, tokens: Arc<dyn TokenService>) -> Self {
        Self { users, tokens }
    }

    /// Verifies the token and returns the up-to-date principal.
    pub async fn execute(&self, token: &str) -> AppResult<Principal> {
        let user_id = self.tokens.verify(token)?;
        let user = self
            .users
            .find_by_id(user_id)
            .await?
            .ok_or_else(|| AppError::Unauthorized("unknown user".into()))?;
        if user.is_banned() {
            return Err(AppError::Unauthorized("account banned".into()));
        }

        Ok(principal_of(&user))
    }
}

#[async_trait]
impl PrincipalResolver for Authenticate {
    /// Delegates to [`Authenticate::execute`].
    async fn resolve(&self, token: &str) -> AppResult<Principal> {
        self.execute(token).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::application::ports::MockTokenService;
    use crate::application::test_support::{now, user};
    use crate::domain::{MockUserRepository, Role, UserId};

    #[tokio::test]
    async fn returns_the_principal_with_its_current_role() {
        let admin = user("admin", Role::Admin);
        let admin_id = admin.id;
        let mut tokens = MockTokenService::new();
        tokens.expect_verify().returning(move |_| Ok(admin_id));
        let mut users = MockUserRepository::new();
        users
            .expect_find_by_id()
            .returning(move |_| Ok(Some(admin.clone())));

        let principal = Authenticate::new(Arc::new(users), Arc::new(tokens))
            .resolve("jwt")
            .await
            .unwrap();

        assert_eq!(principal.user_id, admin_id.as_uuid());
        assert!(principal.is_admin);
    }

    #[tokio::test]
    async fn rejects_invalid_tokens() {
        let mut tokens = MockTokenService::new();
        tokens
            .expect_verify()
            .returning(|_| Err(AppError::Unauthorized("invalid token".into())));

        let error = Authenticate::new(Arc::new(MockUserRepository::new()), Arc::new(tokens))
            .execute("bad")
            .await
            .unwrap_err();

        assert!(matches!(error, AppError::Unauthorized(_)));
    }

    #[tokio::test]
    async fn rejects_tokens_of_deleted_users() {
        let mut tokens = MockTokenService::new();
        tokens.expect_verify().returning(|_| Ok(UserId::generate()));
        let mut users = MockUserRepository::new();
        users.expect_find_by_id().returning(|_| Ok(None));

        let error = Authenticate::new(Arc::new(users), Arc::new(tokens))
            .execute("jwt")
            .await
            .unwrap_err();

        assert!(matches!(error, AppError::Unauthorized(_)));
    }

    #[tokio::test]
    async fn rejects_tokens_of_banned_users() {
        let mut banned = user("john", Role::Member);
        banned
            .ban(None, UserId::generate(), Role::Admin, now())
            .unwrap();
        let banned_id = banned.id;
        let mut tokens = MockTokenService::new();
        tokens.expect_verify().returning(move |_| Ok(banned_id));
        let mut users = MockUserRepository::new();
        users
            .expect_find_by_id()
            .returning(move |_| Ok(Some(banned.clone())));

        let error = Authenticate::new(Arc::new(users), Arc::new(tokens))
            .execute("jwt")
            .await
            .unwrap_err();

        assert_eq!(error, AppError::Unauthorized("account banned".into()));
    }
}
