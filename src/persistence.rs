use anyhow::Result;
use std::{pin::Pin, sync::Arc, time::Duration};
use tokio::sync::Mutex;
use tracing::trace;

use sqlx::{
    sqlite::{SqliteConnectOptions, SqlitePoolOptions},
    ConnectOptions, Connection, Pool, Sqlite, SqliteConnection,
};

pub trait WithDBConnection {
    type Conn;
    fn conn() -> Self::Conn;
}

pub struct StorageConfig {
    pub db_path: String,
}

pub struct Storage {
    pool: Pool<Sqlite>,
}

impl Storage {
    pub async fn open_migrate(config: StorageConfig) -> Result<Storage> {
        let pool = SqlitePoolOptions::new()
            .max_connections(5)
            .idle_timeout(None)
            .max_lifetime(None)
            .acquire_timeout(Duration::from_millis(300))
            .connect(&config.db_path)
            .await?;

        match sqlx::migrate!("./migrations").run(&pool).await {
            Ok(_) => {
                tracing::info!("Migrations passed successfully!");
                println!("Migrations passed successfully!")
            }
            Err(e) => tracing::error!("Failed to migrate. Errors is : {:}", e),
        };

        Ok(Storage { pool })
    }

    pub fn conn(&self) -> Pool<Sqlite> {
        self.pool.clone()
    }

    pub async fn close(&self) {
        self.pool.close().await
    }
}
#[cfg(test)]
mod tests {
    use crate::persistence::{Storage, StorageConfig};

    #[tokio::test]
    async fn migration_test() -> anyhow::Result<()> {
        let storage = Storage::open_migrate(StorageConfig {
            db_path: "sqlite::memory:".into(),
        })
        .await;

        assert!(storage.is_ok());

        Ok(())
    }
}
