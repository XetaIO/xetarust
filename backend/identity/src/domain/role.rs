use std::{fmt, str::FromStr};

use xetaravel_kernel::DomainError;

/// Authorization level of a user. Every new account is a [`Role::Member`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Role {
    #[default]
    Member,
    Admin,
}

impl Role {
    /// Returns the canonical lowercase name of the role.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Member => "member",
            Self::Admin => "admin",
        }
    }

    /// Tells whether the role grants access to the administration.
    pub fn is_admin(&self) -> bool {
        matches!(self, Self::Admin)
    }
}

impl FromStr for Role {
    type Err = DomainError;

    /// Parses a role from its canonical lowercase name.
    fn from_str(raw: &str) -> Result<Self, Self::Err> {
        match raw {
            "member" => Ok(Self::Member),
            "admin" => Ok(Self::Admin),
            _ => Err(DomainError::validation(
                "role",
                "must be 'member' or 'admin'",
            )),
        }
    }
}

impl fmt::Display for Role {
    /// Formats the role with its canonical name.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_to_member() {
        assert_eq!(Role::default(), Role::Member);
    }

    #[test]
    fn round_trips_through_str() {
        for role in [Role::Member, Role::Admin] {
            assert_eq!(role.as_str().parse::<Role>().unwrap(), role);
        }
        assert!("root".parse::<Role>().is_err());
    }

    #[test]
    fn only_admin_is_admin() {
        assert!(Role::Admin.is_admin());
        assert!(!Role::Member.is_admin());
    }
}
