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
- **Persistence**: Seamless integration with a persistent storage backend for durable data storage.
- **Simplicity**: Intuitive operations to add, query, and manage objects in your spatial universe.

## Key Features 🎉
- **Spatial Indexing**: Utilizes RTree for efficient 3D spatial querying.
- **Region Management**: Create and manage multiple spatial regions.
- **Persistent Storage**: Store your spatial data for long-term preservation.
- **Rust Reliability**: Built with Rust for maximum performance and safety.

## Core Components 🧱

### VaultManager (vault_manager.rs)
The VaultManager is the central component of PebbleVault, orchestrating all spatial operations.

Key features:
- **Region Creation**: Create new regions or load existing ones from persistent storage.
- **Object Management**: Add, query, and manage objects within regions.
- **In-Memory Storage**: Uses RTree for high-speed spatial indexing.
- **Persistence**: Periodically saves spatial data to persistent storage.

## API Overview 🛠️

### VaultManager Operations

```rust
// Create a new VaultManager
let vault_manager = VaultManager::new("path/to/database.db")?;

// Create or load a region
let region_id = vault_manager.create_or_load_region([0.0, 0.0, 0.0], 100.0)?;

// Add an object to a region
let object_uuid = Uuid::new_v4();
vault_manager.add_object(region_id, object_uuid, 10.0, 20.0, 30.0, "Object data")?;

// Query objects in a region
let objects = vault_manager.query_region(region_id, -50.0, -50.0, -50.0, 50.0, 50.0, 50.0)?;

// Transfer a player between regions
let player_uuid = Uuid::new_v4();
let from_region_id = Uuid::new_v4();
let to_region_id = Uuid::new_v4();
vault_manager.transfer_player(player_uuid, from_region_id, to_region_id)?;

// Save all data to persistent storage
vault_manager.persist_to_disk()?;
```

## Example Usage 🚀

```rust
use pebblevault::{VaultManager, SpatialObject};
use uuid::Uuid;

fn main() -> Result<(), String> {
    // Create a new VaultManager
    let mut vault_manager = VaultManager::new("spatial_db.pv")?;

    // Create a new region
    let region_id = vault_manager.create_or_load_region([0.0, 0.0, 0.0], 500.0)?;

    // Add some objects to our collection
    let object1_uuid = Uuid::new_v4();
    vault_manager.add_object(region_id, object1_uuid, 10.0, 20.0, 30.0, "First object")?;

    let object2_uuid = Uuid::new_v4();
    vault_manager.add_object(region_id, object2_uuid, -15.0, 25.0, -5.0, "Second object")?;

    // Find objects in a specific area
    let found_objects = vault_manager.query_region(region_id, -20.0, 0.0, -10.0, 20.0, 30.0, 40.0)?;
    println!("Found {} objects in the area!", found_objects.len());

    // Transfer an object to a new region
    let new_region_id = vault_manager.create_or_load_region([100.0, 100.0, 100.0], 500.0)?;
    vault_manager.transfer_player(object1_uuid, region_id, new_region_id)?;
    println!("Transferred object to new region!");

    // Save our spatial data collection
    vault_manager.persist_to_disk()?;

    println!("Our spatial data is safely stored!");
    Ok(())
}
```

## Load Testing 🏋️‍♂️
PebbleVault includes a built-in load testing module to ensure optimal performance under various conditions. The `run_load_test` function in `load_test.rs` allows you to stress-test the system with a large number of objects across multiple regions.

Here's how to run the load test:

```rust
use pebblevault::load_test::run_load_test;

fn main() -> Result<(), String> {
    let db_path = "load_test.db";
    let num_objects = 100_000;
    let num_regions = 10;

    run_load_test(db_path, num_objects, num_regions)?;
    println!("Load test completed successfully!");
    Ok(())
}
```

This load test will:
1. Create 10 regions
2. Add 100,000 randomly positioned objects across these regions
3. Persist all data to disk
4. Create a new VaultManager instance to verify data persistence
5. Retrieve all objects to ensure they were correctly stored and can be queried

The load test helps verify the system's performance, persistence capabilities, and ability to handle large datasets. Feel free to adjust the `num_objects` and `num_regions` parameters to suit your testing needs!

## Contribute 🤝
We welcome contributions to make PebbleVault even better! If you have ideas for improvements or new features, please check out our contributing guide and join our community of spatial data enthusiasts.

## License 📜
PebbleVault is licensed under the Apache 2.0 License. Explore the spatial universe with confidence! 🌠