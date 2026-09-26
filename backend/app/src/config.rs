//! Runtime configuration read from the environment (see `.env.example`).

use std::env;
use std::path::PathBuf;

use chrono::Duration;
use thiserror::Error;
use xetaravel_identity::JwtSettings;

/// Minimum length of the JWT secret, in bytes.
const MIN_JWT_SECRET_LENGTH: usize = 32;

/// Configuration errors raised at startup.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum ConfigError {
    #[error("environment variable {0} is missing")]
    Missing(&'static str),
    #[error("environment variable {0} is invalid: {1}")]
    Invalid(&'static str, String),
}

/// Runtime configuration read from the environment (see `.env.example`).
#[derive(Debug, Clone)]
pub struct Config {
    pub database_url: String,
    pub jwt: JwtSettings,
    pub app_addr: String,
    pub cors_origin: String,
    /// Root directory of the uploaded files (cover images...).
    pub uploads_dir: PathBuf,
}

impl Config {
    /// Reads and validates the configuration from environment variables.
    pub fn from_env() -> Result<Self, ConfigError> {
        Self::from_lookup(|key| env::var(key).ok())
    }

    /// Reads the configuration through `lookup` (injectable for tests).
    pub fn from_lookup(lookup: impl Fn(&str) -> Option<String>) -> Result<Self, ConfigError> {
        let required = |key: &'static str| lookup(key).ok_or(ConfigError::Missing(key));

        let jwt_secret = required("JWT_SECRET")?;
        if jwt_secret.len() < MIN_JWT_SECRET_LENGTH {
            return Err(ConfigError::Invalid(
                "JWT_SECRET",
                format!("must contain at least {MIN_JWT_SECRET_LENGTH} bytes"),
            ));
        }

        let ttl_seconds = lookup("JWT_TTL_SECONDS")
            .map(|raw| {
                raw.parse::<i64>()
                    .ok()
                    .filter(|seconds| *seconds > 0)
                    .ok_or_else(|| {
                        ConfigError::Invalid("JWT_TTL_SECONDS", "must be a positive integer".into())
                    })
            })
            .transpose()?
            .unwrap_or(7 * 24 * 3600);

        Ok(Self {
            database_url: required("DATABASE_URL")?,
            jwt: JwtSettings {
                secret: jwt_secret,
                ttl: Duration::seconds(ttl_seconds),
            },
            app_addr: lookup("APP_ADDR").unwrap_or_else(|| "127.0.0.1:8080".into()),
            cors_origin: lookup("CORS_ORIGIN").unwrap_or_else(|| "http://localhost:3000".into()),
            uploads_dir: lookup("UPLOADS_DIR")
                .unwrap_or_else(|| "storage/uploads".into())
                .into(),
        })
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::*;

    /// Builds a lookup function over the given pairs.
    fn lookup(pairs: &[(&str, &str)]) -> impl Fn(&str) -> Option<String> {
        let map: HashMap<String, String> = pairs
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect();
        move |key| map.get(key).cloned()
    }

    const SECRET: &str = "0123456789abcdef0123456789abcdef";

    #[test]
    fn reads_required_values_and_defaults() {
        let config = Config::from_lookup(lookup(&[
            ("DATABASE_URL", "postgres://db"),
            ("JWT_SECRET", SECRET),
        ]))
        .unwrap();
        assert_eq!(config.database_url, "postgres://db");
        assert_eq!(config.jwt.secret, SECRET);
        assert_eq!(config.jwt.ttl, Duration::days(7));
        assert_eq!(config.app_addr, "127.0.0.1:8080");
        assert_eq!(config.uploads_dir, PathBuf::from("storage/uploads"));
    }

    #[test]
    fn reads_the_uploads_directory() {
        let config = Config::from_lookup(lookup(&[
            ("DATABASE_URL", "postgres://db"),
            ("JWT_SECRET", SECRET),
            ("UPLOADS_DIR", "/var/lib/xetaravel"),
        ]))
        .unwrap();
        assert_eq!(config.uploads_dir, PathBuf::from("/var/lib/xetaravel"));
    }

    #[test]
    fn rejects_missing_or_weak_values() {
        assert_eq!(
            Config::from_lookup(lookup(&[("JWT_SECRET", SECRET)])).unwrap_err(),
            ConfigError::Missing("DATABASE_URL")
        );
        assert!(matches!(
            Config::from_lookup(lookup(&[("DATABASE_URL", "x"), ("JWT_SECRET", "short")])),
            Err(ConfigError::Invalid("JWT_SECRET", _))
        ));
        assert!(matches!(
            Config::from_lookup(lookup(&[
                ("DATABASE_URL", "x"),
                ("JWT_SECRET", SECRET),
                ("JWT_TTL_SECONDS", "-1")
            ])),
            Err(ConfigError::Invalid("JWT_TTL_SECONDS", _))
        ));
    }
}
