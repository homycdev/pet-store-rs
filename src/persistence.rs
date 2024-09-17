use anyhow::Result;
use std::time::Duration;

use sqlx::{postgres::PgPoolOptions, Pool, Postgres};

pub trait Storage {
    type DB;
    async fn conn(self, config: StorageConfig) -> Result<Self::DB>;
    async fn migrate(self, pool: ArcPgPool) -> Result<()>;
    async fn close(self, pool: ArcPgPool);
}

pub struct StorageConfig {
    pub db_path: String,
}

pub type ArcPgPool = Pool<Postgres>;
pub struct StorageIml;
impl Storage for StorageIml {
    type DB = ArcPgPool;

    async fn conn(self, config: StorageConfig) -> Result<ArcPgPool> {
        let pool = PgPoolOptions::new()
            .max_connections(5)
            .idle_timeout(None)
            .max_lifetime(None)
            .acquire_timeout(Duration::from_millis(300))
            .connect(&config.db_path)
            .await?;

        Ok(pool)
    }

    async fn close(self, pool: ArcPgPool) {
        pool.close().await
    }

    async fn migrate(self, pool: ArcPgPool) -> Result<()> {
        sqlx::migrate!("./migrations").run(&pool.clone()).await?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        config::AppConfig,
        persistence::{Storage, StorageConfig, StorageIml},
    };

    #[tokio::test]
    async fn migration_test() -> anyhow::Result<()> {
        let config = AppConfig::load_config()?;

        let storage_conf = StorageConfig {
            db_path: config.db(),
        };
        let pool = StorageIml.conn(storage_conf).await?;
        let storage = StorageIml.migrate(pool).await;

        assert!(storage.is_ok());

        Ok(())
    }
}
