//! Schema migrations owned by the Identity context (`users` table,
//! `user_role` enum and ban columns). The `migration` crate aggregates the migrations of
//! every context in dependency order.

use sea_orm_migration::MigrationTrait;

mod m20260927_identity_000001_create_users;
mod m20260928_identity_000002_add_user_ban;

/// Returns the migrations of the Identity context, oldest first.
pub fn migrations() -> Vec<Box<dyn MigrationTrait>> {
    vec![
        Box::new(m20260927_identity_000001_create_users::Migration),
        Box::new(m20260928_identity_000002_add_user_ban::Migration),
    ]
}
