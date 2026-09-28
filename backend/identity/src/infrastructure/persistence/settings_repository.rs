use async_trait::async_trait;
use sea_orm::sea_query::OnConflict;
use sea_orm::{DatabaseConnection, EntityTrait};
use xetaravel_kernel::DomainResult;
use xetaravel_kernel::persistence::db_error;

use super::entities::settings;
use super::mappers::{SETTINGS_ROW_ID, from_identity_settings, to_identity_settings};
use crate::domain::{IdentitySettings, SettingsRepository};

/// PostgreSQL implementation of [`SettingsRepository`] (single-row table).
#[derive(Clone)]
pub struct SeaOrmSettingsRepository {
    db: DatabaseConnection,
}

impl SeaOrmSettingsRepository {
    /// Builds the repository on top of a connection pool.
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }
}

#[async_trait]
impl SettingsRepository for SeaOrmSettingsRepository {
    /// Loads the settings row, or the defaults when it is missing.
    async fn get(&self) -> DomainResult<IdentitySettings> {
        settings::Entity::find_by_id(SETTINGS_ROW_ID)
            .one(&self.db)
            .await
            .map_err(db_error)?
            .map_or_else(|| Ok(IdentitySettings::defaults()), to_identity_settings)
    }

    /// Upserts the settings row.
    async fn save(&self, value: &IdentitySettings) -> DomainResult<()> {
        settings::Entity::insert(from_identity_settings(value))
            .on_conflict(
                OnConflict::column(settings::Column::Id)
                    .update_columns([
                        settings::Column::RegistrationEnabled,
                        settings::Column::UpdatedAt,
                    ])
                    .to_owned(),
            )
            .exec(&self.db)
            .await
            .map_err(db_error)?;
        Ok(())
    }
}
