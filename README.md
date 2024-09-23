# PebbleVault

> [!CAUTION]
> PebbleVault is still in early development and is not meant to be used in any production environments yet.

![logo-no-background](https://github.com/Stars-Beyond/PebbleVault/assets/34868944/927902b2-1579-4e3a-9c92-93a0f9e47e3e)

---
Welcome to PebbleVault, the spatial database that rocks your world! 🚀 Imagine a universe where pebbles are more than just tiny stones; they're the building blocks of your galactic data dreams. PebbleVault is a spatial database with a SQLite twist, all wrapped up in the cozy, memory-safe blanket of Rust. It's like having a pet rock collection, but for grown-ups with serious spatial data needs!

## Why PebbleVault? 🌟
- **Speed**: With in-memory storage and spatial indexing, your pebbles are accessible at light speed.
- **Safety**: Thanks to Rust, your data is as safe as pebbles in a vault. No more worrying about memory leaks or data corruption.
- **Flexibility**: Easily manage regions and objects in 3D space. It's like juggling pebbles, but with fewer dropped rocks and more data integrity.
- **Persistence**: Throw your pebbles to SQLite when you need more permanent storage. It's like creating your own little rock garden, but for data!
- **Simplicity**: Simple operations to add, query, and transfer objects make managing your spatial data as easy as skipping stones on a serene pond.

## Key Features 🎉
- **Spatial Indexing**: Keep your pebbles organized in a 3D space using RTree for ultra-fast access.
- **Region Management**: Create and manage multiple regions in your vast data universe.
- **SQLite Persistence**: Store your pebble collection for the long term, ensuring your data stays solid as a rock.
- **Rust Reliability**: Built with Rust, so your pebbles are safe and sound, protected from the elements (and by elements, we mean bugs).

## Core Components 🧱

### VaultManager (lib.rs)
The VaultManager is the heart of PebbleVault. It's like the wise old rockkeeper, managing all your pebbles and regions.

Key features:
- **Region Creation**: Spawn new regions or load existing ones from the persistent storage.
- **Object Management**: Add, query, and transfer objects (your precious pebbles) between regions.
- **In-Memory Storage**: Uses RTree for lightning-fast spatial indexing.
- **Persistence**: Periodically saves your pebble collection to SQLite for safekeeping.

### MySQLGeo (MySQLGeo.rs)
Despite its name, MySQLGeo actually uses SQLite (we know, it's confusing - we're working on renaming it!). It's like the bedrock of PebbleVault, providing a solid foundation for persistent storage.

Key features:
- **Point Storage**: Stores spatial points with associated data in SQLite.
- **Spatial Queries**: Retrieve points within a specified radius.
- **File-based Data Storage**: Handles larger data objects by storing them in separate files.

## How They Rock Together 🎸
VaultManager and MySQLGeo work in harmony, like a well-oiled rock tumbler:

1. VaultManager keeps your pebbles (objects) organized in-memory using RTree.
2. When it's time to save, VaultManager calls on MySQLGeo to store the data.
3. MySQLGeo takes each pebble and carefully places it in the SQLite database.
4. For bigger pebbles (large data objects), MySQLGeo creates special files to hold them.
5. When VaultManager needs to load data, it asks MySQLGeo to fetch the pebbles from SQLite.

It's like having a meticulous rock collector (VaultManager) working with a master stonemason (MySQLGeo) to keep your pebble collection pristine and organized!

## API Overview 🛠️

### VaultManager Operations

```rust
// Create a new VaultManager
let vault_manager = VaultManager::new("path/to/database.sqlite")?;

// Create or load a region
let region_id = vault_manager.create_or_load_region([0.0, 0.0, 0.0], 100.0)?;

// Add an object to a region
vault_manager.add_object(region_id, object_uuid, 10.0, 20.0, 30.0, "Shiny pebble data")?;

// Query objects in a region
let objects = vault_manager.query_region(region_id, -50.0, -50.0, -50.0, 50.0, 50.0, 50.0)?;

// Transfer an object between regions
vault_manager.transfer_player(player_uuid, from_region_id, to_region_id)?;

// Save all data to persistent storage
vault_manager.persist_to_disk()?;
```

### MySQLGeo Operations

```rust
// Create a new Database connection
let db = MySQLGeo::Database::new("path/to/database.sqlite")?;

// Add a point to the database
let point = MySQLGeo::Point::new(Some(uuid), x, y, z, serde_json::Value::String("Pebble data".to_string()));
db.add_point(&point)?;

// Query points within a radius
let points = db.get_points_within_radius(0.0, 0.0, 0.0, 100.0)?;
```

## Example Usage 🚀

```rust
use pebblevault::{VaultManager, MySQLGeo};
use uuid::Uuid;

// Create a new VaultManager
let mut vault_manager = VaultManager::new("my_pebble_collection.sqlite")?;

// Create a new region
let region_id = vault_manager.create_or_load_region([0.0, 0.0, 0.0], 100.0)?;

// Add some pebbles to our collection
let pebble1_uuid = Uuid::new_v4();
vault_manager.add_object(region_id, pebble1_uuid, 10.0, 20.0, 30.0, "Smooth river pebble")?;

let pebble2_uuid = Uuid::new_v4();
vault_manager.add_object(region_id, pebble2_uuid, -15.0, 25.0, -5.0, "Sparkly quartz pebble")?;

// Find pebbles in a specific area
let found_pebbles = vault_manager.query_region(region_id, -20.0, 0.0, -10.0, 20.0, 30.0, 40.0)?;
println!("Found {} pebbles in the area!", found_pebbles.len());

// Save our precious pebble collection
vault_manager.persist_to_disk()?;

println!("Our pebble collection is safe and sound!");
```

## Load Testing 🏋️‍♂️
PebbleVault comes with a built-in load testing module to ensure your pebbles can handle the pressure! Check out `load_test.rs` to see how we put our vault through its paces. It's like a rock tumbler for your database!

## Contribute 🤝
Do you have ideas to make PebbleVault even better? Want to add more fun to our pebble party? Join us in making PebbleVault the best place for all your pebble-keeping needs! Check out our contributing guide and start throwing your ideas our way.

## License 📜
PebbleVault is licensed under the Apache 2.0 License. Rock on! 🤘