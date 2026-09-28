//! End-to-end HTTP tests: real router, the three bounded contexts wired by
//! the composition root, PostgreSQL test database. The HTTP contract (routes,
//! JSON) is the one the frontend relies on.
//!
//! Requires the `postgres_test` service (`docker compose up -d`) and
//! `DATABASE_URL_TEST` (read from `.env`).

use std::net::{IpAddr, Ipv6Addr};

use axum::body::Body;
use axum::body::Bytes;
use axum::http::{HeaderMap, Method, Request, StatusCode, header};
use axum::routing::post;
use axum::{Json, Router};
use http_body_util::BodyExt;
use migration::testing::test_database;
use sea_orm::{ConnectionTrait, DatabaseConnection, DbBackend, Statement};
use serde_json::{Value, json};
use tempfile::TempDir;
use tokio::net::TcpListener;
use tower::ServiceExt;
use uuid::Uuid;
use xetaravel_app::{AppState, Config, RateLimitSettings, router};
use xetaravel_identity::{CaptchaSettings, JwtSettings};

/// Header of a PNG file: enough for the format detection.
const PNG: &[u8] = &[0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A, 1, 2, 3];

/// Turnstile secret the API sends to the fake `siteverify`.
const CAPTCHA_SECRET: &str = "test-secret";

/// Captcha token the fake `siteverify` accepts.
const CAPTCHA_TOKEN: &str = "valid";

/// Fake Cloudflare `siteverify`: succeeds only for [`CAPTCHA_TOKEN`] sent
/// with [`CAPTCHA_SECRET`].
async fn siteverify(Json(body): Json<Value>) -> Json<Value> {
    let success = body["secret"] == CAPTCHA_SECRET && body["response"] == CAPTCHA_TOKEN;
    Json(json!({ "success": success, "error-codes": [] }))
}

/// Starts the fake Cloudflare server on a random local port and returns its
/// `siteverify` URL.
async fn fake_cloudflare() -> String {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let app = Router::new().route("/siteverify", post(siteverify));
    tokio::spawn(async move { axum::serve(listener, app).await });
    format!("http://{addr}/siteverify")
}

/// Test harness holding the router and the test database.
struct TestApp {
    router: Router,
    /// Test database, used to promote admins the way it is done in production.
    db: DatabaseConnection,
    /// Temporary uploads directory, removed when the harness is dropped.
    uploads: TempDir,
}

impl TestApp {
    /// Connects to the (migrated) test database and builds the production
    /// router, with the captcha checked by a fake Cloudflare and a rate limit
    /// out of reach.
    async fn start() -> Self {
        Self::start_with(RateLimitSettings {
            burst: 10_000,
            period_seconds: 1,
        })
        .await
    }

    /// Same as [`Self::start`] with the given rate limit on the credential routes.
    async fn start_with(auth_rate_limit: RateLimitSettings) -> Self {
        let db = test_database().await;
        let uploads = tempfile::tempdir().unwrap();
        let config = Config {
            database_url: String::new(),
            jwt: JwtSettings {
                secret: "test-secret-test-secret-test-secret!".into(),
                ttl: chrono::Duration::hours(1),
            },
            captcha: CaptchaSettings {
                turnstile_secret: CAPTCHA_SECRET.into(),
                siteverify_url: fake_cloudflare().await,
            },
            auth_rate_limit,
            app_addr: "127.0.0.1:0".into(),
            cors_origin: "http://localhost:3000".into(),
            uploads_dir: uploads.path().to_path_buf(),
        };
        let state = AppState::build(db.clone(), &config);

        Self {
            router: router(state),
            db,
            uploads,
        }
    }

    /// Sends a request with a raw body and returns the status, headers and body bytes.
    async fn call_raw(
        &self,
        method: Method,
        uri: &str,
        token: Option<&str>,
        body: Vec<u8>,
    ) -> (StatusCode, HeaderMap, Bytes) {
        let mut request = Request::builder().method(method).uri(uri);
        if let Some(token) = token {
            request = request.header(header::AUTHORIZATION, format!("Bearer {token}"));
        }
        let request = request
            .header(header::CONTENT_TYPE, "application/octet-stream")
            .body(Body::from(body))
            .unwrap();

        let response = self.router.clone().oneshot(request).await.unwrap();
        let status = response.status();
        let headers = response.headers().clone();
        let bytes = response.into_body().collect().await.unwrap().to_bytes();
        (status, headers, bytes)
    }

    /// Tells whether the cover file `name` exists in the uploads directory.
    fn cover_file_exists(&self, name: &str) -> bool {
        self.uploads.path().join("covers").join(name).is_file()
    }

    /// Sends a request and returns the status with the parsed JSON body (Null when empty).
    ///
    /// Every call comes from a unique client IP: the tests run in parallel and
    /// must never share a rate limit bucket.
    async fn call(
        &self,
        method: Method,
        uri: &str,
        token: Option<&str>,
        body: Option<Value>,
    ) -> (StatusCode, Value) {
        self.call_from(unique_ip(), method, uri, token, body).await
    }

    /// Same as [`Self::call`], sent by the client `ip` (`X-Forwarded-For`).
    async fn call_from(
        &self,
        ip: IpAddr,
        method: Method,
        uri: &str,
        token: Option<&str>,
        body: Option<Value>,
    ) -> (StatusCode, Value) {
        let mut request = Request::builder()
            .method(method)
            .uri(uri)
            .header("x-forwarded-for", ip.to_string());
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
                Some(json!({ "username": format!("user_{suffix}"), "email": email, "password": "super-secret", "captcha_token": CAPTCHA_TOKEN })),
            )
            .await;
        assert_eq!(status, StatusCode::CREATED, "{body}");
        (body["token"].as_str().unwrap().to_owned(), email)
    }

    /// Registers a new account and promotes it to admin directly in the
    /// database (there is no promotion command); returns its token.
    async fn register_admin(&self) -> String {
        let (token, email) = self.register().await;
        self.db
            .execute_raw(Statement::from_sql_and_values(
                DbBackend::Postgres,
                "UPDATE users SET role = 'admin' WHERE email = $1",
                [email.into()],
            ))
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

/// Returns an IP address no other request of the test suite uses.
fn unique_ip() -> IpAddr {
    IpAddr::V6(Ipv6Addr::from(Uuid::now_v7().as_u128()))
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
            Some(json!({ "email": email.to_uppercase(), "password": "super-secret", "captcha_token": CAPTCHA_TOKEN })),
        )
        .await;
    assert_eq!(status, StatusCode::OK);
    assert!(login["token"].is_string());

    let (status, error) = app
        .call(
            Method::POST,
            "/api/auth/login",
            None,
            Some(json!({ "email": email, "password": "wrong-password", "captcha_token": CAPTCHA_TOKEN })),
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
            Some(json!({ "username": "x", "email": email, "password": "short", "captcha_token": CAPTCHA_TOKEN })),
        )
        .await;

    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(body["error"], "validation_error");
    assert!(body["fields"]["username"].is_array());
    assert!(body["fields"]["password"].is_array());
}

#[tokio::test]
async fn login_rejects_a_failed_captcha() {
    let app = TestApp::start().await;
    let (_, email) = app.register().await;

    let (status, body) = app
        .call(
            Method::POST,
            "/api/auth/login",
            None,
            Some(json!({ "email": email, "password": "super-secret", "captcha_token": "forged" })),
        )
        .await;

    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(body["error"], "validation_error");
    assert!(body["fields"]["captcha_token"].is_array());
}

#[tokio::test]
async fn register_rejects_a_failed_captcha() {
    let app = TestApp::start().await;
    let suffix = &Uuid::now_v7().simple().to_string()[20..];
    let email = format!("user_{suffix}@example.com");

    let (status, body) = app
        .call(
            Method::POST,
            "/api/auth/register",
            None,
            Some(json!({ "username": format!("user_{suffix}"), "email": email, "password": "super-secret", "captcha_token": "forged" })),
        )
        .await;

    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(body["error"], "validation_error");
    assert!(body["fields"]["captcha_token"].is_array());

    // No account was created: the same identity can still register.
    let (status, body) = app
        .call(
            Method::POST,
            "/api/auth/register",
            None,
            Some(json!({ "username": format!("user_{suffix}"), "email": email, "password": "super-secret", "captcha_token": CAPTCHA_TOKEN })),
        )
        .await;
    assert_eq!(status, StatusCode::CREATED, "{body}");
}

#[tokio::test]
async fn auth_routes_are_rate_limited_per_ip() {
    let app = TestApp::start_with(RateLimitSettings {
        burst: 2,
        period_seconds: 60,
    })
    .await;
    let attacker = unique_ip();
    let attempt = json!({ "email": "ghost@example.com", "password": "wrong-password", "captcha_token": CAPTCHA_TOKEN });

    for _ in 0..2 {
        let (status, _) = app
            .call_from(
                attacker,
                Method::POST,
                "/api/auth/login",
                None,
                Some(attempt.clone()),
            )
            .await;
        assert_eq!(status, StatusCode::UNAUTHORIZED);
    }
    let (status, body) = app
        .call_from(
            attacker,
            Method::POST,
            "/api/auth/login",
            None,
            Some(attempt.clone()),
        )
        .await;
    assert_eq!(status, StatusCode::TOO_MANY_REQUESTS);
    assert_eq!(body["error"], "too_many_requests");
    assert_eq!(body["message"], "too many attempts, try again later");

    // The same IP is also blocked on registration (shared bucket).
    let (status, _) = app
        .call_from(
            attacker,
            Method::POST,
            "/api/auth/register",
            None,
            Some(json!({})),
        )
        .await;
    assert_eq!(status, StatusCode::TOO_MANY_REQUESTS);

    // Another IP is not affected.
    let (status, _) = app
        .call(Method::POST, "/api/auth/login", None, Some(attempt))
        .await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);

    // Other routes are never rate limited.
    for _ in 0..5 {
        let (status, _) = app
            .call_from(attacker, Method::GET, "/api/articles", None, None)
            .await;
        assert_eq!(status, StatusCode::OK);
    }
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
async fn admin_bans_a_member() {
    let app = TestApp::start().await;
    let admin = app.register_admin().await;
    let article = app.publish_article(&admin).await;
    let comments_uri = format!(
        "/api/articles/{}/comments",
        article["slug"].as_str().unwrap()
    );
    let (member, email) = app.register().await;
    let (_, me) = app
        .call(Method::GET, "/api/auth/me", Some(&member), None)
        .await;
    let member_id = me["id"].as_str().unwrap();
    let ban_uri = format!("/api/admin/users/{member_id}/ban");
    let login =
        json!({ "email": email, "password": "super-secret", "captcha_token": CAPTCHA_TOKEN });

    // 1. The member comments.
    let comment = json!({ "content": "Buy cheap stuff!" });
    let (status, _) = app
        .call(
            Method::POST,
            &comments_uri,
            Some(&member),
            Some(comment.clone()),
        )
        .await;
    assert_eq!(status, StatusCode::CREATED);

    // 2. The admin bans them with a reason.
    let (status, banned) = app
        .call(
            Method::PUT,
            &ban_uri,
            Some(&admin),
            Some(json!({ "reason": " Spam " })),
        )
        .await;
    assert_eq!(status, StatusCode::OK, "{banned}");
    assert!(banned["banned_at"].is_string());
    assert_eq!(banned["ban_reason"], "Spam");

    // 3. Their still valid token no longer works anywhere.
    let (status, _) = app
        .call(Method::GET, "/api/auth/me", Some(&member), None)
        .await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
    let (status, _) = app
        .call(Method::POST, &comments_uri, Some(&member), Some(comment))
        .await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);

    // 4. Logging in is refused with the reason.
    let (status, body) = app
        .call(Method::POST, "/api/auth/login", None, Some(login.clone()))
        .await;
    assert_eq!(status, StatusCode::FORBIDDEN);
    assert_eq!(body["message"], "your account has been banned: Spam");

    // 5. The admin purges their comments.
    let (status, purge) = app
        .call(
            Method::DELETE,
            &format!("/api/admin/users/{member_id}/comments"),
            Some(&admin),
            None,
        )
        .await;
    assert_eq!(status, StatusCode::OK, "{purge}");
    assert_eq!(purge["deleted"], 1);
    let (_, remaining) = app.call(Method::GET, &comments_uri, None, None).await;
    assert!(remaining.as_array().unwrap().is_empty());

    // 6. Once unbanned, the member can log in again.
    let (status, unbanned) = app.call(Method::DELETE, &ban_uri, Some(&admin), None).await;
    assert_eq!(status, StatusCode::OK);
    assert!(unbanned["banned_at"].is_null());
    let (status, _) = app
        .call(Method::POST, "/api/auth/login", None, Some(login))
        .await;
    assert_eq!(status, StatusCode::OK);
}

#[tokio::test]
async fn bans_are_restricted() {
    let app = TestApp::start().await;
    let admin = app.register_admin().await;
    let other_admin = app.register_admin().await;
    let (member, _) = app.register().await;
    let id_of = async |token: &str| {
        let (_, me) = app
            .call(Method::GET, "/api/auth/me", Some(token), None)
            .await;
        me["id"].as_str().unwrap().to_owned()
    };
    let (admin_id, other_admin_id) = (id_of(&admin).await, id_of(&other_admin).await);
    let ban = Some(json!({ "reason": null }));

    // An admin can neither ban another admin nor themselves.
    for target in [&other_admin_id, &admin_id] {
        let (status, _) = app
            .call(
                Method::PUT,
                &format!("/api/admin/users/{target}/ban"),
                Some(&admin),
                ban.clone(),
            )
            .await;
        assert_eq!(status, StatusCode::FORBIDDEN);
    }

    // A member cannot call the moderation routes.
    let (status, _) = app
        .call(
            Method::PUT,
            &format!("/api/admin/users/{admin_id}/ban"),
            Some(&member),
            ban,
        )
        .await;
    assert_eq!(status, StatusCode::FORBIDDEN);
    let (status, _) = app
        .call(
            Method::DELETE,
            &format!("/api/admin/users/{admin_id}/ban"),
            Some(&member),
            None,
        )
        .await;
    assert_eq!(status, StatusCode::FORBIDDEN);
    let (status, _) = app
        .call(
            Method::DELETE,
            &format!("/api/admin/users/{admin_id}/comments"),
            Some(&member),
            None,
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

#[tokio::test]
async fn admin_manages_article_covers() {
    let app = TestApp::start().await;
    let admin = app.register_admin().await;
    let article = app.publish_article(&admin).await;
    assert_eq!(article["cover_image"], Value::Null);
    let uri = format!(
        "/api/admin/articles/{}/cover",
        article["id"].as_str().unwrap()
    );

    // Upload: the article references a new file served publicly.
    let (status, _, body) = app
        .call_raw(Method::PUT, &uri, Some(&admin), PNG.to_vec())
        .await;
    assert_eq!(status, StatusCode::OK);
    let updated: Value = serde_json::from_slice(&body).unwrap();
    let first = updated["cover_image"].as_str().unwrap().to_owned();
    assert!(first.ends_with(".png"));

    let (status, headers, bytes) = app
        .call_raw(Method::GET, &format!("/api/covers/{first}"), None, vec![])
        .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(headers[header::CONTENT_TYPE], "image/png");
    assert!(
        headers[header::CACHE_CONTROL]
            .to_str()
            .unwrap()
            .contains("immutable")
    );
    assert_eq!(&bytes[..], PNG);

    // The public listing exposes the cover.
    let slug = article["slug"].as_str().unwrap();
    let (_, public) = app
        .call(Method::GET, &format!("/api/articles/{slug}"), None, None)
        .await;
    assert_eq!(public["cover_image"], first.as_str());

    // Replacing the cover deletes the previous file.
    let (status, _, body) = app
        .call_raw(Method::PUT, &uri, Some(&admin), PNG.to_vec())
        .await;
    assert_eq!(status, StatusCode::OK);
    let second = serde_json::from_slice::<Value>(&body).unwrap()["cover_image"]
        .as_str()
        .unwrap()
        .to_owned();
    assert_ne!(first, second);
    assert!(!app.cover_file_exists(&first));
    assert!(app.cover_file_exists(&second));

    // Removal detaches the cover and deletes its file.
    let (status, removed) = app.call(Method::DELETE, &uri, Some(&admin), None).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(removed["cover_image"], Value::Null);
    assert!(!app.cover_file_exists(&second));
    let (status, _, _) = app
        .call_raw(Method::GET, &format!("/api/covers/{second}"), None, vec![])
        .await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn cover_uploads_are_validated() {
    let app = TestApp::start().await;
    let admin = app.register_admin().await;
    let article = app.publish_article(&admin).await;
    let uri = format!(
        "/api/admin/articles/{}/cover",
        article["id"].as_str().unwrap()
    );

    let (status, _, body) = app
        .call_raw(Method::PUT, &uri, Some(&admin), b"not an image".to_vec())
        .await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
    let error: Value = serde_json::from_slice(&body).unwrap();
    assert!(error["fields"]["cover"].is_array());

    let (member, _) = app.register().await;
    let (status, _, _) = app
        .call_raw(Method::PUT, &uri, Some(&member), PNG.to_vec())
        .await;
    assert_eq!(status, StatusCode::FORBIDDEN);

    let (status, _, _) = app
        .call_raw(
            Method::GET,
            "/api/covers/..%2F..%2Fsecret.png",
            None,
            vec![],
        )
        .await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}
