//! # PebbleVault Structures
//!
//! This module defines the core structures used in the PebbleVault spatial database system.
//! It provides the fundamental building blocks for representing spatial objects and regions
//! in a flexible and efficient manner.
//!
//! ## Key Components
//!
//! - `SpatialObject`: Represents individual entities in the spatial database.
//! - `VaultRegion`: Defines a spatial region containing multiple objects.
//!
//! ## Features
//!
//! - **Generic Custom Data**: Allows associating arbitrary data with spatial objects.
//! - **Efficient Spatial Indexing**: Implements traits for use with R-tree spatial indexing.
//! - **Serialization Support**: Enables easy persistence and network transmission of spatial data.
//!
//! ## Usage Example
//!
//! ```rust
//! use uuid::Uuid;
//! use std::sync::Arc;
//! use serde::{Serialize, Deserialize};
//! use your_crate::{SpatialObject, VaultRegion};
//! use rstar::RTree;
//!
//! #[derive(Clone, Serialize, Deserialize, PartialEq)]
//! struct PlayerData {
//!     name: String,
//!     level: u32,
//! }
//!
//! let player = SpatialObject {
//!     uuid: Uuid::new_v4(),
//!     object_type: "player".to_string(),
//!     point: [1.0, 2.0, 3.0],
//!     custom_data: Arc::new(PlayerData { name: "Alice".to_string(), level: 5 }),
//! };
//!
//! let region = VaultRegion {
//!     id: Uuid::new_v4(),
//!     center: [0.0, 0.0, 0.0],
//!     size: 100.0,
//!     rtree: RTree::new(),
//! };
//! ```

use rstar::*;
use std::sync::Arc;
use serde::{Serialize, Deserialize};
use uuid::Uuid;

/// Represents a spatial object in the game world.
///
/// This struct is the core component for representing entities in the spatial database.
/// It combines spatial information with custom data, allowing for flexible use in various
/// game or simulation scenarios.
///
/// # Type Parameters
///
/// * `T`: The type of custom data associated with the object. This can be any type that
///        implements `Clone`, `Serialize`, `Deserialize`, and `PartialEq`.
///
/// # Fields
///
/// * `uuid`: Unique identifier for the object.
/// * `object_type`: String describing the type of the object (e.g., "player", "building").
/// * `point`: 3D coordinates of the object [x, y, z].
/// * `custom_data`: Reference-counted pointer to associated custom data.
///
/// # Examples
///
/// ```rust
/// use uuid::Uuid;
/// use std::sync::Arc;
/// use serde::{Serialize, Deserialize};
/// use your_crate::SpatialObject;
///
/// #[derive(Clone, Serialize, Deserialize, PartialEq)]
/// struct PlayerData {
///     name: String,
///     level: u32,
/// }
///
/// let player = SpatialObject {
///     uuid: Uuid::new_v4(),
///     object_type: "player".to_string(),
///     point: [1.0, 2.0, 3.0],
///     custom_data: Arc::new(PlayerData { name: "Alice".to_string(), level: 5 }),
/// };
///
/// let resource = SpatialObject {
///     uuid: Uuid::new_v4(),
///     object_type: "resource".to_string(),
///     point: [4.0, 5.0, 6.0],
///     custom_data: Arc::new("Gold Ore".to_string()),
/// };
/// ```
#[derive(Clone, PartialEq, Debug)]
pub struct SpatialObject<T: Clone + Serialize + for<'de> Deserialize<'de> + PartialEq + Sized> {
    /// Unique identifier for the object
    pub uuid: Uuid,
    /// Type of the object (e.g., "player", "building", "resource")
    pub object_type: String,
    /// 3D coordinates of the object [x, y, z]
    pub point: [f64; 3],
    /// Reference-counted pointer to custom data associated with the object
    pub custom_data: Arc<T>,
}

impl<T: Clone + Serialize + for<'de> Deserialize<'de> + PartialEq + Sized> PointDistance for SpatialObject<T> {
    /// Calculates the squared Euclidean distance between this object and a given point.
    ///
    /// This method is crucial for spatial operations and queries within the R-tree.
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
    /// # use uuid::Uuid;
    /// # use std::sync::Arc;
    /// # use your_crate::{SpatialObject, PointDistance};
    /// let object = SpatialObject {
    ///     uuid: Uuid::new_v4(),
    ///     object_type: "player".to_string(),
    ///     point: [1.0, 2.0, 3.0],
    ///     custom_data: Arc::new("Example object".to_string()),
    /// };
    /// let distance = object.distance_2(&[4.0, 5.0, 6.0]);
    /// assert_eq!(distance, 27.0);
    /// ```
    fn distance_2(&self, point: &[f64; 3]) -> f64 {
        let dx = self.point[0] - point[0];
        let dy = self.point[1] - point[1];
        let dz = self.point[2] - point[2];
        dx * dx + dy * dy + dz * dz
    }
}

impl<T: Clone + Serialize + for<'de> Deserialize<'de> + PartialEq + Sized> RTreeObject for SpatialObject<T> {
    type Envelope = AABB<[f64; 3]>;

    /// Creates an Axis-Aligned Bounding Box (AABB) envelope for this object.
    ///
    /// This method is used by the R-tree for efficient spatial indexing and querying.
    ///
    /// # Returns
    ///
    /// An AABB representing the envelope of this object.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use uuid::Uuid;
    /// # use std::sync::Arc;
    /// # use your_crate::{SpatialObject, RTreeObject};
    /// let object = SpatialObject {
    ///     uuid: Uuid::new_v4(),
    ///     object_type: "player".to_string(),
    ///     point: [1.0, 2.0, 3.0],
    ///     custom_data: Arc::new("Example object".to_string()),
    /// };
    /// let envelope = object.envelope();
    /// assert_eq!(envelope.lower(), [1.0, 2.0, 3.0]);
    /// assert_eq!(envelope.upper(), [1.0, 2.0, 3.0]);
    /// ```
    fn envelope(&self) -> Self::Envelope {
        AABB::from_point(self.point)
    }
}

/// Represents a region in the game world for the VaultManager.
///
/// This struct defines a cubic spatial partition containing multiple `SpatialObject`s.
/// It uses an R-tree for efficient spatial indexing and querying of objects within the region.
///
/// # Type Parameters
///
/// * `T`: The type of custom data associated with the `SpatialObject`s in this region.
///        Must implement `Clone`, `Serialize`, `Deserialize`, and `PartialEq`.
///
/// # Fields
///
/// * `id`: Unique identifier for the region.
/// * `center`: 3D coordinates of the region's center [x, y, z].
/// * `size`: Length of each side of the cubic region.
/// * `rtree`: Spatial index (RTree) for objects in this region.
///
/// # Examples
///
/// ```rust
/// use uuid::Uuid;
/// use rstar::RTree;
/// use pebblevault::{VaultRegion, SpatialObject};
///
/// // Define a custom data type for your spatial objects
/// #[derive(Clone, Serialize, Deserialize, PartialEq)]
/// struct CustomData {
///     name: String,
///     value: i32,
/// }
///
/// let region: VaultRegion<CustomData> = VaultRegion {
///     id: Uuid::new_v4(),
///     center: [0.0, 0.0, 0.0],
///     size: 100.0,  // Creates a 100x100x100 cubic region
///     rtree: RTree::new(),
/// };
/// ```
///
/// Note that the custom data type `T` is associated with the `SpatialObject`s
/// that will be stored in this region, not with the region itself.
#[derive(Debug)]
pub struct VaultRegion<T: Clone + Serialize + for<'de> Deserialize<'de> + PartialEq + Sized> {
    /// Unique identifier for the region
    pub id: Uuid,
    /// Center coordinates of the region [x, y, z]
    pub center: [f64; 3],
    /// Length of each side of the cubic region
    pub size: f64,
    /// Spatial index (RTree) for objects in this region
    pub rtree: RTree<SpatialObject<T>>,
}