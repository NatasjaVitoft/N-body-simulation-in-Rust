use bevy::{color::palettes::css::WHITE, prelude::*, state::commands, transform};

const DELTA_T: f32 = 0.1;
const BODY_RADIUS: f32 = 10.0;

fn main() {
    App::new()
    .add_plugins(DefaultPlugins)
    .add_systems(Startup, add_bodies)
    .add_systems(Update, update)
    // .add_systems(Update, update)
    .run();
}


#[derive(Component)]
struct Body {
    mass: f64,
    radius: f32,
    velocity: f32,
}

fn add_bodies(
    mut commands: Commands,
    materials: ResMut<Assets<ColorMaterial>>,
    meshes: ResMut<Assets<Mesh>>,
) {
    let transform: Transform = Transform::from_xyz(-10.0, 0.0, 10.0);
    let body = Body {
        mass: 1.0, 
        radius: BODY_RADIUS,
        velocity: 0.0
    };
    commands.spawn(Camera2d::default());
    spawn_body(body, transform, commands, materials, meshes);
}

fn update(mut query: Query<(&Body, &mut Transform)>) {
    for (_body, mut transform) in &mut query {
        transform.translation.x += DELTA_T * 2.0;
    }
}

fn spawn_body(
    body: Body,
    transform: Transform,
    mut commands: Commands,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    commands.spawn((
        Mesh2d(meshes.add(Circle::new(body.radius))),
        MeshMaterial2d(materials.add(ColorMaterial::from_color(WHITE))),
        body,
        transform,
    ));
}