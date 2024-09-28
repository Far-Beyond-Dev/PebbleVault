//! # PebbleVault Structures
//!
//! This module defines the core structures used in the PebbleVault spatial database system
//! and the Barnes-Hut simulation for N-body problems.
//!
//! It includes implementations for:
//! - Spatial objects and regions for the database system
//! - Structures for the Barnes-Hut algorithm implementation

use rstar::*;
use std::sync::{Arc, Mutex};
use serde::{Serialize, Deserialize};
use uuid::Uuid;

/// Represents a spatial object in the game world.
///
/// This struct implements the necessary traits for use with the R-tree spatial index.
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
    /// Calculates the squared Euclidean distance between this object and a given point.
    ///
    /// # Arguments
    ///
    /// * `point` - A reference to a 3D point [x, y, z]
    ///
    /// # Returns
    ///
    /// The squared Euclidean distance as an f64.
    fn distance_2(&self, point: &[f64; 3]) -> f64 {
        let dx = self.point[0] - point[0];
        let dy = self.point[1] - point[1];
        let dz = self.point[2] - point[2];
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
    fn envelope(&self) -> Self::Envelope {
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

/// Represents a region in the game world for the BarnesHutManager.
pub struct BarnesHutRegion {
    /// Unique identifier for the region
    pub id: Uuid,
    /// Center coordinates of the region [x, y, z]
    pub center: [f64; 3],
    /// Radius of the region
    pub radius: f64,
    /// Spatial index (RTree) for objects in this region
    pub rtree: RTree<SpatialObject>,
    /// Barnes-Hut simulation for this region
    pub simulation: Option<Arc<Mutex<BarnesHutSimulation>>>,
}

// Add these new structs for Barnes-Hut implementation

/// Represents a 3D vector.
///
/// Used in the Barnes-Hut simulation for positions, velocities, and forces.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Vector3D {
    /// X-component of the vector
    pub x: f64,
    /// Y-component of the vector
    pub y: f64,
    /// Z-component of the vector
    pub z: f64,
}

/// Represents a body in the Barnes-Hut simulation.
///
/// Each body has mass, position, velocity, and accumulated force.
#[derive(Debug, Clone)]
pub struct Body {
    /// Mass of the body
    pub mass: f64,
    /// Position of the body in 3D space
    pub position: Vector3D,
    /// Velocity of the body
    pub velocity: Vector3D,
    /// Accumulated force acting on the body
    pub force: Vector3D,
}

/// Configuration for the Barnes-Hut simulation.
///
/// Contains parameters that control the behavior and accuracy of the simulation.
pub struct BarnesHutConfig {
    /// Threshold for the multipole acceptance criterion
    pub theta: f64,
    /// Time step for the simulation
    pub dt: f64,
    /// Softening parameter to prevent division by zero
    pub epsilon: f64,
}

/// Represents the Barnes-Hut simulation.
///
/// Contains the bodies being simulated and the configuration parameters.
pub struct BarnesHutSimulation {
    /// Vector of bodies in the simulation
    pub bodies: Vec<Body>,
    /// Configuration parameters for the simulation
    pub config: BarnesHutConfig,
    /// Octree size for the simulation
    pub octree_size: f64,
}

/// Configuration for the octree and Barnes-Hut simulation.
#[derive(Deserialize)]
pub struct PebbleVaultConfig {
    pub octree: OctreeConfig,
}

/// Configuration for the octree.
#[derive(Deserialize)]
pub struct OctreeConfig {
    pub parent_node_size: f64,
}

// Implement Clone for BarnesHutConfig
impl Clone for BarnesHutConfig {
    fn clone(&self) -> Self {
        Self {
            theta: self.theta,
            dt: self.dt,
            epsilon: self.epsilon,
        }
    }
}
