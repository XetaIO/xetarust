//! Integration tests of the Discussion repository against a real PostgreSQL.
//!
//! Requires the `postgres_test` service (`docker compose up -d`) and
//! `DATABASE_URL_TEST` (read from `.env`). Authors (Identity) and articles
//! (Publishing) are seeded as raw rows.

use chrono::{Duration, SubsecRound, Utc};
use migration::testing::{delete_article, seed_published_article, seed_user, test_database};
use xetaravel_discussion::domain::{
    ArticleId, AuthorId, Comment, CommentRepository, CommentThrottle,
};
use xetaravel_discussion::infrastructure::persistence::SeaOrmCommentRepository;
use xetaravel_kernel::DomainError;

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

#[tokio::test]
async fn comment_repository_deletes_the_comments_of_an_author() {
    let db = test_database().await;
    let repo = SeaOrmCommentRepository::new(db.clone());
    let (banned, other) = (seed_user(&db).await, seed_user(&db).await);
    let (article, _) = seed_published_article(&db, other).await;
    let article_id = ArticleId::from(article);

    let spam = Comment::post(article_id, AuthorId::from(banned), "Spam", now()).unwrap();
    let more = Comment::post(article_id, AuthorId::from(banned), "More", now()).unwrap();
    let kept = Comment::post(article_id, AuthorId::from(other), "Kept", now()).unwrap();
    for comment in [&spam, &more, &kept] {
        repo.create(comment).await.unwrap();
    }

    assert_eq!(
        repo.delete_by_author(AuthorId::from(banned)).await.unwrap(),
        2
    );
    assert_eq!(repo.list_by_article(article_id).await.unwrap(), [kept]);
    assert_eq!(
        repo.delete_by_author(AuthorId::from(banned)).await.unwrap(),
        0
    );
}

/// Tells whether `result` is an anti-flood refusal.
fn is_throttled(result: &Result<(), DomainError>) -> bool {
    matches!(result, Err(DomainError::TooManyRequests(_)))
}

#[tokio::test]
async fn comment_repository_throttles_members() {
    let db = test_database().await;
    let repo = SeaOrmCommentRepository::new(db.clone());
    let throttle = CommentThrottle::default();
    let (member, other) = (seed_user(&db).await, seed_user(&db).await);
    let (article, _) = seed_published_article(&db, other).await;
    let (other_article, _) = seed_published_article(&db, other).await;
    let (article_id, member_id, other_id) = (
        ArticleId::from(article),
        AuthorId::from(member),
        AuthorId::from(other),
    );
    let start = now();
    let post = |article_id, author_id, minutes| {
        Comment::post(
            article_id,
            author_id,
            "Hello",
            start + Duration::minutes(minutes),
        )
        .unwrap()
    };

    // The first comment is accepted, posting twice in a row is refused.
    repo.create_throttled(&post(article_id, member_id, 0), &throttle)
        .await
        .unwrap();
    assert!(is_throttled(
        &repo
            .create_throttled(&post(article_id, member_id, 60), &throttle)
            .await
    ));

    // Another article is not affected.
    repo.create_throttled(
        &post(ArticleId::from(other_article), member_id, 1),
        &throttle,
    )
    .await
    .unwrap();

    // Once someone replied, the member must still wait for the cooldown.
    repo.create_throttled(&post(article_id, other_id, 1), &throttle)
        .await
        .unwrap();
    assert!(is_throttled(
        &repo
            .create_throttled(&post(article_id, member_id, 4), &throttle)
            .await
    ));
    repo.create_throttled(&post(article_id, member_id, 5), &throttle)
        .await
        .unwrap();

    assert_eq!(repo.list_by_article(article_id).await.unwrap().len(), 3);
}

#[tokio::test]
async fn comment_repository_serializes_concurrent_throttled_posts() {
    let db = test_database().await;
    let repo = SeaOrmCommentRepository::new(db.clone());
    let throttle = CommentThrottle::default();
    let author = seed_user(&db).await;
    let (article, _) = seed_published_article(&db, author).await;
    let (article_id, author_id) = (ArticleId::from(article), AuthorId::from(author));

    let first = Comment::post(article_id, author_id, "First", now()).unwrap();
    let second = Comment::post(article_id, author_id, "Second", now()).unwrap();
    let (a, b) = tokio::join!(
        repo.create_throttled(&first, &throttle),
        repo.create_throttled(&second, &throttle)
    );

    assert_eq!(
        [&a, &b].iter().filter(|result| result.is_ok()).count(),
        1,
        "{a:?} {b:?}"
    );
    assert!(is_throttled(&a) || is_throttled(&b));
    assert_eq!(repo.list_by_article(article_id).await.unwrap().len(), 1);
}
