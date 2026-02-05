pub mod ui;
use crate::ui::*;

use bevy::prelude::*;
use bevy::prelude::EaseFunction::{ ElasticInOut, BounceInOut };
use bevy::math::Affine2;
use bevy::asset::embedded_asset;
use bevy::render::render_resource::*;
use bevy::image::*;

// ==========================================
// 2. COMPONENT & RESOURCE DEFINITIONS
// ==========================================

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

#[derive(Debug, Component)] #[require(Transform, Visibility)] struct SafetyDisk;
#[derive(Component)] #[require(Transform, Visibility)] struct CameraAnchor;
#[derive(Component)] struct MainCamera;

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

#[derive(Resource)]
struct StitchedRoundel {
    handle: Handle<Image>,
}

#[derive(Debug, Resource)]
struct RoundelMipmapLoading {
    handles: [Handle<Image>; 5],
    target_handle: Handle<Image>,
}

#[derive(Debug, Resource)] pub struct DiskParms { pub rotation_speed: f32 }
#[derive(Debug, Resource)] pub struct CubeParms { pub rotation_speed: f32 }

#[derive(Debug, Resource, Default)]
pub struct EntityTable {
    pub cube: Option<Entity>,
    pub disk: Option<Entity>,
    pub ground: Option<Entity>,
    pub settings: Option<Entity>,
    pub set_anisotropic: Option<Entity>,
    pub set_mipmaps: Option<Entity>,
    pub set_resolution: Option<Entity>,
    pub set_fps: Option<Entity>,
    pub safety_disk: Option<Entity>,
    pub main_anchor: Option<Entity>,
    pub main_camera: Option<Entity>,
}

// ==========================================
// 3. INTERNAL PLUGINS
// ==========================================

pub struct DemoAssetsPlugin;
impl Plugin for DemoAssetsPlugin {
    fn build(&self, app: &mut App) {
        embedded_asset!(app, "media/WhiteBearCrab512.jpg");
        embedded_asset!(app, "media/WhiteBearCrab256.jpg");
        embedded_asset!(app, "media/WhiteBearCrab128.jpg");
        embedded_asset!(app, "media/WhiteBearCrab64.jpg");
        embedded_asset!(app, "media/WhiteBearCrab32.jpg");
        embedded_asset!(app, "media/wbtekbg2b512.jpg");
        embedded_asset!(app, "media/settings.jpg");
        embedded_asset!(app, "media/diamond_sprite.jpg");
        app.add_systems(Update, stitch_roundel_system);
    }
}

pub struct PlayerPlugin;
impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<RotatingCube>()
           .add_systems(Update, (rotate_cube, update_jump));
    }
}

pub struct EnvironmentPlugin;
impl Plugin for EnvironmentPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup)
           .add_systems(Update, (rotate_disk, debug_roundel_system));
    }
}

pub struct CameraPlugin;
impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (update_camera_zoom, update_mobile_zoom));
    }
}

// ==========================================
// 4. SYSTEM IMPLEMENTATIONS (Original Logic)
// ==========================================

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    asset_server: Res<AssetServer>,
    mut et: ResMut<EntityTable>,
) {
    let roundel_handle = asset_server.load("embedded://bevycube/media/WhiteBearCrab64.jpg");
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

    let cube_id = commands.spawn((
        RotatingCube,
        Transform::from_xyz(0.0, 1.0, 0.0),
    )).id();
    et.cube = Some(cube_id);

    let face_data = [
        (Vec3::new(0.0, 0.0, 0.99), Quat::IDENTITY),
        (Vec3::new(0.0, 0.0, -0.99), Quat::from_rotation_y(std::f32::consts::PI)),
        (Vec3::new(0.99, 0.0, 0.0), Quat::from_rotation_y(std::f32::consts::FRAC_PI_2)),
        (Vec3::new(-0.99, 0.0, 0.0), Quat::from_rotation_y(-std::f32::consts::FRAC_PI_2)),
        (Vec3::new(0.0, 0.99, 0.0), Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2)),
        (Vec3::new(0.0, -0.99, 0.0), Quat::from_rotation_x(std::f32::consts::FRAC_PI_2)),
    ];

    commands.entity(cube_id).with_children(|parent| {
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

    commands.entity(cube_id).observe(move |mut click: On<Pointer<Click>>| { click.propagate(false); });
    commands.entity(cube_id).observe(move |mut drag: On<Pointer<Drag>>, mut settings: ResMut<CubeParms>| {
        drag.propagate(false);
        settings.rotation_speed += drag.delta.x * 0.005;
    });

    let disk_id = commands.spawn((
        RotatingDisk,
        Mesh3d(meshes.add(Circle::new(4.0).mesh().resolution(128))),
        MeshMaterial3d(materials.add(roundel_mat.clone())),
        Transform::from_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2)),
    )).id();
    et.disk = Some(disk_id);

    commands.entity(disk_id).observe(move |mut drag: On<Pointer<Drag>>, mut settings: ResMut<DiskParms>| {
        drag.propagate(false);
        settings.rotation_speed += drag.delta.x * 0.001;
    });

    let safety_id = commands.spawn((
        SafetyDisk,
        Mesh3d(meshes.add(Circle::new(5.4).mesh().resolution(128))),
        MeshMaterial3d(materials.add(Color::srgb(0.5, 0.25, 0.0))),
        Transform::from_xyz(0.0, -0.49, 0.0).with_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2)),
    )).id();
    et.safety_disk = Some(safety_id);

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
        Transform::from_xyz(7.5, 0.01, 7.5),
    )).id();
    et.settings = Some(settings_id);
    commands.entity(ground_id).add_child(settings_id);

    enum SetCatType { Anisotropic, Mipmaps, AssetResolution, FPSDisplay }
    enum SetItem { An16, An8, An4, An2, AnOff, MMOn, MMOff, AResHi, AResMed, AResLow, FPSDispOn, FPSDispOff }
    struct SettingsCategory { cat: SetCatType, y_top: f32, y_bot: f32, x_bounds: Vec<f32>, items: Vec<SetItem> }

    macro_rules! row {
        ($c:ident, $y1:expr, $y2:expr, [$($x:expr),*], [$($i:ident),*]) => {
            SettingsCategory { cat: SetCatType::$c, y_top: $y1, y_bot: $y2, x_bounds: vec![$($x),*], items: vec![$(SetItem::$i),*] }
        };
    }

    let settings_data = [
        row!( Anisotropic, 140., 185., [ 107., 166., 215., 263., 310., 387. ], [ An16, An8, An4, An2, AnOff ]),
        row!( Mipmaps, 230., 275., [ 107., 177., 255.], [ MMOn, MMOff ]),
        row!( AssetResolution, 320., 365., [ 107., 206., 350., 433. ], [ AResHi, AResMed, AResLow ]),
        row!( FPSDisplay, 410., 455., [ 107., 177., 255.], [ FPSDispOn, FPSDispOff ])
    ];

    let set_anisotropic_id = commands.spawn(( SetAnisotropic, Mesh3d(meshes.add(Plane3d::default().mesh().size(5./16., 5./16.))), MeshMaterial3d(materials.add(diamond_mat.clone())), Transform::from_xyz(to_local(107.+14.), 0.01, to_local(140.+22.)) )).id();
    et.set_anisotropic = Some(set_anisotropic_id);
    commands.entity(settings_id).add_child(set_anisotropic_id);

    let set_mipmaps_id = commands.spawn(( SetMipmaps, Mesh3d(meshes.add(Plane3d::default().mesh().size(5./16., 5./16.))), MeshMaterial3d(materials.add(diamond_mat.clone())), Transform::from_xyz(to_local(107.+14.), 0.01, to_local(230.+22.)) )).id();
    et.set_mipmaps = Some(set_mipmaps_id);
    commands.entity(settings_id).add_child(set_mipmaps_id);

    let set_resolution_id = commands.spawn(( SetResolution, Mesh3d(meshes.add(Plane3d::default().mesh().size(5./16., 5./16.))), MeshMaterial3d(materials.add(diamond_mat.clone())), Transform::from_xyz(to_local(107.+14.), 0.01, to_local(320.+22.)) )).id();
    et.set_resolution = Some(set_resolution_id);
    commands.entity(settings_id).add_child(set_resolution_id);

    let set_fps_id = commands.spawn(( SetFps, Mesh3d(meshes.add(Plane3d::default().mesh().size(5./16., 5./16.))), MeshMaterial3d(materials.add(diamond_mat.clone())), Transform::from_xyz(to_local(107.+14.), 0.01, to_local(410.+22.)) )).id();
    et.set_fps = Some(set_fps_id);
    commands.entity(settings_id).add_child(set_fps_id);

    commands.entity(settings_id).observe(move |
        mut click: On<Pointer<Click>>,
        et: Res<EntityTable>,
        stitched: Option<Res<StitchedRoundel>>,
        mut query: Query<(&mut Settings, &GlobalTransform)>,
        mut diamond_query: Query<&mut Transform, Without<Settings>>,
        mut images: ResMut<Assets<Image>>,
        mut materials: ResMut<Assets<StandardMaterial>>,
    | {
        let Ok((settings, settings_global)) = query.get_mut(click.event_target()) else { return };
        if !settings.active || click.duration.as_millis() > 250 { return; }
        let Some(hit_pos) = click.hit.position else { return };
        let local_hit = settings_global.affine().inverse().transform_point3(hit_pos);
        let px = from_local(local_hit.x);
        let py = from_local(local_hit.z);

        let clicked_data = settings_data.iter()
            .find(|row| py >= row.y_top && py <= row.y_bot)
            .and_then(|row| {
                row.x_bounds.windows(2).zip(row.items.iter())
                    .find(|(bounds, _)| px >= bounds[0] && px < bounds[1])
                    .map(|(bounds, item)| (&row.cat, item, row.y_top, bounds[0]))
            });

        if let Some((category, item, y_start, x_start)) = clicked_data {
            let Some(ref stitched_res) = stitched else { return };
            let target_handle = &stitched_res.handle;
            match category {
                SetCatType::Anisotropic => {
                    if let Ok(mut transform) = diamond_query.get_mut(et.set_anisotropic.unwrap()) {
                        transform.translation = Vec3::new(to_local(x_start + 14.0), 0.01, to_local(y_start + 22.0));
                    }
                    if let Some(img) = images.get_mut(target_handle) {
                        let mut is_desc = match img.sampler.clone() { ImageSampler::Descriptor(d) => d, _ => ImageSamplerDescriptor::default() };
                        is_desc.anisotropy_clamp = match item { SetItem::An16 => 16, SetItem::An8 => 8, SetItem::An4 => 4, SetItem::An2 => 2, _ => 1 };
                        if is_desc.anisotropy_clamp > 1 {
                            is_desc.mipmap_filter = ImageFilterMode::Linear;
                            if let Ok(mut mip_transform) = diamond_query.get_mut(et.set_mipmaps.unwrap()) {
                                mip_transform.translation = Vec3::new(to_local(107. + 14.0), 0.01, to_local(230. + 22.0));
                            }
                        }
                        img.sampler = ImageSampler::Descriptor(is_desc);
                        for (_, mat) in materials.iter_mut() {
                            if mat.base_color_texture.as_ref().map(|h| h.id() == target_handle.id()).unwrap_or(false) { mat.base_color_texture = Some(target_handle.clone()); }
                        }
                    }
                },
                SetCatType::Mipmaps => {
                    if let Ok(mut transform) = diamond_query.get_mut(et.set_mipmaps.unwrap()) {
                        transform.translation = Vec3::new(to_local(x_start + 14.0), 0.01, to_local(y_start + 22.0));
                    }
                    if let Some(img) = images.get_mut(target_handle) {
                        let is_desc = match img.sampler.clone() { ImageSampler::Descriptor(d) => d, _ => ImageSamplerDescriptor::default() };
                        if let Ok(mut aniso_transform) = diamond_query.get_mut(et.set_anisotropic.unwrap()) {
                            aniso_transform.translation = Vec3::new(to_local(310. + 14.0), 0.01, to_local(140. + 22.0));
                        }
                        img.sampler = ImageSampler::Descriptor(is_desc);
                        for (_, mat) in materials.iter_mut() {
                            if mat.base_color_texture.as_ref().map(|h| h.id() == target_handle.id()).unwrap_or(false) { mat.base_color_texture = Some(target_handle.clone()); }
                        }
                    }
                },
                SetCatType::AssetResolution => {
                    if let Ok(mut transform) = diamond_query.get_mut(et.set_resolution.unwrap()) {
                        transform.translation = Vec3::new(to_local(x_start + 14.0), 0.01, to_local(y_start + 22.0));
                    }
                    if let Some(img) = images.get_mut(target_handle) {
                        let mut is_desc = match img.sampler.clone() { ImageSampler::Descriptor(d) => d, _ => ImageSamplerDescriptor::default() };
                        is_desc.lod_min_clamp = match item { SetItem::AResHi => 0., SetItem::AResMed => 1., SetItem::AResLow => 2., _ => 3. };
                        img.sampler = ImageSampler::Descriptor(is_desc);
                        for (_, mat) in materials.iter_mut() {
                            if mat.base_color_texture.as_ref().map(|h| h.id() == target_handle.id()).unwrap_or(false) { mat.base_color_texture = Some(target_handle.clone()); }
                        }
                    }
                },
                SetCatType::FPSDisplay => {
                    if let Ok(mut transform) = diamond_query.get_mut(et.set_fps.unwrap()) {
                        transform.translation = Vec3::new(to_local(x_start + 14.0), 0.01, to_local(y_start + 22.0));
                    }
                },
            }
            click.propagate(false);
        }
    });

    commands.entity(ground_id).observe(move |mut drag: On<Pointer<Drag>>, et: Res<EntityTable>, mut query: Query<&mut Transform>| {
        drag.propagate(false);
        if let Some(mut transform) = et.main_anchor.and_then(|id| query.get_mut(id).ok()) {
            transform.translation.x -= drag.delta.x * 0.015;
            transform.translation.z -= drag.delta.y * 0.015;
        }
    });

    let jump_observer = move |mut click: On<Pointer<Click>>, mut commands: Commands, et: Res<EntityTable>, jump_check: Query<&JumpData>, global_query: Query<&GlobalTransform>| {
        click.propagate(false);
        if click.duration.as_millis() > 250 { return; }
        let Some(hit_pos) = click.hit.position else { return };
        let Some(cube_entity) = et.cube else { return };
        if jump_check.contains(cube_entity) { return; }

        let target_entity = click.event_target();
        let Ok(cube_global) = global_query.get(cube_entity) else { return };
        let Ok(target_global) = global_query.get(target_entity) else { return };

        let mut local_target = target_global.affine().inverse().transform_point3(hit_pos);
        if target_entity == disk_id { local_target.z += 1.0; } else { local_target.y += 1.0; }

        commands.entity(cube_entity).remove_parent_in_place();
        commands.entity(cube_entity).insert(JumpData {
            world_start: cube_global.translation(),
            start_rotation: cube_global.compute_transform().rotation,
            local_target, timer: 0.0, duration: 3.0, target_entity, animation: None,
        });
    };

    commands.entity(ground_id).observe(jump_observer.clone());
    commands.entity(disk_id).observe(jump_observer);

    commands.spawn(( PointLight { shadows_enabled: true, ..default() }, Transform::from_xyz(4.0, 8.0, 4.0) ));

    let anchor_id = commands.spawn(( CameraAnchor, Transform::IDENTITY )).id();
    et.main_anchor = Some(anchor_id);

    let camera_id = commands.spawn(( MainCamera, Camera3d::default(), Projection::Perspective(PerspectiveProjection::default()), Transform::from_xyz(0.0, 7.5, 15.0).looking_at(Vec3::ZERO, Vec3::Y) )).id();
    et.main_camera = Some(camera_id);
    commands.entity(anchor_id).add_child(camera_id);
}

fn update_camera_zoom(mut mouse_wheel: MessageReader<bevy::input::mouse::MouseWheel>, et: Res<EntityTable>, mut query: Query<&mut Transform>) {
    if let Some(mut transform) = et.main_camera.and_then(|id| query.get_mut(id).ok()) {
        for event in mouse_wheel.read() {
            let zoom_amount = event.y * 0.005;
            transform.translation.z = (transform.translation.z - zoom_amount).clamp(0.01, 40.0);
            transform.look_at(Vec3::ZERO, Vec3::Y);
        }
    }
}

fn update_mobile_zoom(touches: Res<bevy::input::touch::Touches>, et: Res<EntityTable>, mut query: Query<&mut Transform>) {
    let active: Vec<_> = touches.iter().collect();
    if active.len() != 2 { return; }
    if let Some(mut transform) = et.main_camera.and_then(|id| query.get_mut(id).ok()) {
        let pinch_delta = active[0].position().distance(active[1].position()) - active[0].previous_position().distance(active[1].previous_position());
        if pinch_delta.abs() > 0.1 {
            transform.translation.z = (transform.translation.z - pinch_delta * 0.05).clamp(0.01, 40.0);
            transform.look_at(Vec3::ZERO, Vec3::Y);
        }
    }
}

fn stitch_roundel_system(mut commands: Commands, loading: Option<Res<RoundelMipmapLoading>>, mut images: ResMut<Assets<Image>>, mut materials: ResMut<Assets<StandardMaterial>>) {
    let Some(loading) = loading else { return };
    if loading.handles.iter().all(|h| images.get(h).is_some()) {
        let mut combined_data = Vec::new();
        let format = images.get(&loading.handles[0]).unwrap().texture_descriptor.format;
        for h in &loading.handles { if let Some(ref data) = images.get(h).unwrap().data { combined_data.extend_from_slice(data); } else { return; } }
        let final_handle = images.add(Image {
            data: Some(combined_data),
            texture_descriptor: TextureDescriptor { label: Some("stitched"), size: Extent3d { width: 512, height: 512, depth_or_array_layers: 1 }, mip_level_count: loading.handles.len() as u32, sample_count: 1, dimension: TextureDimension::D2, format, usage: TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST, view_formats: &[] },
            sampler: ImageSampler::Descriptor(ImageSamplerDescriptor { mipmap_filter: ImageFilterMode::Linear, mag_filter: ImageFilterMode::Linear, min_filter: ImageFilterMode::Linear, anisotropy_clamp: 16, ..default() }), ..default()
        });
        for (_, mat) in materials.iter_mut() { if let Some(ref tex) = mat.base_color_texture { if tex.id() == loading.target_handle.id() { mat.base_color_texture = Some(final_handle.clone()); } } }
        commands.insert_resource(StitchedRoundel { handle: final_handle });
        commands.remove_resource::<RoundelMipmapLoading>();
    }
}

fn rotate_cube(et: Res<EntityTable>, mut query: Query<(&mut Transform, &GlobalTransform)>, settings: Res<CubeParms>, time: Res<Time>) {
    if let Some(id) = et.cube {
        if let Ok((mut transform, global)) = query.get_mut(id) {
            let local_up = global.affine().inverse().transform_vector3(Vec3::Y);
            transform.rotate_local_axis(Dir3::new_unchecked(local_up.normalize()), settings.rotation_speed * time.delta_secs());
        }
    }
}

fn rotate_disk(et: Res<EntityTable>, mut query: Query<&mut Transform>, time: Res<Time>, settings: Res<DiskParms>) {
    if let Some(mut transform) = et.disk.and_then(|id| query.get_mut(id).ok()) { transform.rotate_local_z(settings.rotation_speed * time.delta_secs()); }
}

fn update_jump(mut commands: Commands, time: Res<Time>, et: Res<EntityTable>, target_query: Query<&GlobalTransform>, mut cube_query: Query<(&mut Transform, &mut JumpData)>) {
    let Some(cube_entity) = et.cube else { return };
    let Ok((mut transform, mut data)) = cube_query.get_mut(cube_entity) else { return };
    let Ok(target_global) = target_query.get(data.target_entity) else { return };
    let local_start = target_global.affine().inverse().transform_point3(data.world_start);
    let target_pos = data.local_target;
    let anim_type = *data.animation.get_or_insert_with(|| {
        let d = local_start.distance(target_pos);
        if d < 1.5 { AnimationType::Slide }
        else if d < 2.5 { AnimationType::Spin }
        else if d < 4.5 { AnimationType::Jump }
        else { AnimationType::Flip }
    });
    data.timer += time.delta_secs();
    let t = (data.timer / data.duration).clamp(0.0, 1.0);
    let (elastic_t, bounce_t) = (ElasticInOut.sample_unchecked(t), BounceInOut.sample_unchecked(t));
    let mut bounce_height = 4. * (0.5 - (bounce_t - 0.5).abs());
    match anim_type {
        AnimationType::Slide | AnimationType::Spin => { bounce_height = 0.; transform.rotation = data.start_rotation * Quat::from_rotation_y(t * std::f32::consts::PI * if anim_type == AnimationType::Slide { 2. } else { 4. }); }
        AnimationType::Jump => { transform.rotation = data.start_rotation * Quat::from_rotation_y(t * std::f32::consts::PI * 6.0); }
        AnimationType::Flip => { let yaw = 6. * std::f32::consts::PI * t; if t > 0.0909 && t < 0.9090 { transform.rotation = data.start_rotation * Quat::from_rotation_y(yaw) * Quat::from_rotation_x(4. * std::f32::consts::PI * (t - 0.0909) * 1.2222); } else { transform.rotation = data.start_rotation * Quat::from_rotation_y(yaw); } }
    }
    transform.scale = Vec3::splat((2. * (0.5 - t).abs()).clamp(0.5, 1.));
    transform.translation = target_global.transform_point(local_start.lerp(data.local_target, elastic_t)) + Vec3::new(0., bounce_height, 0.);
    if t >= 1. { transform.scale = Vec3::splat(1.); commands.entity(cube_entity).set_parent_in_place(data.target_entity); commands.entity(cube_entity).remove::<JumpData>(); }
}

fn debug_roundel_system(roundel: Option<Res<StitchedRoundel>>, images: Res<Assets<Image>>) {
    if let Some(roundel) = roundel { if let Some(img) = images.get(&roundel.handle) { log::info!("Actual Mip Levels: {}", img.texture_descriptor.mip_level_count); } }
}
