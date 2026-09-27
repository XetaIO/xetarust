//! Assembly of the Identity context: builds its adapters once and injects
//! them into every use case.

use std::sync::Arc;

use sea_orm::DatabaseConnection;
use xetaravel_kernel::{Clock, PrincipalResolver};

use crate::application::contract::IdentityDirectory;
use crate::application::ports::{HumanVerifier, PasswordHasher, TokenService};
use crate::application::use_cases::{
    Authenticate, ChangeUserRole, GetCurrentUser, GetPublicProfiles, ListUsers, LoginUser,
    RegisterUser,
};
use crate::domain::UserRepository;
use crate::infrastructure::persistence::SeaOrmUserRepository;
use crate::infrastructure::security::{
    Argon2PasswordHasher, CaptchaSettings, DisabledHumanVerifier, JwtSettings, JwtTokenService,
    TURNSTILE_SITEVERIFY_URL, TurnstileHumanVerifier,
};

/// Every use case of the Identity context, ready to be shared by the HTTP
/// adapter and the other contexts (through [`Self::principals`]
/// and [`Self::directory`]).
pub struct IdentityModule {
    pub register: RegisterUser,
    pub login: LoginUser,
    pub current_user: GetCurrentUser,
    pub list_users: ListUsers,
    pub change_user_role: ChangeUserRole,
    authenticate: Arc<Authenticate>,
    public_profiles: Arc<GetPublicProfiles>,
}

impl IdentityModule {
    /// Wires the production adapters (PostgreSQL, Argon2, JWT, Turnstile)
    /// into the use cases. Without a Turnstile secret, the captcha is disabled.
    pub fn new(
        db: DatabaseConnection,
        jwt: &JwtSettings,
        captcha: &CaptchaSettings,
        clock: Arc<dyn Clock>,
    ) -> Self {
        let users: Arc<dyn UserRepository> = Arc::new(SeaOrmUserRepository::new(db));
        let hasher: Arc<dyn PasswordHasher> = Arc::new(Argon2PasswordHasher);
        let tokens: Arc<dyn TokenService> = Arc::new(JwtTokenService::new(jwt, clock.clone()));
        let humans = human_verifier(captcha);

        Self {
            register: RegisterUser::new(
                users.clone(),
                hasher.clone(),
                tokens.clone(),
                humans.clone(),
                clock.clone(),
            ),
            login: LoginUser::new(users.clone(), hasher, tokens.clone(), humans),
            current_user: GetCurrentUser::new(users.clone()),
            list_users: ListUsers::new(users.clone()),
            change_user_role: ChangeUserRole::new(users.clone(), clock),
            authenticate: Arc::new(Authenticate::new(users.clone(), tokens)),
            public_profiles: Arc::new(GetPublicProfiles::new(users)),
        }
    }

    /// Returns the resolver turning bearer tokens into principals (kernel port).
    pub fn principals(&self) -> Arc<dyn PrincipalResolver> {
        self.authenticate.clone()
    }

    /// Returns the public directory other contexts use to name authors.
    pub fn directory(&self) -> Arc<dyn IdentityDirectory> {
        self.public_profiles.clone()
    }
}

/// Picks the captcha adapter: Turnstile when a secret is configured,
/// otherwise the always-accepting one.
fn human_verifier(captcha: &CaptchaSettings) -> Arc<dyn HumanVerifier> {
    match &captcha.turnstile_secret {
        Some(secret) => Arc::new(TurnstileHumanVerifier::new(
            secret.clone(),
            TURNSTILE_SITEVERIFY_URL,
        )),
        None => Arc::new(DisabledHumanVerifier),
    }
}
