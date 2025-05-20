use crate::Body;
use bevy::prelude::*;

pub struct Quadtree {
    root: TreeNode,
}

impl Quadtree {
    pub fn new(quad: Quad) -> Self {
        Quadtree {
            root: TreeNode::new(quad),
        }
    }

    pub fn insert(&mut self, entity: Entity, transform: Transform, body: Body) {
        self.root.insert_into_subquad(entity, transform, body);
    }

    pub fn get_total_accel(
        &mut self,
        entity: Entity,
        transform: Transform,
        body: Body,
        g: f32,
        dt: f32,
        theta: f32,
    ) -> Vec3 {
        self.root
            .get_total_accel(entity, transform, body, g, dt, theta)
    }
    pub fn draw_tree(&self, mut gizmos: Gizmos) {
        fn draw_node(node: &TreeNode, gizmos: &mut Gizmos) {
            gizmos.rect_2d(
                Isometry2d::from_xy(node.quad.center.x, node.quad.center.y),
                Vec2::splat(node.quad.size),
                Color::linear_rgba(0.0, 0.0, 1.0, 0.2),
            );
        }

        fn recurse(gizmos: &mut Gizmos, node: &TreeNode) {
            draw_node(&node, gizmos);

            if let Some(child) = &node.nw.node {
                recurse(gizmos, child);
            }
            if let Some(child) = &node.ne.node {
                recurse(gizmos, child);
            }
            if let Some(child) = &node.sw.node {
                recurse(gizmos, child);
            }
            if let Some(child) = &node.se.node {
                recurse(gizmos, child);
            }
        }
        recurse(&mut gizmos, &self.root);
    }
}

struct TreeNode {
    quad: Quad,
    nw: Box<Subquad>,
    ne: Box<Subquad>,
    sw: Box<Subquad>,
    se: Box<Subquad>,
}

impl TreeNode {
    fn new(quad: Quad) -> Self {
        let h = quad.size / 2.0;
        let q = h / 2.0;

        TreeNode {
            nw: Box::new(Subquad::new(quad.center.x - q, quad.center.y + q, h)),
            ne: Box::new(Subquad::new(quad.center.x + q, quad.center.y + q, h)),
            sw: Box::new(Subquad::new(quad.center.x - q, quad.center.y - q, h)),
            se: Box::new(Subquad::new(quad.center.x + q, quad.center.y - q, h)),
            quad,
        }
    }

    fn insert_into_subquad(&mut self, entity: Entity, transform: Transform, body: Body) {
        let pos = Vec2::new(transform.translation.x, transform.translation.y);

        if !self.quad.contains(pos) {
            eprintln!(
                "\nPosition not found in any quads!: {:?}.\nTree node: \n{:?} \nQuads: \nnw: {:?} \nne: {:?} \nsw: {:?} \nse: {:?}",
                transform.translation,
                &self.quad,
                &self.nw.quad,
                &self.ne.quad,
                &self.sw.quad,
                &self.se.quad,
            );
            return;
        }

        if pos.y < self.quad.center.y {
            // going south
            if pos.x < self.quad.center.x {
                // going west
                self.sw.insert_or_divide(entity, transform, body);
            } else {
                // going east with ambiguous cases
                self.se.insert_or_divide(entity, transform, body);
            }
        } else {
            // going north
            if pos.x < self.quad.center.x {
                // going west
                self.nw.insert_or_divide(entity, transform, body);
            } else {
                // going east with amb cases
                self.ne.insert_or_divide(entity, transform, body);
            }
        }

        /*
               if self.nw.quad.contains(position) {
                   self.nw.insert_or_divide(entity, transform, body);
               } else if self.ne.quad.contains(position) {
                   self.ne.insert_or_divide(entity, transform, body);
               } else if self.sw.quad.contains(position) {
                   self.sw.insert_or_divide(entity, transform, body);
               } else if self.se.quad.contains(position) {
                   self.se.insert_or_divide(entity, transform, body);
               } else {
                   eprint!(
                       "\nPosition not found in any quads!: {:?}.\nTree node: \n{:?} \nQuads: \nnw: {:?} \nne: {:?} \nsw: {:?} \nse: {:?}",
                       transform.translation,
                       &self.quad,
                       &self.nw.quad,
                       &self.ne.quad,
                       &self.sw.quad,
                       &self.se.quad,
                   )
               }
        */
    }

    fn get_total_accel(
        &self,
        entity: Entity,
        transform: Transform,
        body: Body,
        g: f32,
        dt: f32,
        theta: f32,
    ) -> Vec3 {
        //  TODO FIX AND DO THE REST OF THE ChILDREN
        let mut cum_accel = Vec3::ZERO;

        cum_accel += get_accel(&self.nw, entity, transform, body, g, dt, theta);
        cum_accel += get_accel(&self.ne, entity, transform, body, g, dt, theta);
        cum_accel += get_accel(&self.sw, entity, transform, body, g, dt, theta);
        cum_accel += get_accel(&self.se, entity, transform, body, g, dt, theta);

        cum_accel
    }
}

fn get_accel(
    subquad: &Box<Subquad>,
    entity: Entity,
    transform: Transform,
    body: Body,
    g: f32,
    dt: f32,
    theta: f32,
) -> Vec3 {
    match &subquad.node {
        None => {
            // Node is a leaf
            match subquad.entity {
                // With an occupant
                Some(tuple) => {
                    if tuple.0.index() == entity.index() {
                        return Vec3::ZERO;
                    } else {
                        return calc_accel(
                            tuple.2.mass,
                            transform.translation,
                            tuple.1.translation,
                            dt,
                            g,
                        );
                    }
                }
                None => {
                    // Nobody home;
                    return Vec3::ZERO;
                }
            }
        }
        Some(next_node) => {
            // Node is an internal node
            // S =  quad size
            // d = distance between node center of mass and body

            let s = subquad.quad.size;
            let d = transform.translation.distance(next_node.nw.pos_mass);

            if s / d < theta {
                return calc_accel(subquad.mass, transform.translation, subquad.pos_mass, dt, g);
            } else {
                // node is too close to be treated as one. DIG DEEPER!!
                next_node.get_total_accel(entity, transform, body, g, dt, theta)
            }
        }
    }
}

fn calc_accel(m2: f32, t1: Vec3, t2: Vec3, dt: f32, g: f32) -> Vec3 {
    let r = t2 - t1;

    let mag = r.length();
    g * (m2 / mag) * r.normalize() * dt
}

struct Subquad {
    quad: Quad,
    entity: Option<(Entity, Transform, Body)>,
    node: Option<TreeNode>,
    mass: f32,
    pos_mass: Vec3,
}

impl Subquad {
    fn new(x: f32, y: f32, size: f32) -> Self {
        Subquad {
            quad: Quad::new(x, y, size),
            entity: Option::None,
            node: Option::None,
            mass: 0.0,
            pos_mass: Vec3 {
                x: 0.0,
                y: 0.0,
                z: 0.0,
            },
        }
    }

    fn insert_or_divide(&mut self, entity: Entity, transform: Transform, body: Body) {
        match &mut self.node {
            Some(node) => {
                // Node Is internal. Updat center of mass and total mass, and insert into subquadrants
                let m1 = self.mass;
                let m2 = body.mass;
                let m = m1 + m2;
                let x1 = self.pos_mass.x;
                let x2 = transform.translation.x;
                let y1 = self.pos_mass.y;
                let y2 = transform.translation.y;

                let x = (x1 * m1 + x2 * m2) / m;
                let y = (y1 * m1 + y2 * m2) / m;

                self.mass = m;
                self.pos_mass.x = x;
                self.pos_mass.y = y;

                node.insert_into_subquad(entity, transform, body);
            }
            None => {
                // Node is leaf. Insert if no body, or subdivide if occupied
                match self.entity {
                    None => {
                        // No body present
                        self.entity = Some((entity, transform, body));
                    }
                    Some(tuple) => {
                        // Node is occupied. We must dig deeper!!!1

                        if self.quad.size < 1.0 {
                            // unless node is too small.
                            // To avoid weird edge cases where it cannot be computed if a position is in a quad,
                            // we just add the mass and update center of mass of the node.
                            let m1 = self.mass;
                            let m2 = body.mass;
                            let m = m1 + m2;
                            let x1 = self.pos_mass.x;
                            let x2 = transform.translation.x;
                            let y1 = self.pos_mass.y;
                            let y2 = transform.translation.y;

                            let x = (x1 * m1 + x2 * m2) / m;
                            let y = (y1 * m1 + y2 * m2) / m;

                            self.mass = m;
                            self.pos_mass.x = x;
                            self.pos_mass.y = y;
                        } else {
                            let mut new_node = TreeNode::new(self.quad);

                            new_node.insert_into_subquad(entity, transform, body);
                            new_node.insert_into_subquad(tuple.0, tuple.1, tuple.2);

                            let m1 = self.mass;
                            let m2 = body.mass;
                            let m = m1 + m2;
                            let x1 = self.pos_mass.x;
                            let x2 = transform.translation.x;
                            let y1 = self.pos_mass.y;
                            let y2 = transform.translation.y;

                            let x = (x1 * m1 + x2 * m2) / m;
                            let y = (y1 * m1 + y2 * m2) / m;

                            self.mass = m;
                            self.pos_mass.x = x;
                            self.pos_mass.y = y;

                            self.node = Some(new_node);
                        }
                    }
                }
            }
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Quad {
    center: Vec2,
    size: f32,
}

impl Quad {
    pub fn new(x: f32, y: f32, size: f32) -> Self {
        Quad {
            center: Vec2::new(x, y),
            size,
        }
    }

    pub fn new_containing(positions: &[Vec2]) -> Self {
        let mut min_x = f32::MAX;
        let mut min_y = f32::MAX;
        let mut max_x = f32::MIN;
        let mut max_y = f32::MIN;

        for p in positions {
            min_x = min_x.min(p.x);
            min_y = min_y.min(p.y);
            max_x = max_x.max(p.x);
            max_y = max_y.max(p.y);
        }

        let center = Vec2::new(min_x + max_x, min_y + max_y) * 0.5;
        let size = (max_x - min_x).max(max_y - min_y) + 1.0;

        Self { center, size }
    }

    fn contains(&self, pos: Vec2) -> bool {
        let hl = self.size / 2.0;
        let eps = hl * 0.0001;
        /*         (pos.x >= self.center.x - hl - eps)
        && (pos.x <= self.center.x + hl + eps)
        && (pos.y >= self.center.y - hl - eps)
        && (pos.y <= self.center.y + hl + eps) */

        (self.center.x - pos.x).abs() - eps <= hl && (self.center.y - pos.y).abs() - eps <= hl
    }
}
