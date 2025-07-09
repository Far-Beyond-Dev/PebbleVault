use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::Instant;
use uuid::Uuid;
use PebbleVault::config::load_config;
use PebbleVault::spacial_store::backend::PersistenceBackend;
use PebbleVault::spacial_store::postgres_backend::PostgresDatabase;
use PebbleVault::spacial_store::sqlite_backend::SqliteDatabase;
use PebbleVault::spacial_store::mysql_backend::MySqlDatabase;
use PebbleVault::VaultManager;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
struct CustomData {
    name: String,
    level: u32,
}

fn main() -> Result<(), String> {
    // Create a new VaultManager with custom data type
    // let backend = Box::new(SqliteDatabase::new("spatial_db.db").map_err(|e| e.to_string())?)
    //     as Box<dyn PersistenceBackend>;
    let config = load_config().map_err(|e| e.to_string())?;
    let backend: Box<dyn PersistenceBackend> = match config.database.backend.as_str() {
        "postgres" => {
            let pg = config
                .database
                .postgres
                .ok_or("Missing [database.postgres] config")?;
            let conn_str = format!(
                "host={} port={} user={} password={} dbname={}",
                pg.host, pg.port, pg.user, pg.password, pg.dbname
            );
            Box::new(PostgresDatabase::new(&conn_str).map_err(|e| e.to_string())?)
        }
        "sqlite" => {
            let sqlite = config
                .database
                .sqlite
                .ok_or("Missing [database.sqlite] config")?;
            Box::new(SqliteDatabase::new(&sqlite.path).map_err(|e| e.to_string())?)
        }
        "mysql" => {
            let mysql = config
                .database
                .mysql
                .ok_or("Missing [database.mysql] config")?;

            let url = format!(
                "mysql://{}:{}@{}:{}/{}",
                mysql.user, mysql.password, mysql.host, mysql.port, mysql.dbname
            );

            Box::new(MySqlDatabase::new(&url).map_err(|e| e.to_string())?)
        }
        other => return Err(format!("Unsupported backend: {}", other)),
    };

    let mut vault_manager: VaultManager<CustomData> =
        VaultManager::new(backend).map_err(|e| e.to_string())?;

    // Create a new region
    let region_id = vault_manager.create_or_load_region([0.0, 0.0, 0.0], 500.0)?;

    // Add an object to the region
    let object_uuid = Uuid::new_v4();
    let custom_data = CustomData {
        name: "Example".to_string(),
        level: 1,
    };
    vault_manager.add_object(
        region_id,
        object_uuid,
        "example_object",
        1.0,
        2.0,
        3.0,
        1.0,
        1.0,
        1.0,
        Arc::new(custom_data),
    )?;

    // Query objects in the region (example bounding box)
    let start = Instant::now();
    let objects = vault_manager.query_region(region_id, 0.0, 0.0, 0.0, 10.0, 10.0, 10.0)?;
    let duration = start.elapsed();
    for obj in objects {
        println!("Found object: {:?}", obj);
    }

    println!("Execution time: {:?}", duration);

    Ok(())
}
