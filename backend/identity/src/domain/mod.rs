//! Business rules of the Identity context: the [`User`] aggregate, its
//! self-validating value objects, the [`IdentitySettings`] singleton and
//! their persistence ports.
//!
//! Framework-free: no SeaORM, no Axum, no serde.

mod ban;
mod ban_reason;
mod email;
mod identity_settings;
mod ids;
mod password_hash;
mod role;
mod settings_repository;
mod user;
mod user_repository;
mod username;

pub use ban::Ban;
pub use ban_reason::BanReason;
pub use email::Email;
pub use identity_settings::IdentitySettings;
pub use ids::UserId;
pub use password_hash::PasswordHash;
pub use role::Role;
pub use settings_repository::SettingsRepository;
pub use user::User;
pub use user_repository::UserRepository;
pub use username::Username;

#[cfg(test)]
pub use settings_repository::MockSettingsRepository;
#[cfg(test)]
pub use user_repository::MockUserRepository;
