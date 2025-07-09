//! MySQLGeo: A module for persistent storage of spatial data.
//!
//! This module provides a `Database` struct for interacting with a SQLite database
//! to store and retrieve spatial data points. It also handles file-based storage
//! for larger data objects associated with each point.

use serde_json::{self, Value};
use serde::{Serialize, Deserialize};
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
    /// Width of object (along X axis)
    pub size_x: f64,
    /// Width of object (along Y axis)
    pub size_y: f64,
    /// Width of object (along Z axis)
    pub size_z: f64,
    /// Object type
    pub object_type: String,
    /// Custom data associated with the point
    pub custom_data: Value,
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
    /// * `size_x` - Width of the object along X axis.
    /// * `size_x` - Width of the object along X axis.
    /// * `size_x` - Width of the object along X axis.
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
    pub fn new(
        id: Option<Uuid>,
        x: f64,
        y: f64,
        z: f64,
        size_x: f64,
        size_y: f64,
        size_z: f64,
        object_type: String,
        custom_data: Value,
    ) -> Self {
        Point {
            id,
            x,
            y,
            z,
            size_x,
            size_y,
            size_z,
            object_type,
            custom_data,
        }
    }
}

/// Represents a region in the spatial database.
#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Region {
    /// Unique identifier for the region
    pub id: Uuid,
    /// Center coordinates of the region [x, y, z]
    pub center: [f64; 3],
    /// Length of each side of the cubic region
    pub size: f64,
}