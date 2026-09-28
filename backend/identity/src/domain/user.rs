use chrono::{DateTime, Utc};
use xetaravel_kernel::{DomainError, DomainResult};

use super::{Ban, BanReason, Email, PasswordHash, Role, UserId, Username};

/// A registered account.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct User {
    pub id: UserId,
    pub username: Username,
    pub email: Email,
    pub password_hash: PasswordHash,
    pub role: Role,
    /// Active ban, if any: a banned account can neither log in nor act.
    pub ban: Option<Ban>,
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
            ban: None,
            created_at: now,
            updated_at: now,
        }
    }

    /// Tells whether the user can access the administration.
    pub fn is_admin(&self) -> bool {
        self.role.is_admin()
    }

    /// Tells whether the account is currently banned.
    pub fn is_banned(&self) -> bool {
        self.ban.is_some()
    }

    /// Changes the role of `self` on behalf of `actor`.
    ///
    /// Rules: only an admin can change roles, an admin can never demote
    /// themselves (it guarantees at least one admin always remains) and a
    /// banned account cannot be promoted.
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
        if role.is_admin() && self.is_banned() {
            return Err(DomainError::Forbidden(
                "a banned account cannot be promoted".into(),
            ));
        }

        self.role = role;
        self.updated_at = now;
        Ok(())
    }

    /// Bans `self` on behalf of `actor`, until an admin lifts the ban.
    ///
    /// Rules: only an admin can ban, nobody can ban themselves and an admin
    /// cannot be banned (demote them first). Banning a banned account only
    /// replaces the reason and keeps the original date.
    pub fn ban(
        &mut self,
        reason: Option<BanReason>,
        actor_id: UserId,
        actor_role: Role,
        now: DateTime<Utc>,
    ) -> DomainResult<()> {
        if !actor_role.is_admin() {
            return Err(DomainError::Forbidden("only an admin can ban users".into()));
        }
        if actor_id == self.id {
            return Err(DomainError::Forbidden("you cannot ban yourself".into()));
        }
        if self.is_admin() {
            return Err(DomainError::Forbidden(
                "an admin cannot be banned, demote them first".into(),
            ));
        }

        let banned_at = self.ban.as_ref().map_or(now, |ban| ban.banned_at);
        self.ban = Some(Ban { reason, banned_at });
        self.updated_at = now;
        Ok(())
    }

    /// Lifts the ban of `self` on behalf of an actor with `actor_role`.
    /// Unbanning an account that is not banned changes nothing.
    pub fn unban(&mut self, actor_role: Role, now: DateTime<Utc>) -> DomainResult<()> {
        if !actor_role.is_admin() {
            return Err(DomainError::Forbidden(
                "only an admin can unban users".into(),
            ));
        }
        if self.ban.take().is_some() {
            self.updated_at = now;
        }
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

    /// Returns a reason for the ban tests.
    fn reason(text: &str) -> Option<BanReason> {
        Some(BanReason::parse(text).unwrap())
    }

    #[test]
    fn register_creates_an_account_that_is_not_banned() {
        assert!(!member(Utc::now()).is_banned());
    }

    #[test]
    fn admin_can_ban_a_member() {
        let now = Utc::now();
        let mut user = member(now);
        let later = now + Duration::minutes(5);

        user.ban(reason("spam"), UserId::generate(), Role::Admin, later)
            .unwrap();

        assert!(user.is_banned());
        let ban = user.ban.clone().unwrap();
        assert_eq!(ban.reason, reason("spam"));
        assert_eq!(ban.banned_at, later);
        assert_eq!(user.updated_at, later);
    }

    #[test]
    fn banning_again_replaces_the_reason_but_keeps_the_date() {
        let now = Utc::now();
        let mut user = member(now);
        user.ban(reason("spam"), UserId::generate(), Role::Admin, now)
            .unwrap();

        let later = now + Duration::minutes(5);
        user.ban(None, UserId::generate(), Role::Admin, later)
            .unwrap();

        let ban = user.ban.clone().unwrap();
        assert_eq!(ban.reason, None);
        assert_eq!(ban.banned_at, now);
    }

    #[test]
    fn member_cannot_ban() {
        let mut user = member(Utc::now());
        let result = user.ban(None, UserId::generate(), Role::Member, Utc::now());
        assert!(matches!(result, Err(DomainError::Forbidden(_))));
        assert!(!user.is_banned());
    }

    #[test]
    fn admin_cannot_ban_themselves() {
        let mut admin = member(Utc::now());
        admin.role = Role::Admin;
        let id = admin.id;

        let result = admin.ban(None, id, Role::Admin, Utc::now());

        assert!(matches!(result, Err(DomainError::Forbidden(_))));
        assert!(!admin.is_banned());
    }

    #[test]
    fn an_admin_cannot_be_banned() {
        let mut admin = member(Utc::now());
        admin.role = Role::Admin;

        let result = admin.ban(None, UserId::generate(), Role::Admin, Utc::now());

        assert!(matches!(result, Err(DomainError::Forbidden(_))));
        assert!(!admin.is_banned());
    }

    #[test]
    fn admin_can_unban() {
        let now = Utc::now();
        let mut user = member(now);
        user.ban(None, UserId::generate(), Role::Admin, now)
            .unwrap();
        let later = now + Duration::minutes(5);

        user.unban(Role::Admin, later).unwrap();

        assert!(!user.is_banned());
        assert_eq!(user.updated_at, later);
    }

    #[test]
    fn unbanning_an_account_that_is_not_banned_is_a_no_op() {
        let now = Utc::now();
        let mut user = member(now);

        user.unban(Role::Admin, now + Duration::minutes(5)).unwrap();

        assert!(!user.is_banned());
        assert_eq!(user.updated_at, now);
    }

    #[test]
    fn member_cannot_unban() {
        let now = Utc::now();
        let mut user = member(now);
        user.ban(None, UserId::generate(), Role::Admin, now)
            .unwrap();

        let result = user.unban(Role::Member, now);

        assert!(matches!(result, Err(DomainError::Forbidden(_))));
        assert!(user.is_banned());
    }

    #[test]
    fn a_banned_account_cannot_be_promoted() {
        let now = Utc::now();
        let mut user = member(now);
        user.ban(None, UserId::generate(), Role::Admin, now)
            .unwrap();

        let result = user.change_role(Role::Admin, UserId::generate(), Role::Admin, now);

        assert!(matches!(result, Err(DomainError::Forbidden(_))));
        assert_eq!(user.role, Role::Member);
    }
}
