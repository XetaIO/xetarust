//! Data Transfer Objects of the Identity context (public API contract).
//!
//! TypeScript bindings are generated in `frontend/src/types/api/identity/`.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use ts_rs::TS;
use uuid::Uuid;
use validator::Validate;

use crate::domain::{Role, User};

/// Body of `POST /api/auth/register`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Validate, TS)]
#[ts(export, export_to = "identity/")]
pub struct RegisterRequest {
    #[validate(length(
        min = 3,
        max = 30,
        message = "must contain between 3 and 30 characters"
    ))]
    pub username: String,
    #[validate(email(message = "is not a valid address"))]
    pub email: String,
    #[validate(length(
        min = 8,
        max = 128,
        message = "must contain between 8 and 128 characters"
    ))]
    pub password: String,
    /// Response of the captcha widget (Cloudflare Turnstile).
    #[serde(default)]
    #[validate(length(min = 1, message = "is required"))]
    pub captcha_token: String,
}

/// Body of `POST /api/auth/login`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Validate, TS)]
#[ts(export, export_to = "identity/")]
pub struct LoginRequest {
    #[validate(length(min = 1, message = "is required"))]
    pub email: String,
    #[validate(length(min = 1, message = "is required"))]
    pub password: String,
    /// Response of the captcha widget (Cloudflare Turnstile).
    #[serde(default)]
    #[validate(length(min = 1, message = "is required"))]
    pub captcha_token: String,
}

/// Returned after a successful registration or login.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(export, export_to = "identity/")]
pub struct AuthResponse {
    /// JWT to send as `Authorization: Bearer <token>`.
    pub token: String,
    pub expires_at: DateTime<Utc>,
    pub user: UserDto,
}

/// Role of a user as exposed by the API.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "lowercase")]
#[ts(export, export_to = "identity/")]
pub enum RoleDto {
    Member,
    Admin,
}

impl From<Role> for RoleDto {
    /// Converts a domain role.
    fn from(role: Role) -> Self {
        match role {
            Role::Member => Self::Member,
            Role::Admin => Self::Admin,
        }
    }
}

impl From<RoleDto> for Role {
    /// Converts back to a domain role.
    fn from(role: RoleDto) -> Self {
        match role {
            RoleDto::Member => Self::Member,
            RoleDto::Admin => Self::Admin,
        }
    }
}

/// Private view of an account (current user, administration).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(export, export_to = "identity/")]
pub struct UserDto {
    pub id: Uuid,
    pub username: String,
    pub email: String,
    pub role: RoleDto,
    pub created_at: DateTime<Utc>,
    pub banned_at: Option<DateTime<Utc>>,
    pub ban_reason: Option<String>,
}

impl From<&User> for UserDto {
    /// Builds the DTO from a domain user (never exposes the password hash).
    fn from(user: &User) -> Self {
        Self {
            id: user.id.as_uuid(),
            username: user.username.to_string(),
            email: user.email.to_string(),
            role: user.role.into(),
            created_at: user.created_at,
            banned_at: user.ban.as_ref().map(|ban| ban.banned_at),
            ban_reason: user
                .ban
                .as_ref()
                .and_then(|ban| ban.reason.as_ref())
                .map(ToString::to_string),
        }
    }
}

/// Body of `PATCH /api/admin/users/{id}/role`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(export, export_to = "identity/")]
pub struct ChangeRoleRequest {
    pub role: RoleDto,
}

/// Body of `PUT /api/admin/users/{id}/ban`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Validate, TS)]
#[ts(export, export_to = "identity/")]
pub struct BanUserRequest {
    /// Optional reason, shown in the dashboard and to the banned user at login.
    #[serde(default)]
    #[ts(optional = nullable)]
    #[validate(length(max = 255, message = "must contain at most 255 characters"))]
    pub reason: Option<String>,
}
