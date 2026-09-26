//! Integration tests of the Discussion repository against a real PostgreSQL.
//!
//! Requires the `postgres_test` service (`docker compose up -d`) and
//! `DATABASE_URL_TEST` (read from `.env`). Authors (Identity) and articles
//! (Publishing) are seeded as raw rows.

use chrono::{Duration, SubsecRound, Utc};
use migration::testing::{delete_article, seed_published_article, seed_user, test_database};
use xetaravel_discussion::domain::{ArticleId, AuthorId, Comment, CommentRepository};
use xetaravel_discussion::infrastructure::persistence::SeaOrmCommentRepository;

/// Returns the current time with PostgreSQL precision.
fn now() -> chrono::DateTime<Utc> {
    Utc::now().trunc_subsecs(6)
}

#[tokio::test]
async fn comment_repository_round_trip() {
    let db = test_database().await;
    let repo = SeaOrmCommentRepository::new(db.clone());
    let author = seed_user(&db).await;
    let (article, _) = seed_published_article(&db, author).await;
    let (article_id, author_id) = (ArticleId::from(article), AuthorId::from(author));

    let first = Comment::post(article_id, author_id, "First!", now()).unwrap();
    let second = Comment::post(
        article_id,
        author_id,
        "Second",
        now() + Duration::seconds(1),
    )
    .unwrap();
    repo.create(&first).await.unwrap();
    repo.create(&second).await.unwrap();

    let comments = repo.list_by_article(article_id).await.unwrap();
    assert_eq!(comments, [first.clone(), second.clone()]);
    assert_eq!(
        repo.find_by_id(first.id).await.unwrap(),
        Some(first.clone())
    );

    assert!(repo.delete(first.id).await.unwrap());
    assert!(!repo.delete(first.id).await.unwrap());
    assert_eq!(repo.find_by_id(first.id).await.unwrap(), None);

    // Deleting the article (Publishing) cascades to its comments.
    delete_article(&db, article).await;
    assert_eq!(repo.find_by_id(second.id).await.unwrap(), None);
}
