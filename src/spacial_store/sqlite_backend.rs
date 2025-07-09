use rusqlite::{params, Connection, Result as SqlResult};
use serde_json::{self, Value};
use std::fs;
use uuid::Uuid;
use anyhow::Result;
use crate::spacial_store::types::{Point, Region};
use crate::spacial_store::backend::PersistenceBackend;

/// Manages the connection to the SQLite database and provides methods for data manipulation.
#[derive(Debug)]
pub struct SqliteDatabase {
    conn: Connection,
}

impl SqliteDatabase {
    /// Creates a new Database instance.
    ///
    /// # Arguments
    ///
    /// * `db_path` - Path to the SQLite database file.
    ///
    /// # Returns
    ///
    /// A Result containing a new Database instance or a SQLite error.
    ///
    /// # Examples
    ///
    /// ```
    /// let db = Database::new("path/to/database.sqlite").expect("Failed to create database");
    /// ```
    pub fn new(db_path: &str) -> SqlResult<Self> {
        // Open a connection to the SQLite database
        let conn = Connection::open(db_path)?;
        Ok(SqliteDatabase { conn })
    }

    /// Creates the necessary tables in the database if they don't exist.
    ///
    /// # Returns
    ///
    /// A Result indicating success or a SQLite error.
    ///
    /// # Examples
    ///
    /// ```
    /// db.create_table().expect("Failed to create tables");
    /// ```
    pub fn create_table(&self) -> SqlResult<()> {
        // Create points table
        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS points (
                id TEXT PRIMARY KEY,
                x REAL NOT NULL,
                y REAL NOT NULL,
                z REAL NOT NULL,
                dataFile TEXT NOT NULL,
                region_id TEXT,
                object_type TEXT NOT NULL
            )",
            [],
        )?;
        // Create regions table
        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS regions (
                id TEXT PRIMARY KEY,
                center_x REAL NOT NULL,
                center_y REAL NOT NULL,
                center_z REAL NOT NULL,
                size REAL NOT NULL
            )",
            [],
        )?;
        Ok(())
    }

    /// Adds a point to the database and stores its data in a file.
    ///
    /// # Arguments
    ///
    /// * `point` - The Point to be added.
    /// * `region_id` - UUID of the region to which the point belongs.
    ///
    /// # Returns
    ///
    /// A Result indicating success or an error.
    ///
    /// # Examples
    ///
    /// ```
    /// use uuid::Uuid;
    /// 
    /// let point = Point::new(Some(Uuid::new_v4()), 1.0, 2.0, 3.0, "Example Type".to_string(), json!({"name": "Example Point"}));
    /// let region_id = Uuid::new_v4();
    /// db.add_point(&point, region_id).expect("Failed to add point");
    /// ```
    pub fn add_point(&self, point: &Point, region_id: Uuid) -> SqlResult<()> {
        let id = point.id.unwrap_or_else(Uuid::new_v4).to_string();
        let custom_data_str = serde_json::to_string(&point.custom_data)
            .map_err(|err| rusqlite::Error::ToSqlConversionFailure(Box::new(err)))?;

        let folder_name: String = id.chars().take(2).collect();
        let file_path: String = format!("./data/{}/{}", folder_name, id);

        fs::create_dir_all(format!("./data/{}", folder_name))
            .map_err(|err| rusqlite::Error::ToSqlConversionFailure(Box::new(err)))?;

        fs::write(&file_path, &custom_data_str)
            .map_err(|err| rusqlite::Error::ToSqlConversionFailure(Box::new(err)))?;

        self.conn.execute(
            "INSERT OR REPLACE INTO points (id, x, y, z, dataFile, region_id, object_type, sizeX, sizeY, sizeZ)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
            params![
                id,
                point.x,
                point.y,
                point.z,
                &file_path,
                region_id.to_string(),
                &point.object_type,
                point.size_x,
                point.size_y,
                point.size_z,
            ],
        )?;
        
        Ok(())
    }

    /// Retrieves points within a specified radius from a given center point.
    ///
    /// # Arguments
    ///
    /// * `x1` - X-coordinate of the center point.
    /// * `y1` - Y-coordinate of the center point.
    /// * `z1` - Z-coordinate of the center point.
    /// * `radius` - The radius within which to search for points.
    ///
    /// # Returns
    ///
    /// A Result containing a vector of Points within the specified radius, or an error.
    ///
    /// # Examples
    ///
    /// ```
    /// let points = db.get_points_within_radius(0.0, 0.0, 0.0, 10.0).expect("Failed to get points");
    /// for point in points {
    ///     println!("Found point: {:?}", point);
    /// }
    /// ```
    pub fn get_points_within_radius(&self, x1: f64, y1: f64, z1: f64, radius: f64) -> SqlResult<Vec<Point>> {
        let radius_sq = radius * radius;
        let mut stmt = self.conn.prepare(
            "SELECT id, x, y, z, dataFile, object_type, sizeX, sizeY, sizeZ FROM points
            WHERE ((x - ?1) * (x - ?1) + (y - ?2) * (y - ?2) + (z - ?3) * (z - ?3)) <= ?4",
        )?;
        
        let points_iter = stmt.query_map(params![x1, y1, z1, radius_sq], |row| {
            let id: String = row.get(0)?;
            let x: f64 = row.get(1)?;
            let y: f64 = row.get(2)?;
            let z: f64 = row.get(3)?;
            let size_x: f64 = row.get(6)?;
            let size_y: f64 = row.get(7)?;
            let size_z: f64 = row.get(8)?;
            let data_file: String = row.get(4)?;
            let object_type: String = row.get(5)?;
            
            let custom_data_str = fs::read_to_string(&data_file)
                .map_err(|err| rusqlite::Error::ToSqlConversionFailure(Box::new(err)))?;
            let custom_data: Value = serde_json::from_str(&custom_data_str)
                .map_err(|err| rusqlite::Error::ToSqlConversionFailure(Box::new(err)))?;
            
            Ok(Point {
                id: Some(Uuid::parse_str(&id).unwrap()),
                x,
                y,
                z,
                size_x,
                size_y,
                size_z,
                object_type,
                custom_data,
            })
        })?;
        
        let mut points = Vec::new();
        for point in points_iter {
            points.push(point?);
        }
        
        Ok(points)
    }

    /// Creates a new region in the database.
    ///
    /// # Arguments
    ///
    /// * `region_id` - UUID of the region to create.
    /// * `center` - Center coordinates of the region.
    /// * `size` - Length of each side of the cubic region.
    ///
    /// # Returns
    ///
    /// A Result indicating success or an error.
    ///
    /// # Examples
    ///
    /// ```
    /// let region_id = Uuid::new_v4();
    /// let center = [0.0, 0.0, 0.0];
    /// let size = 100.0;  // Creates a 100x100x100 cubic region
    /// db.create_region(region_id, center, size).expect("Failed to create region");
    /// ```
    pub fn create_region(&self, region_id: Uuid, center: [f64; 3], size: f64) -> SqlResult<()> {
        // Insert the region into the database
        self.conn.execute(
            "INSERT OR REPLACE INTO regions (id, center_x, center_y, center_z, size) VALUES (?1, ?2, ?3, ?4, ?5)",
            params![region_id.to_string(), center[0], center[1], center[2], size],
        )?;
        Ok(())
    }

    /// Removes a point from the database.
    ///
    /// # Arguments
    ///
    /// * `point_id` - UUID of the point to remove.
    ///
    /// # Returns
    ///
    /// A Result indicating success or an error.
    ///
    /// # Examples
    ///
    /// ```
    /// let point_id = Uuid::new_v4();
    /// db.remove_point(point_id).expect("Failed to remove point");
    /// ```
    pub fn remove_point(&self, point_id: Uuid) -> SqlResult<()> {
        // Delete the point from the database
        self.conn.execute(
            "DELETE FROM points WHERE id = ?1",
            params![point_id.to_string()],
        )?;
        Ok(())
    }

    /// Updates the position of a point in the database.
    ///
    /// # Arguments
    ///
    /// * `point_id` - UUID of the point to update.
    /// * `x` - New X-coordinate of the point.
    /// * `y` - New Y-coordinate of the point.
    /// * `z` - New Z-coordinate of the point.
    ///
    /// # Returns
    ///
    /// A Result indicating success or an error.
    ///
    /// # Examples
    ///
    /// ```
    /// let point_id = Uuid::new_v4();
    /// db.update_point_position(point_id, 4.0, 5.0, 6.0).expect("Failed to update point position");
    /// ```
    pub fn update_point_position(&self, point_id: Uuid, x: f64, y: f64, z: f64) -> SqlResult<()> {
        // Update the point's position in the database
        self.conn.execute(
            "UPDATE points SET x = ?1, y = ?2, z = ?3 WHERE id = ?4",
            params![x, y, z, point_id.to_string()],
        )?;
        Ok(())
    }

    /// Retrieves all regions from the database.
    ///
    /// # Returns
    ///
    /// A Result containing a vector of regions or an error.
    ///
    /// # Examples
    ///
    /// ```
    /// let regions = db.get_all_regions().expect("Failed to get regions");
    /// for region in regions {
    ///     println!("Region: {:?}", region);
    /// }
    /// ```
    pub fn get_all_regions(&self) -> SqlResult<Vec<Region>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, center_x, center_y, center_z, size FROM regions",
        )?;
        
        let regions_iter = stmt.query_map([], |row| {
            let id: String = row.get(0)?;
            let center_x: f64 = row.get(1)?;
            let center_y: f64 = row.get(2)?;
            let center_z: f64 = row.get(3)?;
            let size: f64 = row.get(4)?;
            
            Ok(Region {
                id: Uuid::parse_str(&id).unwrap(),
                center: [center_x, center_y, center_z],
                size,
            })
        })?;
        
        let mut regions = Vec::new();
        for region in regions_iter {
            let region = region?;
            println!("Retrieved region: ID: {}, Center: {:?}, Size: {}", region.id, region.center, region.size);
            regions.push(region);
        }
        
        println!("Total regions retrieved from database: {}", regions.len());
        Ok(regions)
    }

    /// Retrieves all points within a specified region from the database.
    ///
    /// # Arguments
    ///
    /// * `region_id` - UUID of the region to query.
    ///
    /// # Returns
    ///
    /// A Result containing a vector of points or an error.
    ///
    /// # Examples
    ///
    /// ```
    /// let region_id = Uuid::new_v4();
    /// let points = db.get_points_in_region(region_id).expect("Failed to get points in region");
    /// for point in points {
    ///     println!("Point in region: {:?}", point);
    /// }
    /// ```
    pub fn get_points_in_region(&self, region_id: Uuid) -> SqlResult<Vec<Point>> {
    let mut stmt = self.conn.prepare(
        "SELECT id, x, y, z, dataFile, object_type, sizeX, sizeY, sizeZ FROM points WHERE region_id = ?1",
    )?;
        
        let points_iter = stmt.query_map(params![region_id.to_string()], |row| {
            let id: String = row.get(0)?;
            let x: f64 = row.get(1)?;
            let y: f64 = row.get(2)?;
            let z: f64 = row.get(3)?;
            let size_x: f64 = row.get(6)?;
            let size_y: f64 = row.get(7)?;
            let size_z: f64 = row.get(8)?;
            let data_file: String = row.get(4)?;
            let object_type: String = row.get(5)?;
            
            let custom_data_str = fs::read_to_string(&data_file)
                .map_err(|err| rusqlite::Error::ToSqlConversionFailure(Box::new(err)))?;
            let custom_data: Value = serde_json::from_str(&custom_data_str)
                .map_err(|err| rusqlite::Error::ToSqlConversionFailure(Box::new(err)))?;
            
            Ok(Point {
                id: Some(Uuid::parse_str(&id).unwrap()),
                x,
                y,
                z,
                size_x,
                size_y,
                size_z,
                object_type,
                custom_data,
            })
        })?;
        
        let mut points = Vec::new();
        for point in points_iter {
            points.push(point?);
        }
        
        println!("Retrieved {} points for region {}", points.len(), region_id);
        Ok(points)
    }

    /// Clears all points from the database.
    ///
    /// # Returns
    ///
    /// A Result indicating success or an error.
    pub fn clear_all_points(&self) -> SqlResult<()> {
        self.conn.execute("DELETE FROM points", [])?;
        Ok(())
    }
}

impl PersistenceBackend for SqliteDatabase {
    fn create_table(&self) -> Result<()> {
        self.create_table().map_err(Into::into)
    }

    fn add_point(&self, point: &Point, region_id: Uuid) -> Result<()> {
        self.add_point(point, region_id).map_err(Into::into)
    }

    fn get_points_within_radius(&self, x: f64, y: f64, z: f64, radius: f64) -> Result<Vec<Point>> {
        self.get_points_within_radius(x, y, z, radius).map_err(Into::into)
    }

    fn create_region(&self, region_id: Uuid, center: [f64; 3], size: f64) -> Result<()> {
        self.create_region(region_id, center, size).map_err(Into::into)
    }

    fn remove_point(&self, point_id: Uuid) -> Result<()> {
        self.remove_point(point_id).map_err(Into::into)
    }

    fn update_point_position(&self, point_id: Uuid, x: f64, y: f64, z: f64) -> Result<()> {
        self.update_point_position(point_id, x, y, z).map_err(Into::into)
    }

    fn get_all_regions(&self) -> Result<Vec<Region>> {
        self.get_all_regions().map_err(Into::into)
    }

    fn get_points_in_region(&self, region_id: Uuid) -> Result<Vec<Point>> {
        self.get_points_in_region(region_id).map_err(Into::into)
    }

    fn clear_all_points(&self) -> Result<()> {
        self.clear_all_points().map_err(Into::into)
    }
}