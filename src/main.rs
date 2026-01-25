use bevy::prelude::*;
use bevy::asset::AssetMetaCheck;
use bevy::math::Affine2;
use bevy::prelude::ops::abs;

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

#[derive(Component)]
struct JumpData {
    start: Vec3,
    end: Vec3,
    timer: f32,       // Current progress in seconds
    duration: f32,    // Total time the slide should take
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
        .add_systems(Update, (rotate_plane, rotate_cube, update_jump)) // Simplified systems
        .run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    asset_server: Res<AssetServer>,
) {
    // 1. The "Container" (Replaces the Cuboid mesh and SpatialBundle)
    let cube_id = commands.spawn((
        Transform::from_xyz(0.0, 1.0, 0.0), // The parent's position
        RotatingCube,
    )).id();

    // 2. The 6-Circle Geometry
    let face_data = [
        (Vec3::new(0.0, 0.0, 0.99), Quat::IDENTITY),                             // Front
        (Vec3::new(0.0, 0.0, -0.99), Quat::from_rotation_y(std::f32::consts::PI)), // Back
        (Vec3::new(0.99, 0.0, 0.0), Quat::from_rotation_y(std::f32::consts::FRAC_PI_2)), // Right
        (Vec3::new(-0.99, 0.0, 0.0), Quat::from_rotation_y(-std::f32::consts::FRAC_PI_2)), // Left
        (Vec3::new(0.0, 0.99, 0.0), Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2)), // Top
        (Vec3::new(0.0, -0.99, 0.0), Quat::from_rotation_x(std::f32::consts::FRAC_PI_2)), // Bottom
    ];

    // 3. Populate the container
    commands.entity(cube_id).with_children(|parent| {
        parent.spawn((
            Mesh3d(meshes.add(
                Sphere::new(0.1)
                    .mesh()
                    // 32 sectors and 18 stacks is the standard "smooth" sphere
                    // This returns a Mesh directly, not a Result.
                    .uv(32, 18) 
            )), 
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color: Color::srgb(0.75, 0.25, 1.0), 
                unlit: true,
                ..default()
            })),
            // PointLight was here but has problems on some phones.
            // Gotta fake internal illumination by 2nd inner set of
            // glowing internal faces below.
        ));

        // How to make .ktx2 (currently these are 256x256):
        // ktx create --format R8G8B8_SRGB --assign-tf srgb --zstd 20
        // --generate-mipmap --mipmap-filter kaiser in.png out.ktx2

        let outside_mat = materials.add(StandardMaterial {
            base_color_texture: Some(asset_server.load("WhiteBearCrabRealRound.ktx2")),
            cull_mode: Some(bevy::render::render_resource::Face::Back), // Only shows outside
            ..default()
        });

        let inside_mat = materials.add(StandardMaterial {
            base_color_texture: Some(asset_server.load("WhiteBearCrabRealRound.ktx2")),
            // The "Internal Glow" - Adjust 0.02 to your liking for intensity
            emissive: LinearRgba::from(Color::srgb(0.75, 0.25, 1.0)) * 0.03, 
            cull_mode: Some(bevy::render::render_resource::Face::Back), // Flipped: shows inside!
            ..default()
        });

        for (offset, rotation) in face_data {
            parent.spawn(( // outside
                Mesh3d(meshes.add(Circle::new(0.90))),
                MeshMaterial3d(outside_mat.clone()),
                Transform::from_translation(offset).with_rotation(rotation),
            ));

            parent.spawn(( // inside is inset
                Mesh3d(meshes.add(Circle::new(0.90))),
                MeshMaterial3d(inside_mat.clone()),
                Transform { // flip to face inward
                    translation: offset * 0.99, // slight inset
                    rotation: rotation * Quat::from_rotation_y(std::f32::consts::PI),
                    scale: Vec3::splat(0.995), // slight shrink
                },
            ));
        }
    })
    .observe(|drag: On<Pointer<Drag>>, mut settings: ResMut<CubeParms>| {
        settings.rotation_speed += drag.delta.x * 0.005;
    });
    commands.spawn(( // big "record player" spinning logo on the ground
        Mesh3d(meshes.add(Circle::new(4.0).mesh().resolution(128))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color_texture: Some(asset_server.load("WhiteBearCrabRealRound.ktx2")),
            alpha_mode: AlphaMode::Opaque,
            uv_transform: Affine2::from_translation(Vec2::new(0.0000, 0.000)) // left, up
                * Affine2::from_translation(Vec2::splat(0.5))
                * Affine2::from_scale(Vec2::splat(0.98))
                * Affine2::from_translation(Vec2::splat(-0.5)),
            ..default()
        })),
        Transform::from_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2)),
        RotatingPlane,
    ))
    .observe(|drag: On<Pointer<Drag>>, mut settings: ResMut<PlaneParms>| {
        // drag.delta is mouse movement during the drag
        settings.rotation_speed += drag.delta.x * 0.001;
    })
    .observe(|event: On<Pointer<Click>>, 
              mut commands: Commands, 
              cube_query: Query<(Entity, &GlobalTransform), With<RotatingCube>>,
              disk_query: Query<&GlobalTransform>| {
        
        if event.duration.as_secs_f32() < 0.2 {
            if let Some(hit_pos) = event.hit.position {
                if let Ok((cube_entity, cube_global)) = cube_query.single() {
                    let disk_entity = event.event_target();

                    if let Ok(disk_global) = disk_query.get(disk_entity) {
                        // Convert the Cube's current WORLD position to the DISK'S LOCAL space
                        let start_local = disk_global.affine().inverse().transform_point3(cube_global.translation());
                        
                        // Convert the CLICK (hit_pos) to the DISK'S LOCAL space
                        let end_local = disk_global.affine().inverse().transform_point3(hit_pos);

                        // Parent the cube (maintains world position)
                        commands.entity(cube_entity).set_parent_in_place(disk_entity);

                        // Start the move using the calculated local start
                        commands.entity(cube_entity).insert(JumpData {
                            start: start_local,
                            // End point + 1.0 height to keep it on the surface
                            end: end_local + Vec3::new(0.0, 0.0, 1.0), 
                            timer: 0.0,
                            duration: 0.6,
                        });
                    }
                }
            }
        }
    });
    // Light: Up and to side, shadows on
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

fn rotate_cube(
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

fn update_jump(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut Transform, &mut JumpData)>,
) {
    for (entity, mut transform, mut data) in &mut query {
        data.timer += time.delta_secs();
        let t = (data.timer / data.duration).clamp(0.0, 1.0);

        // Horizontal Movement (Ease-In-Ease-Out)
        let smooth_t = t * t * (3.0 - 2.0 * t);
        let current_pos = data.start.lerp(data.end, smooth_t);

        // Vertical Arc (The Jump)
        let jump_height = 2.0;
        let arc_offset = 4.0 * t * (1.0 - t) * jump_height;
        transform.translation = current_pos + Vec3::new(0.0, 0.0, arc_offset);

        if t >= 1.0 {
            // Reset scale perfectly on landing
            transform.scale = Vec3::splat(1.0);
            commands.entity(entity).remove::<JumpData>();
        } else {
            let squash_factor = 0.5 + abs(0.5 - t); 

            transform.scale.x = 1.0 + (1.0 - squash_factor) * 1.0;
            transform.scale.y = squash_factor; // vertical
            transform.scale.z = 1.0 + (1.0 - squash_factor) * 1.0;
        }
    }
}

