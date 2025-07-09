//! # VaultManager: Spatial Data Management System
//!
//! This module provides the `VaultManager` struct, which is responsible for managing spatial regions and objects
//! in a game or simulation environment. It offers functionality for creating and managing regions, adding spatial
//! objects to these regions, querying objects within regions, and persisting data to a database.
//!
//! ## Key Features
//!
//! - **Spatial Partitioning**: Divides the world into cubic regions for efficient spatial queries.
//! - **Custom Data Support**: Allows associating arbitrary data with spatial objects using `Arc<T>`.
//! - **Persistent Storage**: Saves and loads spatial data to/from a database.
//! - **Efficient Querying**: Uses R-trees for fast spatial lookups within regions.
//! - **Object Transfer**: Supports moving objects between regions.
//!
//! ## Usage Example
//!
//! ```rust
//! use your_crate::{VaultManager, CustomData};
//! use uuid::Uuid;
//! use std::sync::Arc;
//!
//! // Initialize VaultManager
//! let mut vault_manager: VaultManager<CustomData> = VaultManager::new("path/to/database.db").unwrap();
//!
//! // Create a new region
//! let region_id = vault_manager.create_or_load_region([0.0, 0.0, 0.0], 100.0).unwrap();
//!
//! // Add an object to the region
//! let object_id = Uuid::new_v4();
//! let custom_data = Arc::new(CustomData { /* ... */ });
//! vault_manager.add_object(region_id, object_id, "player", 1.0, 2.0, 3.0, custom_data).unwrap();
//!
//! // Query objects in a region
//! let objects = vault_manager.query_region(region_id, 0.0, 0.0, 0.0, 10.0, 10.0, 10.0).unwrap();
//!
//! // Get a specific object
//! if let Some(object) = vault_manager.get_object(object_id).unwrap() {
//!     println!("Found object: {:?}", object);
//! }
//!
//! // Persist changes to disk
//! vault_manager.persist_to_disk().unwrap();
//! ```
//!
//! ## Performance Considerations
//!
//! - The `VaultManager` uses R-trees for spatial indexing, providing O(log n) complexity for insertions and queries.
//! - Regions are stored in-memory for fast access, with periodic persistence to disk.
//! - Consider the trade-off between region size and number: larger regions mean fewer region transfers but potentially slower queries.
//! - Custom data is stored as `Arc<T>`, allowing for efficient sharing of data between objects and reducing memory usage.

use crate::structs::{VaultRegion, SpatialObject};
use uuid::Uuid;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use rstar::{RTree, AABB};
use indicatif::{ProgressBar, ProgressStyle};
use serde::{Serialize, Deserialize};
use crate::spacial_store::types::Point;
use crate::spacial_store::backend::PersistenceBackend;

/// Manages spatial regions and objects within a persistent database.
///
/// `VaultManager` is the core struct of the spatial management system. It maintains a collection of regions,
/// each containing spatial objects. The manager handles the creation of regions, addition and removal of objects,
/// spatial queries, and persistence of data.
///
/// The generic parameter `T` allows for custom data to be associated with each spatial object, providing
/// flexibility for various use cases. The custom data is stored as `Arc<T>` to allow efficient sharing and
/// reduce memory usage.
///
/// # Type Parameters
///
/// * `T`: The type of custom data associated with spatial objects. Must implement `Clone`, `Serialize`,
///        `Deserialize`, and `PartialEq`.
pub struct VaultManager<T: Clone + Serialize + for<'de> Deserialize<'de> + PartialEq + Sized> {
    /// HashMap storing regions, keyed by their UUID
    pub regions: HashMap<Uuid, Arc<Mutex<VaultRegion<T>>>>,

    /// Persistent backend implementing spatial storage (e.g., SQLite, Postgres, binary)
    pub persistent_db: Box<dyn PersistenceBackend>,

    /// HashMap storing object types
    pub object_types: HashMap<String, String>,
}

impl<T: Clone + Serialize + for<'de> Deserialize<'de> + PartialEq + Sized> VaultManager<T> {
    /// Creates a new instance of `VaultManager`.
    ///
    /// This function initializes a new VaultManager, sets up the persistent database,
    /// and loads existing regions from the database. It's the entry point for using the spatial management system.
    ///
    /// # Arguments
    ///
    /// * `db_path` - A string slice that holds the path to the database file.
    ///
    /// # Returns
    ///
    /// * `Result<Self, String>` - A new `VaultManager` instance if successful, or an error message if not.
    ///
    /// # Examples
    ///
    /// ```
    /// use your_crate::{VaultManager, CustomData};
    ///
    /// let vault_manager: VaultManager<CustomData> = VaultManager::new("path/to/database.db").expect("Failed to create VaultManager");
    /// ```
    ///
    /// # Errors
    ///
    /// This function will return an error if:
    /// - The database connection cannot be established
    /// - The necessary tables cannot be created in the database
    /// - Existing regions cannot be loaded from the database
    pub fn new(persistent_db: Box<dyn PersistenceBackend>) -> Result<Self, String> {
        persistent_db
            .create_table()
            .map_err(|e| format!("Failed to create table: {}", e))?;

        let mut vault_manager = VaultManager {
            regions: HashMap::new(),
            persistent_db,
            object_types: HashMap::new(),
        };

        vault_manager.object_types.insert("player".into(), "player".into());
        vault_manager.object_types.insert("building".into(), "building".into());
        vault_manager.object_types.insert("resource".into(), "resource".into());

        vault_manager.load_regions_from_db()?;

        Ok(vault_manager)
    }

    /// Loads existing regions and their objects from the persistent database.
    ///
    /// This function is called during VaultManager initialization to populate
    /// the in-memory structures with data from the persistent storage. It's crucial for
    /// maintaining consistency between sessions and after application restarts.
    ///
    /// # Returns
    ///
    /// * `Result<(), String>` - Ok if successful, or an error message if not.
    ///
    /// # Notes
    ///
    /// This method is private and is automatically called by `new()`. It shouldn't be called directly by users.
    /// Each region is loaded as a cubic area defined by its center point and size.
    fn load_regions_from_db(&mut self) -> Result<(), String> {
        let regions = self.persistent_db.get_all_regions()
            .map_err(|e| format!("Failed to load regions from database: {}", e))?;

        println!("Loaded {} regions from the database", regions.len());

        for region in regions {
            println!("Loading region: ID: {}, Center: {:?}, Size: {}", region.id, region.center, region.size);
            let vault_region = VaultRegion {
                id: region.id,
                center: region.center,
                size: region.size,
                rtree: RTree::new(),
            };

            self.regions.insert(region.id, Arc::new(Mutex::new(vault_region)));

            let points = self.persistent_db.get_points_in_region(region.id)
                .map_err(|e| format!("Failed to load points for region {}: {}", region.id, e))?;

            println!("Loaded {} points for region {}", points.len(), region.id);

            if let Some(region_arc) = self.regions.get(&region.id) {
                let mut region = region_arc.lock().unwrap();
                for point in points {
                    let custom_data: T = serde_json::from_value(point.custom_data)
                        .map_err(|e| format!("Failed to deserialize custom data: {}", e))?;
                    let spatial_object = SpatialObject {
                        uuid: point.id.unwrap(),
                        object_type: point.object_type,
                        point: [point.x, point.y, point.z],
                        size: [point.size_x, point.size_y, point.size_z],
                        custom_data: Arc::new(custom_data),
                    };
                    region.rtree.insert(spatial_object);
                }
            }
        }

        Ok(())
    }

    /// Creates a new region or loads an existing one from the persistent database.
    ///
    /// This function is used to define spatial partitions in your world. If a region with the given
    /// center and size already exists, it returns the existing region's ID. Otherwise, it creates a new region.
    ///
    /// # Arguments
    ///
    /// * `center` - An array of 3 f64 values representing the x, y, z coordinates of the region's center.
    /// * `size` - The size (length of each side) of the cubic region.
    ///
    /// # Returns
    ///
    /// * `Result<Uuid, String>` - The UUID of the created or loaded region if successful, or an error message if not.
    ///
    /// # Examples
    ///
    /// ```
    /// # use your_crate::{VaultManager, CustomData};
    /// # let mut vault_manager: VaultManager<CustomData> = VaultManager::new("path/to/database.db").unwrap();
    /// let center = [0.0, 0.0, 0.0];
    /// let size = 100.0;  // Creates a 100x100x100 cubic region
    /// let region_id = vault_manager.create_or_load_region(center, size).expect("Failed to create region");
    /// ```
    ///
    /// # Notes
    ///
    /// - Regions are cubic, defined by a center point and a size (length of each side).
    /// - Overlapping regions are allowed, but may impact performance for objects in the overlapped areas.
    pub fn create_or_load_region(&mut self, center: [f64; 3], size: f64) -> Result<Uuid, String> {
        // Check if a region with the same center and size already exists
        if let Some(existing_region) = self.regions.values().find(|r| {
            let r = r.lock().unwrap();
            r.center == center && r.size == size
        }) {
            return Ok(existing_region.lock().unwrap().id);
        }

        // Generate a new UUID for the region
        let region_id = Uuid::new_v4();
        // Create a new RTree for the region
        let rtree = RTree::new();

        // Create a new VaultRegion
        let region = VaultRegion {
            id: region_id,
            center,
            size,
            rtree,
        };

        // Insert the new region into the regions HashMap
        self.regions.insert(region_id, Arc::new(Mutex::new(region)));

        // Persist the region to the database
        self.persistent_db.create_region(region_id, center, size)
            .map_err(|e| format!("Failed to persist region to database: {}", e))?;

        Ok(region_id)
    }

    /// Adds an object to a specific region.
    ///
    /// This function creates a new SpatialObject and adds it to both the in-memory RTree
    /// and the persistent database. It's used to populate your world with entities.
    ///
    /// # Arguments
    ///
    /// * `region_id` - The UUID of the region to add the object to.
    /// * `uuid` - The UUID of the object being added.
    /// * `object_type` - The type of the object being added (e.g., "player", "building", "resource").
    /// * `x` - The x-coordinate of the object.
    /// * `y` - The y-coordinate of the object.
    /// * `z` - The z-coordinate of the object.
    /// * `size_x` - The width of the object along the X-axis.
    /// * `size_y` - The height of the object along the Y-axis.
    /// * `size_z` - The depth of the object along the Z-axis.
    /// 
    /// * `custom_data` - The custom data associated with the object, wrapped in an `Arc`.
    ///
    /// # Returns
    ///
    /// * `Result<(), String>` - An empty result if successful, or an error message if not.
    ///
    /// # Examples
    ///
    /// ```
    /// # use your_crate::{VaultManager, CustomData};
    /// # use uuid::Uuid;
    /// # use std::sync::Arc;
    /// # let mut vault_manager: VaultManager<CustomData> = VaultManager::new("path/to/database.db").unwrap();
    /// # let region_id = vault_manager.create_or_load_region([0.0, 0.0, 0.0], 100.0).unwrap();
    /// let object_id = Uuid::new_v4();
    /// let custom_data = Arc::new(CustomData { /* ... */ });
    /// vault_manager.add_object(region_id, object_id, "player", 1.0, 2.0, 3.0, custom_data).expect("Failed to add object");
    /// ```
    ///
    /// # Notes
    ///
    /// - The object is added to the specified region regardless of its coordinates. Ensure the coordinates fall within the region's cubic bounds for consistent behavior.
    /// - If an object with the same UUID already exists, it will be overwritten.
    /// - The `custom_data` is stored as an `Arc<T>` to allow efficient sharing of data between objects.
    pub fn add_object(
        &self,
        region_id: Uuid,
        uuid: Uuid,
        object_type: &str,
        x: f64,
        y: f64,
        z: f64,
        size_x: f64,
        size_y: f64,
        size_z: f64,
        custom_data: Arc<T>,
    ) -> Result<(), String> {
        let region = self.regions.get(&region_id)
            .ok_or_else(|| format!("Region not found: {}", region_id))?;
        
        let mut region = region.lock().unwrap();
        
        let object = SpatialObject {
            uuid,
            object_type: object_type.to_string(),
            point: [x, y, z],
            size: [size_x, size_y, size_z],
            custom_data: custom_data.clone(),
        };
        
        region.rtree.insert(object.clone());

        let point = Point {
            id: Some(uuid),
            x,
            y,
            z,
            size_x,
            size_y,
            size_z,
            object_type: object_type.to_string(),
            custom_data: serde_json::to_value((*custom_data).clone()).map_err(|e| format!("Failed to serialize custom data: {}", e))?,
        };
        
        self.persistent_db.add_point(&point, region_id)
            .map_err(|e| format!("Failed to add point to persistent database: {}", e))?;

        Ok(())
    }

    /// Queries objects within a specific region.
    ///
    /// This function searches for objects within a given cubic bounding box in a specified region.
    /// It's useful for finding all objects in a particular area, such as for rendering or game logic.
    ///
    /// # Arguments
    ///
    /// * `region_id` - The UUID of the region to query.
    /// * `min_x`, `min_y`, `min_z` - The minimum coordinates of the bounding box.
    /// * `max_x`, `max_y`, `max_z` - The maximum coordinates of the bounding box.
    ///
    /// # Returns
    ///
    /// * `Result<Vec<SpatialObject<T>>, String>` - A vector of `SpatialObject`s within the bounding box if successful, or an error message if not.
    ///
    /// # Examples
    ///
    /// ```
    /// # use your_crate::{VaultManager, CustomData};
    /// # use uuid::Uuid;
    /// # let vault_manager: VaultManager<CustomData> = VaultManager::new("path/to/database.db").unwrap();
    /// # let region_id = Uuid::new_v4();
    /// // Query a 10x10x10 cubic area
    /// let objects = vault_manager.query_region(region_id, 0.0, 0.0, 0.0, 10.0, 10.0, 10.0).expect("Failed to query region");
    /// for object in objects {
    ///     println!("Found object: {:?}", object.uuid);
    /// }
    /// ```
    ///
    /// # Notes
    ///
    /// - The query is performed using an R-tree, which provides efficient spatial searching.
    /// - Objects intersecting the cubic bounding box are included in the results.
    /// - The query box does not need to align with region boundaries.
    pub fn query_region(&self, region_id: Uuid, min_x: f64, min_y: f64, min_z: f64, max_x: f64, max_y: f64, max_z: f64) -> Result<Vec<SpatialObject<T>>, String> {
        let region = self.regions.get(&region_id)
            .ok_or_else(|| format!("Region not found: {}", region_id))?;
        
        let region = region.lock().unwrap();
        let envelope = AABB::from_corners([min_x, min_y, min_z], [max_x, max_y, max_z]);
        let results: Vec<SpatialObject<T>> = region.rtree.locate_in_envelope(&envelope)
            .cloned()
            .collect();

        Ok(results)
    }

    /// Transfers a player (object) from one region to another.
    ///
    /// This function moves a player object from its current region to a new region,
    /// updating the in-memory RTree structure. The object's size and custom data are preserved.
    ///
    /// # Arguments
    ///
    /// * `player_uuid` - The UUID of the player to transfer.
    /// * `from_region_id` - The UUID of the source region.
    /// * `to_region_id` - The UUID of the destination region.
    ///
    /// # Returns
    ///
    /// * `Result<(), String>` - Ok if successful, or an error string if the transfer fails.
    ///
    /// # Examples
    ///
    /// ```
    /// # use your_crate::{VaultManager, CustomData};
    /// # use uuid::Uuid;
    /// # let mut vault_manager: VaultManager<CustomData> = VaultManager::new("path/to/database.db").unwrap();
    /// # let from_region_id = vault_manager.create_or_load_region([0.0, 0.0, 0.0], 100.0).unwrap();
    /// # let to_region_id = vault_manager.create_or_load_region([200.0, 200.0, 200.0], 100.0).unwrap();
    /// # let player_id = Uuid::new_v4();
    /// # let custom_data = Arc::new(CustomData { /* ... */ });
    /// # vault_manager.add_object(from_region_id, player_id, "player", 1.0, 2.0, 3.0, 1.0, 1.0, 1.0, custom_data).unwrap();
    /// vault_manager.transfer_player(player_id, from_region_id, to_region_id).expect("Failed to transfer player");
    /// ```
    ///
    /// # Notes
    ///
    /// - The player’s position is set to the center of the destination region.
    /// - The player's size and custom data are preserved.
    /// - This does **not** persist the change to the database; call `persist_to_disk()` to flush to disk.
    pub fn transfer_player(&self, player_uuid: Uuid, from_region_id: Uuid, to_region_id: Uuid) -> Result<(), String> {
        let from_region = self.regions.get(&from_region_id)
            .ok_or_else(|| format!("Source region not found: {}", from_region_id))?;
        let to_region = self.regions.get(&to_region_id)
            .ok_or_else(|| format!("Destination region not found: {}", to_region_id))?;

        let mut from_region = from_region.lock().unwrap();
        let mut to_region = to_region.lock().unwrap();

        let player = from_region.rtree.iter()
            .find(|obj| obj.uuid == player_uuid)
            .cloned()
            .ok_or_else(|| format!("Player not found in source region: {}", player_uuid))?;

        from_region.rtree.remove(&player);

        let updated_player = SpatialObject {
            uuid: player.uuid,
            object_type: player.object_type,
            point: to_region.center,
            size: player.size, // <- preserve size
            custom_data: player.custom_data.clone(),
        };

        to_region.rtree.insert(updated_player);

        // TODO: Update the player's position in the persistent database

        Ok(())
    }

    /// Persists all in-memory databases to disk.
    ///
    /// This function saves all objects from all regions to the persistent database.
    /// It's important to call this method periodically to ensure data is not lost in case of unexpected shutdowns.
    ///
    /// # Returns
    ///
    /// * `Result<(), String>` - An empty result if successful, or an error message if not.
    ///
    /// # Examples
    ///
    /// ```
    /// # use your_crate::{VaultManager, CustomData};
    /// # let mut vault_manager: VaultManager<CustomData> = VaultManager::new("path/to/database.db").unwrap();
    /// vault_manager.persist_to_disk().expect("Failed to persist data to disk");
    /// ```
    ///
    /// # Notes
    ///
    /// - This operation can be time-consuming for large datasets. Consider running it in a separate thread.
    /// - The method provides progress feedback using a progress bar.
    /// - All existing points in the database are cleared before persisting the current state.
    pub fn persist_to_disk(&self) -> Result<(), String> {
        let start_time = std::time::Instant::now();
        let mut total_points = 0;

        self.persistent_db.clear_all_points()
            .map_err(|e| format!("Failed to clear existing points from database: {}", e))?;

        for (_, region) in &self.regions {
            let region = region.lock().unwrap();
            total_points += region.rtree.size();
        }

        let pb = ProgressBar::new(total_points as u64);
        pb.set_style(ProgressStyle::default_bar()
            .template("[{elapsed_precise}] {bar:40.cyan/blue} {pos:>7}/{len:7} {msg}")
            .unwrap()
            .progress_chars("##-"));

        for (region_id, region) in &self.regions {
            let region = region.lock().unwrap();
            for obj in region.rtree.iter() {
                let point = Point {
                    id: Some(obj.uuid),
                    x: obj.point[0],
                    y: obj.point[1],
                    z: obj.point[2],
                    size_x: obj.size[0],
                    size_y: obj.size[1],
                    size_z: obj.size[2],
                    object_type: obj.object_type.clone(),
                    custom_data: serde_json::to_value((*obj.custom_data).clone())
                        .map_err(|e| format!("Failed to serialize custom data: {}", e))?,
                };
                self.persistent_db.add_point(&point, *region_id)
                    .map_err(|e| format!("Failed to persist point to database: {}", e))?;
                pb.inc(1);
            }
        }

        pb.finish_with_message("Points persisted");

        let duration = start_time.elapsed();
        println!("Persisted {} points in {:?}", total_points, duration);
        println!("Average time per point: {:?}", duration / total_points as u32);
        Ok(())
    }

    /// Gets a reference to a region by its ID.
    ///
    /// This method is useful when you need to perform operations on a specific region.
    ///
    /// # Arguments
    ///
    /// * `region_id` - The UUID of the region to retrieve.
    ///
    /// # Returns
    ///
    /// * `Option<Arc<Mutex<VaultRegion<T>>>>` - An `Option` containing a reference to the region if found, or `None` if not found.
    pub fn get_region(&self, region_id: Uuid) -> Option<Arc<Mutex<VaultRegion<T>>>> {
        self.regions.get(&region_id).cloned()
    }

    /// Removes an object from its region and the persistent database.
    ///
    /// # Arguments
    ///
    /// * `object_id` - The UUID of the object to remove.
    ///
    /// # Returns
    ///
    /// * `Result<(), String>` - An empty result if successful, or an error message if not.
    pub fn remove_object(&mut self, object_id: Uuid) -> Result<(), String> {
        // Find the region containing the object
        for (_, region) in &mut self.regions {
            let mut region = region.lock().unwrap();
            // Find and remove the object from the RTree
            let mut object_to_remove = None;
            for obj in region.rtree.iter() {
                if obj.uuid == object_id {
                    object_to_remove = Some(obj.clone());
                    break;
                }
            }
            
            if let Some(obj) = object_to_remove {
                region.rtree.remove(&obj);
                // Remove the object from the persistent database
                self.persistent_db.remove_point(object_id)
                    .map_err(|e| format!("Failed to remove point from persistent database: {}", e))?;
                return Ok(());
            }
        }
        Err(format!("Object not found: {}", object_id))
    }

    /// Gets a reference to an object by its ID.
    ///
    /// This method searches for an object with the given UUID across all regions.
    ///
    /// # Arguments
    ///
    /// * `object_id` - The UUID of the object to retrieve.
    ///
    /// # Returns
    ///
    /// * `Result<Option<SpatialObject<T>>, String>` - An `Option` containing a clone of the object if found, or `None` if not found.
    ///
    /// # Examples
    ///
    /// ```
    /// # use your_crate::{VaultManager, CustomData};
    /// # use uuid::Uuid;
    /// # let vault_manager: VaultManager<CustomData> = VaultManager::new("path/to/database.db").unwrap();
    /// # let object_id = Uuid::new_v4();
    /// if let Ok(Some(object)) = vault_manager.get_object(object_id) {
    ///     println!("Found object: {:?}", object);
    /// } else {
    ///     println!("Object not found");
    /// }
    /// ```
    ///
    /// # Notes
    ///
    /// - This method returns a clone of the `SpatialObject`, including the `Arc<T>` custom data.
    /// - The search is performed across all regions, which may be slow for a large number of regions or objects.
    pub fn get_object(&self, object_id: Uuid) -> Result<Option<SpatialObject<T>>, String> {
        for (_, region) in &self.regions {
            let region = region.lock().unwrap();
            let object = region.rtree.iter().find(|obj| obj.uuid == object_id).cloned();
            if let Some(obj) = object {
                return Ok(Some(obj));
            }
        }
        Ok(None)
    }

    /// Updates an existing object in the VaultManager's in-memory storage.
    ///
    /// This method updates only the in-memory representation of the object.
    /// It does not update the persistent storage. Use `persist_to_disk` for saving changes to the database.
    ///
    /// # Arguments
    ///
    /// * `object` - A reference to the updated SpatialObject.
    ///
    /// # Returns
    ///
    /// * `Result<(), String>` - Ok if the update is successful, or an error message if it fails.
    ///
    /// # Examples
    ///
    /// ```
    /// # use your_crate::{VaultManager, SpatialObject, CustomData};
    /// # use uuid::Uuid;
    /// # use std::sync::Arc;
    /// # let mut vault_manager: VaultManager<CustomData> = VaultManager::new("path/to/database.db").unwrap();
    /// # let object_id = Uuid::new_v4();
    /// # let mut object = vault_manager.get_object(object_id).unwrap().unwrap();
    /// // Modify the object
    /// object.custom_data = Arc::new(CustomData { /* ... */ });
    /// vault_manager.update_object(&object).expect("Failed to update object");
    /// ```
    pub fn update_object(&mut self, object: &SpatialObject<T>) -> Result<(), String> {
        let mut updated = false;

        // Find the region containing the object
        for (_, region) in &mut self.regions {
            let mut region = region.lock().unwrap();
            let existing_obj = region.rtree.iter().find(|obj| obj.uuid == object.uuid).cloned();
            
            if let Some(existing) = existing_obj {
                // Remove the existing object and insert the updated one
                region.rtree.remove(&existing);
                region.rtree.insert(object.clone());
                updated = true;
                break;
            }
        }

        if !updated {
            return Err(format!("Object not found in any region: {}", object.uuid));
        }

        Ok(())
    }
}