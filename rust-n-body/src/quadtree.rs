use crate::Body;
use bevy::prelude::*;

const MIN_QUADRANT_LENGTH: f32 = 5e-3;
const THETA_THRESHOLD: f32 = 0.7;
enum Corner {
    NW,
    NE,
    SW,
    SE,
}

pub struct BHTree {
    quad: Quadrant,
    node: Option<Node>,
}

impl BHTree {
    pub fn new(quad: Quadrant) -> Self {
        BHTree {
            quad,
            node: Option::None,
        }
    }

    // TODO TEST!!!
    pub fn insert(&mut self, particle: Entity, body: Body, transform: Transform) {
        if let Some(current_node) = &mut self.node {
            match &mut current_node.item {
                NodeItem::Internal(subquad) => {
                    current_node.mass += current_node.body.mass;
                    current_node.mass_pos = current_node.mass_pos - transform.translation;
                    subquad.insert_to_quadrant(particle, body, transform);
                }

                NodeItem::Leaf(node_particle) => {
                    if self.quad.len > MIN_QUADRANT_LENGTH {
                        let mut subquad = SubQuadrants::new(&self.quad);
                        subquad.insert_to_quadrant(
                            node_particle.clone(),
                            current_node.body,
                            transform,
                        );
                        subquad.insert_to_quadrant(particle, body, transform);
                        current_node.item = NodeItem::Internal(subquad);
                    }
                    // implied else: if we've already got too small of a grid, we still add the mass for a cheap estimate
                    current_node.mass += current_node.body.mass;
                    current_node.mass_pos = current_node.mass_pos - transform.translation;
                }
            }
        } else {
            self.node = Some(Node::new(
                body,
                NodeItem::Leaf(particle),
                body.mass,
                transform.translation,
            ))
        }
    }

    pub fn get_force(&self, p: &Entity, b: &Body, other_transform: &Transform, g: f32) -> Vec3 {
        if let Some(current_node) = &self.node {
            match &current_node.item {
                NodeItem::Internal(subquad) => {
                    let dist = current_node.mass_pos.distance(other_transform.translation);
                    if self.quad.len / dist < THETA_THRESHOLD {
                        // treat node as a single body
                        // Gravitational interraction
                        let m2 = current_node.mass;

                        let r =  current_node.mass_pos - other_transform.translation;

                        let mag = r.length();

                        return g * (m2 / (/* mag_sqrt * */mag)) * r.normalize();
                    } else {
                        // traverse the tree, returning the total force
                        subquad.get_force(p, b, other_transform, g)
                    }
                }
                NodeItem::Leaf(node_particle) => {
                    if node_particle.index() != p.index() {
                        let m2 = current_node.mass;

                        let r = current_node.mass_pos - other_transform.translation;

                        let mag = r.length();

                        return g * (m2 / (/* mag_sqrt * */mag)) * r.normalize();
                    } else {
                        // index was the same, this is the same particle
                        Vec3::new(0., 0., 10.)
                    }
                }
            }
        } else {
            // there's no body at self, so there's no force
            Vec3::new(0., 0., 10.)
        }
    }
}

#[derive(Debug)]
pub struct Quadrant {
    center: Vec3,
    len: f32,
}

impl Quadrant {

    pub fn new(length: f32) -> Self {
        Quadrant {
            len: length,
            center: Vec3::new(0., 0., 10.)
        }
    }
    /// return true if this Quadrant contains (x,y)
    fn contains(&self, x: f32, y: f32) -> bool {
        let hl = self.len / 2.0;
        (x >= self.center.x - hl)
            && (x < self.center.x + hl)
            && (y >= self.center.y - hl)
            && (y < self.center.y + hl)
    }

    fn subquad(&self, corner: Corner) -> Self {
        let hl = self.len / 2.0;
        let ql = hl / 2.0;
        match corner {
            Corner::NW => Quadrant {
                center: Vec3::new(self.center.x - ql, self.center.y + ql, 0.0),
                len: hl,
            },
            Corner::NE => Quadrant {
                center: Vec3::new(self.center.x + ql, self.center.y + ql, 0.0),
                len: hl,
            },
            Corner::SW => Quadrant {
                center: Vec3::new(self.center.x - ql, self.center.y - ql, 0.0),
                len: hl,
            },
            Corner::SE => Quadrant {
                center: Vec3::new(self.center.x + ql, self.center.y - ql, 0.0),
                len: hl,
            },
        }
    }
}

struct Node {
    body: Body,
    mass: f32,
    mass_pos: Vec3,
    item: NodeItem,
}

impl Node {
    fn new(body: Body, item: NodeItem, mass: f32, mass_pos: Vec3) -> Self {
        Node {
            body,
            item,
            mass,
            mass_pos,
        }
    }
}

struct SubQuadrants {
    nw: Box<BHTree>,
    ne: Box<BHTree>,
    sw: Box<BHTree>,
    se: Box<BHTree>,
}

impl SubQuadrants {
    fn new(q: &Quadrant) -> Self {
        SubQuadrants {
            nw: Box::new(BHTree::new(q.subquad(Corner::NW))),
            ne: Box::new(BHTree::new(q.subquad(Corner::NE))),
            sw: Box::new(BHTree::new(q.subquad(Corner::SW))),
            se: Box::new(BHTree::new(q.subquad(Corner::SE))),
        }
    }

    fn insert_to_quadrant(&mut self, p: Entity, b: Body, t: Transform) {
        // this is an internal node, we must have a subtree
        match b {
            b if self.nw.quad.contains(t.translation.x, t.translation.y) => self.nw.insert(p, b, t),
            b if self.ne.quad.contains(t.translation.x, t.translation.y) => self.ne.insert(p, b, t),
            b if self.sw.quad.contains(t.translation.x, t.translation.y) => self.sw.insert(p, b, t),
            b if self.se.quad.contains(t.translation.x, t.translation.y) => self.se.insert(p, b, t),
            b => panic!(
                "position {}, {} was not in any quadrant?\n {:#?}, {:#?}, {:#?}, {:#?}",
                t.translation.x,
                t.translation.y,
                self.nw.quad,
                self.ne.quad,
                self.sw.quad,
                self.se.quad
            ),
        }
    }

    fn get_force(&self, p: &Entity, b: &Body, other_transform: &Transform, g: f32) -> Vec3 {
        self.nw.get_force(p, b, other_transform, g)
            + self.ne.get_force(p, b, other_transform, g)
            + self.sw.get_force(p, b, other_transform, g)
            + self.se.get_force(p, b, other_transform, g)
    }
}

enum NodeItem {
    Internal(SubQuadrants),
    Leaf(Entity),
}
