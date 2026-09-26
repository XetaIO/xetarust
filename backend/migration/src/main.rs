use sea_orm_migration::prelude::*;

/// Entry point of the migration CLI (`up`, `down`, `fresh`, `status`...).
#[tokio::main]
async fn main() {
    cli::run_cli(migration::Migrator).await;
}
