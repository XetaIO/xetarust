//! Data Transfer Objects of the Resume context (public API contract).
//!
//! TypeScript bindings are generated in `frontend/src/types/api/resume/`.

use serde::{Deserialize, Serialize};
use ts_rs::TS;
use validator::Validate;

/// Body of `POST /api/cv`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Validate, TS)]
#[ts(export, export_to = "resume/")]
pub struct DownloadResumeRequest {
    /// Response of the captcha widget (Cloudflare Turnstile).
    #[serde(default)]
    #[validate(length(min = 1, message = "is required"))]
    pub captcha_token: String,
}
