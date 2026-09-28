use std::sync::Arc;

use axum::Json;
use axum::extract::State;
use xetaravel_kernel::http::{AdminPrincipal, ApiResult, JsonBody};

use crate::IdentityModule;
use crate::application::dto::{IdentitySettingsDto, UpdateIdentitySettingsRequest};

/// `GET /api/settings/identity` — public settings (are registrations open?).
pub async fn get_identity_settings(
    State(identity): State<Arc<IdentityModule>>,
) -> ApiResult<Json<IdentitySettingsDto>> {
    Ok(Json(identity.identity_settings.execute().await?))
}

/// `PUT /api/admin/settings/identity` — opens or closes registrations.
pub async fn update_identity_settings(
    State(identity): State<Arc<IdentityModule>>,
    AdminPrincipal(principal): AdminPrincipal,
    JsonBody(input): JsonBody<UpdateIdentitySettingsRequest>,
) -> ApiResult<Json<IdentitySettingsDto>> {
    Ok(Json(
        identity
            .update_identity_settings
            .execute(principal, input)
            .await?,
    ))
}
