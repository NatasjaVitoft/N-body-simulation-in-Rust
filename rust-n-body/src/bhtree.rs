use bevy::{math::VectorSpace, prelude::*};

use crate::Body;

pub struct Quadtree {
    node: Node,
}

impl Quadtree {
    fn new(quad: Quad) -> Quadtree {
        Quadtree {
            node: Node {
                quad,
                node_type: NodeType::Leaf(None),
                mass: 0.0,
                pos: Vec2::ZERO,
            },
        }
    }

    pub fn insert(&mut self, entity: Entity, body: Body, transform: Transform) {
        match &self.node.node_type {
            NodeType::Leaf(option) => {
                if let Some(tuple) = option {
                    // occupied. must subdivide
                    let mut subs = SubQuads::new(&self.node.quad);
                    self.node.subdivide_and_insert(&mut subs, entity, body, transform);
                    self.node.node_type = NodeType::Internal(subs);
                    self.node.mass += body.mass;
                }   
                else {
                    // empty node
                    self.node.node_type = NodeType::Leaf(Some((entity, body, transform)));
                    self.node.mass = body.mass;
                }
            },
            NodeType::Internal(subquads) => {
                // TODO
            }
        }
    }
}

#[derive(Debug)]
pub struct Quad {
    center: Vec2,
    size: f32,
}

impl Quad {
    pub fn new_containing(positions: &[Vec2]) -> Self {
        // calculate base quadrant size
        let mut min_x = f32::MAX;
        let mut min_y = f32::MAX;
        let mut max_x = f32::MIN;
        let mut max_y = f32::MIN;

        for pos in positions {
            min_x = min_x.min(pos.x);
            min_y = min_y.min(pos.y);
            max_x = max_x.max(pos.x);
            max_y = max_y.max(pos.y);
        }

        let center = Vec2::new(min_x + max_x, min_y + max_y) * 0.5;
        let size = (max_x - min_x).max(max_y - min_y);

        Self { center, size }
    }

    fn subquad(&self, c: Corner) -> Self {
        let h = self.size / 2.;
        let q = h / 2.;
        match c {
            Corner::NW => Quad {center: Vec2::new(self.center.x - q, self.center.y + q), size: h},
            Corner::NE => Quad {center: Vec2::new(self.center.x + q, self.center.y + q), size: h},
            Corner::SW => Quad {center: Vec2::new(self.center.x - q, self.center.y - q), size: h},
            Corner::SE => Quad {center: Vec2::new(self.center.x + q, self.center.y - q), size: h},
        }
    }

    fn contains(&self, x: f32, y:f32) -> bool {
        let hl = self.size / 2.0;
        (x >= self.center.x - hl) && (x < self.center.x + hl) && (y >= self.center.y - hl) && (y < self.center.y + hl)
    }
}

struct Node {
    quad: Quad,
    node_type: NodeType,
    mass: f32,
    pos: Vec2,
}

impl Node {
    fn subdivide_and_insert(&mut self, subs: &mut SubQuads, entity: Entity, body: Body, transform: Transform) {
        
         match subs {
            subs if subs.nw.node.quad.contains(transform.translation.x, transform.translation.y) => subs.nw.insert(entity, body, transform),
            subs if subs.ne.node.quad.contains(transform.translation.x, transform.translation.y) => subs.ne.insert(entity, body, transform),
            subs if subs.sw.node.quad.contains(transform.translation.x, transform.translation.y) => subs.sw.insert(entity, body, transform),
            subs if subs.se.node.quad.contains(transform.translation.x, transform.translation.y) => subs.se.insert(entity, body, transform),
            subs => panic!("position {}, {} was not in any quadrant?\n {:#?}, {:#?}, {:#?}, {:#?}", transform.translation.x, transform.translation.y, subs.nw.node.quad, subs.ne.node.quad, subs.sw.node.quad, subs.se.node.quad)
        }
    }
}

struct SubQuads {
    nw: Box<Quadtree>,
    ne: Box<Quadtree>,
    sw: Box<Quadtree>,
    se: Box<Quadtree>
}

impl SubQuads {
    fn new(q: &Quad) -> Self {
        SubQuads {
            nw: Box::new(Quadtree::new(q.subquad(Corner::NW))),
            ne: Box::new(Quadtree::new(q.subquad(Corner::NE))),
            sw: Box::new(Quadtree::new(q.subquad(Corner::SW))),
            se: Box::new(Quadtree::new(q.subquad(Corner::SE))),
        }
    }
}

enum Corner {
    NW,
    NE,
    SW,
    SE,
}
enum NodeType {
    Internal(SubQuads),
    Leaf(Option<(Entity, Body, Transform)>),
}
