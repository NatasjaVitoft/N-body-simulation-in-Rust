pub(crate) mod tests;

use bevy::prelude::*;
use bevy_egui::{EguiContextPass, EguiContexts, EguiPlugin, egui};
use rand::Rng;
use std::{collections::HashMap, ops::RangeInclusive};

#[derive(Resource)]
pub struct SimulationSettings {
    // live tweakables
    delta_t: f32,
    g: f32,
    // needs simulation reset
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
            z: 10.0,
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
        .add_systems(Update, (reset_handler, update))
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
) {
    let mut accel_map: HashMap<u32, Vec3> = HashMap::new();
    for (entity1, _body1, transform1, _velocity) in query.iter() {
        // transform.translation.x += DELTA_T * 2.0; // TEST
        let mut accel_cum = Vec3 {
            x: (0.0),
            y: (0.0),
            z: (settings.z),
        };

        for (entity2, body2, transform2, _velocity) in query.iter() {
            if entity1.index() == entity2.index() {
                // dont consider itself
                continue;
            }

            // Gravitational interraction
            let m2 = body2.mass;

            let r = transform2.translation - transform1.translation;
            // let mag_sqr = r.x * r.x + r.y * r.y;
            // let mag = mag_sqr.sqrt();

            let mag = r.length();

            let a1: Vec3 = settings.g * (m2 / (/* mag_sqrt * */mag)) * r * settings.delta_t;

            accel_cum += a1;
        }
        accel_map.insert(entity1.index(), accel_cum);
    }

    for (entity1, _body1, mut transform1, mut velocity) in query.iter_mut() {
        velocity.0 += accel_map.get(&entity1.index()).unwrap();
        transform1.translation.x += velocity.0.x * settings.delta_t;
        transform1.translation.y += velocity.0.y * settings.delta_t;
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
        MeshMaterial2d(materials.add(ColorMaterial::from_color(Srgba::rgb(body.hue, 0.3, 0.3)))),
        body,
        transform,
        velocity,
    ));
}
