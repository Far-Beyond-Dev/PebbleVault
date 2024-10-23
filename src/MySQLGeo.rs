//! MySQLGeo: A module for persistent storage of spatial data.
//!
//! This module provides a `Database` struct for interacting with a SQLite database
//! to store and retrieve spatial data points. It also handles file-based storage
//! for larger data objects associated with each point.

use rusqlite::{params, Connection, Result as SqlResult};
use serde_json::{self, Value};
use serde::{Serialize, Deserialize};
use std::fs;
use uuid::Uuid;

/// Represents a spatial point with associated data.
#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Point {
    /// Unique identifier for the point
    pub id: Option<Uuid>,
    /// X-coordinate
    pub x: f64,
    /// Y-coordinate
    pub y: f64,
    /// Z-coordinate
    pub z: f64,
    /// Object type
    pub object_type: String,
    /// Custom data associated with the point
    pub custom_data: Value,
}

/// Represents a region in the spatial database.
#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Region {
    /// Unique identifier for the region
    pub id: Uuid,
    /// Center coordinates of the region [x, y, z]
    pub center: [f64; 3],
    /// Radius of the region
    pub radius: f64,
}

/// Manages the connection to the SQLite database and provides methods for data manipulation.
pub struct Database {
    conn: Connection,
}

impl Point {
    /// Creates a new Point instance.
    ///
    /// # Arguments
    ///
    /// * `id` - Optional UUID for the point.
    /// * `x` - X-coordinate of the point.
    /// * `y` - Y-coordinate of the point.
    /// * `z` - Z-coordinate of the point.
    /// * `object_type` - Object type of the point.
    /// * `custom_data` - Custom data associated with the point.
    ///
    /// # Returns
    ///
    /// A new Point instance.
    ///
    /// # Examples
    ///
    /// ```
    /// let point = Point::new(Some(Uuid::new_v4()), 1.0, 2.0, 3.0, "Example Type".to_string(), json!({"name": "Example Point"}));
    /// ```
    pub fn new(id: Option<Uuid>, x: f64, y: f64, z: f64, object_type: String, custom_data: Value) -> Self {
        Point { id, x, y, z, object_type, custom_data }
    }
}

impl Database {
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
        Ok(Database { conn })
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
                radius REAL NOT NULL
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
            "INSERT OR REPLACE INTO points (id, x, y, z, dataFile, region_id, object_type) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![id, point.x, point.y, point.z, &file_path, region_id.to_string(), &point.object_type],
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
            "SELECT id, x, y, z, dataFile, object_type FROM points
             WHERE ((x - ?1) * (x - ?1) + (y - ?2) * (y - ?2) + (z - ?3) * (z - ?3)) <= ?4",
        )?;
        
        let points_iter = stmt.query_map(params![x1, y1, z1, radius_sq], |row| {
            let id: String = row.get(0)?;
            let x: f64 = row.get(1)?;
            let y: f64 = row.get(2)?;
            let z: f64 = row.get(3)?;
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
    /// * `radius` - Radius of the region.
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
    /// let radius = 100.0;
    /// db.create_region(region_id, center, radius).expect("Failed to create region");
    /// ```
    pub fn create_region(&self, region_id: Uuid, center: [f64; 3], radius: f64) -> SqlResult<()> {
        // Insert the region into the database
        self.conn.execute(
            "INSERT OR REPLACE INTO regions (id, center_x, center_y, center_z, radius) VALUES (?1, ?2, ?3, ?4, ?5)",
            params![region_id.to_string(), center[0], center[1], center[2], radius],
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
            "SELECT id, center_x, center_y, center_z, radius FROM regions",
        )?;
        
        let regions_iter = stmt.query_map([], |row| {
            let id: String = row.get(0)?;
            let center_x: f64 = row.get(1)?;
            let center_y: f64 = row.get(2)?;
            let center_z: f64 = row.get(3)?;
            let radius: f64 = row.get(4)?;
            
            Ok(Region {
                id: Uuid::parse_str(&id).unwrap(),
                center: [center_x, center_y, center_z],
                radius,
            })
        })?;
        
        let mut regions = Vec::new();
        for region in regions_iter {
            let region = region?;
            println!("Retrieved region: ID: {}, Center: {:?}, Radius: {}", region.id, region.center, region.radius);
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
            "SELECT id, x, y, z, dataFile, object_type FROM points WHERE region_id = ?1",
        )?;
        
        let points_iter = stmt.query_map(params![region_id.to_string()], |row| {
            let id: String = row.get(0)?;
            let x: f64 = row.get(1)?;
            let y: f64 = row.get(2)?;
            let z: f64 = row.get(3)?;
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