//! Runtime configuration read from the environment (see `.env.example`).

use std::env;
use std::path::PathBuf;

use chrono::Duration;
use thiserror::Error;
use xetaravel_identity::{CaptchaSettings, JwtSettings, TURNSTILE_SITEVERIFY_URL};

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

/// Per-IP rate limit of the credential routes (login, register).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RateLimitSettings {
    /// Requests a single IP may send in a row.
    pub burst: u32,
    /// Seconds after which one more request is allowed.
    pub period_seconds: u64,
}

impl Default for RateLimitSettings {
    /// 5 attempts in a row, then one every 12 seconds (5 per minute).
    fn default() -> Self {
        Self {
            burst: 5,
            period_seconds: 12,
        }
    }
}

/// Runtime configuration read from the environment (see `.env.example`).
#[derive(Debug, Clone)]
pub struct Config {
    pub database_url: String,
    pub jwt: JwtSettings,
    /// Captcha of the credential routes (mandatory `TURNSTILE_SECRET`).
    pub captcha: CaptchaSettings,
    pub auth_rate_limit: RateLimitSettings,
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

        let turnstile_secret = required("TURNSTILE_SECRET")?;
        if turnstile_secret.is_empty() {
            return Err(ConfigError::Invalid(
                "TURNSTILE_SECRET",
                "must not be empty".into(),
            ));
        }

        let ttl_seconds = positive(&lookup, "JWT_TTL_SECONDS")?.unwrap_or(7 * 24 * 3600);
        let defaults = RateLimitSettings::default();

        Ok(Self {
            database_url: required("DATABASE_URL")?,
            jwt: JwtSettings {
                secret: jwt_secret,
                ttl: Duration::seconds(ttl_seconds),
            },
            captcha: CaptchaSettings {
                turnstile_secret,
                siteverify_url: TURNSTILE_SITEVERIFY_URL.into(),
            },
            auth_rate_limit: RateLimitSettings {
                burst: positive(&lookup, "AUTH_RATE_LIMIT_BURST")?.unwrap_or(defaults.burst),
                period_seconds: positive(&lookup, "AUTH_RATE_LIMIT_PERIOD_SECONDS")?
                    .unwrap_or(defaults.period_seconds),
            },
            app_addr: lookup("APP_ADDR").unwrap_or_else(|| "127.0.0.1:8080".into()),
            cors_origin: lookup("CORS_ORIGIN").unwrap_or_else(|| "http://localhost:3000".into()),
            uploads_dir: lookup("UPLOADS_DIR")
                .unwrap_or_else(|| "storage/uploads".into())
                .into(),
        })
    }
}

/// Reads the optional variable `key` as a strictly positive integer.
fn positive<T>(
    lookup: &impl Fn(&str) -> Option<String>,
    key: &'static str,
) -> Result<Option<T>, ConfigError>
where
    T: std::str::FromStr + PartialOrd + Default,
{
    lookup(key)
        .map(|raw| {
            raw.parse::<T>()
                .ok()
                .filter(|value| *value > T::default())
                .ok_or_else(|| ConfigError::Invalid(key, "must be a positive integer".into()))
        })
        .transpose()
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::*;

    /// Builds a lookup function over the given pairs.
    fn lookup(pairs: &[(&str, &str)]) -> impl Fn(&str) -> Option<String> + use<> {
        let map: HashMap<String, String> = pairs
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect();
        move |key| map.get(key).cloned()
    }

    const SECRET: &str = "0123456789abcdef0123456789abcdef";

    /// Required variables every valid configuration contains.
    const BASE: [(&str, &str); 3] = [
        ("DATABASE_URL", "postgres://db"),
        ("JWT_SECRET", SECRET),
        ("TURNSTILE_SECRET", "turnstile"),
    ];

    /// Builds a lookup over [`BASE`] overridden by `extra`.
    fn with_base(extra: &[(&str, &str)]) -> impl Fn(&str) -> Option<String> {
        let mut pairs: Vec<(&str, &str)> = BASE
            .iter()
            .filter(|(key, _)| extra.iter().all(|(k, _)| k != key))
            .copied()
            .collect();
        pairs.extend_from_slice(extra);
        lookup(&pairs)
    }

    /// Builds a lookup over [`BASE`] without the variable `missing`.
    fn without(missing: &str) -> impl Fn(&str) -> Option<String> {
        let pairs: Vec<(&str, &str)> = BASE
            .iter()
            .filter(|(key, _)| *key != missing)
            .copied()
            .collect();
        lookup(&pairs)
    }

    #[test]
    fn reads_required_values_and_defaults() {
        let config = Config::from_lookup(with_base(&[])).unwrap();
        assert_eq!(config.database_url, "postgres://db");
        assert_eq!(config.jwt.secret, SECRET);
        assert_eq!(config.jwt.ttl, Duration::days(7));
        assert_eq!(config.app_addr, "127.0.0.1:8080");
        assert_eq!(config.uploads_dir, PathBuf::from("storage/uploads"));
        assert_eq!(config.captcha.turnstile_secret, "turnstile");
        assert_eq!(config.captcha.siteverify_url, TURNSTILE_SITEVERIFY_URL);
        assert_eq!(
            config.auth_rate_limit,
            RateLimitSettings {
                burst: 5,
                period_seconds: 12
            }
        );
    }

    #[test]
    fn reads_the_rate_limit() {
        let config = Config::from_lookup(with_base(&[
            ("AUTH_RATE_LIMIT_BURST", "10"),
            ("AUTH_RATE_LIMIT_PERIOD_SECONDS", "30"),
        ]))
        .unwrap();
        assert_eq!(
            config.auth_rate_limit,
            RateLimitSettings {
                burst: 10,
                period_seconds: 30
            }
        );
    }

    #[test]
    fn requires_the_captcha_secret() {
        assert_eq!(
            Config::from_lookup(without("TURNSTILE_SECRET")).unwrap_err(),
            ConfigError::Missing("TURNSTILE_SECRET")
        );
    }

    #[test]
    fn rejects_an_empty_captcha_secret() {
        assert_eq!(
            Config::from_lookup(with_base(&[("TURNSTILE_SECRET", "")])).unwrap_err(),
            ConfigError::Invalid("TURNSTILE_SECRET", "must not be empty".into())
        );
    }

    #[test]
    fn reads_the_uploads_directory() {
        let config =
            Config::from_lookup(with_base(&[("UPLOADS_DIR", "/var/lib/xetaravel")])).unwrap();
        assert_eq!(config.uploads_dir, PathBuf::from("/var/lib/xetaravel"));
    }

    #[test]
    fn rejects_missing_or_weak_values() {
        assert_eq!(
            Config::from_lookup(without("DATABASE_URL")).unwrap_err(),
            ConfigError::Missing("DATABASE_URL")
        );
        assert!(matches!(
            Config::from_lookup(with_base(&[("JWT_SECRET", "short")])),
            Err(ConfigError::Invalid("JWT_SECRET", _))
        ));
        assert!(matches!(
            Config::from_lookup(with_base(&[("JWT_TTL_SECONDS", "-1")])),
            Err(ConfigError::Invalid("JWT_TTL_SECONDS", _))
        ));
        for key in ["AUTH_RATE_LIMIT_BURST", "AUTH_RATE_LIMIT_PERIOD_SECONDS"] {
            for value in ["0", "-3", "abc"] {
                assert_eq!(
                    Config::from_lookup(with_base(&[(key, value)])).unwrap_err(),
                    ConfigError::Invalid(key, "must be a positive integer".into()),
                    "{key}={value}"
                );
            }
        }
    }
}
