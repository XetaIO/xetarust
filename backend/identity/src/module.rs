//! Assembly of the Identity context: builds its adapters once and injects
//! them into every use case.

use std::sync::Arc;

use sea_orm::DatabaseConnection;
use xetaravel_kernel::{Clock, PrincipalResolver};

use crate::application::contract::IdentityDirectory;
use crate::application::ports::{HumanVerifier, PasswordHasher, TokenService};
use crate::application::use_cases::{
    Authenticate, BanUser, ChangeUserRole, GetCurrentUser, GetPublicProfiles, ListUsers, LoginUser,
    RegisterUser, UnbanUser,
};
use crate::domain::UserRepository;
use crate::infrastructure::persistence::SeaOrmUserRepository;
use crate::infrastructure::security::{
    Argon2PasswordHasher, CaptchaSettings, JwtSettings, JwtTokenService, TurnstileHumanVerifier,
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
    pub ban_user: BanUser,
    pub unban_user: UnbanUser,
    authenticate: Arc<Authenticate>,
    public_profiles: Arc<GetPublicProfiles>,
}

impl IdentityModule {
    /// Wires the production adapters (PostgreSQL, Argon2, JWT, Turnstile)
    /// into the use cases. The captcha is always checked against `siteverify`.
    pub fn new(
        db: DatabaseConnection,
        jwt: &JwtSettings,
        captcha: &CaptchaSettings,
        clock: Arc<dyn Clock>,
    ) -> Self {
        let users: Arc<dyn UserRepository> = Arc::new(SeaOrmUserRepository::new(db));
        let hasher: Arc<dyn PasswordHasher> = Arc::new(Argon2PasswordHasher);
        let tokens: Arc<dyn TokenService> = Arc::new(JwtTokenService::new(jwt, clock.clone()));
        let humans: Arc<dyn HumanVerifier> = Arc::new(TurnstileHumanVerifier::new(
            captcha.turnstile_secret.clone(),
            captcha.siteverify_url.clone(),
        ));

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
            change_user_role: ChangeUserRole::new(users.clone(), clock.clone()),
            ban_user: BanUser::new(users.clone(), clock.clone()),
            unban_user: UnbanUser::new(users.clone(), clock),
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
