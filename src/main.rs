use bevy::prelude::*;
use bevy::asset::AssetMetaCheck;
use bevy::math::Affine2;

#[derive(Component)]
struct RotatingCube;

#[derive(Component)]
struct RotatingPlane;

#[derive(Resource)]
struct PlaneParms {
    rotation_speed: f32,
}

#[derive(Resource)]
struct CubeParms {
    rotation_speed: f32,
}

fn main() {
    App::new()
        .insert_resource(PlaneParms { rotation_speed: 0.2 })
        .insert_resource(CubeParms { rotation_speed: -1.0 }) // Removed world_click
        .add_plugins(DefaultPlugins.set(AssetPlugin {
            meta_check: AssetMetaCheck::Never,
            ..default()
        }))
        .add_plugins(MeshPickingPlugin)
        .add_systems(Startup, setup)
        .add_systems(Update, (rotate_plane, rotate_cube_out)) // Simplified systems
        .run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    asset_server: Res<AssetServer>,
) {
    // Spawn the Outer Cube
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::from_size(Vec3::splat(2.0)))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color_texture: Some(asset_server.load("WhiteBearCrabRoundWC.png")),
            alpha_mode: AlphaMode::Mask(0.5), 
            ..default()
        })),
        Transform::from_xyz(0.0, 1.01, 0.0),
        RotatingCube,
    ))
    .observe(|drag: On<Pointer<Drag>>, mut settings: ResMut<CubeParms>| {
        settings.rotation_speed += drag.delta.x * 0.005;
    })
    // Attach the Inner Cube as a child
    .with_children(|parent| {
        parent.spawn((
            Mesh3d(meshes.add(Cuboid::from_size(Vec3::splat(1.99)))),
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color_texture: Some(asset_server.load("WhiteBearCrabRoundWC.png")),
                alpha_mode: AlphaMode::Mask(0.5), 
                cull_mode: Some(bevy::render::render_resource::Face::Front), // See inside
                ..default()
            })),
        ));
    });
    // Circular Ground Plane with Logo
    commands.spawn((
        Mesh3d(meshes.add(Circle::new(4.0))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color_texture: Some(asset_server.load("WhiteBearCrabRoundWC.png")),
            alpha_mode: AlphaMode::Blend, // Enables transparency
            uv_transform: Affine2::from_translation(Vec2::new(0.0045, 0.004)) // left, up
                * Affine2::from_translation(Vec2::splat(0.5))
                * Affine2::from_scale(Vec2::splat(0.91))
                * Affine2::from_translation(Vec2::splat(-0.5)),
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
              cube_query: Query<Entity, With<RotatingCube>>,
              // We need to query the disk's transform to do the math
              disk_query: Query<&GlobalTransform>| {
        
        if event.duration.as_secs_f32() < 0.2 {
            if let Some(hit_pos) = event.hit.position {
                if let Some(cube_entity) = cube_query.iter().next() {
                    let disk_entity = event.event_target();

                    // Get the disk's current orientation in the world
                    if let Ok(disk_global_transform) = disk_query.get(disk_entity) {
                        
                        // Convert the World hit_pos into the Disk's Local Space
                        // This math "un-rotates" the click point relative to the disk
                        let local_hit = disk_global_transform.affine().inverse().transform_point3(hit_pos);

                        // Parent the cube
                        commands.entity(cube_entity).set_parent_in_place(disk_entity);

                        // Insert the corrected Local Transform
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

fn rotate_cube_out(
    mut query: Query<(&mut Transform, &GlobalTransform), With<RotatingCube>>,
    settings: Res<CubeParms>,
    time: Res<Time>,
) {
    let seconds_passed = time.delta_secs();

    for (mut transform, global_transform) in &mut query {
        let world_up = Vec3::Y;
        // Correctly maps the world vertical axis to the cube's local space
        let local_up = global_transform.affine().inverse().transform_vector3(world_up);
        
        transform.rotate_local_axis(
            Dir3::new_unchecked(local_up.normalize()), 
            settings.rotation_speed * seconds_passed
        );
    }
}

fn rotate_plane(
    mut query: Query<&mut Transform, With<RotatingPlane>>, 
    time: Res<Time>,
    settings: Res<PlaneParms>,
) {
    for mut transform in &mut query {
        // Uses delta_secs to maintain constant rotation speed
        transform.rotate_local_z(settings.rotation_speed * time.delta_secs());
    }
}

