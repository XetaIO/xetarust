//! Harness shared by the HTTP test binaries (`api.rs`, `registration.rs`):
//! production router wired on the PostgreSQL test database, fake Cloudflare
//! `siteverify` and request helpers.
//!
//! Every binary compiles this module but uses only part of it.
#![allow(dead_code)]

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

/// Turnstile secret the API sends to the fake `siteverify`.
pub const CAPTCHA_SECRET: &str = "test-secret";

/// Captcha token the fake `siteverify` accepts.
pub const CAPTCHA_TOKEN: &str = "valid";

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
pub struct TestApp {
    pub router: Router,
    /// Test database, used to promote admins the way it is done in production.
    pub db: DatabaseConnection,
    /// Temporary uploads directory, removed when the harness is dropped.
    uploads: TempDir,
}

impl TestApp {
    /// Connects to the (migrated) test database and builds the production
    /// router, with the captcha checked by a fake Cloudflare and a rate limit
    /// out of reach.
    pub async fn start() -> Self {
        Self::start_with(RateLimitSettings {
            burst: 10_000,
            period_seconds: 1,
        })
        .await
    }

    /// Same as [`Self::start`] with the given rate limit on the credential routes.
    pub async fn start_with(auth_rate_limit: RateLimitSettings) -> Self {
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
    pub async fn call_raw(
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
    pub fn cover_file_exists(&self, name: &str) -> bool {
        self.uploads.path().join("covers").join(name).is_file()
    }

    /// Sends a request and returns the status with the parsed JSON body (Null when empty).
    ///
    /// Every call comes from a unique client IP: the tests run in parallel and
    /// must never share a rate limit bucket.
    pub async fn call(
        &self,
        method: Method,
        uri: &str,
        token: Option<&str>,
        body: Option<Value>,
    ) -> (StatusCode, Value) {
        self.call_from(unique_ip(), method, uri, token, body).await
    }

    /// Same as [`Self::call`], sent by the client `ip` (`X-Forwarded-For`).
    pub async fn call_from(
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
    pub async fn register(&self) -> (String, String) {
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
    pub async fn register_admin(&self) -> String {
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
    pub async fn publish_article(&self, admin: &str) -> Value {
        self.publish_article_with(admin, json!({})).await
    }

    /// Same as [`TestApp::publish_article`], with the `extra` fields (e.g.
    /// `comments_enabled`) merged into the article form.
    pub async fn publish_article_with(&self, admin: &str, extra: Value) -> Value {
        let (status, category) = self
            .call(
                Method::POST,
                "/api/admin/categories",
                Some(admin),
                Some(json!({ "name": format!("Category {}", Uuid::now_v7()), "slug": null, "description": null })),
            )
            .await;
        assert_eq!(status, StatusCode::CREATED, "{category}");

        let mut form = json!({
            "category_id": category["id"],
            "title": format!("Article {}", Uuid::now_v7()),
            "slug": null,
            "excerpt": "Short intro",
            "content": "# Hello\n\nMarkdown **body**.",
            "publish": true
        });
        if let (Some(form), Some(extra)) = (form.as_object_mut(), extra.as_object()) {
            form.extend(extra.clone());
        }
        let (status, article) = self
            .call(Method::POST, "/api/admin/articles", Some(admin), Some(form))
            .await;
        assert_eq!(status, StatusCode::CREATED, "{article}");
        article
    }
}

/// Returns an IP address no other request of the test suite uses.
pub fn unique_ip() -> IpAddr {
    IpAddr::V6(Ipv6Addr::from(Uuid::now_v7().as_u128()))
}
