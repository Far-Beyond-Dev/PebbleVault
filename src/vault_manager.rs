use crate::structs::{VaultRegion, SpatialObject};
use crate::MySQLGeo;
use uuid::Uuid;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use rstar::{RTree, AABB};

/// Manages spatial regions and objects within a persistent database.
///
/// The `VaultManager` is responsible for creating and managing regions,
/// adding spatial objects to these regions, querying objects within regions,
/// and persisting data to a MySQL database.
pub struct VaultManager {
    pub regions: HashMap<Uuid, Arc<Mutex<VaultRegion>>>,
    pub persistent_db: MySQLGeo::Database,
}

impl VaultManager {
    /// Creates a new instance of `VaultManager`.
    ///
    /// This method initializes the persistent database and creates the necessary table.
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
    /// let vault_manager = VaultManager::new("/data").expect("Failed to create VaultManager");
    /// ```
    pub fn new(db_path: &str) -> Result<Self, String> {
        let persistent_db = MySQLGeo::Database::new(db_path)
            .map_err(|e| format!("Failed to create persistent database: {}", e))?;
        persistent_db.create_table()
            .map_err(|e| format!("Failed to create table: {}", e))?;
        
        Ok(VaultManager {
            regions: std::collections::HashMap::new(),
            persistent_db,
        })
    }

    /// Creates a new region or loads an existing one from the persistent database.
    ///
    /// If a region with the same center and radius already exists, it returns the existing region's ID.
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
    /// let region_id = vault_manager.create_or_load_region([0.0, 0.0, 0.0], 100.0).expect("Failed to create region");
    /// ```
    pub fn create_or_load_region(&mut self, center: [f64; 3], radius: f64) -> Result<Uuid, String> {
        // Check if a region with the same center and radius already exists
        if let Some(existing_region) = self.regions.values().find(|r| {
            let r = r.lock().unwrap();
            r.center == center && r.radius == radius
        }) {
            return Ok(existing_region.lock().unwrap().id);
        }

        let region_id = Uuid::new_v4();
        let rtree = RTree::new();

        let region = VaultRegion {
            id: region_id,
            center,
            radius,
            rtree,
        };

        self.regions.insert(region_id, Arc::new(Mutex::new(region)));
        Ok(region_id)
    }

    /// Adds an object to a specific region.
    ///
    /// This method adds a spatial object to both the in-memory R-tree and the persistent database.
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
    /// let object_uuid = Uuid::new_v4();
    /// vault_manager.add_object(region_id, object_uuid, 1.0, 2.0, 3.0, "Player data").expect("Failed to add object");
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

        // Add to persistent database
        let point = MySQLGeo::Point::new(Some(uuid), x, y, z, serde_json::Value::String(data.to_string()));
        self.persistent_db.add_point(&point)
            .map_err(|e| format!("Failed to add point to persistent database: {}", e))?;

        Ok(())
    }

    /// Queries objects within a specific region.
    ///
    /// This method returns all objects within the specified bounding box in the given region.
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
    /// let objects = vault_manager.query_region(region_id, 0.0, 0.0, 0.0, 10.0, 10.0, 10.0).expect("Failed to query region");
    /// ```
    pub fn query_region(&self, region_id: Uuid, min_x: f64, min_y: f64, min_z: f64, max_x: f64, max_y: f64, max_z: f64) -> Result<Vec<SpatialObject>, String> {
        let region = self.regions.get(&region_id)
            .ok_or_else(|| format!("Region not found: {}", region_id))?;
        
        let region = region.lock().unwrap();
        let envelope = AABB::from_corners([min_x, min_y, min_z], [max_x, max_y, max_z]);
        let results: Vec<SpatialObject> = region.rtree.locate_in_envelope(&envelope)
            .cloned()
            .collect();

        Ok(results)
    }

    /// Transfers a player (object) from one region to another.
    ///
    /// This method removes the player from the source region and adds them to the destination region,
    /// updating their position to the center of the destination region.
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
    /// vault_manager.transfer_player(player_uuid, source_region_id, destination_region_id).expect("Failed to transfer player");
    /// ```
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

        // Update player position to the center of the destination region
        let updated_player = SpatialObject {
            uuid: player.uuid,
            data: player.data,
            point: to_region.center,
        };

        to_region.rtree.insert(updated_player);

        Ok(())
    }

    /// Persists all in-memory databases to disk.
    ///
    /// This method writes all objects from all regions to the persistent database.
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
        for region in self.regions.values() {
            let region = region.lock().unwrap();
            for obj in region.rtree.iter() {
                let point = MySQLGeo::Point::new(
                    Some(obj.uuid),
                    obj.point[0],
                    obj.point[1],
                    obj.point[2],
                    serde_json::Value::String(obj.data.clone()),
                );
                self.persistent_db.add_point(&point)
                    .map_err(|e| format!("Failed to persist point to database: {}", e))?;
            }
        }
        Ok(())
    }
}