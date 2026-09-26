use std::sync::Arc;

use axum::Json;
use axum::extract::State;
use uuid::Uuid;
use xetaravel_kernel::dto::{PageQuery, Paginated};
use xetaravel_kernel::http::{AdminPrincipal, ApiResult, JsonBody, PathParam, QueryParams};

use crate::IdentityModule;
use crate::application::dto::{ChangeRoleRequest, UserDto};

/// `GET /api/admin/users` — registered users.
pub async fn list_users(
    State(identity): State<Arc<IdentityModule>>,
    AdminPrincipal(principal): AdminPrincipal,
    QueryParams(query): QueryParams<PageQuery>,
) -> ApiResult<Json<Paginated<UserDto>>> {
    Ok(Json(identity.list_users.execute(principal, query).await?))
}

/// `PATCH /api/admin/users/{id}/role` — promotes or demotes a user.
pub async fn change_user_role(
    State(identity): State<Arc<IdentityModule>>,
    AdminPrincipal(principal): AdminPrincipal,
    PathParam(id): PathParam<Uuid>,
    JsonBody(input): JsonBody<ChangeRoleRequest>,
) -> ApiResult<Json<UserDto>> {
    Ok(Json(
        identity
            .change_user_role
            .execute(principal, id, input)
            .await?,
    ))
}
