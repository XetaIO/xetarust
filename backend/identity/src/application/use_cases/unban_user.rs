use std::sync::Arc;

use uuid::Uuid;
use xetaravel_kernel::{AppError, AppResult, Clock, Principal};

use super::actor_role;
use crate::application::dto::UserDto;
use crate::domain::{UserId, UserRepository};

/// Lifts the ban of an account.
pub struct UnbanUser {
    users: Arc<dyn UserRepository>,
    clock: Arc<dyn Clock>,
}

impl UnbanUser {
    /// Builds the use case with its dependencies.
    pub fn new(users: Arc<dyn UserRepository>, clock: Arc<dyn Clock>) -> Self {
        Self { users, clock }
    }

    /// Unbans the user (a no-op when the account is not banned).
    pub async fn execute(&self, principal: Principal, user_id: Uuid) -> AppResult<UserDto> {
        principal.require_admin()?;
        let mut user = self
            .users
            .find_by_id(UserId::from(user_id))
            .await?
            .ok_or_else(|| AppError::NotFound("user not found".into()))?;

        user.unban(actor_role(&principal), self.clock.now())?;
        self.users.update(&user).await?;

        Ok((&user).into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::application::test_support::{admin_principal, clock, member_principal, now, user};
    use crate::domain::{MockUserRepository, Role};

    #[tokio::test]
    async fn unbans_a_banned_member() {
        let mut banned = user("john", Role::Member);
        banned
            .ban(None, UserId::generate(), Role::Admin, now())
            .unwrap();
        let mut users = MockUserRepository::new();
        users
            .expect_find_by_id()
            .returning(move |_| Ok(Some(banned.clone())));
        users
            .expect_update()
            .withf(|u| !u.is_banned())
            .times(1)
            .returning(|_| Ok(()));

        let dto = UnbanUser::new(Arc::new(users), clock())
            .execute(admin_principal(), Uuid::now_v7())
            .await
            .unwrap();

        assert_eq!(dto.banned_at, None);
    }

    #[tokio::test]
    async fn members_cannot_unban() {
        let error = UnbanUser::new(Arc::new(MockUserRepository::new()), clock())
            .execute(member_principal(), Uuid::now_v7())
            .await
            .unwrap_err();

        assert!(matches!(error, AppError::Forbidden(_)));
    }

    #[tokio::test]
    async fn missing_user_is_not_found() {
        let mut users = MockUserRepository::new();
        users.expect_find_by_id().returning(|_| Ok(None));

        let error = UnbanUser::new(Arc::new(users), clock())
            .execute(admin_principal(), Uuid::now_v7())
            .await
            .unwrap_err();

        assert!(matches!(error, AppError::NotFound(_)));
    }
}
