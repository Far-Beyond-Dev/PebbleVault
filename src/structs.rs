//! # PebbleVault Structures
//!
//! This module defines the core structures used in the PebbleVault spatial database system

use rstar::*;
use std::sync::{Arc, Mutex};
use serde::{Serialize, Deserialize};
use uuid::Uuid;

/// Represents a spatial object in the game world.
#[derive(Clone, Serialize, Deserialize, PartialEq)]
pub struct SpatialObject {
    /// Unique identifier for the object
    pub uuid: Uuid,
    /// Type of the object (e.g., player, building, resource)
    pub object_type: &'static str,
    /// Associated data with the object (e.g., player info, item details)
    pub data: String,
    /// 3D coordinates of the object [x, y, z]
    pub point: [f64; 3],
}

impl PointDistance for SpatialObject {
    /// Calculates the squared Euclidean distance between this object and a given point.
    ///
    /// # Arguments
    ///
    /// * `point` - A reference to a 3D point [x, y, z]
    ///
    /// # Returns
    ///
    /// The squared Euclidean distance as an f64.
    ///
    /// # Examples
    ///
    /// ```rust
    /// let object = SpatialObject {
    ///     uuid: Uuid::new_v4(),
    ///     object_type: "player",
    ///     data: "Example object".to_string(),
    ///     point: [1.0, 2.0, 3.0],
    /// };
    /// let distance = object.distance_2(&[4.0, 5.0, 6.0]);
    /// assert_eq!(distance, 27.0);
    /// ```
    fn distance_2(&self, point: &[f64; 3]) -> f64 {
        // Calculate the difference in each dimension
        let dx = self.point[0] - point[0];
        let dy = self.point[1] - point[1];
        let dz = self.point[2] - point[2];
        // Return the sum of squared differences
        dx * dx + dy * dy + dz * dz
    }
}

impl RTreeObject for SpatialObject {
    type Envelope = AABB<[f64; 3]>;

    /// Creates an Axis-Aligned Bounding Box (AABB) envelope for this object.
    ///
    /// # Returns
    ///
    /// An AABB representing the envelope of this object.
    ///
    /// # Examples
    ///
    /// ```rust
    /// let object = SpatialObject {
    ///     uuid: Uuid::new_v4(),
    ///     object_type: "player",
    ///     data: "Example object".to_string(),
    ///     point: [1.0, 2.0, 3.0],
    /// };
    /// let envelope = object.envelope();
    /// assert_eq!(envelope.lower(), [1.0, 2.0, 3.0]);
    /// assert_eq!(envelope.upper(), [1.0, 2.0, 3.0]);
    /// ```
    fn envelope(&self) -> Self::Envelope {
        // Create an AABB from the object's point
        AABB::from_point(self.point)
    }
}

/// Represents a region in the game world for the VaultManager.
pub struct VaultRegion {
    /// Unique identifier for the region
    pub id: Uuid,
    /// Center coordinates of the region [x, y, z]
    pub center: [f64; 3],
    /// Radius of the region
    pub radius: f64,
    /// Spatial index (RTree) for objects in this region
    pub rtree: RTree<SpatialObject>,
}
