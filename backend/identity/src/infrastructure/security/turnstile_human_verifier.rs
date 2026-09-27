//! [`HumanVerifier`] backed by Cloudflare Turnstile's `siteverify` API.

use std::net::IpAddr;
use std::time::Duration;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use xetaravel_kernel::{AppError, AppResult};

use crate::application::ports::HumanVerifier;

/// Official endpoint validating Turnstile tokens.
pub const TURNSTILE_SITEVERIFY_URL: &str =
    "https://challenges.cloudflare.com/turnstile/v0/siteverify";

/// Maximum time spent waiting for Cloudflare.
const TIMEOUT: Duration = Duration::from_secs(5);

/// Body sent to `siteverify`.
#[derive(Serialize)]
struct SiteverifyRequest<'a> {
    secret: &'a str,
    response: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    remoteip: Option<String>,
}

/// Relevant part of the `siteverify` answer.
#[derive(Deserialize)]
struct SiteverifyResponse {
    success: bool,
}

/// Validates Turnstile tokens server-side against Cloudflare.
pub struct TurnstileHumanVerifier {
    client: reqwest::Client,
    secret: String,
    endpoint: String,
}

impl TurnstileHumanVerifier {
    /// Builds a verifier using `secret` and posting to `endpoint`
    /// (usually [`TURNSTILE_SITEVERIFY_URL`]; a local server in tests).
    pub fn new(secret: impl Into<String>, endpoint: impl Into<String>) -> Self {
        Self {
            client: reqwest::Client::new(),
            secret: secret.into(),
            endpoint: endpoint.into(),
        }
    }
}

#[async_trait]
impl HumanVerifier for TurnstileHumanVerifier {
    /// Asks Cloudflare whether `token` is valid. Network failures, timeouts
    /// and unexpected answers become internal errors (logged, never exposed).
    async fn verify(&self, token: &str, remote_ip: Option<IpAddr>) -> AppResult<bool> {
        let body = SiteverifyRequest {
            secret: &self.secret,
            response: token,
            remoteip: remote_ip.map(|ip| ip.to_string()),
        };
        let answer: SiteverifyResponse = self
            .client
            .post(&self.endpoint)
            .timeout(TIMEOUT)
            .json(&body)
            .send()
            .await
            .and_then(reqwest::Response::error_for_status)
            .map_err(turnstile_failure)?
            .json()
            .await
            .map_err(turnstile_failure)?;
        Ok(answer.success)
    }
}

/// Wraps a transport error into an internal error.
fn turnstile_failure(error: reqwest::Error) -> AppError {
    AppError::Internal(format!("turnstile verification failed: {error}"))
}

#[cfg(test)]
mod tests {
    use axum::routing::post;
    use axum::{Json, Router};
    use serde_json::{Value, json};
    use tokio::net::TcpListener;

    use super::*;

    /// Secret accepted by the fake Cloudflare server.
    const SECRET: &str = "test-secret";

    /// Fake `siteverify`: succeeds only for the token "valid" sent with the
    /// right secret and the client IP 203.0.113.7.
    async fn siteverify(Json(body): Json<Value>) -> Json<Value> {
        let success = body["secret"] == SECRET
            && body["response"] == "valid"
            && body["remoteip"] == "203.0.113.7";
        Json(json!({ "success": success, "error-codes": [] }))
    }

    /// Starts the fake Cloudflare server and returns its endpoint URL.
    async fn fake_cloudflare() -> String {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        let app = Router::new().route("/siteverify", post(siteverify));
        tokio::spawn(async move { axum::serve(listener, app).await });
        format!("http://{addr}/siteverify")
    }

    /// Returns the client IP used by the tests.
    fn ip() -> Option<IpAddr> {
        Some("203.0.113.7".parse().unwrap())
    }

    #[tokio::test]
    async fn accepts_a_token_cloudflare_validates() {
        let verifier = TurnstileHumanVerifier::new(SECRET, fake_cloudflare().await);
        assert!(verifier.verify("valid", ip()).await.unwrap());
    }

    #[tokio::test]
    async fn rejects_a_token_cloudflare_refuses() {
        let verifier = TurnstileHumanVerifier::new(SECRET, fake_cloudflare().await);
        assert!(!verifier.verify("forged", ip()).await.unwrap());
        assert!(!verifier.verify("valid", None).await.unwrap());
    }

    #[tokio::test]
    async fn turns_network_failures_into_internal_errors() {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        drop(listener);
        let verifier = TurnstileHumanVerifier::new(SECRET, format!("http://{addr}/siteverify"));

        let error = verifier.verify("valid", ip()).await.unwrap_err();

        assert!(matches!(error, AppError::Internal(_)));
    }
}
