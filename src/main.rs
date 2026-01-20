use bevy::prelude::*;
use bevy::asset::AssetMetaCheck;

#[derive(Component)]
struct RotatingCubeOut;

#[derive(Component)]
struct RotatingCubeIn;

#[derive(Component)]
struct RotatingPlane;

#[derive(Resource)]
struct PlaneParms {
    rotation_speed: f32,
}

#[derive(Resource)]
struct CubeParms {
    rotation_speed: f32,
    world_click: Vec3,
}

fn main() {
    App::new()
        .insert_resource(PlaneParms { rotation_speed: 0.2 })
        .insert_resource(CubeParms {
            rotation_speed: -1.0,
            world_click: Vec3::ZERO
        })
        .add_plugins(DefaultPlugins.set(AssetPlugin {
            meta_check: AssetMetaCheck::Never,
            ..default()
        }))
        .add_plugins(MeshPickingPlugin)
        .add_systems(Startup, setup)
//        .add_systems(Update, (rotate_plane, position_cube_out, position_cube_in))
        .add_systems(Update, (rotate_plane, rotate_cube_out, rotate_cube_in, position_cube_in))
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
        RotatingCubeOut,
    ))
    .observe(|drag: On<Pointer<Drag>>, mut settings: ResMut<CubeParms>| {
        // drag.delta is the mouse movement during the drag
        settings.rotation_speed += drag.delta.x * 0.005;
    });
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
        RotatingCubeIn,
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
    .observe(|drag: On<Pointer<Drag>>, mut settings: ResMut<PlaneParms>| {
        // drag.delta is the mouse movement during the drag
        settings.rotation_speed += drag.delta.x * 0.001;
    })
    .observe(|event: On<Pointer<Click>>, mut commands: Commands, cube_query: Query<Entity, With<RotatingCubeOut>>| {
        if event.duration.as_secs_f32() < 0.2 {
            if let Some(hit_pos) = event.hit.position {
                if let Some(cube_entity) = cube_query.iter().next() {
                    let disk_entity = event.event_target();

                    commands.entity(cube_entity).set_parent_in_place(disk_entity);

                    // CORRECTED SWIZZLE:
                    // World X -> Local X
                    // World Z -> Local -Y
                    // World Y -> Local -Z
                    let local_translation = Vec3::new(hit_pos.x, -hit_pos.z, -hit_pos.y);

                    commands.entity(cube_entity).insert(Transform {
                        // Sit 1.0 units "above" the disk surface (which is local Z)
                        translation: local_translation + Vec3::new(0.0, 0.0, 1.0),
                        // Counteract the disk's tilt to stand upright
                        rotation: Quat::from_rotation_x(std::f32::consts::FRAC_PI_2),
                        ..default()
                    });
                }
            }
        }
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

fn position_cube_out(
    mut cube_query: Query<&mut Transform, With<RotatingCubeOut>>,
    params: Res<CubeParms>,
) {
    // Just get the cube and move it to the world click point
    if let Some(mut cube_tx) = cube_query.iter_mut().next() {
        cube_tx.translation = params.world_click + Vec3::new(0.0, 1.01, 0.0);
    }
}

fn position_cube_in(
    mut cube_query: Query<&mut Transform, With<RotatingCubeIn>>,
    params: Res<CubeParms>,
) {
    // Just get the cube and move it to the world click point
    if let Some(mut cube_tx) = cube_query.iter_mut().next() {
        cube_tx.translation = params.world_click + Vec3::new(0.0, 1.01, 0.0);
    }
}

fn rotate_cube_out(
    mut query: Query<&mut Transform, With<RotatingCubeOut>>,
    time: Res<Time>,
    settings: Res<CubeParms>,
) {
    for mut transform in &mut query {
        transform.rotate_y(settings.rotation_speed * time.delta_secs());
    }
}

fn rotate_cube_in(
    mut query: Query<&mut Transform, With<RotatingCubeIn>>,
    time: Res<Time>,
    settings: Res<CubeParms>,
) {
    for mut transform in &mut query {
        transform.rotate_y(settings.rotation_speed * time.delta_secs());
    }
}

fn rotate_plane(
    mut query: Query<&mut Transform, With<RotatingPlane>>, 
    time: Res<Time>,
    settings: Res<PlaneParms>,
) {
    for mut transform in &mut query {
        transform.rotate_local_z(settings.rotation_speed * time.delta_secs());
    }
}

