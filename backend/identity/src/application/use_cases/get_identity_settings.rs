use std::sync::Arc;

use xetaravel_kernel::AppResult;

use crate::application::dto::IdentitySettingsDto;
use crate::domain::SettingsRepository;

/// Returns the public settings of the Identity context (e.g. whether the
/// frontend should offer to sign up).
pub struct GetIdentitySettings {
    settings: Arc<dyn SettingsRepository>,
}

impl GetIdentitySettings {
    /// Builds the use case with its dependencies.
    pub fn new(settings: Arc<dyn SettingsRepository>) -> Self {
        Self { settings }
    }

    /// Loads the current settings (public, no principal needed).
    pub async fn execute(&self) -> AppResult<IdentitySettingsDto> {
        Ok((&self.settings.get().await?).into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::{IdentitySettings, MockSettingsRepository};

    #[tokio::test]
    async fn returns_the_stored_settings() {
        let mut settings = MockSettingsRepository::new();
        settings.expect_get().returning(|| {
            Ok(IdentitySettings {
                registration_enabled: false,
                updated_at: None,
            })
        });

        let dto = GetIdentitySettings::new(Arc::new(settings))
            .execute()
            .await
            .unwrap();

        assert!(!dto.registration_enabled);
    }
}
