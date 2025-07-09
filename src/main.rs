use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::Instant;
use uuid::Uuid;
use PebbleVault::spacial_store::backend::PersistenceBackend;
// use PebbleVault::spacial_store::sqlite_backend::SqliteDatabase;
use PebbleVault::spacial_store::postgres_backend::PostgresDatabase;
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

    let backend = Box::new(
        PostgresDatabase::new("host=localhost port=5433 user=postgres password=postgres dbname=spatial")
            .map_err(|e| e.to_string())?,
    ) as Box<dyn PersistenceBackend>;
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
