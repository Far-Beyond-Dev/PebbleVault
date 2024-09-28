use crate::structs::{Vector3D, Body};

/// Represents a node in the octree used for the Barnes-Hut algorithm.
pub struct OctreeNode {
    /// The center point of this node's cube in 3D space.
    pub center: Vector3D,
    /// The side length of this node's cube.
    pub size: f64,
    /// The total mass of all bodies contained within this node and its children.
    pub mass: f64,
    /// The center of mass of all bodies contained within this node and its children.
    pub com: Vector3D,
    /// The child nodes of this node, if any. There are always either 0 or 8 children.
    pub children: Option<Box<[OctreeNode; 8]>>,
    /// The body contained in this node, if it's a leaf node containing exactly one body.
    pub body: Option<Body>,
}

impl OctreeNode {
    /// Creates a new `OctreeNode` with the given center and size.
    ///
    /// # Arguments
    ///
    /// * `center` - The center point of the node's cube.
    /// * `size` - The side length of the node's cube.
    ///
    /// # Returns
    ///
    /// A new `OctreeNode` instance.
    pub fn new(center: Vector3D, size: f64) -> Self {
        Self {
            center,
            size,
            mass: 0.0,
            com: Vector3D::zero(),
            children: None,
            body: None,
        }
    }

    /// Inserts a body into the octree, subdividing nodes as necessary.
    ///
    /// # Arguments
    ///
    /// * `body` - The `Body` to insert into the octree.
    pub fn insert(&mut self, body: Body) {
        if self.body.is_none() && self.children.is_none() {
            self.body = Some(body.clone());
            self.mass = body.mass;
            self.com = body.position;
            return;
        }

        if self.children.is_none() {
            self.subdivide();
            if let Some(existing_body) = self.body.take() {
                self.insert_into_children(existing_body);
            }
        }

        self.insert_into_children(body);
    }

    fn insert_into_children(&mut self, body: Body) {
        let octant = self.get_octant(&body.position);
        if let Some(children) = &mut self.children {
            children[octant].insert(body.clone());
        }

        // Update mass and center of mass
        let new_mass = self.mass + body.mass;
        self.com = self.com.mul(self.mass).add(&body.position.mul(body.mass)).mul(1.0 / new_mass);
        self.mass = new_mass;
    }

    /// Subdivides this node into 8 child nodes.
    fn subdivide(&mut self) {
        let new_size = self.size / 2.0;
        let offset = new_size / 2.0;

        self.children = Some(Box::new([
            OctreeNode::new(self.center.add(&Vector3D::new(-offset, -offset, -offset)), new_size),
            OctreeNode::new(self.center.add(&Vector3D::new(offset, -offset, -offset)), new_size),
            OctreeNode::new(self.center.add(&Vector3D::new(-offset, offset, -offset)), new_size),
            OctreeNode::new(self.center.add(&Vector3D::new(offset, offset, -offset)), new_size),
            OctreeNode::new(self.center.add(&Vector3D::new(-offset, -offset, offset)), new_size),
            OctreeNode::new(self.center.add(&Vector3D::new(offset, -offset, offset)), new_size),
            OctreeNode::new(self.center.add(&Vector3D::new(-offset, offset, offset)), new_size),
            OctreeNode::new(self.center.add(&Vector3D::new(offset, offset, offset)), new_size),
        ]));
    }

    /// Determines which octant of this node a given position belongs to.
    ///
    /// # Arguments
    ///
    /// * `position` - The `Vector3D` position to check.
    ///
    /// # Returns
    ///
    /// An index from 0 to 7 representing the octant.
    fn get_octant(&self, position: &Vector3D) -> usize {
        let mut index = 0;
        if position.x >= self.center.x { index |= 1; }
        if position.y >= self.center.y { index |= 2; }
        if position.z >= self.center.z { index |= 4; }
        index
    }

    /// Updates the mass and center of mass of this node after inserting a new body.
    ///
    /// # Arguments
    ///
    /// * `body` - The `Body` that was just inserted.
    fn update_mass_and_com(&mut self, body: &Body) {
        let new_mass = self.mass + body.mass;
        self.com = self.com.mul(self.mass).add(&body.position.mul(body.mass)).mul(1.0 / new_mass);
        self.mass = new_mass;
    }
}