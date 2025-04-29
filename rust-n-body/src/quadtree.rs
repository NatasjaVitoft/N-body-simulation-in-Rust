use bevy::prelude::*;
use crate::Body;

pub const MIN_QUADRANT_LENGTH: f32 = 0.5; // How small a quandrant can be
pub const THETA_THRESHOLD: f32 = 0.5; // What is Theta?

#[derive(Debug, Default, Clone, Copy)] // More powerful and flexible
pub enum Corner {
    #[default]
    NW,
    NE,
    SW,
    SE,
}

#[derive(Clone, Debug, Default)] // Ekstra attributes
pub struct Quadrant {
    pub center: Vec2,
    pub len: f32,
}

impl Quadrant {
    
    // This functions creates a new quadrant and takes two parameters: Center which is a 2D point (x, y) and a 32-bit float number. 
    // f32 is not as precicse as f64 but is faster and fine for 2D
    pub fn new(center: Vec2, length: f32) -> Self {
        Self { center, len: length } // Then returns self with those values
    }

    // Checking if the position of the point is inside the quadrant 
    pub fn contains(&self, pos: Vec2) -> bool {
        let half_len = self.len / 2.0;
        (pos.x >= self.center.x - half_len)
            && (pos.x <= self.center.x + half_len) 
            && (pos.y >= self.center.y - half_len)
            && (pos.y <= self.center.y + half_len) 
    }

    // Divides the current quadrant into four smaller boxes and takes a corner as input paramter
    pub fn subquad(&self, corner: Corner) -> Self {
        let half = self.len / 2.0;
        let quarter = half / 2.0;
        match corner {
            Corner::NW => Quadrant::new(Vec2::new(self.center.x - quarter, self.center.y + quarter), half),
            Corner::NE => Quadrant::new(Vec2::new(self.center.x + quarter, self.center.y + quarter), half),
            Corner::SW => Quadrant::new(Vec2::new(self.center.x - quarter, self.center.y - quarter), half),
            Corner::SE => Quadrant::new(Vec2::new(self.center.x + quarter, self.center.y - quarter), half),
        }
    }

    // Helper function to print boundaries
    pub fn print_bounds(&self) {
        
        let half_len = self.len / 2.0;
        let left = self.center.x - half_len;
        let right = self.center.x + half_len;
        let bottom = self.center.y - half_len;
        let top = self.center.y + half_len;

        println!(
            "Quadrant bounds: Left: {}, Right: {}, Bottom: {}, Top: {}",
            left, right, bottom, top
        );
    }
}

// An enum that can either represnet a internal nodeitem or leaf nodeitem 
enum NodeItem {
    Internal(SubQuadrants), // A node with 4 subschildren
    External(Entity), // A node without children
}

// Represents a single node in the three
struct Node {
    pub pos: Vec2,
    pub mass: f32,
    pub item: NodeItem,
}

impl Node {
    // Constructor for the node
    fn new(pos: Vec2, mass: f32, item: NodeItem) -> Self {
        Self { pos, mass, item } // Takes in position, mass and a nodeitem
    }

    // Updates the nodes position and mass when another body is added into the node
    fn add(&mut self, other_pos: Vec2, other_mass: f32) {
        let total_mass = self.mass + other_mass;
        self.pos = (self.pos * self.mass + other_pos * other_mass) / total_mass;
        self.mass = total_mass;
    }
}

#[derive(Default)]
pub struct BHTree {
    quad: Quadrant,
    node: Option<Node>,
}

impl BHTree {

    pub fn new(quad: Quadrant) -> Self {
        BHTree {quad, ..default()} // Self { quad, node: None } creates a new BHTree with a root quadrant but no nodes yet
    }

    // Inserts new body into the BHTree
    pub fn insert(&mut self, entity: Entity, body: &Body, position: Vec2) {
        
        if let Some(current_node) = &mut self.node {
            match &mut current_node.item {
                NodeItem::Internal(subquads) => {
                    subquads.insert(entity, body, position); 
                    current_node.add(position, body.mass);
                },
                NodeItem::External(existing_entity) => {
                    if self.quad.len > MIN_QUADRANT_LENGTH {

                        let mut subquads = SubQuadrants::new(&self.quad);
    
                        subquads.insert(*existing_entity, body, current_node.pos);
                        subquads.insert(entity, body, position);
    
                        current_node.item = NodeItem::Internal(subquads);
                    }
                    current_node.add(position, body.mass);
                }
            }
        } else {
            self.node = Some(Node::new(position, body.mass, NodeItem::External(entity)));
        }
    }
    
    pub fn compute_force(&self, entity: Entity, body: &Body, position: Vec2) -> Vec2 {
        if let Some(node) = &self.node {
            match &node.item {
                NodeItem::Internal(subquads) => {
                    let distance = node.pos.distance(position);
                    if self.quad.len / distance < THETA_THRESHOLD {
                        self.approximate_force(node, position, body.mass)
                    } else {
                        subquads.compute_force(entity, body, position)
                    }
                }
                NodeItem::External(other_entity) => {
                    if *other_entity != entity {
                        self.approximate_force(node, position, body.mass)
                    } else {
                        Vec2::ZERO
                    }
                }
            }
        } else {
            Vec2::ZERO
        }
    }

    fn approximate_force(&self, node: &Node, pos: Vec2, mass: f32) -> Vec2 {
        let dir = node.pos - pos;
        let dist_sq = dir.length_squared().max(1.0);
        let force_mag = (mass * node.mass) / dist_sq;
        dir.normalize_or_zero() * force_mag
    }
}

struct SubQuadrants {
    nw: Box<BHTree>,
    ne: Box<BHTree>,
    sw: Box<BHTree>,
    se: Box<BHTree>,
}

impl SubQuadrants {
    fn new(parent_quad: &Quadrant) -> Self {
        Self {
            nw: Box::new(BHTree::new(parent_quad.subquad(Corner::NW))),
            ne: Box::new(BHTree::new(parent_quad.subquad(Corner::NE))),
            sw: Box::new(BHTree::new(parent_quad.subquad(Corner::SW))),
            se: Box::new(BHTree::new(parent_quad.subquad(Corner::SE))),
        }
    }

    fn insert(&mut self, entity: Entity, body: &Body, pos: Vec2) {
        if !self.nw.quad.contains(pos) && !self.ne.quad.contains(pos) &&
            !self.sw.quad.contains(pos) && !self.se.quad.contains(pos) {
    
            let half_len = self.nw.quad.len; 
            let parent_center = self.nw.quad.center + Vec2::new(half_len / 2.0, -half_len / 2.0);
    
            eprintln!("WARNING: Position {:?} is out of bounds!", pos);
            eprintln!("Parent quadrant center: {:?}, length: {}", parent_center, half_len * 2.0);
            
            return;
        }
        if self.nw.quad.contains(pos) {
            self.nw.insert(entity, body, pos);
        } else if self.ne.quad.contains(pos) {
            self.ne.insert(entity, body, pos);
        } else if self.sw.quad.contains(pos) {
            self.sw.insert(entity, body, pos);
        } else if self.se.quad.contains(pos) {
            self.se.insert(entity, body, pos);
        }
    }

    fn compute_force(&self, entity: Entity, body: &Body, pos: Vec2) -> Vec2 {
        self.nw.compute_force(entity, body, pos)
            + self.ne.compute_force(entity, body, pos)
            + self.sw.compute_force(entity, body, pos)
            + self.se.compute_force(entity, body, pos)
    }
}
