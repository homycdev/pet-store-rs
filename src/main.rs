use std::sync::Arc;

use anyhow::Result;
use config::AppConfig;
use persistence::ArcPgPool;
use persistence::StorageConfig;
use persistence::StorageI;

pub mod config;
pub mod orders;
pub mod persistence;
pub mod pet;
pub mod user;
use persistence::Storage;

pub struct AppStateInner {
    pub db: ArcPgPool,
}

struct AppState {
    pub inner: Arc<AppStateInner>,
}

impl AppState {
    pub async fn shutdown(self) -> Result<()> {
        self.inner.db.close().await;
        Ok(())
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    println!("starting..");
    println!("connecting...");

    let app_config = AppConfig::load_config()?;
    let storage_config = StorageConfig {
        db_path: app_config.db(),
    };

    let storage = StorageI.conn(storage_config).await?;
    StorageI.migrate(storage.pool.clone()).await?;

    let state = AppState {
        inner: Arc::new(AppStateInner { db: storage.pool }),
    };

    println!("connected to DB");

    println!("shutting down...");
    let _ = state.shutdown().await;
    Ok(())
}
