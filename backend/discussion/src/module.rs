//! Assembly of the Discussion context: builds its adapters once and injects
//! them into every use case.

use std::sync::Arc;

use sea_orm::DatabaseConnection;
use xetaravel_kernel::Clock;

use crate::application::ports::{ArticleCatalog, AuthorDirectory};
use crate::application::use_cases::{
    DeleteAuthorComments, DeleteComment, ListComments, PostComment,
};
use crate::domain::CommentRepository;
use crate::infrastructure::persistence::SeaOrmCommentRepository;

/// Every use case of the Discussion context.
pub struct DiscussionModule {
    pub list_comments: ListComments,
    pub post_comment: PostComment,
    pub delete_comment: DeleteComment,
    pub delete_author_comments: DeleteAuthorComments,
}

impl DiscussionModule {
    /// Wires the PostgreSQL repository and the given outgoing ports into the
    /// use cases. `articles` and `authors` are provided by the composition
    /// root (anti-corruption layer).
    pub fn new(
        db: DatabaseConnection,
        clock: Arc<dyn Clock>,
        articles: Arc<dyn ArticleCatalog>,
        authors: Arc<dyn AuthorDirectory>,
    ) -> Self {
        let comments: Arc<dyn CommentRepository> = Arc::new(SeaOrmCommentRepository::new(db));

        Self {
            list_comments: ListComments::new(comments.clone(), articles.clone(), authors.clone()),
            post_comment: PostComment::new(comments.clone(), articles, authors, clock),
            delete_comment: DeleteComment::new(comments.clone()),
            delete_author_comments: DeleteAuthorComments::new(comments),
        }
    }
}
