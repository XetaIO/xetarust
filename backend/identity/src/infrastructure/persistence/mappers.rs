//! Conversions between `users` rows and the [`User`] aggregate.
//!
//! Loading a row re-runs the value object validation; a failure means the
//! database holds corrupted data and is reported as a repository error.

use chrono::{DateTime, Utc};
use sea_orm::ActiveValue::Set;
use xetaravel_kernel::DomainResult;
use xetaravel_kernel::persistence::corrupted;

use super::entity::{self as user, UserRole};
use crate::domain::{Ban, BanReason, Email, PasswordHash, Role, User, Username};

impl From<UserRole> for Role {
    /// Converts the database enum into the domain role.
    fn from(role: UserRole) -> Self {
        match role {
            UserRole::Member => Role::Member,
            UserRole::Admin => Role::Admin,
        }
    }
}

impl From<Role> for UserRole {
    /// Converts the domain role into the database enum.
    fn from(role: Role) -> Self {
        match role {
            Role::Member => UserRole::Member,
            Role::Admin => UserRole::Admin,
        }
    }
}

/// Converts a `users` row into a domain user.
pub(super) fn to_user(model: user::Model) -> DomainResult<User> {
    Ok(User {
        id: model.id.into(),
        username: Username::parse(&model.username).map_err(|e| corrupted("users", e))?,
        email: Email::parse(&model.email).map_err(|e| corrupted("users", e))?,
        password_hash: PasswordHash::new(model.password_hash),
        role: model.role.into(),
        ban: to_ban(model.banned_at, model.ban_reason.as_deref())?,
        created_at: model.created_at,
        updated_at: model.updated_at,
    })
}

/// Rebuilds the optional ban from its columns: an account is banned when
/// `banned_at` is set (the reason is ignored otherwise).
fn to_ban(banned_at: Option<DateTime<Utc>>, reason: Option<&str>) -> DomainResult<Option<Ban>> {
    let Some(banned_at) = banned_at else {
        return Ok(None);
    };
    let reason = reason
        .map(BanReason::parse)
        .transpose()
        .map_err(|e| corrupted("users", e))?;
    Ok(Some(Ban { reason, banned_at }))
}

/// Converts a domain user into a fully set active model.
pub(super) fn from_user(user: &User) -> user::ActiveModel {
    user::ActiveModel {
        id: Set(user.id.as_uuid()),
        username: Set(user.username.to_string()),
        email: Set(user.email.to_string()),
        password_hash: Set(user.password_hash.as_str().to_owned()),
        role: Set(user.role.into()),
        banned_at: Set(user.ban.as_ref().map(|ban| ban.banned_at)),
        ban_reason: Set(user
            .ban
            .as_ref()
            .and_then(|ban| ban.reason.as_ref())
            .map(|reason| reason.to_string())),
        created_at: Set(user.created_at),
        updated_at: Set(user.updated_at),
    }
}
