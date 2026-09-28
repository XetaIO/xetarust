//! Settings of the Identity context an admin can change at runtime
//! (whether new accounts can be created).

use chrono::{DateTime, Utc};
use xetaravel_kernel::{DomainError, DomainResult};

/// Runtime settings of the Identity context (a single instance exists).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IdentitySettings {
    /// Whether visitors can create an account.
    pub registration_enabled: bool,
    /// Last time an admin changed the settings (`None`: never changed).
    pub updated_at: Option<DateTime<Utc>>,
}

impl IdentitySettings {
    /// Returns the settings used until an admin changes them: registrations open.
    pub fn defaults() -> Self {
        Self {
            registration_enabled: true,
            updated_at: None,
        }
    }

    /// Fails with [`DomainError::Forbidden`] when registrations are closed.
    pub fn ensure_registration_open(&self) -> DomainResult<()> {
        if self.registration_enabled {
            Ok(())
        } else {
            Err(DomainError::Forbidden("registration is disabled".into()))
        }
    }

    /// Opens or closes registrations at `now`.
    pub fn set_registration(&mut self, enabled: bool, now: DateTime<Utc>) {
        self.registration_enabled = enabled;
        self.updated_at = Some(now);
    }
}

#[cfg(test)]
mod tests {
    use chrono::TimeZone;

    use super::*;

    /// Returns a fixed instant for the tests.
    fn now() -> DateTime<Utc> {
        Utc.with_ymd_and_hms(2026, 1, 1, 12, 0, 0).unwrap()
    }

    #[test]
    fn registrations_are_open_by_default() {
        let settings = IdentitySettings::defaults();

        assert!(settings.registration_enabled);
        assert_eq!(settings.updated_at, None);
        assert_eq!(settings.ensure_registration_open(), Ok(()));
    }

    #[test]
    fn closed_registrations_are_forbidden() {
        let mut settings = IdentitySettings::defaults();
        settings.set_registration(false, now());

        assert_eq!(
            settings.ensure_registration_open(),
            Err(DomainError::Forbidden("registration is disabled".into()))
        );
    }

    #[test]
    fn set_registration_records_the_change_time() {
        let mut settings = IdentitySettings::defaults();
        settings.set_registration(false, now());

        assert!(!settings.registration_enabled);
        assert_eq!(settings.updated_at, Some(now()));
    }
}
