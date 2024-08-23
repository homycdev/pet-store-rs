use anyhow::Result;
use figment::providers::{Format, Toml};
use serde::Deserialize;

#[derive(Deserialize)]
struct Db {
    url: String,
    user: String,
    pwd: String,
}

#[derive(Deserialize)]
pub struct AppConfig {
    db: Db,
}
impl AppConfig {
    pub fn load_config() -> Result<AppConfig> {
        let raw_config = figment::Figment::from(Toml::file("./configs/qa/config.toml"));
        let config: AppConfig = raw_config.extract()?;

        Ok(config)
    }

    pub fn db(&self) -> String {
        format!(
            "postgres://{}:{}@{}",
            self.db.user, self.db.pwd, self.db.url
        )
    }
}
