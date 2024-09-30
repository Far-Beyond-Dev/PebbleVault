//! VaultManager: A module for managing spatial regions and objects.
//!
//! This module provides the VaultManager struct, which is responsible for creating and managing regions,
//! adding spatial objects to these regions, querying objects within regions, and persisting data to a database.

use crate::structs::{VaultRegion, SpatialObject};
use crate::MySQLGeo;
use uuid::Uuid;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use rstar::{RTree, AABB};
use indicatif::{ProgressBar, ProgressStyle};

/// Manages spatial regions and objects within a persistent database.
pub struct VaultManager {
    /// HashMap storing regions, keyed by their UUID
    pub regions: HashMap<Uuid, Arc<Mutex<VaultRegion>>>,
    /// Persistent database connection
    pub persistent_db: MySQLGeo::Database,
}

impl VaultManager {
    /// Creates a new instance of `VaultManager`.
    ///
    /// # Arguments
    ///
    /// * `db_path` - A string slice that holds the path to the database.
    ///
    /// # Returns
    ///
    /// * `Result<Self, String>` - A new `VaultManager` instance if successful, or an error message if not.
    ///
    /// # Examples
    ///
    /// ```
    /// let vault_manager = VaultManager::new("path/to/database.db").expect("Failed to create VaultManager");
    /// ```
    pub fn new(db_path: &str) -> Result<Self, String> {
        // Create a new persistent database connection
        let persistent_db = MySQLGeo::Database::new(db_path)
            .map_err(|e| format!("Failed to create persistent database: {}", e))?;
        // Create the necessary tables in the database
        persistent_db.create_table()
            .map_err(|e| format!("Failed to create table: {}", e))?;
        
        let mut vault_manager = VaultManager {
            regions: std::collections::HashMap::new(),
            persistent_db,
        };

        // Load existing regions from the persistent database
        vault_manager.load_regions_from_db()?;

        Ok(vault_manager)
    }

    fn load_regions_from_db(&mut self) -> Result<(), String> {
        // Get all regions from the persistent database
        let regions = self.persistent_db.get_all_regions()
            .map_err(|e| format!("Failed to load regions from database: {}", e))?;

        println!("Loaded {} regions from the database", regions.len());

        // Iterate through the loaded regions and add them to the VaultManager
        for region in regions {
            println!("Loading region: ID: {}, Center: {:?}, Radius: {}", region.id, region.center, region.radius);
            let vault_region = VaultRegion {
                id: region.id,
                center: region.center,
                radius: region.radius,
                rtree: RTree::new(),
            };

            // Insert the region into the regions HashMap
            self.regions.insert(region.id, Arc::new(Mutex::new(vault_region)));

            // Load points for this region
            let points = self.persistent_db.get_points_in_region(region.id)
                .map_err(|e| format!("Failed to load points for region {}: {}", region.id, e))?;

            println!("Loaded {} points for region {}", points.len(), region.id);

            // Add points to the region's RTree
            if let Some(region_arc) = self.regions.get(&region.id) {
                let mut region = region_arc.lock().unwrap();
                for point in points {
                    let spatial_object = SpatialObject {
                        uuid: point.id.unwrap(),
                        data: point.data.to_string(),
                        point: [point.x, point.y, point.z],
                    };
                    region.rtree.insert(spatial_object);
                }
            }
        }

        Ok(())
    }

    /// Creates a new region or loads an existing one from the persistent database.
    ///
    /// # Arguments
    ///
    /// * `center` - An array of 3 f64 values representing the x, y, z coordinates of the region's center.
    /// * `radius` - The radius of the region.
    ///
    /// # Returns
    ///
    /// * `Result<Uuid, String>` - The UUID of the created or loaded region if successful, or an error message if not.
    ///
    /// # Examples
    ///
    /// ```
    /// let center = [0.0, 0.0, 0.0];
    /// let radius = 100.0;
    /// let region_id = vault_manager.create_or_load_region(center, radius).expect("Failed to create region");
    /// ```
    pub fn create_or_load_region(&mut self, center: [f64; 3], radius: f64) -> Result<Uuid, String> {
        // Check if a region with the same center and radius already exists
        if let Some(existing_region) = self.regions.values().find(|r| {
            let r = r.lock().unwrap();
            r.center == center && r.radius == radius
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
            radius,
            rtree,
        };

        // Insert the new region into the regions HashMap
        self.regions.insert(region_id, Arc::new(Mutex::new(region)));

        // Persist the region to the database
        self.persistent_db.create_region(region_id, center, radius)
            .map_err(|e| format!("Failed to persist region to database: {}", e))?;

        Ok(region_id)
    }

    /// Adds an object to a specific region.
    ///
    /// # Arguments
    ///
    /// * `region_id` - The UUID of the region to add the object to.
    /// * `uuid` - The UUID of the object being added.
    /// * `x` - The x-coordinate of the object.
    /// * `y` - The y-coordinate of the object.
    /// * `z` - The z-coordinate of the object.
    /// * `data` - A string slice containing additional data associated with the object.
    ///
    /// # Returns
    ///
    /// * `Result<(), String>` - An empty result if successful, or an error message if not.
    ///
    /// # Examples
    ///
    /// ```
    /// let region_id = Uuid::new_v4();
    /// let object_id = Uuid::new_v4();
    /// vault_manager.add_object(region_id, object_id, 1.0, 2.0, 3.0, "Example object").expect("Failed to add object");
    /// ```
    pub fn add_object(&self, region_id: Uuid, uuid: Uuid, x: f64, y: f64, z: f64, data: &str) -> Result<(), String> {
        let region = self.regions.get(&region_id)
            .ok_or_else(|| format!("Region not found: {}", region_id))?;
        
        let mut region = region.lock().unwrap();
        let object = SpatialObject {
            uuid,
            data: data.to_string(),
            point: [x, y, z],
        };
        
        region.rtree.insert(object.clone());

        let point = MySQLGeo::Point::new(Some(uuid), x, y, z, serde_json::Value::String(data.to_string()));
        self.persistent_db.add_point(&point, region_id)
            .map_err(|e| format!("Failed to add point to persistent database: {}", e))?;

        Ok(())
    }

    /// Queries objects within a specific region.
    ///
    /// # Arguments
    ///
    /// * `region_id` - The UUID of the region to query.
    /// * `min_x`, `min_y`, `min_z` - The minimum coordinates of the bounding box.
    /// * `max_x`, `max_y`, `max_z` - The maximum coordinates of the bounding box.
    ///
    /// # Returns
    ///
    /// * `Result<Vec<SpatialObject>, String>` - A vector of `SpatialObject`s within the bounding box if successful, or an error message if not.
    ///
    /// # Examples
    ///
    /// ```
    /// let region_id = Uuid::new_v4();
    /// let objects = vault_manager.query_region(region_id, 0.0, 0.0, 0.0, 10.0, 10.0, 10.0).expect("Failed to query region");
    /// for object in objects {
    ///     println!("Found object: {:?}", object);
    /// }
    /// ```
    pub fn query_region(&self, region_id: Uuid, min_x: f64, min_y: f64, min_z: f64, max_x: f64, max_y: f64, max_z: f64) -> Result<Vec<SpatialObject>, String> {
        // Get the region from the regions HashMap
        let region = self.regions.get(&region_id)
            .ok_or_else(|| format!("Region not found: {}", region_id))?;
        
        // Lock the region for reading
        let region = region.lock().unwrap();
        // Create an AABB envelope for the query
        let envelope = AABB::from_corners([min_x, min_y, min_z], [max_x, max_y, max_z]);
        // Query the RTree and collect the results
        let results: Vec<SpatialObject> = region.rtree.locate_in_envelope(&envelope)
            .cloned()
            .collect();

        Ok(results)
    }

    /// Transfers a player (object) from one region to another.
    ///
    /// # Arguments
    ///
    /// * `player_uuid` - The UUID of the player to transfer.
    /// * `from_region_id` - The UUID of the source region.
    /// * `to_region_id` - The UUID of the destination region.
    ///
    /// # Returns
    ///
    /// * `Result<(), String>` - An empty result if successful, or an error message if not.
    ///
    /// # Examples
    ///
    /// ```
    /// let player_id = Uuid::new_v4();
    /// let from_region_id = Uuid::new_v4();
    /// let to_region_id = Uuid::new_v4();
    /// vault_manager.transfer_player(player_id, from_region_id, to_region_id).expect("Failed to transfer player");
    /// ```
    pub fn transfer_player(&self, player_uuid: Uuid, from_region_id: Uuid, to_region_id: Uuid) -> Result<(), String> {
        // Get the source and destination regions
        let from_region = self.regions.get(&from_region_id)
            .ok_or_else(|| format!("Source region not found: {}", from_region_id))?;
        let to_region = self.regions.get(&to_region_id)
            .ok_or_else(|| format!("Destination region not found: {}", to_region_id))?;

        // Lock both regions for mutation
        let mut from_region = from_region.lock().unwrap();
        let mut to_region = to_region.lock().unwrap();

        // Find the player in the source region
        let player = from_region.rtree.iter()
            .find(|obj| obj.uuid == player_uuid)
            .cloned()
            .ok_or_else(|| format!("Player not found in source region: {}", player_uuid))?;

        // Remove the player from the source region
        from_region.rtree.remove(&player);

        // Update player position to the center of the destination region
        let updated_player = SpatialObject {
            uuid: player.uuid,
            data: player.data,
            point: to_region.center,
        };

        // Insert the updated player into the destination region
        to_region.rtree.insert(updated_player);

        Ok(())
    }

    /// Persists all in-memory databases to disk.
    ///
    /// # Returns
    ///
    /// * `Result<(), String>` - An empty result if successful, or an error message if not.
    ///
    /// # Examples
    ///
    /// ```
    /// vault_manager.persist_to_disk().expect("Failed to persist data to disk");
    /// ```
    pub fn persist_to_disk(&self) -> Result<(), String> {
        let start_time = std::time::Instant::now();
        let mut total_points = 0;

        // First, clear all existing points from the database
        self.persistent_db.clear_all_points()
            .map_err(|e| format!("Failed to clear existing points from database: {}", e))?;

        // Count total points
        for (_, region) in &self.regions {
            let region = region.lock().unwrap();
            total_points += region.rtree.size();
        }

        // DEBUG: Progress bar for persisting points
        let pb = ProgressBar::new(total_points as u64);
        pb.set_style(ProgressStyle::default_bar()
            .template("[{elapsed_precise}] {bar:40.cyan/blue} {pos:>7}/{len:7} {msg}")
            .unwrap()
            .progress_chars("##-"));

        for (region_id, region) in &self.regions {
            let region = region.lock().unwrap();
            for obj in region.rtree.iter() {
                let point = MySQLGeo::Point::new(
                    Some(obj.uuid),
                    obj.point[0],
                    obj.point[1],
                    obj.point[2],
                    serde_json::Value::String(obj.data.clone()),
                );
                self.persistent_db.add_point(&point, *region_id)
                    .map_err(|e| format!("Failed to persist point to database: {}", e))?;
                // DEBUG: Increment progress bar
                pb.inc(1);
            }
        }

        // DEBUG: Finish progress bar
        pb.finish_with_message("Points persisted");

        let duration = start_time.elapsed();
        println!("Persisted {} points in {:?}", total_points, duration);
        println!("Average time per point: {:?}", duration / total_points as u32);
        Ok(())
    }

    /// Gets a reference to a region by its ID.
    ///
    /// # Arguments
    ///
    /// * `region_id` - The UUID of the region to retrieve.
    ///
    /// # Returns
    ///
    /// * `Option<Arc<Mutex<VaultRegion>>>` - An `Option` containing a reference to the region if found, or `None` if not found.
    pub fn get_region(&self, region_id: Uuid) -> Option<Arc<Mutex<VaultRegion>>> {
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
        for (region_id, region) in &mut self.regions {
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
}