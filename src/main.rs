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
#[require(Transform, Visibility)]
struct Ground;

#[derive(Debug, Component, Reflect)]
#[reflect(Component)]
pub struct Settings {
    pub active: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Self { active: true }
    }
}

#[derive(Debug, Component)]
#[require(Transform, Visibility)]
struct SetAnisotropic;

#[derive(Debug, Component)]
#[require(Transform, Visibility)]
struct SetMipmaps;

#[derive(Debug, Component)]
#[require(Transform, Visibility)]
struct SetResolution;

#[derive(Debug, Component)]
#[require(Transform, Visibility)]
struct SetFps;

#[derive(Debug, Component)]
#[require(Transform, Visibility)]
struct SafetyDisk;

#[derive(Component)]
#[require(Transform, Visibility)]
struct CameraAnchor;

#[derive(Component)]
struct MainCamera;

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

#[derive(Debug, Resource, Default)]
struct EntityTable {
    cube: Option<Entity>,
    disk: Option<Entity>,
    ground: Option<Entity>,
    settings: Option<Entity>,
    set_anisotropic: Option<Entity>,
    set_mipmaps: Option<Entity>,
    set_resolution: Option<Entity>,
    set_fps: Option<Entity>,
    safety_disk: Option<Entity>,
    main_anchor: Option<Entity>,
    main_camera: Option<Entity>,
}

// --- Main ---

fn main() {
    #[cfg(target_arch = "wasm32")]
    console_log::init_with_level(log::Level::Debug).ok();

    App::new()
        .init_resource::<EntityTable>()
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
            embedded_asset!(app, "media/wbtekbg2b512.jpg");
            embedded_asset!(app, "media/settings.jpg");
            embedded_asset!(app, "media/diamond_sprite.jpg");
        })
        .add_plugins(MeshPickingPlugin)
        .add_systems(Startup, setup)
        .add_systems(Update, (rotate_disk, rotate_cube, update_jump, stitch_roundel_system,
            update_camera_zoom, update_mobile_zoom))
        .run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    asset_server: Res<AssetServer>,
    mut et: ResMut<EntityTable>,
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
    et.cube = Some(cube_id);


    let face_data = [ // front/back, right/left, top/bottom
        (Vec3::new(0.0, 0.0, 0.99), Quat::IDENTITY),
        (Vec3::new(0.0, 0.0, -0.99), Quat::from_rotation_y(std::f32::consts::PI)),
        (Vec3::new(0.99, 0.0, 0.0), Quat::from_rotation_y(std::f32::consts::FRAC_PI_2)),
        (Vec3::new(-0.99, 0.0, 0.0), Quat::from_rotation_y(-std::f32::consts::FRAC_PI_2)),
        (Vec3::new(0.0, 0.99, 0.0), Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2)),
        (Vec3::new(0.0, -0.99, 0.0), Quat::from_rotation_x(std::f32::consts::FRAC_PI_2)),
    ];

    commands.entity(cube_id)
    .with_children(|parent| {
        // Core center sphere
        parent.spawn((
            Mesh3d(meshes.add(Sphere::new(0.1).mesh().uv(32, 18))),
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color: Color::srgb(0.75, 0.25, 1.0),
                unlit: true,
                ..default()
            })),
        ));

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
    });

    commands.entity(cube_id)
    .observe(move |mut click: On<Pointer<Click>>| {
        click.propagate(false);
    });

    commands.entity(cube_id)
    .observe(move |mut drag: On<Pointer<Drag>>, mut settings: ResMut<CubeParms>| {
        drag.propagate(false);
        settings.rotation_speed += drag.delta.x * 0.005;
    });

    // 2. The Turntable
    let disk_id = commands.spawn((
        RotatingDisk,
        Mesh3d(meshes.add(Circle::new(4.0).mesh().resolution(128))),
        MeshMaterial3d(materials.add(roundel_mat.clone())),
        Transform::from_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2)),
    )).id();
    et.disk = Some(disk_id);

    commands.entity(disk_id)
    .observe(move |mut drag: On<Pointer<Drag>>, mut settings: ResMut<DiskParms>| {
        drag.propagate(false);
        settings.rotation_speed += drag.delta.x * 0.001;
    });

    // 3. The Safety Zone
    let safety_id = commands.spawn((
        SafetyDisk,
        Mesh3d(meshes.add(Circle::new(5.4).mesh().resolution(128))),
        MeshMaterial3d(materials.add(Color::srgb(0.5, 0.25, 0.0))),
        Transform::from_xyz(0.0, -0.49, 0.0).with_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2)),
    )).id();
    et.safety_disk = Some(safety_id);

    // 4. The Ground
    let ocean_floor_handle = asset_server.load("embedded://bevycube/media/wbtekbg2b512.jpg");
    let settings_handle = asset_server.load("embedded://bevycube/media/settings.jpg");
    let diamond_handle = asset_server.load("embedded://bevycube/media/diamond_sprite.jpg");

    let ocean_floor_mat = StandardMaterial {
        base_color_texture: Some(ocean_floor_handle.clone()),
        alpha_mode: AlphaMode::Opaque,
        cull_mode: Some(bevy::render::render_resource::Face::Back),
        ..default()
    };

    let settings_mat = StandardMaterial {
        base_color_texture: Some(settings_handle.clone()),
        alpha_mode: AlphaMode::Add,
        reflectance: 0.0,
        cull_mode: Some(bevy::render::render_resource::Face::Back),
        ..default()
    };

    let diamond_mat = StandardMaterial {
        base_color_texture: Some(diamond_handle.clone()),
        alpha_mode: AlphaMode::Add,
        reflectance: 0.0,
        cull_mode: Some(bevy::render::render_resource::Face::Back),
        ..default()
    };

    let ground_id = commands.spawn((
        Ground,
        Mesh3d(meshes.add(Plane3d::default().mesh().size(20., 20.))),
        MeshMaterial3d(materials.add(ocean_floor_mat.clone())),
        Transform::from_xyz(0.0, -0.5, 0.0),
    )).id();
    et.ground = Some(ground_id);

    let settings_id = commands.spawn((
        Settings { active: true },
        Mesh3d(meshes.add(Plane3d::default().mesh().size(5.0, 5.0))),
        MeshMaterial3d(materials.add(settings_mat.clone())),
        // Parent is 20x20, so bounds are -10 to +10.
        // Center of 5x5 square in corner is at 7.5.
        Transform::from_xyz(7.5, 0.01, 7.5),
    )).id();
    et.settings = Some(settings_id);

    commands.entity(ground_id).add_child(settings_id);

    let to_local = |pixel: f32| (pixel - 256.0) / 512.0 * 5.0;
    let from_local = |pixel: f32| (pixel / 5.0) * 512.0 + 256.0;

    enum SetCatType {
        Anisotropic,
        Mipmaps,
        AssetResolution,
        FPSDisplay,
    }

    enum SetItem {
        An16, An8, An4, An2, AnOff,
        MMOn, MMOff,
        AResHi, AResMed, AResLow,
        FPSDispOn, FPSDispOff,
    }

    struct SettingsCategory {
        cat: SetCatType,
        y_top: f32,
        y_bot: f32,
        x_bounds: Vec<f32>,
        items: Vec<SetItem>,
    }

    macro_rules! row {
        ($c:ident, $y1:expr, $y2:expr,
        [$($x:expr),*],
        [$($i:ident),*]) =>
        {
            SettingsCategory {
                cat: SetCatType::$c,
                y_top: $y1,
                y_bot: $y2,
                x_bounds: vec![$($x),*],
                items: vec![$(SetItem::$i),*],
            }
        };
    }

    let settings_data = [
        row!( Anisotropic, 140., 185.,
            [ 107., 166., 215., 263., 310., 387. ],
            [ An16, An8, An4, An2, AnOff ]),
        row!( Mipmaps, 230., 275.,
            [ 107., 177., 255.],
            [ MMOn, MMOff ]),
        row!( AssetResolution, 320., 365.,
            [ 107., 206., 350., 433. ],
            [ AResHi, AResMed, AResLow ]),
        row!( FPSDisplay, 410., 455.,
            [ 107., 177., 255.],
            [ FPSDispOn, FPSDispOff ])
    ];

    let set_anisotropic_id = commands.spawn((
        SetAnisotropic,
        Mesh3d(meshes.add(Plane3d::default().mesh().size(5./16., 5./16.))),
        MeshMaterial3d(materials.add(diamond_mat.clone())),
        Transform::from_xyz(to_local(107.+14.), 0.01, to_local(140.+22.)),
    )).id();
    et.set_anisotropic = Some(set_anisotropic_id);

    commands.entity(settings_id).add_child(set_anisotropic_id);

    let set_mipmaps_id = commands.spawn((
        SetMipmaps,
        Mesh3d(meshes.add(Plane3d::default().mesh().size(5./16., 5./16.))),
        MeshMaterial3d(materials.add(diamond_mat.clone())),
        Transform::from_xyz(to_local(107.+14.), 0.01, to_local(230.+22.)),
    )).id();
    et.set_mipmaps = Some(set_mipmaps_id);

    commands.entity(settings_id).add_child(set_mipmaps_id);

    let set_resolution_id = commands.spawn((
        SetResolution,
        Mesh3d(meshes.add(Plane3d::default().mesh().size(5./16., 5./16.))),
        MeshMaterial3d(materials.add(diamond_mat.clone())),
        Transform::from_xyz(to_local(107.+14.), 0.01, to_local(320.+22.)),
    )).id();
    et.set_resolution = Some(set_resolution_id);

    commands.entity(settings_id).add_child(set_resolution_id);

    let set_fps_id = commands.spawn((
        SetFps,
        Mesh3d(meshes.add(Plane3d::default().mesh().size(5./16., 5./16.))),
        MeshMaterial3d(materials.add(diamond_mat.clone())),
        Transform::from_xyz(to_local(107.+14.), 0.01, to_local(410.+22.)),
    )).id();
    et.set_fps = Some(set_fps_id);

    commands.entity(settings_id).add_child(set_fps_id);

    commands.entity(settings_id)
    .observe(move |
        mut click: On<Pointer<Click>>,
        et: Res<EntityTable>,
        mut query: Query<(&mut Settings, &GlobalTransform)>,
        mut diamond_query: Query<&mut Transform, Without<Settings>>,
        // mut cmd: Commands,
    | {
        let Ok((settings, settings_global)) = query.get_mut(click.event_target()) else { return };

        if !settings.active { return; }
        if click.duration.as_millis() > 250 { return; }

        let Some(hit_pos) = click.hit.position else { return };
        let local_hit = settings_global.affine().inverse().transform_point3(hit_pos);
        let px = from_local(local_hit.x);
        let py = from_local(local_hit.z);

        let clicked_data = settings_data.iter()
            .find(|row| py >= row.y_top && py <= row.y_bot)
            .and_then(|row| {
                row.x_bounds.windows(2)
                    .zip(row.items.iter())
                    .find(|(bounds, _)| px >= bounds[0] && px < bounds[1])
                    .map(|(bounds, item)| (&row.cat, item, row.y_top, bounds[0]))
            });

        if let Some((category, _item, y_start, x_start)) = clicked_data {
            match category {
                SetCatType::Anisotropic => {
                    if let Ok(mut transform) = diamond_query.get_mut(et.set_anisotropic.unwrap()) {
                        let new_x = to_local(x_start as f32 + 14.0);
                        let new_z = to_local(y_start as f32 + 22.0);
                        transform.translation = Vec3::new(new_x, 0.01, new_z);
                    }
                    click.propagate(false);
                },
                SetCatType::Mipmaps => {
                    if let Ok(mut transform) = diamond_query.get_mut(et.set_mipmaps.unwrap()) {
                        let new_x = to_local(x_start as f32 + 14.0);
                        let new_z = to_local(y_start as f32 + 22.0);
                        transform.translation = Vec3::new(new_x, 0.01, new_z);
                    }
                    click.propagate(false);
                },
                SetCatType::AssetResolution => {
                    if let Ok(mut transform) = diamond_query.get_mut(et.set_resolution.unwrap()) {
                        let new_x = to_local(x_start as f32 + 14.0);
                        let new_z = to_local(y_start as f32 + 22.0);
                        transform.translation = Vec3::new(new_x, 0.01, new_z);
                    }
                    click.propagate(false);
                },
                SetCatType::FPSDisplay => {
                    if let Ok(mut transform) = diamond_query.get_mut(et.set_fps.unwrap()) {
                        let new_x = to_local(x_start as f32 + 14.0);
                        let new_z = to_local(y_start as f32 + 22.0);
                        transform.translation = Vec3::new(new_x, 0.01, new_z);
                    }
                    click.propagate(false);
                },
            }
        }
    });

    commands.entity(ground_id)
    .observe(move |mut drag: On<Pointer<Drag>>, et: Res<EntityTable>, mut query: Query<&mut Transform>| {
        drag.propagate(false);
        if let Some(mut transform) = et.main_anchor.and_then(|id| query.get_mut(id).ok()) {
            let sensitivity = 0.015;
            transform.translation.x -= drag.delta.x * sensitivity;
            transform.translation.z -= drag.delta.y * sensitivity;
        }
    });

    // common click observer for disk and ground click
    let jump_observer = move |mut click: On<Pointer<Click>>,
                              mut commands: Commands,
                              et: Res<EntityTable>,
                              jump_check: Query<&JumpData>,
                              global_query: Query<&GlobalTransform>| {

        click.propagate(false);
        if click.duration.as_millis() > 250 { return; }
        let Some(hit_pos) = click.hit.position else { return };
        let Some(cube_entity) = et.cube else { return };
        if jump_check.contains(cube_entity) { return; }

        let target_entity = click.event_target(); // Automatically gets disk_id or ground_id
        let Ok(cube_global) = global_query.get(cube_entity) else { return };
        let Ok(target_global) = global_query.get(target_entity) else { return };

        let mut local_target = target_global.affine().inverse().transform_point3(hit_pos);

        // The only "difference" check:
        if target_entity == disk_id {
            local_target.z += 1.0;
        } else {
            local_target.y += 1.0;
        }

        commands.entity(cube_entity).remove_parent_in_place();
        commands.entity(cube_entity).insert(JumpData {
            world_start: cube_global.translation(),
            start_rotation: cube_global.compute_transform().rotation,
            local_target,
            timer: 0.0,
            duration: 3.0,
            target_entity,
            animation: None,
        });
    };

    // attach same observer to both
    commands.entity(ground_id).observe(jump_observer.clone());
    commands.entity(disk_id).observe(jump_observer);

    // 5. Lighting
    commands.spawn((
        PointLight {
            shadows_enabled: true,
            ..default()
        },
        Transform::from_xyz(4.0, 8.0, 4.0),
    ));

    // 6. Camera
    let anchor_id = commands.spawn((
        CameraAnchor,
        Transform::IDENTITY,
    )).id();
    et.main_anchor = Some(anchor_id);

    let camera_id = commands.spawn((
        MainCamera,
        Camera3d::default(),
        // Simple perspective. No scaling_mode field exists here.
        Projection::Perspective(PerspectiveProjection::default()),
        Transform::from_xyz(0.0, 7.5, 15.0).looking_at(Vec3::ZERO, Vec3::Y),
    ))
    .id();
    et.main_camera = Some(camera_id);

    commands.entity(anchor_id).add_child(camera_id);
}

// --- Systems ---

fn update_camera_zoom(
    mut mouse_wheel: MessageReader<bevy::input::mouse::MouseWheel>,
    et: Res<EntityTable>,
    mut query: Query<&mut Transform>,
) {
    if let Some(mut transform) = et.main_camera.and_then(|id| query.get_mut(id).ok()) {
        for event in mouse_wheel.read() {
            let zoom_amount = event.y * 0.005;

            transform.translation.z = (transform.translation.z - zoom_amount).clamp(0.01, 40.0);
            transform.look_at(Vec3::ZERO, Vec3::Y);
        }
    }
}

fn update_mobile_zoom(
    touches: Res<bevy::input::touch::Touches>,
    et: Res<EntityTable>,
    mut query: Query<&mut Transform>,
) {
    // We only zoom if exactly two fingers are on the screen
    let active: Vec<_> = touches.iter().collect();
    if active.len() != 2 { return; }

    if let Some(mut transform) = et.main_camera.and_then(|id| query.get_mut(id).ok()) {
        let p1 = active[0].position();
        let p2 = active[1].position();
        let prev_p1 = active[0].previous_position();
        let prev_p2 = active[1].previous_position();

        // Calculate how much the gap between fingers changed
        let current_dist = p1.distance(p2);
        let prev_dist = prev_p1.distance(prev_p2);
        let pinch_delta = current_dist - prev_dist;

        if pinch_delta.abs() > 0.1 {
            let zoom_speed = 0.05; // Adjust for touch sensitivity
            // Move local Z
            transform.translation.z = (transform.translation.z - pinch_delta * zoom_speed).clamp(0.01, 40.0);
            // Stay focused on the anchor
            transform.look_at(Vec3::ZERO, Vec3::Y);
        }
    }
}

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
                    depth_or_array_layers: 1,
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
    et: Res<EntityTable>,
    mut query: Query<(&mut Transform, &GlobalTransform)>,
    settings: Res<CubeParms>,
    time: Res<Time>,
) {
    let seconds_passed = time.delta_secs();
    if let Some(id) = et.cube {
        if let Ok((mut transform, global_transform)) = query.get_mut(id) {
            let local_up = global_transform.affine().inverse().transform_vector3(Vec3::Y);
            transform.rotate_local_axis(
                Dir3::new_unchecked(local_up.normalize()),
                settings.rotation_speed * seconds_passed
            );
        }
    }
}

fn rotate_disk(
    et: Res<EntityTable>,
    mut query: Query<&mut Transform>,
    time: Res<Time>,
    settings: Res<DiskParms>,
) {
    if let Some(mut transform) = et.disk.and_then(|id| query.get_mut(id).ok()) {
        transform.rotate_local_z(settings.rotation_speed * time.delta_secs());
    }
}

fn update_jump(
    mut commands: Commands,
    time: Res<Time>,
    et: Res<EntityTable>,
    target_query: Query<&GlobalTransform>,
    mut cube_query: Query<(&mut Transform, &mut JumpData)>,
) {
    let Some(cube_entity) = et.cube else { return };
    let Ok((mut transform, mut data)) = cube_query.get_mut(cube_entity) else { return };

    let Ok(target_global) = target_query.get(data.target_entity) else { return };

    // 1. Resolve Animation Type Once per Animation
    let local_start = target_global.affine().inverse().transform_point3(data.world_start);
    let dist = local_start.distance(data.local_target);

    let anim_type = *data.animation.get_or_insert_with(|| {
        if dist < 1.5 { AnimationType::Slide }
        else if dist < 2.5 { AnimationType::Spin }
        else if dist < 4.5 { AnimationType::Jump }
        else { AnimationType::Flip }
    });

    // 2. Update Timer and Progress
    data.timer += time.delta_secs();
    let t = (data.timer / data.duration).clamp(0.0, 1.0);
    let elastic_t = ElasticInOut.sample_unchecked(t);
    let bounce_t = BounceInOut.sample_unchecked(t);
    let mut bounce_height = 4. * (0.5 - (bounce_t - 0.5).abs());
    let s = (2. * (0.5 - t).abs()).clamp(0.5, 1.);

    // 3. Match on resolved type
    match anim_type {
        AnimationType::Slide => {
            bounce_height = 0.;
            let yaw_angle = 2.0 * std::f32::consts::PI * t;
            transform.rotation = data.start_rotation * Quat::from_rotation_y(yaw_angle);
        }
        AnimationType::Spin => {
            bounce_height = 0.;
            let yaw_angle = 4.0 * std::f32::consts::PI * t;
            transform.rotation = data.start_rotation * Quat::from_rotation_y(yaw_angle);
        }
        AnimationType::Jump => {
            let yaw_angle = 6.0 * std::f32::consts::PI * t;
            transform.rotation = data.start_rotation * Quat::from_rotation_y(yaw_angle);
        }
        AnimationType::Flip => {
            let yaw_angle = 6.0 * std::f32::consts::PI * t;
            let pitch_angle = 4.0 * std::f32::consts::PI * (t - 0.0909) * 1.2222;
            if t > 0.0909 && t < 0.9090 {
                transform.rotation = data.start_rotation
                    * Quat::from_rotation_y(yaw_angle) * Quat::from_rotation_x(pitch_angle);
            } else {
                transform.rotation = data.start_rotation * Quat::from_rotation_y(yaw_angle);
            }
        }
    }

    // 4. Finalize Position including which axis is up
    transform.scale = Vec3::splat(s);
    let world_pos_horizontal = target_global.transform_point(local_start.lerp(data.local_target, elastic_t));
    let final_world_pos = world_pos_horizontal + Vec3::new(0., bounce_height, 0.);
    transform.translation = final_world_pos;

    if t >= 1. {
        transform.scale = Vec3::splat(1.);
        commands.entity(cube_entity).set_parent_in_place(data.target_entity);
        commands.entity(cube_entity).remove::<JumpData>();
    }
}
// log::info!("Foo: {:#?}", data);
// panic!("Boo! Did line {} scare ya?!?", line!());
// if let Some(id) = et.set_anisotropic { cmd.entity(id).insert(Transform::from_xyz(0.0, 0.5, 0.0)); }
        // How to make .ktx2 (currently these are 256x256):
        // ktx create --format R8G8B8_SRGB --assign-tf srgb --zstd 20
        // --generate-mipmap --mipmap-filter kaiser in.png out.ktx2
