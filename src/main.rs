use std::ops::Deref;
use std::sync::Arc;

use anyhow::Context;
use anyhow::Result;
use axum::extract::State;
use axum::http::StatusCode;
use axum::routing::get;
use axum::Json;
use axum::Router;
use config::AppConfig;
use orders::Order;
use persistence::ArcPgPool;
use persistence::StorageConfig;
use persistence::StorageIml;

pub mod config;
pub mod error;
pub mod orders;
pub mod persistence;
pub mod pet;
pub mod user;
use persistence::Storage;
use tokio::net::TcpListener;
use tokio::signal;
use tracing::info;

#[derive(Debug)]
pub struct AppStateInner {
    pub db: ArcPgPool,
    pub version: String,
}

#[derive(Debug, Clone)]
struct AppState {
    pub inner: Arc<AppStateInner>,
}

impl AppState {
    pub async fn shutdown(self) -> Result<()> {
        self.inner.db.close().await;
        Ok(())
    }
}

impl Deref for AppState {
    type Target = Arc<AppStateInner>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .init();

    info!("Loading config");
    let app_config = AppConfig::load_config()?;
    let storage_config = StorageConfig {
        db_path: app_config.db(),
    };
    info!("Connecting to DB");
    let pool = StorageIml.conn(storage_config).await?;
    StorageIml.migrate(pool.clone()).await?;

    let state = AppState {
        inner: Arc::new(AppStateInner {
            db: pool,
            version: "0.0.1".to_string(),
        }),
    };

    info!("Connected to DB");

    info!("Starting server");

    let version_router = Router::new().route(
        "/version",
        get(|state: State<AppState>| async move {
            (StatusCode::OK, Json(state.0.clone().version.clone()))
        }),
    );
    let listener = TcpListener::bind("127.0.0.1:8080").await?;
    let routes = Router::new()
        .nest("/orders", orders::api::create_router())
        .nest("/", version_router)
        .with_state(state.clone());

    axum::serve(listener, routes)
        .with_graceful_shutdown(shutdown_signal())
        .await
        .context("Fucked up here")?;

    info!("Shutting down...");
    let _ = state.shutdown().await;
    info!("Shutdown complete");
    Ok(())
}

async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }
}
