# PebbleVault

> [!IMPORTANT]
> PebbleVault is still in early development and is not meant to be used in any production environments yet.

![logo-no-background](https://github.com/Stars-Beyond/PebbleVault/assets/34868944/927902b2-1579-4e3a-9c92-93a0f9e47e3e)

---
Welcome to PebbleVault, the spatial database that rocks your world! 🚀 PebbleVault is a high-performance spatial database written in Rust, designed for managing 3D spatial data with efficiency and safety in mind.

## Why PebbleVault? 🌟
- **Speed**: In-memory storage with RTree spatial indexing for lightning-fast queries.
- **Safety**: Leveraging Rust's memory safety guarantees for robust and reliable operations.
- **Flexibility**: Easily manage regions and objects in 3D space with a simple yet powerful API.
- **Persistence**: Seamless integration with a SQLite-based persistent storage backend for durable data storage.
- **Simplicity**: Intuitive operations to add, query, and manage objects in your spatial universe.
- **Custom Data**: Support for arbitrary custom data associated with spatial objects.

## Key Features 🎉
- **Spatial Indexing**: Utilizes RTree for efficient 3D spatial querying.
- **Region Management**: Create and manage multiple spatial regions.
- **Persistent Storage**: Store your spatial data for long-term preservation using SQLite.
- **Rust Reliability**: Built with Rust for maximum performance and safety.
- **Object Types**: Support for different object types with custom data.
- **Generic Implementation**: Flexible custom data support through generics.

## Core Components 🧱

### VaultManager (vault_manager.rs)
The VaultManager is the central component of PebbleVault, orchestrating all spatial operations.

Key features:
- **Region Creation**: Create new regions or load existing ones from persistent storage.
- **Object Management**: Add, query, and manage objects within regions.
- **In-Memory Storage**: Uses RTree for high-speed spatial indexing.
- **Persistence**: Periodically saves spatial data to persistent storage.
- **Player Transfer**: Move players between regions seamlessly.
- **Generic Custom Data**: Support for arbitrary custom data types.

### SpatialObject (structs.rs)
Represents individual objects within the spatial database.

Key attributes:
- **UUID**: Unique identifier for each object.
- **Object Type**: Categorizes objects (e.g., player, building, resource).
- **Point**: 3D coordinates of the object.
- **Custom Data**: Generic type for associating arbitrary data with objects.

### VaultRegion (structs.rs)
Represents a spatial region in the game world.

Key attributes:
- **ID**: Unique identifier for the region.
- **Center**: 3D coordinates of the region's center.
- **Radius**: Defines the size of the region.
- **RTree**: Spatial index for efficient object querying within the region.

### Database (MySQLGeo.rs)
Manages persistent storage of spatial data using SQLite.

Key features:
- **Point Storage**: Efficiently store and retrieve spatial points.
- **Region Management**: Create and manage spatial regions.
- **Custom Data Handling**: Store arbitrary custom data associated with points.
- **Spatial Queries**: Perform radius-based queries on stored points.

## API Overview 🛠️

### VaultManager Operations

```rust
// Create a new VaultManager with custom data type
let mut vault_manager: VaultManager<CustomData> = VaultManager::new("path/to/database.db")?;

// Create or load a region
let region_id = vault_manager.create_or_load_region([0.0, 0.0, 0.0], 100.0)?;

// Add an object to a region with custom data
let object_uuid = Uuid::new_v4();
let custom_data = CustomData { /* ... */ };
vault_manager.add_object(region_id, object_uuid, "player", 10.0, 20.0, 30.0, Arc::new(custom_data))?;

// Query objects in a region
let objects = vault_manager.query_region(region_id, -50.0, -50.0, -50.0, 50.0, 50.0, 50.0)?;

// Transfer a player between regions
let player_uuid = Uuid::new_v4();
let from_region_id = Uuid::new_v4();
let to_region_id = Uuid::new_v4();
vault_manager.transfer_player(player_uuid, from_region_id, to_region_id)?;

// Remove an object
vault_manager.remove_object(object_uuid)?;

// Update an object
let updated_object = SpatialObject { /* ... */ };
vault_manager.update_object(&updated_object)?;

// Save all data to persistent storage
vault_manager.persist_to_disk()?;
```

## Example Usage 🚀

```rust
use pebblevault::{VaultManager, SpatialObject, CustomData};
use uuid::Uuid;
use std::sync::Arc;

fn main() -> Result<(), String> {
    // Create a new VaultManager with custom data type
    let mut vault_manager: VaultManager<CustomData> = VaultManager::new("spatial_db.db")?;

    // Create a new region
    let region_id = vault_manager.create_or_load_region([0.0, 0.0, 0.0], 500.0)?;

    // Add some objects to our collection
    let object1_uuid = Uuid::new_v4();
    let custom_data1 = CustomData { name: "Player One".to_string(), level: 1 };
    vault_manager.add_object(region_id, object1_uuid, "player", 10.0, 20.0, 30.0, Arc::new(custom_data1))?;

    let object2_uuid = Uuid::new_v4();
    let custom_data2 = CustomData { name: "Town Hall".to_string(), level: 5 };
    vault_manager.add_object(region_id, object2_uuid, "building", -15.0, 25.0, -5.0, Arc::new(custom_data2))?;

    // Find objects in a specific area
    let found_objects = vault_manager.query_region(region_id, -20.0, 0.0, -10.0, 20.0, 30.0, 40.0)?;
    println!("Found {} objects in the area!", found_objects.len());

    // Transfer a player to a new region
    let new_region_id = vault_manager.create_or_load_region([100.0, 100.0, 100.0], 500.0)?;
    vault_manager.transfer_player(object1_uuid, region_id, new_region_id)?;
    println!("Transferred player to new region!");

    // Remove an object
    vault_manager.remove_object(object2_uuid)?;
    println!("Removed building from the region!");

    // Save our spatial data collection
    vault_manager.persist_to_disk()?;

    println!("Our spatial data is safely stored!");
    Ok(())
}
```

## Load Testing 🏋️‍♂️
PebbleVault includes comprehensive load testing modules to ensure optimal performance under various conditions. The `load_test.rs` file provides two main load testing functions:

1. `run_load_test`: A detailed load test with predefined custom data.
2. `run_arbitrary_data_load_test`: A load test using arbitrary struct data to demonstrate flexibility.

Here's how to run the load tests:

```rust
use pebblevault::load_test::{run_load_test, run_arbitrary_data_load_test};

fn main() -> Result<(), String> {
    // Run the standard load test
    let db_path = "load_test.db";
    let num_objects = 100_000;
    let num_regions = 10;
    let num_operations = 5;

    let mut vault_manager = VaultManager::new(db_path)?;
    run_load_test(&mut vault_manager, num_objects, num_regions, num_operations)?;
    println!("Standard load test completed successfully!");

    // Run the arbitrary data load test
    run_arbitrary_data_load_test(50_000, 5)?;
    println!("Arbitrary data load test completed successfully!");

    Ok(())
}
```

These load tests will:
1. Create or use existing regions
2. Add a specified number of randomly positioned objects across these regions
3. Persist all data to disk
4. Perform additional operations like querying, updating, and verifying data
5. Test the system's ability to handle arbitrary custom data structures

The load tests help verify the system's performance, persistence capabilities, and ability to handle large datasets with various data types. Feel free to adjust the parameters to suit your testing needs!

## Contribute 🤝
We welcome contributions to make PebbleVault even better! If you have ideas for improvements or new features, please check out our contributing guide and join our community of spatial data enthusiasts.

## License 📜
PebbleVault is licensed under the Apache 2.0 License. Explore the spatial universe with confidence! 🌠