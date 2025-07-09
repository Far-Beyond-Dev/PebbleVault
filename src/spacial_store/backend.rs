use uuid::Uuid;
use anyhow::Result;
use crate::spacial_store::types::{Point, Region};


pub trait PersistenceBackend: std::fmt::Debug {
    fn create_table(&self) -> Result<()>;
    fn add_point(&self, point: &Point, region_id: Uuid) -> Result<()>;
    fn get_points_within_radius(&self, x: f64, y: f64, z: f64, radius: f64) -> Result<Vec<Point>>;
    fn create_region(&self, region_id: Uuid, center: [f64; 3], size: f64) -> Result<()>;
    fn remove_point(&self, point_id: Uuid) -> Result<()>;
    fn update_point_position(&self, point_id: Uuid, x: f64, y: f64, z: f64) -> Result<()>;
    fn get_all_regions(&self) -> Result<Vec<Region>>;
    fn get_points_in_region(&self, region_id: Uuid) -> Result<Vec<Point>>;
    fn clear_all_points(&self) -> Result<()>;
}
