//! Integration tests of the Publishing repositories against a real PostgreSQL.
//!
//! Requires the `postgres_test` service (`docker compose up -d`) and
//! `DATABASE_URL_TEST` (read from `.env`). Every test creates its own uniquely
//! named data, so tests can run in parallel on the same database. Authors
//! (owned by Identity) are seeded as raw rows.

use chrono::{SubsecRound, Utc};
use migration::testing::{seed_user, test_database, unique};
use sea_orm::DatabaseConnection;
use xetaravel_kernel::DomainError;
use xetaravel_kernel::pagination::PageRequest;
use xetaravel_publishing::domain::{
    Article, ArticleDraft, ArticleFilter, ArticleRepository, AuthorId, Category,
    CategoryRepository, CoverImage, ImageFormat, Slug,
};
use xetaravel_publishing::infrastructure::persistence::{
    SeaOrmArticleRepository, SeaOrmCategoryRepository,
};

/// Returns the current time with PostgreSQL precision.
fn now() -> chrono::DateTime<Utc> {
    Utc::now().trunc_subsecs(6)
}

/// Seeds an author row and returns its id.
async fn create_author(db: &DatabaseConnection) -> AuthorId {
    AuthorId::from(seed_user(db).await)
}

/// Builds and stores a new category.
async fn create_category(db: &DatabaseConnection) -> Category {
    let category = Category::create(&format!("Category {}", unique()), None, None, now()).unwrap();
    SeaOrmCategoryRepository::new(db.clone())
        .create(&category)
        .await
        .unwrap();
    category
}

/// Builds and stores a new article.
async fn create_article(
    db: &DatabaseConnection,
    author: AuthorId,
    category: &Category,
    publish: bool,
) -> Article {
    let article = Article::write(
        author,
        ArticleDraft {
            category_id: category.id,
            title: format!("Article {}", unique()),
            slug: None,
            excerpt: Some("Excerpt".into()),
            content: "# Title\n\nBody".into(),
            publish,
        },
        now(),
    )
    .unwrap();
    SeaOrmArticleRepository::new(db.clone())
        .create(&article)
        .await
        .unwrap();
    article
}

#[tokio::test]
async fn category_repository_round_trip() {
    let db = test_database().await;
    let repo = SeaOrmCategoryRepository::new(db.clone());
    let mut category = create_category(&db).await;

    assert_eq!(
        repo.find_by_id(category.id).await.unwrap(),
        Some(category.clone())
    );
    assert!(repo.slug_exists(&category.slug, None).await.unwrap());
    assert!(
        !repo
            .slug_exists(&category.slug, Some(category.id))
            .await
            .unwrap()
    );
    assert!(repo.list_all().await.unwrap().contains(&category));

    category
        .update("Renamed category", None, Some("desc"), now())
        .unwrap();
    repo.update(&category).await.unwrap();
    assert_eq!(
        repo.find_by_id(category.id).await.unwrap(),
        Some(category.clone())
    );

    assert!(repo.delete(category.id).await.unwrap());
    assert!(!repo.delete(category.id).await.unwrap());
}

#[tokio::test]
async fn article_repository_categorizes_and_filters() {
    let db = test_database().await;
    let repo = SeaOrmArticleRepository::new(db.clone());
    let author = create_author(&db).await;
    let category = create_category(&db).await;
    let published = create_article(&db, author, &category, true).await;
    let draft = create_article(&db, author, &category, false).await;

    let entry = repo
        .find_categorized_by_slug(&published.slug)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(entry.article, published);
    assert_eq!(entry.category, category);
    assert_eq!(
        repo.find_categorized_by_id(draft.id)
            .await
            .unwrap()
            .unwrap()
            .article,
        draft
    );

    let public = repo
        .list_categorized(
            ArticleFilter::published(Some(category.slug.clone())),
            PageRequest::default(),
        )
        .await
        .unwrap();
    assert_eq!(public.total, 1);
    assert_eq!(public.items[0].article.id, published.id);

    let all = ArticleFilter {
        published_only: false,
        category: Some(category.slug.clone()),
    };
    assert_eq!(
        repo.list_categorized(all, PageRequest::default())
            .await
            .unwrap()
            .total,
        2
    );

    let unknown =
        ArticleFilter::published(Some(Slug::parse(&format!("nope-{}", unique())).unwrap()));
    assert_eq!(
        repo.list_categorized(unknown, PageRequest::default())
            .await
            .unwrap()
            .total,
        0
    );

    assert_eq!(repo.count_by_category(category.id).await.unwrap(), 2);
    assert!(repo.slug_exists(&published.slug, None).await.unwrap());
    assert!(
        !repo
            .slug_exists(&published.slug, Some(published.id))
            .await
            .unwrap()
    );
}

#[tokio::test]
async fn article_repository_update_and_delete() {
    let db = test_database().await;
    let repo = SeaOrmArticleRepository::new(db.clone());
    let author = create_author(&db).await;
    let category = create_category(&db).await;
    let mut article = create_article(&db, author, &category, false).await;

    article.publish(now());
    article.title = "Updated title".into();
    article.replace_cover(CoverImage::new(ImageFormat::Webp), now());
    repo.update(&article).await.unwrap();
    assert_eq!(
        repo.find_by_id(article.id).await.unwrap(),
        Some(article.clone())
    );

    article.remove_cover(now());
    repo.update(&article).await.unwrap();
    assert_eq!(
        repo.find_by_id(article.id).await.unwrap().unwrap().cover,
        None
    );

    let error = SeaOrmCategoryRepository::new(db.clone())
        .delete(category.id)
        .await
        .unwrap_err();
    assert!(
        matches!(error, DomainError::Conflict(_)),
        "category in use must not be deleted"
    );

    assert!(repo.delete(article.id).await.unwrap());
    assert_eq!(repo.find_by_id(article.id).await.unwrap(), None);
}

#[tokio::test]
async fn articles_require_an_existing_author() {
    let db = test_database().await;
    let category = create_category(&db).await;
    let article = Article::write(
        AuthorId::generate(),
        ArticleDraft {
            category_id: category.id,
            title: format!("Orphan {}", unique()),
            slug: None,
            excerpt: None,
            content: "Body".into(),
            publish: false,
        },
        now(),
    )
    .unwrap();

    let error = SeaOrmArticleRepository::new(db)
        .create(&article)
        .await
        .unwrap_err();

    assert!(matches!(error, DomainError::Conflict(_)));
}
