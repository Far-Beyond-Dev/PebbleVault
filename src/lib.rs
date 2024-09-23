#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

mod MySQLGeo;

use rstar::*;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use serde::{Serialize, Deserialize};
use uuid::Uuid;

/// Represents a spatial object in the game world.
#[derive(Clone, Serialize, Deserialize, PartialEq)]
pub struct SpatialObject {
    /// Unique identifier for the object
    pub uuid: Uuid,
    /// Associated data with the object (e.g., player info, item details)
    pub data: String,
    /// 3D coordinates of the object [x, y, z]
    pub point: [f64; 3],
}

impl PointDistance for SpatialObject {
    fn distance_2(&self, point: &[f64; 3]) -> f64 {
        let dx = self.point[0] - point[0];
        let dy = self.point[1] - point[1];
        let dz = self.point[2] - point[2];
        dx * dx + dy * dy + dz * dz
    }
}

impl RTreeObject for SpatialObject {
    type Envelope = AABB<[f64; 3]>;

    fn envelope(&self) -> Self::Envelope {
        AABB::from_point(self.point)
    }
}

/// Represents a region in the game world.
pub struct Region {
    /// Unique identifier for the region
    id: Uuid,
    /// Center coordinates of the region
    center: [f64; 3],
    /// Radius of the region
    radius: f64,
    /// Spatial index (RTree) for objects in this region
    rtree: RTree<SpatialObject>,
}

/// Manages the spatial databases for different regions in the game world.
pub struct VaultManager {
    /// Map of region IDs to their corresponding Region structures
    regions: HashMap<Uuid, Arc<Mutex<Region>>>,
    /// Connection to the persistent database
    persistent_db: MySQLGeo::Database,
}

impl VaultManager {
    /// Creates a new instance of VaultManager.
    ///
    /// # Arguments
    ///
    /// * `db_path` - The path to the persistent database file.
    ///
    /// # Returns
    ///
    /// A Result containing a new instance of VaultManager or an error message.
    pub fn new(db_path: &str) -> Result<Self, String> {
        let persistent_db = MySQLGeo::Database::new(db_path)
            .map_err(|e| format!("Failed to create persistent database: {}", e))?;
        persistent_db.create_table()
            .map_err(|e| format!("Failed to create table: {}", e))?;
        
        Ok(VaultManager {
            regions: HashMap::new(),
            persistent_db,
        })
    }

    /// Creates a new region or loads an existing one from the persistent database.
    ///
    /// # Arguments
    ///
    /// * `center` - The center coordinates of the region.
    /// * `radius` - The radius of the region.
    ///
    /// # Returns
    ///
    /// A Result containing the UUID of the created or loaded region, or an error message.
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

        let region = Region {
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
    /// # Arguments
    ///
    /// * `region_id` - The UUID of the region.
    /// * `uuid` - The UUID of the object.
    /// * `x` - The x-coordinate of the object.
    /// * `y` - The y-coordinate of the object.
    /// * `z` - The z-coordinate of the object.
    /// * `data` - The data associated with the object.
    ///
    /// # Returns
    ///
    /// A Result indicating success or an error message.
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
    /// # Arguments
    ///
    /// * `region_id` - The UUID of the region.
    /// * `min_x` - The minimum x-coordinate of the query box.
    /// * `min_y` - The minimum y-coordinate of the query box.
    /// * `min_z` - The minimum z-coordinate of the query box.
    /// * `max_x` - The maximum x-coordinate of the query box.
    /// * `max_y` - The maximum y-coordinate of the query box.
    /// * `max_z` - The maximum z-coordinate of the query box.
    ///
    /// # Returns
    ///
    /// A Result containing a vector of objects within the query box, or an error message.
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
    /// # Arguments
    ///
    /// * `player_uuid` - The UUID of the player object.
    /// * `from_region_id` - The UUID of the source region.
    /// * `to_region_id` - The UUID of the destination region.
    ///
    /// # Returns
    ///
    /// A Result indicating success or an error message.
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
    /// This method should be called periodically to ensure data is not lost.
    ///
    /// # Returns
    ///
    /// A Result indicating success or an error message.
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

pub mod tests;

pub mod load_test;
