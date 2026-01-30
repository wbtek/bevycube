use bevy::prelude::*;
use bevy::prelude::EaseFunction::{ ElasticInOut, BounceInOut };
use bevy::asset::AssetMetaCheck;
use bevy::math::Affine2;
use bevy::asset::embedded_asset;

// --- Components ---

#[derive(Debug, Component, Default, Reflect)]
#[reflect(Component)]
#[require(Transform, Visibility)]
struct RotatingCube;

#[derive(Debug, Component, Default, Reflect)]
#[reflect(Component)]
#[require(Transform, Visibility)]
struct RotatingDisk;

#[derive(Debug, Component)]
struct Ground;

#[derive(Debug, Component)]
struct SafetyDisk;

#[derive(Debug, Clone, Copy, PartialEq, Reflect)]
enum AnimationType {
    Jump,
    Slide,
    Spin,
    Flip,
}

#[derive(Debug, Component)]
struct JumpData {
    world_start: Vec3,
    start_rotation: Quat,
    local_target: Vec3,
    timer: f32,
    duration: f32,
    target_entity: Entity,
    animation: Option<AnimationType>,
}

// --- Resources ---

#[derive(Debug, Resource)]
struct RoundelMipmapLoading {
    // [512, 256, 128, 64, 32]
    handles: [Handle<Image>; 5],
    target_handle: Handle<Image>,
}

#[derive(Debug, Resource)]
struct DiskParms {
    rotation_speed: f32,
}

#[derive(Debug, Resource)]
struct CubeParms {
    rotation_speed: f32,
}

// --- Main ---

fn main() {
    #[cfg(target_arch = "wasm32")]
    console_log::init_with_level(log::Level::Debug).ok();

    App::new()
        .insert_resource(DiskParms { rotation_speed: 0.2 })
        .insert_resource(CubeParms { rotation_speed: -1.0 })
        .add_plugins(DefaultPlugins.set(AssetPlugin {
            meta_check: AssetMetaCheck::Never,
            ..default()
        }))
        .add_plugins(|app: &mut App| {
            embedded_asset!(app, "media/WhiteBearCrab512.jpg");
            embedded_asset!(app, "media/WhiteBearCrab256.jpg");
            embedded_asset!(app, "media/WhiteBearCrab128.jpg");
            embedded_asset!(app, "media/WhiteBearCrab64.jpg");
            embedded_asset!(app, "media/WhiteBearCrab32.jpg");
        })
        .add_plugins(MeshPickingPlugin)
        .add_systems(Startup, setup)
        .add_systems(Update, (rotate_disk, rotate_cube, update_jump, stitch_roundel_system))
        .run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    asset_server: Res<AssetServer>,
) {

    let roundel_handle = asset_server.load("embedded://bevycube/media/WhiteBearCrab64.jpg"); // swapped later
    let roundel_mat = StandardMaterial {
        base_color_texture: Some(roundel_handle.clone()),
        alpha_mode: AlphaMode::Opaque,
        uv_transform: Affine2::from_translation(Vec2::splat(0.5))
            * Affine2::from_scale(Vec2::splat(0.98))
            * Affine2::from_translation(Vec2::splat(-0.5)),
            cull_mode: Some(bevy::render::render_resource::Face::Back),
        ..default()
    };

    let handles = [
        asset_server.load("embedded://bevycube/media/WhiteBearCrab512.jpg"),
        asset_server.load("embedded://bevycube/media/WhiteBearCrab256.jpg"),
        asset_server.load("embedded://bevycube/media/WhiteBearCrab128.jpg"),
        asset_server.load("embedded://bevycube/media/WhiteBearCrab64.jpg"),
        asset_server.load("embedded://bevycube/media/WhiteBearCrab32.jpg"),
    ];
    commands.insert_resource(RoundelMipmapLoading {
        handles,
        target_handle: roundel_handle.clone(),
    });

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

        for (offset, rotation) in face_data {
            parent.spawn((
                Mesh3d(meshes.add(Circle::new(0.90).mesh().resolution(128))),
                MeshMaterial3d(materials.add(roundel_mat.clone())),
                Transform::from_translation(offset).with_rotation(rotation),
            ));

            parent.spawn((
                Mesh3d(meshes.add(Circle::new(0.90).mesh().resolution(128))),
                MeshMaterial3d(materials.add(StandardMaterial {
                    emissive: LinearRgba::from(Color::srgb(0.75, 0.25, 1.0)) * 0.03,
                    ..roundel_mat.clone()
                })),
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

    // 2a. The Turntable
    commands.spawn((
        RotatingDisk,
        Mesh3d(meshes.add(Circle::new(4.0).mesh().resolution(128))),
        MeshMaterial3d(materials.add(roundel_mat.clone())),
        Transform::from_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2)),
    ))
    .observe(|drag: On<Pointer<Drag>>, mut settings: ResMut<DiskParms>| {
        settings.rotation_speed += drag.delta.x * 0.001;
    })
    .observe(|event: On<Pointer<Click>>,
              mut commands: Commands,
              cube_query: Query<(Entity, &GlobalTransform), With<RotatingCube>>,
              jump_check: Query<&JumpData>,
              disk_query: Query<&GlobalTransform, With<RotatingDisk>>| {

        if let Some(hit_pos) = event.hit.position {
            if let Ok((cube_entity, cube_global)) = cube_query.single() {
                if jump_check.contains(cube_entity) { return; }
                let target_entity = event.event_target();
                if let Ok(target_global) = disk_query.get(target_entity) {
                    let world_start = cube_global.translation();
                    let mut local_target = target_global.affine().inverse().transform_point3(hit_pos);
                    local_target.z += 1.0;
                    commands.entity(cube_entity).remove_parent_in_place();
                    let start_rotation = cube_global.compute_transform().rotation;
                    commands.entity(cube_entity).insert(JumpData {
                        world_start,
                        start_rotation,
                        local_target,
                        timer: 0.0,
                        duration: 0.6,
                        target_entity,
                        animation: None,
                    });
                }
            }
        }
    });

    // 2b. The Safety Zone
    commands.spawn((
        SafetyDisk,
        Mesh3d(meshes.add(Circle::new(5.4).mesh().resolution(128))),
        MeshMaterial3d(materials.add(Color::srgb(0.5, 0.25, 0.0))),
        Transform::from_xyz(0.0, -0.49, 0.0).with_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2)),
    ));

    // 2c. The Ground
    commands.spawn((
        Ground,
        Mesh3d(meshes.add(Plane3d::default().mesh().size(20., 20.))),
        MeshMaterial3d(materials.add(Color::srgb(0.3, 0.5, 0.3))),
        Transform::from_xyz(0.0, -0.5, 0.0),
    ))
    .observe(|event: On<Pointer<Click>>, mut commands: Commands, cube_query: Query<(Entity, &GlobalTransform), With<RotatingCube>>, jump_check: Query<&JumpData>, ground_query: Query<&GlobalTransform, With<Ground>>| {
        // Exact same logic as the Disk observer, but using the ground's transform!
        if let Some(hit_pos) = event.hit.position {
            if let Ok((cube_entity, cube_global)) = cube_query.single() {
                if jump_check.contains(cube_entity) { return; }
                let target_entity = event.event_target();
                if let Ok(target_global) = ground_query.get(target_entity) {
                    let world_start = cube_global.translation();
                    let mut local_target = target_global.affine().inverse().transform_point3(hit_pos);
                    // Adjust Z or Y depending on your coordinate preference
                    local_target.y += 1.0; 
                    commands.entity(cube_entity).remove_parent_in_place();
                    let start_rotation = cube_global.compute_transform().rotation;
                    commands.entity(cube_entity).insert(JumpData {
                        world_start,
                        start_rotation,
                        local_target,
                        timer: 0.0,
                        duration: 0.8, // Maybe a bit slower for big ground jumps?
                        target_entity,
                        animation: None,
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

fn stitch_roundel_system(
    mut commands: Commands,
    loading: Option<Res<RoundelMipmapLoading>>,
    mut images: ResMut<Assets<Image>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let Some(loading) = loading else { return };

    if loading.handles.iter().all(|h| images.get(h).is_some()) {
        let mut combined_data = Vec::new();
        
        let first_img = images.get(&loading.handles[0]).unwrap();
        let detected_format = first_img.texture_descriptor.format;

        for handle in &loading.handles {
            let img = images.get(handle).unwrap();
            // 1. Unwrap the Option to get the actual bytes
            if let Some(ref data) = img.data {
                combined_data.extend_from_slice(data);
            } else {
                return; 
            }
        }

        let mip_count = loading.handles.len() as u32;
        let stitched_image = Image {
            // 2. Wrap the final Vec in Some()
            data: Some(combined_data),
            texture_descriptor: bevy::render::render_resource::TextureDescriptor {
                label: Some("stitched_roundel"),
                size: bevy::render::render_resource::Extent3d { 
                    width: 512, 
                    height: 512, 
                    depth_or_array_layers: 1 
                },
                mip_level_count: mip_count,
                sample_count: 1,
                dimension: bevy::render::render_resource::TextureDimension::D2,
                format: detected_format,
                usage: bevy::render::render_resource::TextureUsages::TEXTURE_BINDING | bevy::render::render_resource::TextureUsages::COPY_DST,
                view_formats: &[],
            },
            sampler: bevy::image::ImageSampler::Descriptor(bevy::image::ImageSamplerDescriptor {
                mipmap_filter: bevy::image::ImageFilterMode::Linear,
                mag_filter: bevy::image::ImageFilterMode::Linear,
                min_filter: bevy::image::ImageFilterMode::Linear,
                anisotropy_clamp: 16,
                ..default()
            }),
            ..default()
        };

        let final_handle = images.add(stitched_image);

        for (_, mat) in materials.iter_mut() {
            if let Some(ref current_tex) = mat.base_color_texture {
                if current_tex.id() == loading.target_handle.id() {
                    mat.base_color_texture = Some(final_handle.clone());
                }
            }
        }

        commands.remove_resource::<RoundelMipmapLoading>();
    }
}

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

fn rotate_disk(
    mut query: Query<&mut Transform, With<RotatingDisk>>,
    time: Res<Time>,
    settings: Res<DiskParms>,
) {
    for mut transform in &mut query {
        transform.rotate_local_z(settings.rotation_speed * time.delta_secs());
    }
}

fn update_jump(
    mut commands: Commands,
    time: Res<Time>,
    target_query: Query<&GlobalTransform>,
    mut cube_query: Query<(Entity, &mut Transform, &mut JumpData), With<RotatingCube>>,
) {
    for (cube_entity, mut transform, mut data) in &mut cube_query {
        let Ok(target_global) = target_query.get(data.target_entity) else { continue };

        // 1. Resolve Animation Type Once per Animation
        let local_start = target_global.affine().inverse().transform_point3(data.world_start);
        let dist = local_start.distance(data.local_target);

        let anim_type = *data.animation.get_or_insert_with(|| {
            if dist < 4.0 { AnimationType::Slide }
            else if dist < 5.0 { AnimationType::Jump }
            else if dist < 6.0 { AnimationType::Spin }
            else { AnimationType::Flip }
        });

        // 2. Update Timer and Progress
        data.timer += time.delta_secs();
        let t = (data.timer / data.duration).clamp(0.0, 1.0);
        let elastic_t = ElasticInOut.sample_unchecked(t);
        let bounce_t = BounceInOut.sample_unchecked(t);
        let mut bounce_height = 4. * (0.5 - (bounce_t - 0.5).abs());
        let mut local_pos = local_start.lerp(data.local_target, elastic_t);
        let distance_to_go = local_pos.distance(data.local_target);

        // 3. Match on resolved type
        match anim_type {
            AnimationType::Slide => {
                bounce_height = 0.;
                transform.scale = Vec3::splat(1.0);
                transform.rotation = data.start_rotation;
            }
            AnimationType::Jump => {
                local_pos.z += bounce_height;
                let s = 0.5 + (0.5 - t).abs();
                transform.scale = Vec3::new(1.0 + (1.0 - s), s, 1.0 + (1.0 - s));
                transform.rotation = data.start_rotation;
            }
            AnimationType::Spin => {
                // Height and squash/stretch same as jump
                local_pos.z += bounce_height;
                let s = 0.5 + (0.5 - t).abs();
                transform.scale = Vec3::new(1.0 + (1.0 - s), s, 1.0 + (1.0 - s));
                let angle = 2.0 * std::f32::consts::PI * t;
                transform.rotation = data.start_rotation * Quat::from_rotation_y(angle);
            }
            AnimationType::Flip => {
                // Height and squash/stretch same as jump
                local_pos.z += bounce_height;
                let s = 0.5 + (0.5 - t).abs();
                transform.scale = Vec3::new(1.0 + (1.0 - s), s, 1.0 + (1.0 - s));
                let angle = 2.0 * std::f32::consts::PI * t;
                if distance_to_go > 0.5 { transform.rotation = data.start_rotation * Quat::from_rotation_x(angle); }
                else {
                    let (y, _, _) = transform.rotation.to_euler(EulerRot::YXZ);
                    transform.rotation = Quat::from_rotation_y(y);
                }
            }
        }

        // 4. Finalize Position including which axis is up
        let world_pos_horizontal = target_global.transform_point(local_start.lerp(data.local_target, elastic_t));
        let final_world_pos = world_pos_horizontal + Vec3::new(0.0, bounce_height, 0.0);
        transform.translation = final_world_pos;

        if t >= 1.0 {
            transform.scale = Vec3::splat(1.0);
            commands.entity(cube_entity).set_parent_in_place(data.target_entity);
            commands.entity(cube_entity).remove::<JumpData>();
        }
    }
}
// log::info!("Foo: {:#?}", data);
// panic!("Boo! Did line {} scare ya?!?", line!());
