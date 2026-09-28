use std::sync::Arc;

use xetaravel_kernel::{AppResult, Clock, Principal};

use crate::application::dto::{IdentitySettingsDto, UpdateIdentitySettingsRequest};
use crate::domain::SettingsRepository;

/// Changes the runtime settings of the Identity context (admin only).
pub struct UpdateIdentitySettings {
    settings: Arc<dyn SettingsRepository>,
    clock: Arc<dyn Clock>,
}

impl UpdateIdentitySettings {
    /// Builds the use case with its dependencies.
    pub fn new(settings: Arc<dyn SettingsRepository>, clock: Arc<dyn Clock>) -> Self {
        Self { settings, clock }
    }

    /// Opens or closes registrations and returns the saved settings.
    pub async fn execute(
        &self,
        principal: Principal,
        input: UpdateIdentitySettingsRequest,
    ) -> AppResult<IdentitySettingsDto> {
        principal.require_admin()?;
        let mut settings = self.settings.get().await?;

        settings.set_registration(input.registration_enabled, self.clock.now());
        self.settings.save(&settings).await?;

        Ok((&settings).into())
    }
}

#[cfg(test)]
mod tests {
    use xetaravel_kernel::AppError;

    use super::*;
    use crate::application::test_support::{admin_principal, clock, member_principal, now};
    use crate::domain::{IdentitySettings, MockSettingsRepository};

    /// Returns a request closing registrations.
    fn close() -> UpdateIdentitySettingsRequest {
        UpdateIdentitySettingsRequest {
            registration_enabled: false,
        }
    }

    #[tokio::test]
    async fn admins_close_registrations() {
        let mut settings = MockSettingsRepository::new();
        settings
            .expect_get()
            .returning(|| Ok(IdentitySettings::defaults()));
        settings
            .expect_save()
            .withf(|s| !s.registration_enabled && s.updated_at == Some(now()))
            .times(1)
            .returning(|_| Ok(()));

        let dto = UpdateIdentitySettings::new(Arc::new(settings), clock())
            .execute(admin_principal(), close())
            .await
            .unwrap();

        assert!(!dto.registration_enabled);
    }

    #[tokio::test]
    async fn members_cannot_change_the_settings() {
        let mut settings = MockSettingsRepository::new();
        settings.expect_save().times(0);

        let error = UpdateIdentitySettings::new(Arc::new(settings), clock())
            .execute(member_principal(), close())
            .await
            .unwrap_err();

        assert!(matches!(error, AppError::Forbidden(_)));
    }
}
