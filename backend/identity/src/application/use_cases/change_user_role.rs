use std::sync::Arc;

use uuid::Uuid;
use xetaravel_kernel::{AppError, AppResult, Clock, Principal};

use crate::application::dto::{ChangeRoleRequest, UserDto};
use crate::domain::{Role, UserId, UserRepository};

/// Promotes a member to admin or demotes an admin to member.
pub struct ChangeUserRole {
    users: Arc<dyn UserRepository>,
    clock: Arc<dyn Clock>,
}

impl ChangeUserRole {
    /// Builds the use case with its dependencies.
    pub fn new(users: Arc<dyn UserRepository>, clock: Arc<dyn Clock>) -> Self {
        Self { users, clock }
    }

    /// Changes the role of the user (the domain forbids self-demotion).
    pub async fn execute(
        &self,
        principal: Principal,
        user_id: Uuid,
        input: ChangeRoleRequest,
    ) -> AppResult<UserDto> {
        principal.require_admin()?;
        let mut user = self
            .users
            .find_by_id(UserId::from(user_id))
            .await?
            .ok_or_else(|| AppError::NotFound("user not found".into()))?;

        let actor_role = if principal.is_admin {
            Role::Admin
        } else {
            Role::Member
        };
        user.change_role(
            input.role.into(),
            UserId::from(principal.user_id),
            actor_role,
            self.clock.now(),
        )?;
        self.users.update(&user).await?;

        Ok((&user).into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::application::dto::RoleDto;
    use crate::application::test_support::{admin_principal, clock, principal_of, user};
    use crate::domain::MockUserRepository;

    #[tokio::test]
    async fn promotes_a_member() {
        let mut users = MockUserRepository::new();
        users
            .expect_find_by_id()
            .returning(|_| Ok(Some(user("john", Role::Member))));
        users
            .expect_update()
            .withf(|u| u.role == Role::Admin)
            .times(1)
            .returning(|_| Ok(()));

        let dto = ChangeUserRole::new(Arc::new(users), clock())
            .execute(
                admin_principal(),
                Uuid::now_v7(),
                ChangeRoleRequest {
                    role: RoleDto::Admin,
                },
            )
            .await
            .unwrap();

        assert_eq!(dto.role, RoleDto::Admin);
    }

    #[tokio::test]
    async fn admin_cannot_demote_themselves() {
        let admin = user("admin", Role::Admin);
        let principal = principal_of(&admin);
        let mut users = MockUserRepository::new();
        users
            .expect_find_by_id()
            .returning(move |_| Ok(Some(admin.clone())));

        let error = ChangeUserRole::new(Arc::new(users), clock())
            .execute(
                principal,
                principal.user_id,
                ChangeRoleRequest {
                    role: RoleDto::Member,
                },
            )
            .await
            .unwrap_err();

        assert!(matches!(error, AppError::Forbidden(_)));
    }

    #[tokio::test]
    async fn missing_user_is_not_found() {
        let mut users = MockUserRepository::new();
        users.expect_find_by_id().returning(|_| Ok(None));

        let error = ChangeUserRole::new(Arc::new(users), clock())
            .execute(
                admin_principal(),
                Uuid::now_v7(),
                ChangeRoleRequest {
                    role: RoleDto::Admin,
                },
            )
            .await
            .unwrap_err();

        assert!(matches!(error, AppError::NotFound(_)));
    }
}
