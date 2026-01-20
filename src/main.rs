use bevy::prelude::*;
use bevy::asset::AssetMetaCheck;

#[derive(Component)]
struct RotatingCube;

#[derive(Component)]
struct RotatingPlane;

#[derive(Resource)]
struct RotationSettings {
    speed: f32,
}

fn main() {
    App::new()
        .insert_resource(RotationSettings { speed: 0.2 })
        .add_plugins(DefaultPlugins.set(AssetPlugin {
            meta_check: AssetMetaCheck::Never,
            ..default()
        }))
        .add_plugins(MeshPickingPlugin)
        .add_systems(Startup, setup)
        .add_systems(Update, (rotate_cube, rotate_plane))
        .run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    asset_server: Res<AssetServer>,
) {
    // Cube with logo on all sides, outside
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::from_size(Vec3::splat(2.0)))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color_texture: Some(asset_server.load("WhiteBearCrabRound.png")),
            // This ensures the transparent parts of your logo show the background
            alpha_mode: AlphaMode::Mask(0.5), 
            ..default()
        })),
        Transform::from_xyz(0.0, 1.01, 0.0),
        RotatingCube,
    ));
    // Cube with logo on all sides, inside
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::from_size(Vec3::splat(1.99)))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color_texture: Some(asset_server.load("WhiteBearCrabRound.png")),
            // This ensures the transparent parts of your logo show the background
            alpha_mode: AlphaMode::Mask(0.5), 
            cull_mode: Some(bevy::render::render_resource::Face::Front),
            ..default()
        })),
        Transform::from_xyz(0.0, 1.01, 0.0),
        RotatingCube,
    ));
    // Circular Ground Plane with Logo
    commands.spawn((
        Mesh3d(meshes.add(Circle::new(4.0))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color_texture: Some(asset_server.load("WhiteBearCrabRound.png")),
            alpha_mode: AlphaMode::Blend, // Enables transparency
            unlit: false, // Optional: makes logo bright regardless of lighting
            ..default()
        })),
        Transform::from_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2)),
        RotatingPlane,
    ))
    .observe(|drag: On<Pointer<Drag>>, mut settings: ResMut<RotationSettings>| {
        // drag.delta is the mouse movement during the drag
        settings.speed += drag.delta.x * 0.001;
    });
    // Light: Up and to the side, with shadows on
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
        transform.rotate_y(-1.0 * time.delta_secs());
    }
}

fn rotate_plane(
    mut query: Query<&mut Transform, With<RotatingPlane>>, 
    time: Res<Time>,
    settings: Res<RotationSettings>,
) {
    for mut transform in &mut query {
        transform.rotate_local_z(settings.speed * time.delta_secs());
    }
}

