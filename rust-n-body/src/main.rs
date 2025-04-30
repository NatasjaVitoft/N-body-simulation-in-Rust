pub(crate) mod tests;

use bevy::prelude::*;
use bevy_egui::{EguiContextPass, EguiContexts, EguiPlugin, egui};
use rand::Rng;
use std::{collections::HashMap, ops::RangeInclusive};
use bevy::prelude::{Color, Gizmos};

mod quadtree;
use crate::quadtree::{BHTree, Quadrant};



// Ressource to draw root quadrant
#[derive(Resource, Clone)]
struct RootQuadrant(pub Quadrant);

#[derive(Resource)]
pub struct SimulationSettings {

    delta_t: f32,
    g: f32,
    min_body_mass: f32,
    max_body_mass: f32,
    n_bodies: u32,
    spawn_area: RangeInclusive<f32>,
    z: f32,
}

impl Default for SimulationSettings {
    fn default() -> Self {
        SimulationSettings {
            delta_t: 0.001,
            g: 1.0,
            min_body_mass: 10.0,
            max_body_mass: 100.0,
            n_bodies: 1500,
            spawn_area: -300.0..=300.0,
            z: 0.0,
        }
    }
}

#[derive(Component)]
struct Velocity(Vec3);

#[derive(Component)]
struct Body {
    mass: f32,
    radius: f32,
    hue: f32,
}

#[derive(Event)]
struct ResetEvent;

fn main() {
    App::new()
        .insert_resource(ClearColor(Color::BLACK))
        .insert_resource(SimulationSettings::default())
        .add_plugins(DefaultPlugins)
        .add_plugins(EguiPlugin {
            enable_multipass_for_primary_context: true,
        })
        .add_event::<ResetEvent>()
        .add_systems(EguiContextPass, ui_window)
        .add_systems(Startup, (spawn_camera, add_bodies))
        .add_systems(Update, (reset_handler, update, draw_quadrant))
        .run();
}

fn ui_window(
    mut contexts: EguiContexts,
    mut settings: ResMut<SimulationSettings>,
    mut reset_writer: EventWriter<ResetEvent>,
) {
    egui::Window::new("Settings").show(contexts.ctx_mut(), |ui| {
        ui.add(egui::Slider::new(&mut settings.g, 0.0..=10.0).text("Gravity constant"));
        ui.add(egui::Slider::new(&mut settings.delta_t, 0.00000001..=0.01).text("Delta T"));

        ui.add(egui::Label::new("Reset Sim after tweaking these"));
        ui.add(egui::Slider::new(&mut settings.n_bodies, 2..=5000).text("Num Bodies"));
        ui.add(egui::Slider::new(&mut settings.min_body_mass, 1.0..=5000.0).text("Min Body Mass"));
        ui.add(egui::Slider::new(&mut settings.max_body_mass, 1.0..=5000.0).text("Max Body Mass"));
        if ui.button("Reset").clicked() {
            reset_writer.write(ResetEvent);
        }

        let num_bodies = settings.n_bodies as f32;
        let simulation_size = num_bodies * 2.0; 
        let root_quadrant = Quadrant::new(Vec2::new(0.0, 0.0), simulation_size);

    
    });
}

fn reset_handler(
    query: Query<Entity, With<Body>>,
    reset_event: EventReader<ResetEvent>,
    mut commands: Commands,
    materials: ResMut<Assets<ColorMaterial>>,
    meshes: ResMut<Assets<Mesh>>,
    settings: Res<SimulationSettings>,
) {
    if reset_event.is_empty() {
        return;
    }

    for entity in &query {
        commands.entity(entity).despawn();
    }

    add_bodies(commands, materials, meshes, settings);
}

fn spawn_camera(mut commands: Commands) {
    commands.spawn(Camera2d::default());
}

fn mass_to_radius(m: f32) -> f32 {
    // Radius determination: r=sqrt(G * Mass/accel)
    let g = 1.0; // constant for size
    let a = 10.0; // initial accel
    let res = g * m / a;
    res.sqrt()
}

pub fn mass_to_hue(m: f32, min_mass: f32, max_mass: f32) -> f32 {
    // linear conversion
    // NewValue = (((OldValue - OldMin) * (NewMax - NewMin)) / (OldMax - OldMin)) + NewMin
    let new_max = 1.0;

    if min_mass == max_mass {
        return 1.0;
    }

    ((m - min_mass) * new_max) / (max_mass - min_mass)
}

/*
fn body_collide(
    body1: &Body,
    body2: &Body,
    vel1: &Velocity,
    vel2: &Velocity,
    imp: &Vec3,
    dist: f32,
) -> Vec3 {
    // elastic colliison
    let m_sum = body1.mass + body2.mass;
    let v_diff = vel2.0 - vel1.0;

    let num_a = 2.0 * body2.mass * v_diff.dot(*imp);
    let den_a = m_sum * dist * dist;

    imp * (num_a / den_a)
} */

fn add_bodies(
    mut commands: Commands,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
    settings: Res<SimulationSettings>,
) {
    let norm_min = if settings.min_body_mass < settings.max_body_mass {
        settings.min_body_mass
    } else {
        settings.max_body_mass
    };

    let mut rng = rand::rng();
    for _ in 0..settings.n_bodies {
        let rng_mass = rng.random_range(norm_min..=settings.max_body_mass);
        let body = Body {
            mass: rng_mass,
            radius: mass_to_radius(rng_mass),
            hue: mass_to_hue(rng_mass, settings.min_body_mass, settings.max_body_mass),
        };
        let x = rng.random_range(settings.spawn_area.clone());
        let y = rng.random_range(settings.spawn_area.clone());
        let transform: Transform = Transform::from_xyz(x, y, settings.z);
        let velocity = Velocity(Vec3::ZERO);
        spawn_body(
            body,
            transform,
            velocity,
            &mut commands,
            &mut materials,
            &mut meshes,
        );
    }
}

fn update(
    mut query: Query<(Entity, &mut Body, &mut Transform, &mut Velocity)>,
    settings: Res<SimulationSettings>,
    mut commands: Commands,
) {
    let mut accel_map: HashMap<u32, Vec3> = HashMap::new();

    let mut min = Vec2::splat(f32::MAX);
    let mut max = Vec2::splat(f32::MIN);

    for (_entity, _body, transform, _velocity) in query.iter() { // Marked with "_" for entity, body and velocity to supress warnings. We are only interested in the position which is in transform 
        let pos = transform.translation.truncate();
        min = min.min(pos);
        max = max.max(pos);
    }

    let center = (min + max) / 2.0;
    let size = (max - min).max_element(); 
    
    let root_quad = Quadrant::new(center, size * 1.1);
    commands.insert_resource(RootQuadrant(root_quad.clone()));

    root_quad.print_bounds(); 

    let mut tree = BHTree::new(root_quad);


    for (entity, body, transform, _velocity) in query.iter() {
        let pos = transform.translation.truncate();
        tree.insert(entity, body, pos);
    }

    for (entity, body, transform, velocity) in query.iter() {
        let mut accel_cum = Vec3 {
            x: 0.0,
            y: 0.0,
            z: settings.z,
        };

        let pos = transform.translation.truncate();

        let force = tree.compute_force(entity, body, pos);

        accel_cum += Vec3::new(force.x, force.y, 0.0) * settings.g * settings.delta_t;

        accel_map.insert(entity.index(), accel_cum);
    }

    for (entity, _body, mut transform, mut velocity) in query.iter_mut() {
        velocity.0 += accel_map.get(&entity.index()).unwrap_or(&Vec3::ZERO);

        transform.translation.x += velocity.0.x * settings.delta_t;
        transform.translation.y += velocity.0.y * settings.delta_t;
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
        MeshMaterial2d(materials.add(ColorMaterial::from_color(Srgba::rgb(body.hue, 0.5, 0.1)))),
        body,
        transform,
        velocity,
    ));
}




fn draw_quadrant(mut gizmos: Gizmos, root_quad: Option<Res<RootQuadrant>>) {
    if let Some(root_quad) = root_quad {
        let root = &root_quad.0;

        let half = root.len / 2.0;
        let top_left = Vec3::new(root.center.x - half, root.center.y + half, 0.0);
        let top_right = Vec3::new(root.center.x + half, root.center.y + half, 0.0);
        let bottom_left = Vec3::new(root.center.x - half, root.center.y - half, 0.0);
        let bottom_right = Vec3::new(root.center.x + half, root.center.y - half, 0.0);

        gizmos.line(top_left, top_right, Color::srgb(1.0, 1.0, 0.0));
        gizmos.line(top_right, bottom_right, Color::srgb(1.0, 1.0, 0.0));
        gizmos.line(bottom_right, bottom_left, Color::srgb(1.0, 1.0, 0.0));
        gizmos.line(bottom_left, top_left, Color::srgb(1.0, 1.0, 0.0));
    }
}