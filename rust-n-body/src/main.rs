use std::{collections::HashMap, ops::RangeInclusive};

use bevy::{color::palettes::css::WHITE, prelude::*};
use rand::Rng;

const DELTA_T: f32 = 0.001;
const BODY_RADIUS: f32 = 2.0;
const BODY_MASS: f32 = 50.0;
const N_BODIES: u32 = 1000;
const SPAWN_AREA: RangeInclusive<f32> = -100.0..=100.0;
const G: f32 = 1.0;
const Z: f32 = 10.0;
fn main() {
    App::new()
    .insert_resource(ClearColor(
        Color::BLACK,
    ))
    .add_plugins(DefaultPlugins)
    .add_systems(Startup, add_bodies)
    .add_systems(Update, update)
    .run();
}

#[derive(Component)]
struct Velocity(Vec3);


#[derive(Component)]
struct Body {
    mass: f32,
    radius: f32,
}

fn add_bodies(
    mut commands: Commands,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
) {

    let mut rng = rand::rng();
    commands.spawn(Camera2d::default());
    for _ in 0..N_BODIES {
        let body = Body {
            mass: BODY_MASS, 
            radius: BODY_RADIUS,
        };
        let x = rng.random_range(SPAWN_AREA);
        let y = rng.random_range(SPAWN_AREA);
        let transform: Transform = Transform::from_xyz(x, y, Z);
        let velocity = Velocity(Vec3 { x: (0.0), y: (0.0), z: (0.0) });
        spawn_body(body, transform, velocity, &mut commands, &mut materials, &mut meshes);
    }

}

fn update(mut query: Query<(Entity, &mut Body, &mut Transform, &mut Velocity)>) {
    let mut accel_map: HashMap<u32, Vec3> = HashMap::new();
    for (entity1, _body1, transform1, _velocity) in query.iter() {
        // transform.translation.x += DELTA_T * 2.0; // TEST
        let mut accel_cum = Vec3 { x: (0.0), y: (0.0), z: (Z) };

        for (entity2, body2, transform2, _velocity) in query.iter() {
            if entity1.index() == entity2.index() { // dont consider itself
                continue;
            }

            // Gravitational interraction
            let m2 = body2.mass;
            
            let r = transform2.translation - transform1.translation;
            // let mag_sqr = r.x * r.x + r.y * r.y;
            // let mag = mag_sqr.sqrt();

            let mag = r.length();

            let a1:Vec3  = G * (m2 / (/* mag_sqrt * */mag)) * r;
            
            accel_cum += a1;
        }
        accel_map.insert(entity1.index(), accel_cum);
    }

    for (entity1, _body1, mut transform1, mut velocity) in query.iter_mut() {
        velocity.0 += accel_map.get(&entity1.index()).unwrap() * DELTA_T;
        transform1.translation.x += velocity.0.x * DELTA_T;
        transform1.translation.y += velocity.0.y * DELTA_T;
    }
}

fn spawn_body(
    body: Body,
    transform: Transform,
    velocity: Velocity,
    commands: &mut Commands,
    materials: &mut ResMut<Assets<ColorMaterial>>,
    meshes: &mut ResMut<Assets<Mesh>>,
) {
    commands.spawn((
        Mesh2d(meshes.add(Circle::new(body.radius))),
        MeshMaterial2d(materials.add(ColorMaterial::from_color(WHITE))),
        body,
        transform,
        velocity,
    ));
}