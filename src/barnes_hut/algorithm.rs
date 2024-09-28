use crate::structs::{BarnesHutSimulation, Body, Vector3D, BarnesHutConfig};
use super::octree::OctreeNode;

/// Implements the Barnes-Hut algorithm for N-body simulations.
impl BarnesHutSimulation {
    /// Creates a new Barnes-Hut simulation with the given bodies and configuration.
    ///
    /// # Arguments
    ///
    /// * `bodies` - A vector of `Body` objects representing the particles in the simulation.
    /// * `config` - A `BarnesHutConfig` object containing simulation parameters.
    ///
    /// # Returns
    ///
    /// A new `BarnesHutSimulation` instance.
    pub fn new(bodies: Vec<Body>, config: BarnesHutConfig, octree_size: f64) -> Self {
        Self { bodies, config, octree_size }
    }

    /// Advances the simulation by one time step.
    ///
    /// This method performs three main steps:
    /// 1. Builds the octree
    /// 2. Calculates forces on each body
    /// 3. Updates positions and velocities of bodies
    pub fn step(&mut self) {
        let root = self.build_tree();
        self.calculate_forces(&root);
        self.update_positions();
    }

    /// Constructs the octree for the current state of the simulation.
    ///
    /// # Returns
    ///
    /// The root `OctreeNode` of the constructed tree.
    fn build_tree(&self) -> OctreeNode {
        let center = Vector3D::new(0.0, 0.0, 0.0);  // Assume the center of the universe is at (0,0,0)
        let size = self.octree_size;

        let mut root = OctreeNode::new(center, size);
        for body in &self.bodies {
            root.insert(body.clone());
        }
        root
    }

    /// Calculates the bounding box that contains all bodies in the simulation.
    ///
    /// # Returns
    ///
    /// A tuple of two `Vector3D` objects representing the minimum and maximum corners of the bounding box.
    fn bounding_box(&self) -> (Vector3D, Vector3D) {
        let mut min = Vector3D::new(f64::MAX, f64::MAX, f64::MAX);
        let mut max = Vector3D::new(f64::MIN, f64::MIN, f64::MIN);

        for body in &self.bodies {
            min.x = min.x.min(body.position.x);
            min.y = min.y.min(body.position.y);
            min.z = min.z.min(body.position.z);
            max.x = max.x.max(body.position.x);
            max.y = max.y.max(body.position.y);
            max.z = max.z.max(body.position.z);
        }

        (min, max)
    }

    /// Calculates the forces acting on all bodies in the simulation using the Barnes-Hut approximation.
    ///
    /// # Arguments
    ///
    /// * `root` - A reference to the root `OctreeNode` of the constructed tree.
    fn calculate_forces(&mut self, root: &OctreeNode) {
        let forces: Vec<Vector3D> = self.bodies.iter().map(|body| {
            self.calculate_force_for_body(body, root)
        }).collect();

        // Update forces for all bodies
        for (body, force) in self.bodies.iter_mut().zip(forces.iter()) {
            body.force = *force;
        }
    }

    /// Recursively calculates the force on a single body using the Barnes-Hut approximation.
    ///
    /// # Arguments
    ///
    /// * `body` - A reference to the `Body` for which to calculate the force.
    /// * `node` - A reference to the current `OctreeNode` being evaluated.
    ///
    /// # Returns
    ///
    /// The calculated force as a `Vector3D`.
    fn calculate_force_for_body(&self, body: &Body, node: &OctreeNode) -> Vector3D {
        let mut force = Vector3D::zero();

        if let Some(ref node_body) = node.body {
            if node_body.position != body.position {
                let r = node_body.position.sub(&body.position);
                let r_squared = r.length_squared();
                force = r.mul(node_body.mass / (r_squared * r_squared.sqrt() + self.config.epsilon));
            }
        } else if node.children.is_some() {
            let s = node.size;
            let d = node.com.sub(&body.position).length();
            if s / d < self.config.theta {
                let r = node.com.sub(&body.position);
                let r_squared = r.length_squared();
                force = r.mul(node.mass / (r_squared * r_squared.sqrt() + self.config.epsilon));
            } else {
                for child in node.children.as_ref().unwrap().iter() {
                    force = force.add(&self.calculate_force_for_body(body, child));
                }
            }
        }

        force
    }

    /// Updates the positions and velocities of all bodies based on the calculated forces.
    fn update_positions(&mut self) {
        for body in &mut self.bodies {
            body.velocity = body.velocity.add(&body.force.mul(self.config.dt));
            body.position = body.position.add(&body.velocity.mul(self.config.dt));
        }
    }
}