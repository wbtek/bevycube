use bevy::prelude::*;

#[derive(Component)]
struct RotatingCube;
fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, setup)
        .add_systems(Update, rotate_cube)
        .run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Cube
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::default())),
        MeshMaterial3d(materials.add(Color::srgb(0.8, 0.7, 0.6))),
        Transform::from_xyz(0.0, 0.5, 0.0),
        RotatingCube,
    ));
    // Circular Ground Plane
    commands.spawn((
        Mesh3d(meshes.add(Circle::new(4.0))),
        MeshMaterial3d(materials.add(Color::srgb(0.2, 0.2, 0.3))),
        Transform::from_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2)),
    ));
    // Light
    // Light: Moved up and to the side, with shadows on
    commands.spawn((
        PointLight {
            shadows_enabled: true,
            ..default()
        },
        Transform::from_xyz(4.0, 8.0, 4.0),
    ));
    // Camera
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(-2.0, 5.0, 8.0).looking_at(Vec3::ZERO, Vec3::Y)
    ));
}

fn rotate_cube(mut query: Query<&mut Transform, With<RotatingCube>>, time: Res<Time>) {
    for mut transform in &mut query {
        transform.rotate_y(1.0 * time.delta_secs());
    }
}
