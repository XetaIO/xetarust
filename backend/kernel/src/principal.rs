//! The authenticated caller, as every bounded context sees it.
//!
//! Identity owns users and roles; the other contexts only need to know *who*
//! calls (an opaque user id) and whether they are an administrator.

use async_trait::async_trait;
use uuid::Uuid;

use crate::error::{AppError, AppResult};

/// The authenticated user performing a use case.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Principal {
    pub user_id: Uuid,
    pub is_admin: bool,
}

impl Principal {
    /// Builds a principal without administration rights.
    pub fn member(user_id: Uuid) -> Self {
        Self {
            user_id,
            is_admin: false,
        }
    }

    /// Builds a principal with administration rights.
    pub fn admin(user_id: Uuid) -> Self {
        Self {
            user_id,
            is_admin: true,
        }
    }

    /// Fails with [`AppError::Forbidden`] unless the principal is an admin.
    pub fn require_admin(&self) -> AppResult<()> {
        if self.is_admin {
            Ok(())
        } else {
            Err(AppError::Forbidden(
                "this action is reserved to administrators".into(),
            ))
        }
    }
}

/// Resolves an access token into an up-to-date [`Principal`].
/// Implemented by the Identity context.
#[async_trait]
pub trait PrincipalResolver: Send + Sync {
    /// Verifies `token` and returns the principal it identifies.
    async fn resolve(&self, token: &str) -> AppResult<Principal>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn require_admin_only_accepts_admins() {
        let id = Uuid::now_v7();
        assert!(Principal::admin(id).require_admin().is_ok());
        assert!(matches!(
            Principal::member(id).require_admin(),
            Err(AppError::Forbidden(_))
        ));
    }
}
