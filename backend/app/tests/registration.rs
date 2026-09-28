//! End-to-end HTTP test of closing registrations.
//!
//! Closing registrations is global state: it lives in its own test binary,
//! with a single test, because `cargo test` runs test binaries one after the
//! other while the (registering) tests of `api.rs` run in parallel.
//!
//! Requires the `postgres_test` service (`docker compose up -d`) and
//! `DATABASE_URL_TEST` (read from `.env`).

mod common;

use axum::http::{Method, StatusCode};
use serde_json::{Value, json};
use uuid::Uuid;

use common::{CAPTCHA_TOKEN, TestApp};

/// Returns a registration body for a brand new account.
fn new_account() -> Value {
    let suffix = &Uuid::now_v7().simple().to_string()[20..];
    json!({
        "username": format!("user_{suffix}"),
        "email": format!("user_{suffix}@example.com"),
        "password": "super-secret",
        "captcha_token": CAPTCHA_TOKEN
    })
}

/// Opens or closes registrations as `admin` and checks the answer.
async fn set_registration(app: &TestApp, admin: &str, enabled: bool) {
    let (status, body) = app
        .call(
            Method::PUT,
            "/api/admin/settings/identity",
            Some(admin),
            Some(json!({ "registration_enabled": enabled })),
        )
        .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(body["registration_enabled"], enabled);
}

#[tokio::test]
async fn admin_closes_and_reopens_registration() {
    let app = TestApp::start().await;
    let admin = app.register_admin().await;
    let (_, email) = app.register().await;

    set_registration(&app, &admin, false).await;

    let (status, settings) = app
        .call(Method::GET, "/api/settings/identity", None, None)
        .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(settings["registration_enabled"], false);

    let (status, error) = app
        .call(
            Method::POST,
            "/api/auth/register",
            None,
            Some(new_account()),
        )
        .await;
    assert_eq!(status, StatusCode::FORBIDDEN);
    assert_eq!(error["error"], "forbidden");

    // Existing accounts can still log in.
    let (status, login) = app
        .call(
            Method::POST,
            "/api/auth/login",
            None,
            Some(json!({ "email": email, "password": "super-secret", "captcha_token": CAPTCHA_TOKEN })),
        )
        .await;
    assert_eq!(status, StatusCode::OK, "{login}");

    set_registration(&app, &admin, true).await;

    let (status, body) = app
        .call(
            Method::POST,
            "/api/auth/register",
            None,
            Some(new_account()),
        )
        .await;
    assert_eq!(status, StatusCode::CREATED, "{body}");
}
