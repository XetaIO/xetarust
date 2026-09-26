//! Test support (feature `test-support`): a migrated PostgreSQL test
//! database and seeds for rows owned by other contexts.
//!
//! The persistence tests of a context must satisfy the cross-context foreign
//! keys (an article needs a user, a comment needs an article) without
//! depending on the other context crates: the seeds below insert raw rows.

use sea_orm_migration::sea_orm::{
    ConnectOptions, ConnectionTrait, Database, DatabaseConnection, DbBackend, Statement,
};
use tokio::sync::OnceCell;
use uuid::Uuid;

use crate::{Migrator, MigratorTrait};

/// Guards the migrations so they run once per test binary.
static MIGRATED: OnceCell<()> = OnceCell::const_new();

/// Connects to `DATABASE_URL_TEST` (read from `.env`), applying the
/// migrations on first use.
pub async fn test_database() -> DatabaseConnection {
    dotenvy::dotenv().ok();
    let url = std::env::var("DATABASE_URL_TEST")
        .expect("DATABASE_URL_TEST must be set (start it with `docker compose up -d`)");
    let mut options = ConnectOptions::new(url);
    options.max_connections(10).sqlx_logging(false);
    let db = Database::connect(options)
        .await
        .expect("cannot connect to the test database");
    MIGRATED
        .get_or_init(|| async { Migrator::up(&db, None).await.expect("migrations failed") })
        .await;
    db
}

/// Returns a short random suffix to keep test data unique.
pub fn unique() -> String {
    Uuid::now_v7().simple().to_string()[20..].to_owned()
}

/// Inserts a member row in `users` and returns its id.
pub async fn seed_user(db: &DatabaseConnection) -> Uuid {
    let id = Uuid::now_v7();
    let name = format!("seed_{}", unique());
    execute(
        db,
        "INSERT INTO users (id, username, email, password_hash, created_at, updated_at)
         VALUES ($1, $2, $3, 'seed', NOW(), NOW())",
        vec![
            id.into(),
            name.clone().into(),
            format!("{name}@example.com").into(),
        ],
    )
    .await;
    id
}

/// Inserts a published article (and its category) written by `author`;
/// returns the article id and slug.
pub async fn seed_published_article(db: &DatabaseConnection, author: Uuid) -> (Uuid, String) {
    let category = Uuid::now_v7();
    let suffix = unique();
    execute(
        db,
        "INSERT INTO categories (id, name, slug, created_at, updated_at)
         VALUES ($1, $2, $3, NOW(), NOW())",
        vec![
            category.into(),
            format!("Seed {suffix}").into(),
            format!("seed-{suffix}").into(),
        ],
    )
    .await;

    let article = Uuid::now_v7();
    let slug = format!("seed-article-{suffix}");
    execute(
        db,
        "INSERT INTO articles (id, author_id, category_id, title, slug, content, published_at, created_at, updated_at)
         VALUES ($1, $2, $3, 'Seed article', $4, 'Body', NOW(), NOW(), NOW())",
        vec![article.into(), author.into(), category.into(), slug.clone().into()],
    )
    .await;
    (article, slug)
}

/// Deletes an article row (its comments cascade).
pub async fn delete_article(db: &DatabaseConnection, id: Uuid) {
    execute(db, "DELETE FROM articles WHERE id = $1", vec![id.into()]).await;
}

/// Runs a parameterized statement, panicking on failure (tests only).
async fn execute(
    db: &DatabaseConnection,
    sql: &str,
    values: Vec<sea_orm_migration::sea_orm::Value>,
) {
    db.execute_raw(Statement::from_sql_and_values(
        DbBackend::Postgres,
        sql,
        values,
    ))
    .await
    .expect("seed statement failed");
}
