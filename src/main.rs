use std::sync::Arc;

use anyhow::Result;
use persistence::Storage;
use persistence::StorageConfig;

pub mod api;
pub mod orders;
pub mod persistence;
pub mod pet;
pub mod user;

pub struct AppStateInner {
    pub db_pool: Arc<Storage>,
}

struct AppState {
    pub inner: Arc<AppStateInner>,
}

impl AppState {
    pub async fn shutdown(self) -> Result<()> {
        self.inner.db_pool.close().await;
        Ok(())
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    println!("starting..");
    println!("connecting...");

    let storage_config = StorageConfig {
        db_path: "sqlite::memory:".into(),
    };

    let storage = Arc::new(Storage::open_migrate(storage_config).await?);
    let state = AppState {
        inner: Arc::new(AppStateInner { db_pool: storage }),
    };

    // let test = std::panic::catch_unwind(|| (1..10).product::<i8>()).is_err();
    // println!("{}", test);

    println!("connected to SQLite");

    println!("shutting down...");
    let _ = state.shutdown().await;
    // storage.close();
    Ok(())
}
