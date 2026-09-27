//! Fixtures shared by the use case tests.

use std::sync::Arc;

use chrono::{DateTime, TimeZone, Utc};
use xetaravel_kernel::{FixedClock, Principal};

use crate::application::ports::MockHumanVerifier;
pub(crate) use crate::application::use_cases::principal_of;
use crate::domain::{Email, PasswordHash, Role, User, Username};

/// Returns the instant every test clock is frozen at.
pub fn now() -> DateTime<Utc> {
    Utc.with_ymd_and_hms(2026, 1, 1, 12, 0, 0).unwrap()
}

/// Returns a fixed clock frozen at [`now`].
pub fn clock() -> Arc<FixedClock> {
    Arc::new(FixedClock(now()))
}

/// Builds a registered user with the given username and role.
pub fn user(username: &str, role: Role) -> User {
    let mut user = User::register(
        Username::parse(username).unwrap(),
        Email::parse(&format!("{username}@example.com")).unwrap(),
        PasswordHash::new("hashed"),
        now(),
    );
    user.role = role;
    user
}

/// Builds an admin principal.
pub fn admin_principal() -> Principal {
    principal_of(&user("admin", Role::Admin))
}

/// Builds a member principal.
pub fn member_principal() -> Principal {
    principal_of(&user("member", Role::Member))
}

/// Returns a captcha verifier mock answering `valid` to every challenge.
pub fn human(valid: bool) -> Arc<MockHumanVerifier> {
    let mut verifier = MockHumanVerifier::new();
    verifier.expect_verify().returning(move |_, _| Ok(valid));
    Arc::new(verifier)
}
