use async_trait::async_trait;

use xetaravel_kernel::DomainResult;

use super::IdentitySettings;

/// Persistence port of the [`IdentitySettings`] singleton.
#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait SettingsRepository: Send + Sync {
    /// Loads the settings, or [`IdentitySettings::defaults`] when none were saved.
    async fn get(&self) -> DomainResult<IdentitySettings>;

    /// Saves the settings (creates them when missing).
    async fn save(&self, settings: &IdentitySettings) -> DomainResult<()>;
}
