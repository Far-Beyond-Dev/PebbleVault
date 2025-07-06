use PebbleVault::{VaultManager};
use uuid::Uuid;
use serde::{Serialize, Deserialize};
use std::sync::Arc;
use std::time::Instant;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
struct CustomData {
    name: String,
    level: u32,
}

fn main() -> Result<(), String> {
    
    // Create a new VaultManager with custom data type
    let mut vault_manager: VaultManager<CustomData> = VaultManager::new("spatial_db.db")?;
    
    // Create a new region
    let region_id = vault_manager.create_or_load_region([0.0, 0.0, 0.0], 500.0)?;
    
    // Add an object to the region
    let object_uuid = Uuid::new_v4();
    let custom_data = CustomData { name: "Example".to_string(), level: 1 };
    vault_manager.add_object(region_id, object_uuid, "example_object", 1.0, 2.0, 3.0, Arc::new(custom_data))?;
    
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