//! Request extractors shared by every HTTP adapter: authentication and
//! rejection-aware wrappers around Axum's `Json`, `Path` and `Query` so every
//! error shares the same JSON shape.
//!
//! The principal extractors are generic over the router state: any state
//! exposing an `Arc<dyn PrincipalResolver>` through [`FromRef`] works.

use std::sync::Arc;

use axum::extract::{FromRef, FromRequest, FromRequestParts};
use axum::http::header::AUTHORIZATION;
use axum::http::request::Parts;

use super::error::ApiError;
use crate::error::AppError;
use crate::principal::{Principal, PrincipalResolver};

/// JSON request body; malformed payloads become `422` JSON errors.
#[derive(FromRequest)]
#[from_request(via(axum::Json), rejection(ApiError))]
pub struct JsonBody<T>(pub T);

/// Path parameters; unparsable values (e.g. bad UUID) become `404` JSON errors.
#[derive(FromRequestParts)]
#[from_request(via(axum::extract::Path), rejection(ApiError))]
pub struct PathParam<T>(pub T);

/// Query string; invalid values become `422` JSON errors.
#[derive(FromRequestParts)]
#[from_request(via(axum::extract::Query), rejection(ApiError))]
pub struct QueryParams<T>(pub T);

/// The authenticated caller, resolved from the `Authorization: Bearer` header.
pub struct CurrentPrincipal(pub Principal);

impl<S> FromRequestParts<S> for CurrentPrincipal
where
    S: Send + Sync,
    Arc<dyn PrincipalResolver>: FromRef<S>,
{
    type Rejection = ApiError;

    /// Extracts the bearer token and resolves it into an up-to-date principal.
    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let token = bearer_token(parts)
            .ok_or_else(|| AppError::Unauthorized("missing bearer token".into()))?;
        let resolver = Arc::<dyn PrincipalResolver>::from_ref(state);
        Ok(Self(resolver.resolve(token).await?))
    }
}

/// An authenticated administrator; members get a `403`.
pub struct AdminPrincipal(pub Principal);

impl<S> FromRequestParts<S> for AdminPrincipal
where
    S: Send + Sync,
    Arc<dyn PrincipalResolver>: FromRef<S>,
{
    type Rejection = ApiError;

    /// Authenticates the request, then requires the admin role.
    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let CurrentPrincipal(principal) =
            CurrentPrincipal::from_request_parts(parts, state).await?;
        principal.require_admin()?;
        Ok(Self(principal))
    }
}

/// Reads the token of an `Authorization: Bearer <token>` header.
fn bearer_token(parts: &Parts) -> Option<&str> {
    parts
        .headers
        .get(AUTHORIZATION)?
        .to_str()
        .ok()?
        .strip_prefix("Bearer ")
        .map(str::trim)
        .filter(|token| !token.is_empty())
}

#[cfg(test)]
mod tests {
    use async_trait::async_trait;
    use axum::http::{Request, StatusCode};
    use axum::response::IntoResponse;
    use uuid::Uuid;

    use super::*;
    use crate::error::AppResult;

    /// Resolver accepting the token "admin" and "member" only.
    struct FakeResolver(Uuid);

    #[async_trait]
    impl PrincipalResolver for FakeResolver {
        /// Maps the two known tokens to principals.
        async fn resolve(&self, token: &str) -> AppResult<Principal> {
            match token {
                "admin" => Ok(Principal::admin(self.0)),
                "member" => Ok(Principal::member(self.0)),
                _ => Err(AppError::Unauthorized("invalid token".into())),
            }
        }
    }

    /// Builds request parts carrying the given Authorization header.
    fn parts(header: Option<&str>) -> Parts {
        let mut builder = Request::builder();
        if let Some(value) = header {
            builder = builder.header(AUTHORIZATION, value);
        }
        builder.body(()).unwrap().into_parts().0
    }

    /// Returns a router state made of the fake resolver only.
    fn state() -> Arc<dyn PrincipalResolver> {
        Arc::new(FakeResolver(Uuid::now_v7()))
    }

    /// Returns the HTTP status of an extraction failure.
    fn status(error: ApiError) -> StatusCode {
        error.into_response().status()
    }

    #[test]
    fn reads_bearer_tokens_only() {
        assert_eq!(bearer_token(&parts(Some("Bearer abc"))), Some("abc"));
        assert_eq!(bearer_token(&parts(Some("Basic abc"))), None);
        assert_eq!(bearer_token(&parts(Some("Bearer "))), None);
        assert_eq!(bearer_token(&parts(None)), None);
    }

    #[tokio::test]
    async fn current_principal_requires_a_valid_token() {
        let state = state();
        let CurrentPrincipal(principal) =
            CurrentPrincipal::from_request_parts(&mut parts(Some("Bearer member")), &state)
                .await
                .ok()
                .unwrap();
        assert!(!principal.is_admin);

        for header in [None, Some("Bearer garbage")] {
            let Err(error) = CurrentPrincipal::from_request_parts(&mut parts(header), &state).await
            else {
                panic!("{header:?} must be rejected");
            };
            assert_eq!(status(error), StatusCode::UNAUTHORIZED);
        }
    }

    #[tokio::test]
    async fn admin_principal_rejects_members() {
        let state = state();
        assert!(
            AdminPrincipal::from_request_parts(&mut parts(Some("Bearer admin")), &state)
                .await
                .is_ok()
        );
        let Err(error) =
            AdminPrincipal::from_request_parts(&mut parts(Some("Bearer member")), &state).await
        else {
            panic!("members must be rejected");
        };
        assert_eq!(status(error), StatusCode::FORBIDDEN);
    }
}
