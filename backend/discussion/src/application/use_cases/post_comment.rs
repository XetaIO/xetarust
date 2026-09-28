use std::sync::Arc;

use validator::Validate;
use xetaravel_kernel::{AppError, AppResult, Clock, Principal};

use super::{commentable_article, to_dtos};
use crate::application::dto::{CommentDto, CreateCommentRequest};
use crate::application::ports::{ArticleCatalog, AuthorDirectory};
use crate::domain::{AuthorId, Comment, CommentRepository, CommentThrottle};

/// Posts a comment on a published article on behalf of a logged-in user.
pub struct PostComment {
    comments: Arc<dyn CommentRepository>,
    articles: Arc<dyn ArticleCatalog>,
    authors: Arc<dyn AuthorDirectory>,
    clock: Arc<dyn Clock>,
    throttle: CommentThrottle,
}

impl PostComment {
    /// Builds the use case with its dependencies.
    pub fn new(
        comments: Arc<dyn CommentRepository>,
        articles: Arc<dyn ArticleCatalog>,
        authors: Arc<dyn AuthorDirectory>,
        clock: Arc<dyn Clock>,
        throttle: CommentThrottle,
    ) -> Self {
        Self {
            comments,
            articles,
            authors,
            clock,
            throttle,
        }
    }

    /// Validates and stores the comment on the article identified by `slug`,
    /// then returns it with its author. Fails with `Forbidden` when the
    /// comments of the article are closed, and with `TooManyRequests` when a
    /// member floods it (admins are exempt from the anti-flood policy).
    pub async fn execute(
        &self,
        principal: Principal,
        slug: &str,
        input: CreateCommentRequest,
    ) -> AppResult<CommentDto> {
        input.validate()?;
        let article = commentable_article(self.articles.as_ref(), slug).await?;
        article.ensure_open()?;

        let author = AuthorId::from(principal.user_id);
        let comment = Comment::post(article.id, author, &input.content, self.clock.now())?;
        if principal.is_admin {
            self.comments.create(&comment).await?;
        } else {
            self.comments
                .create_throttled(&comment, &self.throttle)
                .await?;
        }

        to_dtos(self.authors.as_ref(), std::slice::from_ref(&comment))
            .await?
            .pop()
            .ok_or_else(|| AppError::Internal("comment vanished after creation".into()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::application::ports::{MockArticleCatalog, MockAuthorDirectory};
    use crate::application::test_support::{
        admin_principal, authors, catalog, clock, closed_catalog, member_principal, throttle,
    };
    use crate::domain::MockCommentRepository;
    use xetaravel_kernel::DomainError;

    /// Builds the use case over `comments` and `catalog`, naming every author "john".
    fn use_case(comments: MockCommentRepository, catalog: MockArticleCatalog) -> PostComment {
        PostComment::new(
            Arc::new(comments),
            Arc::new(catalog),
            Arc::new(authors()),
            clock(),
            throttle(),
        )
    }

    /// Returns a valid comment form with `content`.
    fn request(content: &str) -> CreateCommentRequest {
        CreateCommentRequest {
            content: content.into(),
        }
    }

    #[tokio::test]
    async fn posts_a_comment() {
        let principal = member_principal();
        let mut comments = MockCommentRepository::new();
        comments.expect_create().times(0);
        comments
            .expect_create_throttled()
            .withf(move |c, t| {
                c.author_id.as_uuid() == principal.user_id
                    && c.content == "Great post!"
                    && *t == throttle()
            })
            .times(1)
            .returning(|_, _| Ok(()));

        let dto = use_case(comments, catalog(true))
            .execute(principal, "hello-rust", request(" Great post! "))
            .await
            .unwrap();

        assert_eq!(dto.author.username, "john");
        assert_eq!(dto.content, "Great post!");
    }

    #[tokio::test]
    async fn refuses_a_flooding_member() {
        let mut comments = MockCommentRepository::new();
        comments.expect_create_throttled().returning(|_, _| {
            Err(DomainError::TooManyRequests(
                "please wait 2 minutes before commenting again".into(),
            ))
        });

        let error = use_case(comments, catalog(true))
            .execute(member_principal(), "hello-rust", request("Hello"))
            .await
            .unwrap_err();

        assert_eq!(
            error,
            AppError::TooManyRequests("please wait 2 minutes before commenting again".into())
        );
    }

    #[tokio::test]
    async fn admins_are_not_throttled() {
        let mut comments = MockCommentRepository::new();
        comments.expect_create_throttled().times(0);
        comments.expect_create().times(1).returning(|_| Ok(()));

        let dto = use_case(comments, catalog(true))
            .execute(admin_principal(), "hello-rust", request("Hello"))
            .await
            .unwrap();

        assert_eq!(dto.content, "Hello");
    }

    #[tokio::test]
    async fn refuses_comments_on_drafts() {
        let error = PostComment::new(
            Arc::new(MockCommentRepository::new()),
            Arc::new(catalog(false)),
            Arc::new(MockAuthorDirectory::new()),
            clock(),
            throttle(),
        )
        .execute(member_principal(), "hello-rust", request("Hello"))
        .await
        .unwrap_err();

        assert!(matches!(error, AppError::NotFound(_)));
    }

    #[tokio::test]
    async fn refuses_comments_when_closed() {
        let mut comments = MockCommentRepository::new();
        comments.expect_create().times(0);
        comments.expect_create_throttled().times(0);

        let error = PostComment::new(
            Arc::new(comments),
            Arc::new(closed_catalog()),
            Arc::new(MockAuthorDirectory::new()),
            clock(),
            throttle(),
        )
        .execute(member_principal(), "hello-rust", request("Hello"))
        .await
        .unwrap_err();

        assert_eq!(
            error,
            AppError::Forbidden("comments are closed on this article".into())
        );
    }

    #[tokio::test]
    async fn validates_content() {
        let error = PostComment::new(
            Arc::new(MockCommentRepository::new()),
            Arc::new(MockArticleCatalog::new()),
            Arc::new(MockAuthorDirectory::new()),
            clock(),
            throttle(),
        )
        .execute(member_principal(), "hello-rust", request("x"))
        .await
        .unwrap_err();

        assert!(matches!(error, AppError::Validation(_)));
    }
}
