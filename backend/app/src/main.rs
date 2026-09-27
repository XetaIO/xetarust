//! Binary of Xetaravel.
//!
//! ```text
//! xetaravel  # start the HTTP server
//! ```

use std::net::SocketAddr;
use std::process::ExitCode;

use axum::http::{HeaderValue, Method, header};
use migration::{Migrator, MigratorTrait};
use sea_orm::DatabaseConnection;
use tokio::net::TcpListener;
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;
use tracing_subscriber::EnvFilter;
use xetaravel_app::{AppState, Config, router};
use xetaravel_kernel::persistence;

/// Parses the command line and runs the requested command.
#[tokio::main]
async fn main() -> ExitCode {
    dotenvy::dotenv().ok();
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| {
                "xetaravel_app=info,xetaravel_kernel=info,tower_http=info".into()
            }),
        )
        .init();

    let args: Vec<String> = std::env::args().skip(1).collect();
    let result = match args
        .iter()
        .map(String::as_str)
        .collect::<Vec<_>>()
        .as_slice()
    {
        [] | ["serve"] => serve().await,
        _ => Err("usage: xetaravel [serve]".into()),
    };

    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            tracing::error!("{error}");
            ExitCode::FAILURE
        }
    }
}

/// Error type of the binary commands.
type CliResult = Result<(), Box<dyn std::error::Error>>;

/// Loads the configuration, connects and migrates the database, then builds
/// every bounded context.
async fn bootstrap() -> Result<(Config, AppState), Box<dyn std::error::Error>> {
    let config = Config::from_env()?;
    let db: DatabaseConnection = persistence::connect(&config.database_url).await?;
    Migrator::up(&db, None).await?;
    let state = AppState::build(db, &config);
    Ok((config, state))
}

/// Starts the HTTP server until Ctrl+C.
async fn serve() -> CliResult {
    let (config, state) = bootstrap().await?;

    let cors = CorsLayer::new()
        .allow_origin(config.cors_origin.parse::<HeaderValue>()?)
        .allow_methods([
            Method::GET,
            Method::POST,
            Method::PUT,
            Method::PATCH,
            Method::DELETE,
        ])
        .allow_headers([header::AUTHORIZATION, header::CONTENT_TYPE]);

    let app = router(state).layer(cors).layer(TraceLayer::new_for_http());

    let listener = TcpListener::bind(&config.app_addr).await?;
    tracing::info!("listening on http://{}", config.app_addr);
    // The peer address is the rate limit fallback when `X-Forwarded-For` is absent.
    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .with_graceful_shutdown(async {
        tokio::signal::ctrl_c().await.ok();
    })
    .await?;
    Ok(())
}
