//! HTTP adapter of the Resume context: the resume download.
//! Handlers stay thin: extract, call one use case, serialize.

use std::sync::Arc;

use axum::Router;
use axum::extract::{FromRef, State};
use axum::http::header;
use axum::response::IntoResponse;
use axum::routing::post;
use xetaravel_kernel::http::{ApiResult, ClientIp, JsonBody};

use crate::ResumeModule;
use crate::application::dto::DownloadResumeRequest;

/// Routes of the Resume context. The router state must expose the
/// [`ResumeModule`] through [`FromRef`].
pub fn router<S>() -> Router<S>
where
    S: Clone + Send + Sync + 'static,
    Arc<ResumeModule>: FromRef<S>,
{
    Router::new().route("/api/cv", post(download_resume))
}

/// `POST /api/cv` — the resume PDF as an attachment, once the captcha is solved.
/// The response is never cached: each download needs a fresh challenge.
async fn download_resume(
    State(resume): State<Arc<ResumeModule>>,
    ClientIp(ip): ClientIp,
    JsonBody(input): JsonBody<DownloadResumeRequest>,
) -> ApiResult<impl IntoResponse> {
    let file = resume.download.execute(input, ip).await?;
    Ok((
        [
            (header::CONTENT_TYPE, "application/pdf".to_owned()),
            (
                header::CONTENT_DISPOSITION,
                format!("attachment; filename=\"{}\"", file.filename),
            ),
            (header::X_CONTENT_TYPE_OPTIONS, "nosniff".to_owned()),
            (header::CACHE_CONTROL, "private, no-store".to_owned()),
        ],
        file.bytes,
    ))
}
