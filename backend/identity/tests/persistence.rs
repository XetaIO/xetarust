//! Integration tests of the Identity repository against a real PostgreSQL.
//!
//! Requires the `postgres_test` service (`docker compose up -d`) and
//! `DATABASE_URL_TEST` (read from `.env`). Every test creates its own uniquely
//! named data, so tests can run in parallel on the same database.

use chrono::{SubsecRound, Utc};
use migration::testing::{test_database, unique};
use sea_orm::DatabaseConnection;
use xetaravel_identity::domain::{
    Email, PasswordHash, Role, User, UserId, UserRepository, Username,
};
use xetaravel_identity::infrastructure::persistence::SeaOrmUserRepository;
use xetaravel_kernel::DomainError;
use xetaravel_kernel::pagination::PageRequest;

/// Returns the current time with PostgreSQL precision.
fn now() -> chrono::DateTime<Utc> {
    Utc::now().trunc_subsecs(6)
}

/// Builds and stores a new member.
async fn create_user(db: &DatabaseConnection) -> User {
    let suffix = unique();
    let user = User::register(
        Username::parse(&format!("user_{suffix}")).unwrap(),
        Email::parse(&format!("user_{suffix}@example.com")).unwrap(),
        PasswordHash::new("$argon2id$fake"),
        now(),
    );
    SeaOrmUserRepository::new(db.clone())
        .create(&user)
        .await
        .unwrap();
    user
}

#[tokio::test]
async fn user_repository_round_trip() {
    let db = test_database().await;
    let repo = SeaOrmUserRepository::new(db.clone());
    let mut user = create_user(&db).await;

    assert_eq!(repo.find_by_id(user.id).await.unwrap(), Some(user.clone()));
    assert_eq!(
        repo.find_by_email(&user.email).await.unwrap(),
        Some(user.clone())
    );
    assert!(repo.email_exists(&user.email).await.unwrap());
    let upper = Username::parse(&user.username.as_str().to_uppercase()).unwrap();
    assert!(repo.username_exists(&upper).await.unwrap());

    user.change_role(Role::Admin, UserId::generate(), Role::Admin, now())
        .unwrap();
    repo.update(&user).await.unwrap();
    assert_eq!(
        repo.find_by_id(user.id).await.unwrap().unwrap().role,
        Role::Admin
    );

    let page = repo
        .list(PageRequest::new(Some(1), Some(50)))
        .await
        .unwrap();
    assert!(page.total >= 1);
}

#[tokio::test]
async fn user_repository_finds_users_by_ids() {
    let db = test_database().await;
    let repo = SeaOrmUserRepository::new(db.clone());
    let first = create_user(&db).await;
    let second = create_user(&db).await;

    let mut found = repo
        .find_by_ids(&[first.id, second.id, UserId::generate()])
        .await
        .unwrap();
    found.sort_by_key(|user| user.id);

    assert_eq!(found, [first, second]);
    assert!(repo.find_by_ids(&[]).await.unwrap().is_empty());
}

#[tokio::test]
async fn user_repository_reports_duplicates_as_conflicts() {
    let db = test_database().await;
    let user = create_user(&db).await;
    let mut clone = user.clone();
    clone.id = UserId::generate();

    let error = SeaOrmUserRepository::new(db)
        .create(&clone)
        .await
        .unwrap_err();

    assert!(matches!(error, DomainError::Conflict(_)));
}
