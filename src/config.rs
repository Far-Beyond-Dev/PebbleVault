use serde::Deserialize;
use config::Config as ConfigLoader;
use std::error::Error;

#[derive(Debug, Deserialize)]
pub struct AppConfig {
    pub database: DatabaseConfig,
}

#[derive(Debug, Deserialize)]
pub struct DatabaseConfig {
    pub backend: String,
    pub postgres: Option<PostgresConfig>,
    pub sqlite: Option<SqliteConfig>,
    pub mysql: Option<MySqlConfig>,
}

#[derive(Debug, Deserialize)]
pub struct PostgresConfig {
    pub host: String,
    pub port: u16,
    pub user: String,
    pub password: String,
    pub dbname: String,
}

#[derive(Debug, Deserialize)]
pub struct MySqlConfig {
    pub host: String,
    pub port: u16,
    pub user: String,
    pub password: String,
    pub dbname: String,
}

#[derive(Debug, Deserialize)]
pub struct SqliteConfig {
    pub path: String,
}

pub fn load_config() -> Result<AppConfig, Box<dyn Error>> {
    let settings = ConfigLoader::builder()
        .add_source(config::File::with_name("Config").required(true))
        .build()?
        .try_deserialize::<AppConfig>()?;
    Ok(settings)
}
