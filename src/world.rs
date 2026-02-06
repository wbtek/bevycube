// MIT License
//
// Copyright (c) 2026 - WBTek: Greg Slocum
// Division of WhiteBear Family, Inc.
//
// Permission is hereby granted, free of charge, to any person obtaining a copy
// of this software and associated documentation files (the "Software"), to deal
// in the Software without restriction, including without limitation the rights
// to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
// copies of the Software, and to permit persons to whom the Software is
// furnished to do so, subject to the following conditions:
//
// The above copyright notice and this permission notice shall be included in all
// copies or substantial portions of the Software.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
// IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
// FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
// AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
// LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
// OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
// SOFTWARE.

use crate::{camera::*, roundel, DiskParms, EntityTable, OceanBuffer};
use bevy::mesh::VertexAttributeValues;
use bevy::prelude::*;

pub mod cube;

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
pub struct Ocean;

#[derive(Debug, Component)]
#[require(Transform, Visibility)]
pub struct SafetyDisk;

#[derive(Debug, Component)]
#[require(Transform, Visibility)]
pub struct SafetyDiskHidden;

pub struct WorldPlugin;

impl Plugin for WorldPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(Time::<Fixed>::from_seconds(1.0 / 10.0));
        app.add_systems(FixedUpdate, simulate_waves);
        app.register_type::<RotatingCube>()
            .add_systems(Startup, setup)
            .add_systems(
                Update,
                (
                    cube::rotate_cube,
                    rotate_disk,
                    cube::update_jump,
                    // simulate_waves,
                    apply_camera_repulsion,
                    update_ocean_mesh,
                )
                    .chain(),
            );
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
    cube::spawn_rotating_cube(
        &mut commands,
        &mut meshes,
        &mut materials,
        roundel_mat.clone(),
        &mut et,
    );

    // Disk Spawning
    let disk_id = commands
        .spawn((
            RotatingDisk,
            Mesh3d(meshes.add(Circle::new(4.0).mesh().resolution(128))),
            MeshMaterial3d(materials.add(roundel_mat.clone())),
            Transform::from_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2)),
        ))
        .id();
    et.disk = Some(disk_id);

    commands.entity(disk_id).observe(
        |mut drag: On<Pointer<Drag>>, mut settings: ResMut<DiskParms>| {
            drag.propagate(false);
            settings.rotation_speed += drag.delta.x * 0.001;
        },
    );

    let safety_id = commands
        .spawn((
            SafetyDisk,
            Mesh3d(meshes.add(Circle::new(5.4).mesh().resolution(128))),
            MeshMaterial3d(materials.add(Color::srgb(0.5, 0.25, 0.0))),
            // Transform::from_xyz(0.0, -0.49, 0.0)
            Transform::from_xyz(0.0, -0.99, 0.0)
                .with_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2)),
        ))
        .id();
    et.safety_disk = Some(safety_id);

    let safety_hidden_id = commands
        .spawn((
            SafetyDiskHidden,
            Mesh3d(meshes.add(Circle::new(5.4).mesh().resolution(128))),
            Transform::from_xyz(0.0, -0.01, 0.0)
                .with_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2)),
        ))
        .id();
    et.safety_disk_hidden = Some(safety_hidden_id);

    // Ground and Environment
    let ocean_floor_handle = asset_server.load("embedded://bevycube/media/wbtekbg2b512.jpg");
    let settings_handle = asset_server.load("embedded://bevycube/media/settings.jpg");
    let diamond_handle = asset_server.load("embedded://bevycube/media/diamond_sprite.jpg");

    let ground_id = commands
        .spawn((
            Ground,
            Mesh3d(meshes.add(Plane3d::default().mesh().size(20., 20.))),
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color_texture: Some(ocean_floor_handle),
                ..default()
            })),
            // Transform::from_xyz(0.0, -0.5, 0.0),
            Transform::from_xyz(0.0, -1.0, 0.0),
        ))
        .id();
    et.ground = Some(ground_id);

    let grid_size = 12;
    commands.insert_resource(OceanBuffer::new(grid_size));

    let ocean_mesh = Plane3d::default().mesh().size(23.0, 23.0).subdivisions(10);

    let ocean_id = commands
        .spawn((
            Ocean,
            Mesh3d(meshes.add(ocean_mesh)),
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color: Color::srgba(0.0, 0.3, 0.6, 0.4), // Sea blue, 40% opaque
                alpha_mode: AlphaMode::Blend,
                perceptual_roughness: 0.08, // Shiny surface
                metallic: 0.2,
                ..default()
            })),
            // Transform::from_xyz(0.0, -0.25, 0.0),
            Transform::from_xyz(0.0, -0.25, 0.0),
        ))
        .id();
    et.ocean = Some(ocean_id);

    // Observers
    commands.entity(ground_id).observe(
        |mut drag: On<Pointer<Drag>>, et: Res<EntityTable>, mut query: Query<&mut Transform>| {
            drag.propagate(false);
            if let Some(mut transform) = et.main_anchor.and_then(|id| query.get_mut(id).ok()) {
                transform.translation.x -= drag.delta.x * 0.015;
                transform.translation.z -= drag.delta.y * 0.015;
            }
        },
    );

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
        &mut et,
    );

    commands.spawn((
        PointLight {
            shadows_enabled: true,
            ..default()
        },
        Transform::from_xyz(-7.0, 10.0, -7.0),
    ));

    commands.spawn((
        PointLight {
            shadows_enabled: true,
            ..default()
        },
        Transform::from_xyz(7.0, 10.0, -7.0),
    ));

    commands.spawn((
        PointLight {
            shadows_enabled: true,
            ..default()
        },
        Transform::from_xyz(4.0, 8.0, 4.0),
    ));

    let anchor_id = commands.spawn((CameraAnchor, Transform::IDENTITY)).id();
    et.main_anchor = Some(anchor_id);

    let camera_id = commands
        .spawn((
            MainCamera,
            Camera3d::default(),
            Projection::Perspective(PerspectiveProjection::default()),
            Transform::from_xyz(0.0, 7.5, 15.0).looking_at(Vec3::ZERO, Vec3::Y),
        ))
        .id();
    et.main_camera = Some(camera_id);
    commands.entity(anchor_id).add_child(camera_id);

    commands.entity(ocean_id).observe(
        |mut drag: On<Pointer<Drag>>, et: Res<EntityTable>, mut query: Query<&mut Transform>| {
            drag.propagate(false);
            if let Some(mut transform) = et.main_anchor.and_then(|id| query.get_mut(id).ok()) {
                transform.translation.x -= drag.delta.x * 0.015;
                transform.translation.z -= drag.delta.y * 0.015;
            }
        },
    );

    commands
        .entity(ground_id)
        .observe(cube::handle_jump_request);
    commands.entity(ocean_id).observe(cube::handle_jump_request);
    commands.entity(disk_id).observe(cube::handle_jump_request);
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

fn apply_camera_repulsion(
    mut water: ResMut<OceanBuffer>,
    et: Res<EntityTable>,
    global_transforms: Query<&GlobalTransform>,
) {
    let (Some(cam_id), Some(anchor_id)) = (et.main_camera, et.main_anchor) else {
        return;
    };

    // Get world positions
    let Ok(cam_gtf) = global_transforms.get(cam_id) else {
        return;
    };
    let Ok(anchor_gtf) = global_transforms.get(anchor_id) else {
        return;
    };

    let cam_pos = cam_gtf.translation();
    let anchor_pos = anchor_gtf.translation();

    // Calculate 3D distance for the "zoom" level
    let dist = cam_pos.distance(anchor_pos);

    // radius grows as distance shrinks (15.0 is your start dist)
    let repulsion_radius = ((15.0 - dist) / 15.0 * 6.0).max(0.0);
    let r_sq = repulsion_radius * repulsion_radius;

    // Depth: Sink below ground (-0.5) when close
    // let push_depth = ((15.0 - dist) / 15.0 * -0.6).min(0.0);
    let push_depth = ((15.0 - dist) / 15.0 * -2.0).min(0.0);

    let size = water.size;
    let step = 2.0;
    let anchor_xz = anchor_pos.xz();

    for z in 1..size - 1 {
        for x in 1..size - 1 {
            let i = z * size + x;
            let w_pos = Vec2::new((x as f32 * step) - 10.0, (z as f32 * step) - 10.0);

            if w_pos.distance_squared(anchor_xz) < r_sq {
                water.current[i] = push_depth;
            }
        }
    }
}

fn simulate_waves(
    mut water: ResMut<OceanBuffer>,
    et: Res<EntityTable>,
    global_transforms: Query<&GlobalTransform>,
) {
    let Some(disk_id) = et.disk else { return };
    let Ok(disk_gtf) = global_transforms.get(disk_id) else {
        return;
    };
    let disk_xz = disk_gtf.translation().xz();

    let size = water.size;
    for z in 1..size - 1 {
        for x in 1..size - 1 {
            let i = z * size + x;
            let w_pos = Vec2::new((x as f32 * 2.0) - 10.0, (z as f32 * 2.0) - 10.0);

            // Mirror boundary at Disk radius (4.0)
            if w_pos.distance_squared(disk_xz) < 16.5 {
                water.previous[i] = 0.0;
                continue;
            }

            // let avg = (water.current[i-1] + water.current[i+1] +
            //            water.current[i-size] + water.current[i+size]) / 4.0;
            let avg = (water.current[i - size - 1]
                + water.current[i - size]
                + water.current[i - size + 1]
                + water.current[i - 1]
                + water.current[i]
                + water.current[i + 1]
                + water.current[i + size - 1]
                + water.current[i + size]
                + water.current[i + size + 1])
                / 9.0;

            water.previous[i] = (avg * 2.0 - water.previous[i]) * 0.98;
        }
    }
    water.swap();
}

fn update_ocean_mesh(
    water: Res<OceanBuffer>,
    mut meshes: ResMut<Assets<Mesh>>,
    query: Query<&Mesh3d, With<Ocean>>,
) {
    // A loop is the most robust way to avoid the get_single/single drama
    for mesh_3d in &query {
        if let Some(mesh) = meshes.get_mut(&mesh_3d.0) {
            if let Some(VertexAttributeValues::Float32x3(pos)) =
                mesh.attribute_mut(Mesh::ATTRIBUTE_POSITION)
            {
                for (i, p) in pos.iter_mut().enumerate() {
                    // Safety check to ensure we don't overflow the buffer
                    if i < water.current.len() {
                        p[1] = water.current[i];
                    }
                }
            }
        }
    }
}
