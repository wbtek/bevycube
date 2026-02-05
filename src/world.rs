use bevy::prelude::*;
use bevy::prelude::EaseFunction::{ElasticInOut, BounceInOut};
use crate::{ui::Settings, EntityTable, roundel, DiskParms, CubeParms, camera::*};

// --- Components ---
#[derive(Debug, Component, Default, Reflect)]
#[reflect(Component)]
#[require(Transform, Visibility)]
pub struct RotatingCube;

#[derive(Debug, Component, Default, Reflect)]
#[reflect(Component)]
#[require(Transform, Visibility)]
pub struct RotatingDisk;

#[derive(Debug, Component)]
#[require(Transform, Visibility)]
pub struct Ground;

#[derive(Debug, Component)] 
#[require(Transform, Visibility)] 
pub struct SafetyDisk;

#[derive(Debug, Clone, Copy, PartialEq, Reflect)]
pub enum AnimationType {
    Jump, Slide, Spin, Flip,
}

#[derive(Debug, Component)]
pub struct JumpData {
    pub world_start: Vec3,
    pub start_rotation: Quat,
    pub local_target: Vec3,
    pub timer: f32,
    pub duration: f32,
    pub target_entity: Entity,
    pub animation: Option<AnimationType>,
}

pub struct WorldPlugin;

impl Plugin for WorldPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<RotatingCube>()
           .add_systems(Startup, setup)
           .add_systems(Update, (rotate_cube, rotate_disk, update_jump));
    }
}

pub fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    asset_server: Res<AssetServer>,
    mut et: ResMut<EntityTable>,
) {
    let roundel_handle = asset_server.load("embedded://bevycube/media/WhiteBearCrab64.jpg");
    let roundel_mat = roundel::get_roundel_material(roundel_handle.clone());

    // Notify roundel module to start stitching
    let handles = [
        asset_server.load("embedded://bevycube/media/WhiteBearCrab512.jpg"),
        asset_server.load("embedded://bevycube/media/WhiteBearCrab256.jpg"),
        asset_server.load("embedded://bevycube/media/WhiteBearCrab128.jpg"),
        asset_server.load("embedded://bevycube/media/WhiteBearCrab64.jpg"),
        asset_server.load("embedded://bevycube/media/WhiteBearCrab32.jpg"),
    ];
    commands.insert_resource(roundel::RoundelMipmapLoading {
        handles,
        target_handle: roundel_handle.clone(),
    });

    // Cube Spawning
    let cube_id = commands.spawn((RotatingCube, Transform::from_xyz(0.0, 1.0, 0.0))).id();
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
            MeshMaterial3d(materials.add(StandardMaterial { base_color: Color::srgb(0.75, 0.25, 1.0), unlit: true, ..default() })),
        ));
        for (offset, rotation) in face_data {
            parent.spawn((Mesh3d(meshes.add(Circle::new(0.90).mesh().resolution(128))), MeshMaterial3d(materials.add(roundel_mat.clone())), Transform::from_translation(offset).with_rotation(rotation)));
            parent.spawn((Mesh3d(meshes.add(Circle::new(0.90).mesh().resolution(128))), MeshMaterial3d(materials.add(StandardMaterial { emissive: LinearRgba::from(Color::srgb(0.75, 0.25, 1.0)) * 0.03, ..roundel_mat.clone() })), Transform { translation: offset * 0.99, rotation: rotation * Quat::from_rotation_y(std::f32::consts::PI), scale: Vec3::splat(0.995) }));
        }
    });

    commands.entity(cube_id).observe(|mut click: On<Pointer<Click>>| { click.propagate(false); });
    commands.entity(cube_id).observe(|mut drag: On<Pointer<Drag>>, mut settings: ResMut<CubeParms>| {
        drag.propagate(false);
        settings.rotation_speed += drag.delta.x * 0.005;
    });

    // Disk Spawning
    let disk_id = commands.spawn((
        RotatingDisk,
        Mesh3d(meshes.add(Circle::new(4.0).mesh().resolution(128))),
        MeshMaterial3d(materials.add(roundel_mat.clone())),
        Transform::from_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2)),
    )).id();
    et.disk = Some(disk_id);

    commands.entity(disk_id).observe(|mut drag: On<Pointer<Drag>>, mut settings: ResMut<DiskParms>| {
        drag.propagate(false);
        settings.rotation_speed += drag.delta.x * 0.001;
    });

    let safety_id = commands.spawn((SafetyDisk, Mesh3d(meshes.add(Circle::new(5.4).mesh().resolution(128))), MeshMaterial3d(materials.add(Color::srgb(0.5, 0.25, 0.0))), Transform::from_xyz(0.0, -0.49, 0.0).with_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2)))).id();
    et.safety_disk = Some(safety_id);

    // Ground and Environment
    let ocean_floor_handle = asset_server.load("embedded://bevycube/media/wbtekbg2b512.jpg");
    let settings_handle = asset_server.load("embedded://bevycube/media/settings.jpg");
    let diamond_handle = asset_server.load("embedded://bevycube/media/diamond_sprite.jpg");

    let ground_id = commands.spawn((
        Ground,
        Mesh3d(meshes.add(Plane3d::default().mesh().size(20., 20.))),
        MeshMaterial3d(materials.add(StandardMaterial { base_color_texture: Some(ocean_floor_handle), ..default() })),
        Transform::from_xyz(0.0, -0.5, 0.0),
    )).id();
    et.ground = Some(ground_id);

    let settings_id = commands.spawn((
        Settings { active: true },
        Mesh3d(meshes.add(Plane3d::default().mesh().size(5.0, 5.0))),
        MeshMaterial3d(materials.add(StandardMaterial { base_color_texture: Some(settings_handle.clone()), alpha_mode: AlphaMode::Add, reflectance: 0.0, perceptual_roughness: 1.0, ..default() })),
        Transform::from_xyz(7.5, 0.01, 7.5),
    )).id();
    et.settings = Some(settings_id);
    commands.entity(ground_id).add_child(settings_id);

    // Observers
    commands.entity(ground_id).observe(|mut drag: On<Pointer<Drag>>, et: Res<EntityTable>, mut query: Query<&mut Transform>| {
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

    crate::ui::spawn_settings_ui(
        &mut commands,
        &mut meshes,
        materials.add(StandardMaterial {
            base_color_texture: Some(settings_handle),
            alpha_mode: AlphaMode::Add,
            reflectance: 0.0,
            ..default()
        }),
        materials.add(StandardMaterial {
            base_color_texture: Some(diamond_handle),
            alpha_mode: AlphaMode::Add,
            reflectance: 0.0,
            ..default()
        }),
        ground_id,
        &mut et
    );

    commands.spawn(( PointLight { shadows_enabled: true, ..default() }, Transform::from_xyz(4.0, 8.0, 4.0) ));

    let anchor_id = commands.spawn(( CameraAnchor, Transform::IDENTITY )).id();
    et.main_anchor = Some(anchor_id);

    let camera_id = commands.spawn(( MainCamera, Camera3d::default(), Projection::Perspective(PerspectiveProjection::default()), Transform::from_xyz(0.0, 7.5, 15.0).looking_at(Vec3::ZERO, Vec3::Y) )).id();
    et.main_camera = Some(camera_id);
    commands.entity(anchor_id).add_child(camera_id);
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
        if d < 1.5 { AnimationType::Slide } else if d < 2.5 { AnimationType::Spin } else if d < 4.5 { AnimationType::Jump } else { AnimationType::Flip }
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
