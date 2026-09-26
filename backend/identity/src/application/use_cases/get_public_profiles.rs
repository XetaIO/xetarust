use std::collections::HashMap;
use std::sync::Arc;

use async_trait::async_trait;
use uuid::Uuid;
use xetaravel_kernel::AppResult;

use crate::application::contract::IdentityDirectory;
use crate::domain::{UserId, UserRepository};

/// Returns the public names of a batch of users (used by other contexts to
/// display authors). This use case implements the [`IdentityDirectory`] contract.
pub struct GetPublicProfiles {
    users: Arc<dyn UserRepository>,
}

impl GetPublicProfiles {
    /// Builds the use case with its dependencies.
    pub fn new(users: Arc<dyn UserRepository>) -> Self {
        Self { users }
    }

    /// Loads the users of `ids` in one query and maps each id to its username.
    pub async fn execute(&self, ids: &[Uuid]) -> AppResult<HashMap<Uuid, String>> {
        let mut ids: Vec<UserId> = ids.iter().copied().map(UserId::from).collect();
        ids.sort_unstable();
        ids.dedup();
        if ids.is_empty() {
            return Ok(HashMap::new());
        }

        let users = self.users.find_by_ids(&ids).await?;
        Ok(users
            .into_iter()
            .map(|user| (user.id.as_uuid(), user.username.to_string()))
            .collect())
    }
}

#[async_trait]
impl IdentityDirectory for GetPublicProfiles {
    /// Delegates to [`GetPublicProfiles::execute`].
    async fn public_names(&self, ids: &[Uuid]) -> AppResult<HashMap<Uuid, String>> {
        self.execute(ids).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::application::test_support::user;
    use crate::domain::{MockUserRepository, Role};

    #[tokio::test]
    async fn maps_known_ids_to_usernames_in_one_query() {
        let john = user("john", Role::Member);
        let john_id = john.id;
        let ghost = Uuid::now_v7();
        let mut users = MockUserRepository::new();
        users
            .expect_find_by_ids()
            .withf(move |ids| ids.len() == 2 && ids.contains(&john_id))
            .times(1)
            .returning(move |_| Ok(vec![john.clone()]));

        let names = GetPublicProfiles::new(Arc::new(users))
            .public_names(&[john_id.as_uuid(), ghost, john_id.as_uuid()])
            .await
            .unwrap();

        assert_eq!(names.len(), 1);
        assert_eq!(names[&john_id.as_uuid()], "john");
    }

    #[tokio::test]
    async fn skips_storage_for_empty_batches() {
        let names = GetPublicProfiles::new(Arc::new(MockUserRepository::new()))
            .execute(&[])
            .await
            .unwrap();
        assert!(names.is_empty());
    }
}
