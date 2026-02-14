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

use crate::{roundel, world::camera::CameraAnchorRes, EntityTable};
use bevy::prelude::*;

pub mod camera;
pub mod cube;
pub mod disk;
pub mod ground;
pub mod lights;
pub mod ocean;

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
    app.insert_resource(ground::GroundConfig { world_y: -2.0 });
    app.insert_resource(camera::CameraAnchorRes::default());
    app.add_systems(
      FixedUpdate,
      (
        ocean::simulate_waves,
        ocean::apply_camera_repulsion,
        ocean::clamp_edges,
        ocean::swap_and_copy,
        ocean::update_ocean_mesh,
      )
        .chain(),
    );
    app.add_systems(FixedUpdate, cube::apply_buoyancy);
    app
      .register_type::<cube::RotatingCube>()
      .add_systems(Startup, setup)
      .add_systems(
        Update,
        (
          cube::rotate_cube,
          disk::rotate_disk,
          cube::update_jump,
          camera::sync_camera_transforms,
        )
          .chain(),
      );
  }
}

pub fn setup(
  mut commands: Commands,
  mut meshes: ResMut<Assets<Mesh>>,
  mut materials: ResMut<Assets<StandardMaterial>>,
  ground_config: Res<ground::GroundConfig>,
  asset_server: Res<AssetServer>,
  mut et: ResMut<EntityTable>,
  mut camera_anchor: ResMut<CameraAnchorRes>,
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

  // Ground and Environment
  let ocean_floor_handle = asset_server.load("embedded://bevycube/media/wbtekbg2b512.jpg");
  let settings_handle = asset_server.load("embedded://bevycube/media/settings.jpg");
  let diamond_handle = asset_server.load("embedded://bevycube/media/diamond_sprite.jpg");

  /*
  let menu_main_handle = asset_server.load("embedded://bevycube/media/menu_main.jpg");
  let menu_roundel_handle = asset_server.load("embedded://bevycube/media/menu_roundel.jpg");
  let menu_ocean_handle = asset_server.load("embedded://bevycube/media/menu_ocean.jpg");
  let menu_instructions_handle = asset_server.load("embedded://bevycube/media/menu_instructions.jpg");
  let menu_about_handle = asset_server.load("embedded://bevycube/media/menu_about.jpg");
  */

  let ocean_id = ocean::spawn_ocean(&mut commands, &mut meshes, &mut materials, &mut et);

  let ground_id = ground::spawn_ground(
    &mut commands,
    &mut meshes,
    &mut materials,
    ground_config,
    ocean_floor_handle,
    &mut et,
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

  crate::ui::main_ui::spawn_main_menu(
    &mut commands,
    &mut meshes,
    &mut materials,
    &asset_server,
    &mut et,
  );

  crate::ui::instructions_ui::spawn_instructions_menu(
    &mut commands,
    &mut meshes,
    &mut materials,
    &asset_server,
    &mut et,
  );

  camera::spawn_camera(&mut commands, &mut et, &mut camera_anchor);

  lights::spawn_lights(&mut commands);

  commands
    .entity(ground_id)
    .observe(cube::handle_jump_request);
  commands.entity(ocean_id).observe(cube::handle_jump_request);
  commands.entity(disk_id).observe(cube::handle_jump_request);
}
