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
        .add_systems(Update, (rotate_plane, rotate_cube_out))
        .run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    asset_server: Res<AssetServer>,
) {
    // 1. Spawn the Outer Cube
    let outer_entity = commands.spawn((
        Mesh3d(meshes.add(Cuboid::from_size(Vec3::splat(2.0)))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color_texture: Some(asset_server.load("WhiteBearCrabRound.png")),
            alpha_mode: AlphaMode::Mask(0.5), 
            ..default()
        })),
        Transform::from_xyz(0.0, 1.01, 0.0),
        RotatingCubeOut,
    ))
    .observe(|drag: On<Pointer<Drag>>, mut settings: ResMut<CubeParms>| {
        settings.rotation_speed += drag.delta.x * 0.005;
    })
    // 2. Attach the Inner Cube as a child
    .with_children(|parent| {
        parent.spawn((
            Mesh3d(meshes.add(Cuboid::from_size(Vec3::splat(1.99)))),
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color_texture: Some(asset_server.load("WhiteBearCrabRound.png")),
                alpha_mode: AlphaMode::Mask(0.5), 
                cull_mode: Some(bevy::render::render_resource::Face::Front), // See inside
                ..default()
            })),
            // We don't need RotatingCubeIn or a special transform anymore.
            // It will sit at (0,0,0) relative to the parent by default.
        ));
    })
    .id();
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
    .observe(|event: On<Pointer<Click>>, 
              mut commands: Commands, 
              cube_query: Query<Entity, With<RotatingCubeOut>>,
              // We need to query the disk's transform to do the math
              disk_query: Query<&GlobalTransform>| {
        
        if event.duration.as_secs_f32() < 0.2 {
            if let Some(hit_pos) = event.hit.position {
                if let Some(cube_entity) = cube_query.iter().next() {
                    let disk_entity = event.event_target();

                    // 1. Get the disk's current orientation in the world
                    if let Ok(disk_global_transform) = disk_query.get(disk_entity) {
                        
                        // 2. Convert the World hit_pos into the Disk's Local Space
                        // This math "un-rotates" the click point relative to the disk
                        let local_hit = disk_global_transform.affine().inverse().transform_point3(hit_pos);

                        // 3. Parent the cube
                        commands.entity(cube_entity).set_parent_in_place(disk_entity);

                        // 4. Insert the corrected Local Transform
                        commands.entity(cube_entity).insert(Transform {
                            // Use the calculated local point + 1.0 "up" from the surface
                            translation: local_hit + Vec3::new(0.0, 0.0, 1.0),
                            // Counteract the disk's -90x tilt so cube stands upright
                            rotation: Quat::from_rotation_x(std::f32::consts::FRAC_PI_2),
                            ..default()
                        });
                    }
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
    mut query: Query<(&mut Transform, &GlobalTransform), With<RotatingCubeOut>>,
    settings: Res<CubeParms>,
    time: Res<Time>, // This is the Bevy clock
) {
    // 1. Get the time elapsed since the last update
    let seconds_passed = time.delta_secs();

    for (mut transform, global_transform) in &mut query {
        let world_up = Vec3::Y;
        let local_up = global_transform.affine().inverse().transform_vector3(world_up);
        
        // 2. Scale the rotation by seconds_passed
        // This makes the RPM independent of the frame rate or distance
        transform.rotate_local_axis(
            Dir3::new_unchecked(local_up.normalize()), 
            settings.rotation_speed * seconds_passed
        );
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

