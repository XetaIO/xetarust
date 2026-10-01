//! Composition root: builds every context once and wires the
//! anti-corruption layer between them.

use std::sync::Arc;

use axum::extract::FromRef;
use axum::http::HeaderValue;
use sea_orm::DatabaseConnection;
use xetaravel_discussion::DiscussionModule;
use xetaravel_identity::IdentityModule;
use xetaravel_kernel::{Clock, PrincipalResolver, SystemClock};
use xetaravel_publishing::PublishingModule;
use xetaravel_resume::ResumeModule;

use crate::config::{Config, RateLimitSettings};
use crate::integration::{
    IdentityAuthorDirectory, IdentityHumanVerifier, PublishingArticleCatalog,
};

/// Shared state of the Axum application (cheap to clone). Each context's
/// router extracts the part it needs through [`FromRef`].
#[derive(Clone, FromRef)]
pub struct AppState {
    pub identity: Arc<IdentityModule>,
    pub publishing: Arc<PublishingModule>,
    pub discussion: Arc<DiscussionModule>,
    pub resume: Arc<ResumeModule>,
    pub principals: Arc<dyn PrincipalResolver>,
    /// Per-IP rate limit applied by the router to the credential routes and
    /// to the resume download (the captcha-protected routes).
    pub auth_rate_limit: RateLimitSettings,
    /// Only origin a browser may read the API from (CORS), applied by the router.
    pub cors_origin: HeaderValue,
}

impl AppState {
    /// Builds the four contexts with their production adapters and connects
    /// them: Identity names the authors of Publishing and Discussion and tells
    /// humans from bots for Resume, and Publishing tells Discussion which
    /// articles can be commented.
    pub fn build(db: DatabaseConnection, config: &Config) -> Self {
        let clock: Arc<dyn Clock> = Arc::new(SystemClock);

        let identity = Arc::new(IdentityModule::new(
            db.clone(),
            &config.jwt,
            &config.captcha,
            clock.clone(),
        ));
        let authors = Arc::new(IdentityAuthorDirectory::new(identity.directory()));
        let resume = Arc::new(ResumeModule::new(Arc::new(IdentityHumanVerifier::new(
            identity.human_check(),
        ))));

        let publishing = Arc::new(PublishingModule::new(
            db.clone(),
            clock.clone(),
            authors.clone(),
            config.uploads_dir.join("covers"),
        ));
        let articles = Arc::new(PublishingArticleCatalog::new(
            publishing.published_articles(),
        ));

        let discussion = Arc::new(DiscussionModule::new(
            db,
            clock,
            articles,
            authors,
            config.comment_throttle,
        ));

        Self {
            principals: identity.principals(),
            identity,
            publishing,
            discussion,
            resume,
            auth_rate_limit: config.auth_rate_limit,
            cors_origin: config.cors_origin.clone(),
        }
    }
}
