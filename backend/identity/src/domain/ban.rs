use chrono::{DateTime, Utc};

use super::BanReason;

/// A ban placed on an account: it stays until an admin lifts it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Ban {
    pub reason: Option<BanReason>,
    pub banned_at: DateTime<Utc>,
}
