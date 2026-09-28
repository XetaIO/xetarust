use std::sync::Arc;

use uuid::Uuid;
use validator::Validate;
use xetaravel_kernel::text::non_blank;
use xetaravel_kernel::{AppError, AppResult, Clock, Principal};

use super::actor_role;
use crate::application::dto::{BanUserRequest, UserDto};
use crate::domain::{BanReason, UserId, UserRepository};

/// Bans an account until an admin lifts the ban.
pub struct BanUser {
    users: Arc<dyn UserRepository>,
    clock: Arc<dyn Clock>,
}

impl BanUser {
    /// Builds the use case with its dependencies.
    pub fn new(users: Arc<dyn UserRepository>, clock: Arc<dyn Clock>) -> Self {
        Self { users, clock }
    }

    /// Bans the user with an optional reason (the domain forbids banning
    /// oneself or an admin).
    pub async fn execute(
        &self,
        principal: Principal,
        user_id: Uuid,
        input: BanUserRequest,
    ) -> AppResult<UserDto> {
        principal.require_admin()?;
        input.validate()?;
        let reason = non_blank(input.reason.as_deref())
            .map(BanReason::parse)
            .transpose()?;

        let mut user = self
            .users
            .find_by_id(UserId::from(user_id))
            .await?
            .ok_or_else(|| AppError::NotFound("user not found".into()))?;

        user.ban(
            reason,
            UserId::from(principal.user_id),
            actor_role(&principal),
            self.clock.now(),
        )?;
        self.users.update(&user).await?;

        Ok((&user).into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::application::test_support::{
        admin_principal, clock, member_principal, now, principal_of, user,
    };
    use crate::domain::{MockUserRepository, Role, User};

    /// Returns a ban request with the given reason.
    fn request(reason: Option<&str>) -> BanUserRequest {
        BanUserRequest {
            reason: reason.map(Into::into),
        }
    }

    /// Returns a repository mock always finding `found`.
    fn users_finding(found: User) -> MockUserRepository {
        let mut users = MockUserRepository::new();
        users
            .expect_find_by_id()
            .returning(move |_| Ok(Some(found.clone())));
        users
    }

    #[tokio::test]
    async fn bans_a_member_with_a_trimmed_reason() {
        let mut users = users_finding(user("john", Role::Member));
        users
            .expect_update()
            .withf(|u| u.is_banned())
            .times(1)
            .returning(|_| Ok(()));

        let dto = BanUser::new(Arc::new(users), clock())
            .execute(admin_principal(), Uuid::now_v7(), request(Some(" spam ")))
            .await
            .unwrap();

        assert_eq!(dto.banned_at, Some(now()));
        assert_eq!(dto.ban_reason.as_deref(), Some("spam"));
    }

    #[tokio::test]
    async fn a_blank_reason_bans_without_reason() {
        let mut users = users_finding(user("john", Role::Member));
        users.expect_update().times(1).returning(|_| Ok(()));

        let dto = BanUser::new(Arc::new(users), clock())
            .execute(admin_principal(), Uuid::now_v7(), request(Some("   ")))
            .await
            .unwrap();

        assert_eq!(dto.banned_at, Some(now()));
        assert_eq!(dto.ban_reason, None);
    }

    #[tokio::test]
    async fn rejects_a_too_long_reason() {
        let long = "a".repeat(256);

        let error = BanUser::new(Arc::new(MockUserRepository::new()), clock())
            .execute(admin_principal(), Uuid::now_v7(), request(Some(&long)))
            .await
            .unwrap_err();

        assert!(matches!(error, AppError::Validation(_)));
    }

    #[tokio::test]
    async fn refuses_to_ban_an_admin() {
        let users = users_finding(user("boss", Role::Admin));

        let error = BanUser::new(Arc::new(users), clock())
            .execute(admin_principal(), Uuid::now_v7(), request(None))
            .await
            .unwrap_err();

        assert!(matches!(error, AppError::Forbidden(_)));
    }

    #[tokio::test]
    async fn refuses_to_ban_oneself() {
        let admin = user("admin", Role::Admin);
        let principal = principal_of(&admin);
        let users = users_finding(admin);

        let error = BanUser::new(Arc::new(users), clock())
            .execute(principal, principal.user_id, request(None))
            .await
            .unwrap_err();

        assert!(matches!(error, AppError::Forbidden(_)));
    }

    #[tokio::test]
    async fn members_cannot_ban() {
        let error = BanUser::new(Arc::new(MockUserRepository::new()), clock())
            .execute(member_principal(), Uuid::now_v7(), request(None))
            .await
            .unwrap_err();

        assert!(matches!(error, AppError::Forbidden(_)));
    }

    #[tokio::test]
    async fn missing_user_is_not_found() {
        let mut users = MockUserRepository::new();
        users.expect_find_by_id().returning(|_| Ok(None));

        let error = BanUser::new(Arc::new(users), clock())
            .execute(admin_principal(), Uuid::now_v7(), request(None))
            .await
            .unwrap_err();

        assert!(matches!(error, AppError::NotFound(_)));
    }
}
