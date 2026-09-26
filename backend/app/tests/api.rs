//! End-to-end HTTP tests: real router, the three bounded contexts wired by
//! the composition root, PostgreSQL test database. The HTTP contract (routes,
//! JSON) is the one the frontend relies on.
//!
//! Requires the `postgres_test` service (`docker compose up -d`) and
//! `DATABASE_URL_TEST` (read from `.env`).

use axum::Router;
use axum::body::Body;
use axum::http::{Method, Request, StatusCode, header};
use http_body_util::BodyExt;
use migration::testing::test_database;
use serde_json::{Value, json};
use tower::ServiceExt;
use uuid::Uuid;
use xetaravel_app::{AppState, Config, router};
use xetaravel_identity::JwtSettings;

/// Test harness holding the router and the application state.
struct TestApp {
    router: Router,
    state: AppState,
}

impl TestApp {
    /// Connects to the (migrated) test database and builds the production router.
    async fn start() -> Self {
        let db = test_database().await;
        let config = Config {
            database_url: String::new(),
            jwt: JwtSettings {
                secret: "test-secret-test-secret-test-secret!".into(),
                ttl: chrono::Duration::hours(1),
            },
            app_addr: "127.0.0.1:0".into(),
            cors_origin: "http://localhost:3000".into(),
        };
        let state = AppState::build(db, &config);

        Self {
            router: router(state.clone()),
            state,
        }
    }

    /// Sends a request and returns the status with the parsed JSON body (Null when empty).
    async fn call(
        &self,
        method: Method,
        uri: &str,
        token: Option<&str>,
        body: Option<Value>,
    ) -> (StatusCode, Value) {
        let mut request = Request::builder().method(method).uri(uri);
        if let Some(token) = token {
            request = request.header(header::AUTHORIZATION, format!("Bearer {token}"));
        }
        let request = match body {
            Some(body) => request
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(body.to_string())),
            None => request.body(Body::empty()),
        }
        .unwrap();

        let response = self.router.clone().oneshot(request).await.unwrap();
        let status = response.status();
        let bytes = response.into_body().collect().await.unwrap().to_bytes();
        let json = if bytes.is_empty() {
            Value::Null
        } else {
            serde_json::from_slice(&bytes).unwrap()
        };
        (status, json)
    }

    /// Registers a new member and returns `(token, email)`.
    async fn register(&self) -> (String, String) {
        let suffix = &Uuid::now_v7().simple().to_string()[20..];
        let email = format!("user_{suffix}@example.com");
        let (status, body) = self
            .call(
                Method::POST,
                "/api/auth/register",
                None,
                Some(json!({ "username": format!("user_{suffix}"), "email": email, "password": "super-secret" })),
            )
            .await;
        assert_eq!(status, StatusCode::CREATED, "{body}");
        (body["token"].as_str().unwrap().to_owned(), email)
    }

    /// Registers a new account and promotes it to admin; returns its token.
    async fn register_admin(&self) -> String {
        let (token, email) = self.register().await;
        self.state
            .identity
            .promote_to_admin
            .execute(&email)
            .await
            .unwrap();
        token
    }

    /// Creates a category and a published article as `admin`; returns the article JSON.
    async fn publish_article(&self, admin: &str) -> Value {
        let (status, category) = self
            .call(
                Method::POST,
                "/api/admin/categories",
                Some(admin),
                Some(json!({ "name": format!("Category {}", Uuid::now_v7()), "slug": null, "description": null })),
            )
            .await;
        assert_eq!(status, StatusCode::CREATED, "{category}");

        let (status, article) = self
            .call(
                Method::POST,
                "/api/admin/articles",
                Some(admin),
                Some(json!({
                    "category_id": category["id"],
                    "title": format!("Article {}", Uuid::now_v7()),
                    "slug": null,
                    "excerpt": "Short intro",
                    "content": "# Hello\n\nMarkdown **body**.",
                    "publish": true
                })),
            )
            .await;
        assert_eq!(status, StatusCode::CREATED, "{article}");
        article
    }
}

#[tokio::test]
async fn health_check() {
    let app = TestApp::start().await;
    let (status, body) = app.call(Method::GET, "/api/health", None, None).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["status"], "ok");
}

#[tokio::test]
async fn register_login_and_me() {
    let app = TestApp::start().await;
    let (token, email) = app.register().await;

    let (status, me) = app
        .call(Method::GET, "/api/auth/me", Some(&token), None)
        .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(me["email"], email.as_str());
    assert_eq!(me["role"], "member");
    assert!(me.get("password_hash").is_none());

    let (status, login) = app
        .call(
            Method::POST,
            "/api/auth/login",
            None,
            Some(json!({ "email": email.to_uppercase(), "password": "super-secret" })),
        )
        .await;
    assert_eq!(status, StatusCode::OK);
    assert!(login["token"].is_string());

    let (status, error) = app
        .call(
            Method::POST,
            "/api/auth/login",
            None,
            Some(json!({ "email": email, "password": "wrong-password" })),
        )
        .await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
    assert_eq!(error["error"], "unauthorized");
}

#[tokio::test]
async fn registration_errors_are_reported_per_field() {
    let app = TestApp::start().await;
    let (_, email) = app.register().await;

    let (status, body) = app
        .call(
            Method::POST,
            "/api/auth/register",
            None,
            Some(json!({ "username": "x", "email": email, "password": "short" })),
        )
        .await;

    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(body["error"], "validation_error");
    assert!(body["fields"]["username"].is_array());
    assert!(body["fields"]["password"].is_array());
}

#[tokio::test]
async fn authentication_is_required_for_protected_routes() {
    let app = TestApp::start().await;

    let (status, _) = app.call(Method::GET, "/api/auth/me", None, None).await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);

    let (status, _) = app
        .call(Method::GET, "/api/auth/me", Some("garbage"), None)
        .await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn members_cannot_access_the_administration() {
    let app = TestApp::start().await;
    let (token, _) = app.register().await;

    let (status, body) = app
        .call(Method::GET, "/api/admin/users", Some(&token), None)
        .await;

    assert_eq!(status, StatusCode::FORBIDDEN);
    assert_eq!(body["error"], "forbidden");
}

#[tokio::test]
async fn admin_publishes_and_readers_comment() {
    let app = TestApp::start().await;
    let admin = app.register_admin().await;
    let article = app.publish_article(&admin).await;
    let slug = article["slug"].as_str().unwrap();

    // Public reading.
    let (status, public) = app
        .call(Method::GET, &format!("/api/articles/{slug}"), None, None)
        .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(public["content"], "# Hello\n\nMarkdown **body**.");

    let category = article["category"]["slug"].as_str().unwrap();
    let (status, list) = app
        .call(
            Method::GET,
            &format!("/api/articles?category={category}"),
            None,
            None,
        )
        .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(list["total"], 1);

    // Anonymous visitors cannot comment, members can.
    let comment = json!({ "content": "Great article!" });
    let uri = format!("/api/articles/{slug}/comments");
    let (status, _) = app
        .call(Method::POST, &uri, None, Some(comment.clone()))
        .await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);

    let (member, _) = app.register().await;
    let (status, created) = app
        .call(Method::POST, &uri, Some(&member), Some(comment))
        .await;
    assert_eq!(status, StatusCode::CREATED, "{created}");

    let (status, comments) = app.call(Method::GET, &uri, None, None).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(comments.as_array().unwrap().len(), 1);

    // Another member cannot delete it, the admin can.
    let comment_uri = format!("/api/comments/{}", created["id"].as_str().unwrap());
    let (other, _) = app.register().await;
    let (status, _) = app
        .call(Method::DELETE, &comment_uri, Some(&other), None)
        .await;
    assert_eq!(status, StatusCode::FORBIDDEN);
    let (status, _) = app
        .call(Method::DELETE, &comment_uri, Some(&admin), None)
        .await;
    assert_eq!(status, StatusCode::NO_CONTENT);
}

#[tokio::test]
async fn drafts_are_hidden_from_the_public() {
    let app = TestApp::start().await;
    let admin = app.register_admin().await;
    let article = app.publish_article(&admin).await;
    let id = article["id"].as_str().unwrap();
    let slug = article["slug"].as_str().unwrap();

    let mut draft = json!({
        "category_id": article["category"]["id"],
        "title": article["title"],
        "slug": slug,
        "excerpt": null,
        "content": "Unpublished",
        "publish": false
    });
    let (status, updated) = app
        .call(
            Method::PUT,
            &format!("/api/admin/articles/{id}"),
            Some(&admin),
            Some(draft.clone()),
        )
        .await;
    assert_eq!(status, StatusCode::OK, "{updated}");
    assert_eq!(updated["is_published"], false);

    let (status, _) = app
        .call(Method::GET, &format!("/api/articles/{slug}"), None, None)
        .await;
    assert_eq!(status, StatusCode::NOT_FOUND);

    let (status, _) = app
        .call(
            Method::GET,
            &format!("/api/admin/articles/{id}"),
            Some(&admin),
            None,
        )
        .await;
    assert_eq!(status, StatusCode::OK);

    draft["title"] = json!("x");
    let (status, _) = app
        .call(
            Method::PUT,
            &format!("/api/admin/articles/{id}"),
            Some(&admin),
            Some(draft),
        )
        .await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);

    // A non-empty category cannot be deleted.
    let category_uri = format!(
        "/api/admin/categories/{}",
        article["category"]["id"].as_str().unwrap()
    );
    let (status, _) = app
        .call(Method::DELETE, &category_uri, Some(&admin), None)
        .await;
    assert_eq!(status, StatusCode::CONFLICT);

    let (status, _) = app
        .call(
            Method::DELETE,
            &format!("/api/admin/articles/{id}"),
            Some(&admin),
            None,
        )
        .await;
    assert_eq!(status, StatusCode::NO_CONTENT);
    let (status, _) = app
        .call(Method::DELETE, &category_uri, Some(&admin), None)
        .await;
    assert_eq!(status, StatusCode::NO_CONTENT);
}

#[tokio::test]
async fn admin_changes_roles() {
    let app = TestApp::start().await;
    let admin = app.register_admin().await;
    let (member, _) = app.register().await;
    let (_, me) = app
        .call(Method::GET, "/api/auth/me", Some(&member), None)
        .await;
    let uri = format!("/api/admin/users/{}/role", me["id"].as_str().unwrap());

    let (status, user) = app
        .call(
            Method::PATCH,
            &uri,
            Some(&admin),
            Some(json!({ "role": "admin" })),
        )
        .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(user["role"], "admin");

    // The new role applies immediately, even with the old token.
    let (status, _) = app
        .call(Method::GET, "/api/admin/users", Some(&member), None)
        .await;
    assert_eq!(status, StatusCode::OK);

    // An admin cannot demote themselves.
    let (_, admin_me) = app
        .call(Method::GET, "/api/auth/me", Some(&admin), None)
        .await;
    let self_uri = format!("/api/admin/users/{}/role", admin_me["id"].as_str().unwrap());
    let (status, _) = app
        .call(
            Method::PATCH,
            &self_uri,
            Some(&admin),
            Some(json!({ "role": "member" })),
        )
        .await;
    assert_eq!(status, StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn malformed_requests_get_json_errors() {
    let app = TestApp::start().await;
    let admin = app.register_admin().await;

    let (status, body) = app
        .call(
            Method::GET,
            "/api/admin/articles/not-a-uuid",
            Some(&admin),
            None,
        )
        .await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    assert_eq!(body["error"], "not_found");

    let (status, body) = app
        .call(
            Method::POST,
            "/api/auth/login",
            None,
            Some(json!({ "email": 42 })),
        )
        .await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
    assert!(body["fields"]["body"].is_array());
}
