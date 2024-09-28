use crate::structs::{BarnesHutSimulation, Body, Vector3D, BarnesHutConfig, BarnesHutRegion, SpatialObject};
use crate::MySQLGeo;
use uuid::Uuid;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use rstar::{AABB, RTree};
use glutin_window::GlutinWindow as Window;
use opengl_graphics::OpenGL;
use piston::event_loop::{EventSettings, Events};
use piston::input::*;
use crate::barnes_hut::visualization::{create_window, App};

/// Manages the spatial databases and Barnes-Hut simulations for different regions in the game world.
pub struct BarnesHutManager {
    /// Map of region IDs to their corresponding Region structures
    pub regions: HashMap<Uuid, Arc<Mutex<BarnesHutRegion>>>,
    /// Connection to the persistent database
    pub persistent_db: Arc<MySQLGeo::Database>,
    /// Barnes-Hut configuration
    pub barnes_hut_config: BarnesHutConfig,
    /// Octree size for Barnes-Hut simulations
    pub octree_size: f64,
}

impl Clone for BarnesHutManager {
    fn clone(&self) -> Self {
        BarnesHutManager {
            regions: self.regions.clone(),
            persistent_db: Arc::clone(&self.persistent_db),
            barnes_hut_config: self.barnes_hut_config.clone(),
            octree_size: self.octree_size,
        }
    }
}

impl BarnesHutManager {
    /// Creates a new instance of `BarnesHutManager`.
    pub fn new(db_path: &str, barnes_hut_config: BarnesHutConfig, octree_size: f64) -> Result<Self, String> {
        let persistent_db = MySQLGeo::Database::new(db_path)
            .map_err(|e| format!("Failed to create persistent database: {}", e))?;
        persistent_db.create_table()
            .map_err(|e| format!("Failed to create table: {}", e))?;
        
        Ok(BarnesHutManager {
            regions: HashMap::new(),
            persistent_db: Arc::new(persistent_db),
            barnes_hut_config,
            octree_size,
        })
    }

    /// Creates a new region or loads an existing one from the persistent database.
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

        let region = BarnesHutRegion {
            id: region_id,
            center,
            radius,
            rtree,
            simulation: None,
        };

        self.regions.insert(region_id, Arc::new(Mutex::new(region)));
        Ok(region_id)
    }

    /// Adds an object to a specific region.
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

    // Barnes-Hut specific methods

    /// Creates a new Barnes-Hut simulation for a given region.
    pub fn create_simulation(&mut self, region_id: Uuid) -> Result<(), String> {
        let region = self.regions.get_mut(&region_id)
            .ok_or_else(|| format!("Region not found: {}", region_id))?;
        let mut region = region.lock().unwrap();
        
        let bodies: Vec<Body> = region.rtree.iter()
            .map(|obj| Body {
                mass: 1.0, // Default mass, adjust as needed
                position: Vector3D::new(obj.point[0], obj.point[1], obj.point[2]),
                velocity: Vector3D::zero(),
                force: Vector3D::zero(),
            })
            .collect();

        let simulation = BarnesHutSimulation::new(bodies, self.barnes_hut_config.clone(), self.octree_size);
        region.simulation = Some(Arc::new(Mutex::new(simulation)));
        Ok(())
    }

    /// Steps the Barnes-Hut simulation for a specific region.
    pub fn step_simulation(&mut self, region_id: Uuid) -> Result<(), String> {
        let region = self.regions.get_mut(&region_id)
            .ok_or_else(|| format!("Region not found: {}", region_id))?;
        
        // Step the simulation and get updated body positions
        let updated_bodies = {
            let mut region_guard = region.lock().unwrap();
            if let Some(simulation) = &mut region_guard.simulation {
                let mut sim = simulation.lock().map_err(|e| e.to_string())?;
                sim.step();
                sim.bodies.clone()
            } else {
                return Err(format!("Simulation for region {} not found", region_id));
            }
        };

        // Update the R-tree with new positions
        {
            let mut region_guard = region.lock().unwrap();
            let mut new_rtree = RTree::new();
            
            for (obj, body) in region_guard.rtree.iter().zip(updated_bodies.iter()) {
                let mut updated_obj = obj.clone();
                updated_obj.point = [body.position.x, body.position.y, body.position.z];
                new_rtree.insert(updated_obj);
            }
            
            region_guard.rtree = new_rtree;
        }

        Ok(())
    }

    /// Retrieves the current state of bodies in a specific simulation.
    pub fn get_bodies(&self, region_id: Uuid) -> Result<Vec<Body>, String> {
        let region = self.regions.get(&region_id)
            .ok_or_else(|| format!("Region not found: {}", region_id))?;
        let region = region.lock().unwrap();
        
        if let Some(simulation) = &region.simulation {
            let sim = simulation.lock().map_err(|e| e.to_string())?;
            Ok(sim.bodies.clone())
        } else {
            Err(format!("Simulation for region {} not found", region_id))
        }
    }

    /// Runs a visualization of the Barnes-Hut simulation for a specific region.
    pub fn run_visualization(&mut self, region_id: Uuid) -> Result<(), String> {
        let region = self.regions.get(&region_id)
            .ok_or_else(|| format!("Region not found: {}", region_id))?;
        let region_guard = region.lock().unwrap();
        let (width, height) = (
            (region_guard.center[0] * 2.0) as u32,
            (region_guard.center[1] * 2.0) as u32
        );
        drop(region_guard);

        let mut window: Window = create_window(width, height);
        let mut app = App::new(OpenGL::V3_2, self.clone(), region_id);
        let mut events = Events::new(EventSettings::new());
        let mut mouse_xy = [0.0, 0.0];

        while let Some(e) = events.next(&mut window) {
            if let Some(Button::Mouse(_button)) = e.press_args() {
                app.click(mouse_xy);
            }

            if let Some(args) = e.render_args() {
                app.render(&args);
            }

            if let Some(args) = e.update_args() {
                app.update(&args);
            }

            e.mouse_cursor(|pos| {
                mouse_xy = pos;
            });
        }

        Ok(())
    }
}