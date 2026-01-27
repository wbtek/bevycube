use bevy::prelude::*;
use bevy::asset::AssetMetaCheck;
use bevy::math::Affine2;

// --- Components ---

#[derive(Component, Default, Reflect)]
#[reflect(Component)]
#[require(Transform, Visibility)]
struct RotatingCube;

#[derive(Component, Default, Reflect)]
#[reflect(Component)]
#[require(Transform, Visibility)]
struct RotatingPlane;

#[derive(Component)]
struct JumpData {
    world_start: Vec3,
    local_target: Vec3,
    timer: f32,
    duration: f32,
    disk_entity: Entity,
}

// --- Resources ---

#[derive(Resource)]
struct PlaneParms {
    rotation_speed: f32,
}

#[derive(Resource)]
struct CubeParms {
    rotation_speed: f32,
}

// --- Main ---

fn main() {
    App::new()
        .insert_resource(PlaneParms { rotation_speed: 0.2 })
        .insert_resource(CubeParms { rotation_speed: -1.0 })
        .add_plugins(DefaultPlugins.set(AssetPlugin {
            meta_check: AssetMetaCheck::Never,
            ..default()
        }))
        .add_plugins(MeshPickingPlugin)
        .add_systems(Startup, setup)
        .add_systems(Update, (rotate_plane, rotate_cube, update_jump))
        .run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    asset_server: Res<AssetServer>,
) {
    // 1. The Cube
    let cube_id = commands.spawn((
        RotatingCube,
        Transform::from_xyz(0.0, 1.0, 0.0),
    )).id();

    let face_data = [ // front/back, right/left, top/bottom
        (Vec3::new(0.0, 0.0, 0.99), Quat::IDENTITY),
        (Vec3::new(0.0, 0.0, -0.99), Quat::from_rotation_y(std::f32::consts::PI)),
        (Vec3::new(0.99, 0.0, 0.0), Quat::from_rotation_y(std::f32::consts::FRAC_PI_2)),
        (Vec3::new(-0.99, 0.0, 0.0), Quat::from_rotation_y(-std::f32::consts::FRAC_PI_2)),
        (Vec3::new(0.0, 0.99, 0.0), Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2)),
        (Vec3::new(0.0, -0.99, 0.0), Quat::from_rotation_x(std::f32::consts::FRAC_PI_2)),
    ];

    commands.entity(cube_id).with_children(|parent| {
        // Core center sphere
        parent.spawn((
            Mesh3d(meshes.add(Sphere::new(0.1).mesh().uv(32, 18))),
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
            cull_mode: Some(bevy::render::render_resource::Face::Back),
            ..default()
        });

        let inside_mat = materials.add(StandardMaterial {
            base_color_texture: Some(asset_server.load("WhiteBearCrabRealRound.ktx2")),
            emissive: LinearRgba::from(Color::srgb(0.75, 0.25, 1.0)) * 0.03,
            cull_mode: Some(bevy::render::render_resource::Face::Back),
            ..default()
        });

        for (offset, rotation) in face_data {
            parent.spawn((
                Mesh3d(meshes.add(Circle::new(0.90))),
                MeshMaterial3d(outside_mat.clone()),
                Transform::from_translation(offset).with_rotation(rotation),
            ));

            parent.spawn((
                Mesh3d(meshes.add(Circle::new(0.90))),
                MeshMaterial3d(inside_mat.clone()),
                Transform {
                    translation: offset * 0.99,
                    rotation: rotation * Quat::from_rotation_y(std::f32::consts::PI),
                    scale: Vec3::splat(0.995),
                },
            ));
        }
    })
    .observe(|drag: On<Pointer<Drag>>, mut settings: ResMut<CubeParms>| {
        settings.rotation_speed += drag.delta.x * 0.005;
    });

    // 2. The Ground Plane (Record Player)
    commands.spawn((
        RotatingPlane,
        Mesh3d(meshes.add(Circle::new(4.0).mesh().resolution(128))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color_texture: Some(asset_server.load("WhiteBearCrabRealRound.ktx2")),
            alpha_mode: AlphaMode::Opaque,
            uv_transform: Affine2::from_translation(Vec2::splat(0.5))
                * Affine2::from_scale(Vec2::splat(0.98))
                * Affine2::from_translation(Vec2::splat(-0.5)),
            ..default()
        })),
        Transform::from_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2)),
    ))
    .observe(|drag: On<Pointer<Drag>>, mut settings: ResMut<PlaneParms>| {
        settings.rotation_speed += drag.delta.x * 0.001;
    })
    .observe(|event: On<Pointer<Click>>,
              mut commands: Commands,
              cube_query: Query<(Entity, &GlobalTransform), With<RotatingCube>>,
              jump_check: Query<&JumpData>,
              disk_query: Query<&GlobalTransform, With<RotatingPlane>>| {

        if let Some(hit_pos) = event.hit.position {
            if let Ok((cube_entity, cube_global)) = cube_query.single() {
                if jump_check.contains(cube_entity) { return; }
                let disk_entity = event.event_target();
                if let Ok(disk_global) = disk_query.get(disk_entity) {
                    let world_start = cube_global.translation();
                    let mut local_target = disk_global.affine().inverse().transform_point3(hit_pos);
                    local_target.z += 1.0;

                    commands.entity(cube_entity).remove_parent_in_place();

                    commands.entity(cube_entity).insert(JumpData {
                        world_start,
                        local_target,
                        timer: 0.0,
                        duration: 0.6,
                        disk_entity,
                    });
                }
            }
        }
    });

    // 3. Lighting & Camera
    commands.spawn((
        PointLight {
            shadows_enabled: true,
            ..default()
        },
        Transform::from_xyz(4.0, 8.0, 4.0),
    ));

    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(-2.0, 5.0, 8.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));
}

// --- Systems ---

fn rotate_cube(
    mut query: Query<(&mut Transform, &GlobalTransform), With<RotatingCube>>,
    settings: Res<CubeParms>,
    time: Res<Time>,
) {
    let seconds_passed = time.delta_secs();
    for (mut transform, global_transform) in &mut query {
        let local_up = global_transform.affine().inverse().transform_vector3(Vec3::Y);
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
        transform.rotate_local_z(settings.rotation_speed * time.delta_secs());
    }
}

fn update_jump(
    mut commands: Commands,
    time: Res<Time>,
    disk_query: Query<&GlobalTransform, With<RotatingPlane>>,
    mut cube_query: Query<(Entity, &mut Transform, &mut JumpData), With<RotatingCube>>,
) {
    for (cube_entity, mut transform, mut data) in &mut cube_query {
        data.timer += time.delta_secs();
        let t = (data.timer / data.duration).clamp(0.0, 1.0);

        let Ok(disk_global) = disk_query.get(data.disk_entity) else { continue };

        let local_start = disk_global.affine().inverse().transform_point3(data.world_start);
        let smooth_t = t * t * (3.0 - 2.0 * t);
        let mut local_pos = local_start.lerp(data.local_target, smooth_t);

        let dist = local_start.distance(data.local_target);
        if dist > 4.0 {
            local_pos.z += 4.0 * t * (1.0 - t) * 2.0;
            let s = 0.5 + (0.5 - t).abs();
            transform.scale = Vec3::new(1.0 + (1.0 - s), s, 1.0 + (1.0 - s));
        } else {
            transform.scale = Vec3::splat(1.0);
        }

        transform.translation = disk_global.transform_point(local_pos);

        if t >= 1.0 {
            transform.scale = Vec3::splat(1.0);
            commands.entity(cube_entity).set_parent_in_place(data.disk_entity);
            commands.entity(cube_entity).remove::<JumpData>();
        }
    }
}
