//! Author names for Publishing and Discussion, served by Identity.

use std::collections::HashMap;
use std::hash::Hash;
use std::sync::Arc;

use async_trait::async_trait;
use uuid::Uuid;
use xetaravel_discussion::domain::AuthorId as DiscussionAuthorId;
use xetaravel_identity::IdentityDirectory;
use xetaravel_kernel::AppResult;
use xetaravel_publishing::domain::AuthorId as PublishingAuthorId;

/// Implements the `AuthorDirectory` port of Publishing and Discussion on top
/// of the Identity public directory, translating ids at the boundary.
pub struct IdentityAuthorDirectory {
    identity: Arc<dyn IdentityDirectory>,
}

impl IdentityAuthorDirectory {
    /// Wraps the Identity public directory.
    pub fn new(identity: Arc<dyn IdentityDirectory>) -> Self {
        Self { identity }
    }

    /// Looks the ids up in Identity and re-keys the names with the local id type.
    async fn names_by<Id>(
        &self,
        ids: &[Id],
        to_uuid: fn(&Id) -> Uuid,
    ) -> AppResult<HashMap<Id, String>>
    where
        Id: Copy + Eq + Hash + From<Uuid>,
    {
        let uuids: Vec<Uuid> = ids.iter().map(to_uuid).collect();
        let names = self.identity.public_names(&uuids).await?;
        Ok(names
            .into_iter()
            .map(|(id, name)| (Id::from(id), name))
            .collect())
    }
}

#[async_trait]
impl xetaravel_publishing::AuthorDirectory for IdentityAuthorDirectory {
    /// Names the authors of articles.
    async fn names(
        &self,
        ids: &[PublishingAuthorId],
    ) -> AppResult<HashMap<PublishingAuthorId, String>> {
        self.names_by(ids, PublishingAuthorId::as_uuid).await
    }
}

#[async_trait]
impl xetaravel_discussion::AuthorDirectory for IdentityAuthorDirectory {
    /// Names the authors of comments.
    async fn names(
        &self,
        ids: &[DiscussionAuthorId],
    ) -> AppResult<HashMap<DiscussionAuthorId, String>> {
        self.names_by(ids, DiscussionAuthorId::as_uuid).await
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Mutex;

    use super::*;

    /// Identity directory knowing a single user, recording the requested ids.
    struct FakeIdentity {
        known: Uuid,
        requested: Mutex<Vec<Uuid>>,
    }

    #[async_trait]
    impl IdentityDirectory for FakeIdentity {
        /// Returns "xety" for the known user only.
        async fn public_names(&self, ids: &[Uuid]) -> AppResult<HashMap<Uuid, String>> {
            self.requested.lock().unwrap().extend_from_slice(ids);
            Ok(ids
                .iter()
                .filter(|id| **id == self.known)
                .map(|id| (*id, "xety".to_owned()))
                .collect())
        }
    }

    /// Builds the adapter over a fake knowing `known`.
    fn directory(known: Uuid) -> (IdentityAuthorDirectory, Arc<FakeIdentity>) {
        let fake = Arc::new(FakeIdentity {
            known,
            requested: Mutex::new(vec![]),
        });
        (IdentityAuthorDirectory::new(fake.clone()), fake)
    }

    #[tokio::test]
    async fn names_publishing_authors() {
        let known = Uuid::now_v7();
        let (directory, fake) = directory(known);
        let ghost = PublishingAuthorId::generate();

        let names = xetaravel_publishing::AuthorDirectory::names(
            &directory,
            &[PublishingAuthorId::from(known), ghost],
        )
        .await
        .unwrap();

        assert_eq!(names.len(), 1);
        assert_eq!(names[&PublishingAuthorId::from(known)], "xety");
        assert_eq!(*fake.requested.lock().unwrap(), [known, ghost.as_uuid()]);
    }

    #[tokio::test]
    async fn names_discussion_authors() {
        let known = Uuid::now_v7();
        let (directory, _) = directory(known);

        let names = xetaravel_discussion::AuthorDirectory::names(
            &directory,
            &[DiscussionAuthorId::from(known)],
        )
        .await
        .unwrap();

        assert_eq!(names[&DiscussionAuthorId::from(known)], "xety");
    }
}
