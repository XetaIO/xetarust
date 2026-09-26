use std::sync::Arc;

use xetaravel_kernel::{AppError, AppResult, Principal};

use crate::application::dto::UserDto;
use crate::domain::{UserId, UserRepository};

/// Returns the profile of the authenticated user.
pub struct GetCurrentUser {
    users: Arc<dyn UserRepository>,
}

impl GetCurrentUser {
    /// Builds the use case with its dependencies.
    pub fn new(users: Arc<dyn UserRepository>) -> Self {
        Self { users }
    }

    /// Loads the account of `principal`.
    pub async fn execute(&self, principal: Principal) -> AppResult<UserDto> {
        self.users
            .find_by_id(UserId::from(principal.user_id))
            .await?
            .map(|user| UserDto::from(&user))
            .ok_or_else(|| AppError::Unauthorized("unknown user".into()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::application::test_support::{principal_of, user};
    use crate::domain::{MockUserRepository, Role};

    #[tokio::test]
    async fn returns_the_profile_of_the_principal() {
        let john = user("john", Role::Member);
        let principal = principal_of(&john);
        let mut users = MockUserRepository::new();
        users
            .expect_find_by_id()
            .withf(move |id| id.as_uuid() == principal.user_id)
            .returning(move |_| Ok(Some(john.clone())));

        let dto = GetCurrentUser::new(Arc::new(users))
            .execute(principal)
            .await
            .unwrap();

        assert_eq!(dto.username, "john");
        assert_eq!(dto.email, "john@example.com");
    }
}
