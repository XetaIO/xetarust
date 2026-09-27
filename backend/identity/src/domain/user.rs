use chrono::{DateTime, Utc};
use xetaravel_kernel::{DomainError, DomainResult};

use super::{Email, PasswordHash, Role, UserId, Username};

/// A registered account.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct User {
    pub id: UserId,
    pub username: Username,
    pub email: Email,
    pub password_hash: PasswordHash,
    pub role: Role,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl User {
    /// Registers a brand new account. New accounts are always members.
    pub fn register(
        username: Username,
        email: Email,
        password_hash: PasswordHash,
        now: DateTime<Utc>,
    ) -> Self {
        Self {
            id: UserId::generate(),
            username,
            email,
            password_hash,
            role: Role::Member,
            created_at: now,
            updated_at: now,
        }
    }

    /// Tells whether the user can access the administration.
    pub fn is_admin(&self) -> bool {
        self.role.is_admin()
    }

    /// Changes the role of `self` on behalf of `actor`.
    ///
    /// Rules: only an admin can change roles, and an admin can never demote
    /// themselves (it guarantees at least one admin always remains).
    pub fn change_role(
        &mut self,
        role: Role,
        actor_id: UserId,
        actor_role: Role,
        now: DateTime<Utc>,
    ) -> DomainResult<()> {
        if !actor_role.is_admin() {
            return Err(DomainError::Forbidden(
                "only an admin can change roles".into(),
            ));
        }
        if actor_id == self.id && !role.is_admin() {
            return Err(DomainError::Forbidden(
                "an admin cannot demote themselves".into(),
            ));
        }

        self.role = role;
        self.updated_at = now;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use chrono::Duration;

    use super::*;

    /// Builds a freshly registered member for the tests.
    fn member(now: DateTime<Utc>) -> User {
        User::register(
            Username::parse("john").unwrap(),
            Email::parse("john@example.com").unwrap(),
            PasswordHash::new("hash"),
            now,
        )
    }

    #[test]
    fn register_creates_a_member() {
        let now = Utc::now();
        let user = member(now);
        assert_eq!(user.role, Role::Member);
        assert!(!user.is_admin());
        assert_eq!(user.created_at, now);
        assert_eq!(user.updated_at, now);
    }

    #[test]
    fn admin_can_promote_another_user() {
        let now = Utc::now();
        let mut user = member(now);
        let later = now + Duration::minutes(5);

        user.change_role(Role::Admin, UserId::generate(), Role::Admin, later)
            .unwrap();

        assert!(user.is_admin());
        assert_eq!(user.updated_at, later);
    }

    #[test]
    fn member_cannot_change_roles() {
        let mut user = member(Utc::now());
        let result = user.change_role(Role::Admin, UserId::generate(), Role::Member, Utc::now());
        assert!(matches!(result, Err(DomainError::Forbidden(_))));
        assert_eq!(user.role, Role::Member);
    }

    #[test]
    fn admin_cannot_demote_themselves() {
        let mut admin = member(Utc::now());
        admin.role = Role::Admin;
        let id = admin.id;

        let result = admin.change_role(Role::Member, id, Role::Admin, Utc::now());

        assert!(matches!(result, Err(DomainError::Forbidden(_))));
        assert!(admin.is_admin());
    }
}
