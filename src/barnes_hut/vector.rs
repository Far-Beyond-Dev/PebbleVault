use crate::structs::Vector3D;

/// Implements operations for 3D vectors used in the Barnes-Hut simulation.
impl Vector3D {
    /// Creates a new `Vector3D` with the given x, y, and z components.
    ///
    /// # Arguments
    ///
    /// * `x` - The x-component of the vector.
    /// * `y` - The y-component of the vector.
    /// * `z` - The z-component of the vector.
    ///
    /// # Returns
    ///
    /// A new `Vector3D` instance.
    pub fn new(x: f64, y: f64, z: f64) -> Self {
        Self { x, y, z }
    }

    /// Creates a new `Vector3D` with all components set to zero.
    ///
    /// # Returns
    ///
    /// A new `Vector3D` instance representing the zero vector.
    pub fn zero() -> Self {
        Self::new(0.0, 0.0, 0.0)
    }

    /// Adds another vector to this vector.
    ///
    /// # Arguments
    ///
    /// * `other` - The `Vector3D` to add to this vector.
    ///
    /// # Returns
    ///
    /// A new `Vector3D` instance representing the sum of the two vectors.
    pub fn add(&self, other: &Vector3D) -> Vector3D {
        Vector3D::new(self.x + other.x, self.y + other.y, self.z + other.z)
    }

    /// Subtracts another vector from this vector.
    ///
    /// # Arguments
    ///
    /// * `other` - The `Vector3D` to subtract from this vector.
    ///
    /// # Returns
    ///
    /// A new `Vector3D` instance representing the difference between the two vectors.
    pub fn sub(&self, other: &Vector3D) -> Vector3D {
        Vector3D::new(self.x - other.x, self.y - other.y, self.z - other.z)
    }

    /// Multiplies this vector by a scalar value.
    ///
    /// # Arguments
    ///
    /// * `scalar` - The scalar value to multiply the vector by.
    ///
    /// # Returns
    ///
    /// A new `Vector3D` instance representing the scaled vector.
    pub fn mul(&self, scalar: f64) -> Vector3D {
        Vector3D::new(self.x * scalar, self.y * scalar, self.z * scalar)
    }

    /// Calculates the squared length (magnitude) of this vector.
    ///
    /// This method is often used instead of `length()` when comparing distances,
    /// as it avoids the computationally expensive square root operation.
    ///
    /// # Returns
    ///
    /// The squared length of the vector as an f64.
    pub fn length_squared(&self) -> f64 {
        self.x * self.x + self.y * self.y + self.z * self.z
    }

    /// Calculates the length (magnitude) of this vector.
    ///
    /// # Returns
    ///
    /// The length of the vector as an f64.
    pub fn length(&self) -> f64 {
        self.length_squared().sqrt()
    }
}