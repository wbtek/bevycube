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

use crate::{camera::*, roundel, EntityTable};
use bevy::prelude::*;

pub mod cube;
pub mod disk;
pub mod ground;
pub mod ocean;

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
        app.add_systems(FixedUpdate, ocean::simulate_waves);
        app.register_type::<RotatingCube>()
            .add_systems(Startup, setup)
            .add_systems(
                Update,
                (
                    cube::rotate_cube,
                    disk::rotate_disk,
                    cube::update_jump,
                    ocean::apply_camera_repulsion,
                    ocean::update_ocean_mesh,
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
    let _cube_id = cube::spawn_rotating_cube(
        &mut commands,
        &mut meshes,
        &mut materials,
        roundel_mat.clone(),
        &mut et,
    );

    // Disk Spawning
    let disk_id = disk::spawn_rotating_disk(
        &mut commands,
        &mut meshes,
        &mut materials,
        roundel_mat.clone(),
        &mut et,
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

    let ocean_id = ocean::spawn_ocean(&mut commands, &mut meshes, &mut materials, &mut et);

    let ground_id = ground::spawn_ground(
        &mut commands,
        &mut meshes,
        &mut materials,
        ocean_floor_handle,
        &mut et,
    );

    // Observers
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

    commands
        .entity(ground_id)
        .observe(cube::handle_jump_request);
    commands.entity(ocean_id).observe(cube::handle_jump_request);
    commands.entity(disk_id).observe(cube::handle_jump_request);
}
