use std::sync::Arc;

use uuid::Uuid;
use xetaravel_kernel::{AppResult, Principal};

use crate::application::dto::DeletedCommentsDto;
use crate::domain::{AuthorId, CommentRepository};

/// Deletes every comment written by an author (moderation, e.g. after a ban).
pub struct DeleteAuthorComments {
    comments: Arc<dyn CommentRepository>,
}

impl DeleteAuthorComments {
    /// Builds the use case with its dependencies.
    pub fn new(comments: Arc<dyn CommentRepository>) -> Self {
        Self { comments }
    }

    /// Checks that `principal` is an admin and permanently deletes the
    /// comments of `author`. Returns how many comments were deleted.
    pub async fn execute(
        &self,
        principal: Principal,
        author: Uuid,
    ) -> AppResult<DeletedCommentsDto> {
        principal.require_admin()?;
        let deleted = self
            .comments
            .delete_by_author(AuthorId::from(author))
            .await?;
        Ok(DeletedCommentsDto { deleted })
    }
}

#[cfg(test)]
mod tests {
    use xetaravel_kernel::AppError;

    use super::*;
    use crate::application::test_support::{admin_principal, member_principal};
    use crate::domain::MockCommentRepository;

    #[tokio::test]
    async fn admin_deletes_the_comments_of_an_author() {
        let author = Uuid::now_v7();
        let mut comments = MockCommentRepository::new();
        comments
            .expect_delete_by_author()
            .withf(move |id| *id == AuthorId::from(author))
            .times(1)
            .returning(|_| Ok(3));

        let dto = DeleteAuthorComments::new(Arc::new(comments))
            .execute(admin_principal(), author)
            .await
            .unwrap();

        assert_eq!(dto, DeletedCommentsDto { deleted: 3 });
    }

    #[tokio::test]
    async fn members_cannot_delete_the_comments_of_an_author() {
        let mut comments = MockCommentRepository::new();
        comments.expect_delete_by_author().times(0);

        let error = DeleteAuthorComments::new(Arc::new(comments))
            .execute(member_principal(), Uuid::now_v7())
            .await
            .unwrap_err();

        assert!(matches!(error, AppError::Forbidden(_)));
    }
}
