use std::sync::Arc;

use xetaravel_kernel::dto::{PageQuery, Paginated};
use xetaravel_kernel::{AppResult, Principal};

use crate::application::dto::UserDto;
use crate::domain::UserRepository;

/// Lists the registered users for the administration.
pub struct ListUsers {
    users: Arc<dyn UserRepository>,
}

impl ListUsers {
    /// Builds the use case with its dependencies.
    pub fn new(users: Arc<dyn UserRepository>) -> Self {
        Self { users }
    }

    /// Returns one page of users, newest first.
    pub async fn execute(
        &self,
        principal: Principal,
        query: PageQuery,
    ) -> AppResult<Paginated<UserDto>> {
        principal.require_admin()?;
        let page = self.users.list(query.to_page_request()).await?;
        Ok(Paginated::from_page(page, |user| (&user).into()))
    }
}

#[cfg(test)]
mod tests {
    use xetaravel_kernel::AppError;
    use xetaravel_kernel::pagination::Page;

    use super::*;
    use crate::application::test_support::{admin_principal, member_principal, user};
    use crate::domain::{MockUserRepository, Role};

    #[tokio::test]
    async fn lists_users() {
        let mut users = MockUserRepository::new();
        users.expect_list().returning(|request| {
            Ok(Page {
                items: vec![user("john", Role::Member)],
                total: 1,
                request,
            })
        });

        let result = ListUsers::new(Arc::new(users))
            .execute(admin_principal(), PageQuery::default())
            .await
            .unwrap();

        assert_eq!(result.items[0].username, "john");
    }

    #[tokio::test]
    async fn is_reserved_to_admins() {
        let error = ListUsers::new(Arc::new(MockUserRepository::new()))
            .execute(member_principal(), PageQuery::default())
            .await
            .unwrap_err();
        assert!(matches!(error, AppError::Forbidden(_)));
    }
}
