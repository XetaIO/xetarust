//! Use cases of the Identity context. Each use case is a struct holding its
//! dependencies as `Arc<dyn Port>` and exposing a single `execute` method.
//! Administration use cases (except the CLI-only [`PromoteUserToAdmin`])
//! start with `principal.require_admin()`.

mod authenticate;
mod change_user_role;
mod get_current_user;
mod get_public_profiles;
mod list_users;
mod login_user;
mod promote_user_to_admin;
mod register_user;

pub use authenticate::Authenticate;
pub use change_user_role::ChangeUserRole;
pub use get_current_user::GetCurrentUser;
pub use get_public_profiles::GetPublicProfiles;
pub use list_users::ListUsers;
pub use login_user::LoginUser;
pub use promote_user_to_admin::PromoteUserToAdmin;
pub use register_user::RegisterUser;

use xetaravel_kernel::{AppResult, Principal};

use crate::application::dto::AuthResponse;
use crate::application::ports::TokenService;
use crate::domain::User;

/// Issues a token for `user` and wraps it in an [`AuthResponse`].
fn authenticated_response(tokens: &dyn TokenService, user: &User) -> AppResult<AuthResponse> {
    let issued = tokens.issue(user)?;
    Ok(AuthResponse {
        token: issued.token,
        expires_at: issued.expires_at,
        user: user.into(),
    })
}

/// Builds the principal other contexts will see for `user`.
pub(crate) fn principal_of(user: &User) -> Principal {
    Principal {
        user_id: user.id.as_uuid(),
        is_admin: user.is_admin(),
    }
}
