use std::sync::Arc;

use xetaravel_kernel::{AppError, AppResult, Clock};

use crate::application::dto::UserDto;
use crate::domain::{Email, UserRepository};

/// Promotes an account to admin by email.
///
/// Trusted system operation used by the CLI to bootstrap the first admin:
/// it performs no actor check and must never be exposed over HTTP.
pub struct PromoteUserToAdmin {
    users: Arc<dyn UserRepository>,
    clock: Arc<dyn Clock>,
}

impl PromoteUserToAdmin {
    /// Builds the use case with its dependencies.
    pub fn new(users: Arc<dyn UserRepository>, clock: Arc<dyn Clock>) -> Self {
        Self { users, clock }
    }

    /// Finds the account by email and makes it an admin.
    pub async fn execute(&self, email: &str) -> AppResult<UserDto> {
        let email = Email::parse(email)?;
        let mut user = self
            .users
            .find_by_email(&email)
            .await?
            .ok_or_else(|| AppError::NotFound(format!("no account uses {email}")))?;

        user.promote_to_admin(self.clock.now());
        self.users.update(&user).await?;

        Ok((&user).into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::application::dto::RoleDto;
    use crate::application::test_support::{clock, user};
    use crate::domain::{MockUserRepository, Role};

    #[tokio::test]
    async fn promotes_the_account() {
        let mut users = MockUserRepository::new();
        users
            .expect_find_by_email()
            .returning(|_| Ok(Some(user("john", Role::Member))));
        users
            .expect_update()
            .withf(|u| u.is_admin())
            .times(1)
            .returning(|_| Ok(()));

        let dto = PromoteUserToAdmin::new(Arc::new(users), clock())
            .execute("john@example.com")
            .await
            .unwrap();

        assert_eq!(dto.role, RoleDto::Admin);
    }

    #[tokio::test]
    async fn unknown_email_is_not_found() {
        let mut users = MockUserRepository::new();
        users.expect_find_by_email().returning(|_| Ok(None));

        let error = PromoteUserToAdmin::new(Arc::new(users), clock())
            .execute("ghost@example.com")
            .await
            .unwrap_err();

        assert!(matches!(error, AppError::NotFound(_)));
    }
}
