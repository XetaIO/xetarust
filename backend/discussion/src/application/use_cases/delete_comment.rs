use std::sync::Arc;

use uuid::Uuid;
use xetaravel_kernel::{AppError, AppResult, Principal};

use crate::domain::{AuthorId, CommentId, CommentRepository};

/// Deletes a comment. Allowed for its author and for admins.
pub struct DeleteComment {
    comments: Arc<dyn CommentRepository>,
}

impl DeleteComment {
    /// Builds the use case with its dependencies.
    pub fn new(comments: Arc<dyn CommentRepository>) -> Self {
        Self { comments }
    }

    /// Checks the permissions of `principal` and deletes the comment.
    pub async fn execute(&self, principal: Principal, comment_id: Uuid) -> AppResult<()> {
        let comment = self
            .comments
            .find_by_id(CommentId::from(comment_id))
            .await?
            .ok_or_else(|| AppError::NotFound("comment not found".into()))?;

        if !comment.can_be_deleted_by(AuthorId::from(principal.user_id), principal.is_admin) {
            return Err(AppError::Forbidden(
                "you can only delete your own comments".into(),
            ));
        }

        self.comments.delete(comment.id).await?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::application::test_support::{admin_principal, comment_by, member_principal};
    use crate::domain::MockCommentRepository;

    /// Builds a repository mock holding one comment written by a member,
    /// expecting `deletions` delete calls. Returns the mock and the author.
    fn comments(deletions: usize) -> (MockCommentRepository, Principal) {
        let john = member_principal();
        let comment = comment_by(john);
        let mut comments = MockCommentRepository::new();
        comments
            .expect_find_by_id()
            .returning(move |_| Ok(Some(comment.clone())));
        comments
            .expect_delete()
            .times(deletions)
            .returning(|_| Ok(true));
        (comments, john)
    }

    #[tokio::test]
    async fn author_can_delete_their_comment() {
        let (repo, john) = comments(1);
        DeleteComment::new(Arc::new(repo))
            .execute(john, Uuid::now_v7())
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn admin_can_delete_any_comment() {
        let (repo, _) = comments(1);
        DeleteComment::new(Arc::new(repo))
            .execute(admin_principal(), Uuid::now_v7())
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn other_members_cannot_delete_it() {
        let (repo, _) = comments(0);
        let error = DeleteComment::new(Arc::new(repo))
            .execute(member_principal(), Uuid::now_v7())
            .await
            .unwrap_err();
        assert!(matches!(error, AppError::Forbidden(_)));
    }

    #[tokio::test]
    async fn missing_comment_is_not_found() {
        let mut repo = MockCommentRepository::new();
        repo.expect_find_by_id().returning(|_| Ok(None));
        let error = DeleteComment::new(Arc::new(repo))
            .execute(member_principal(), Uuid::now_v7())
            .await
            .unwrap_err();
        assert!(matches!(error, AppError::NotFound(_)));
    }
}
