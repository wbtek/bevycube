//! # Ocean Settings UI
//!
/// Ocean mesh settings (mode, dimension).
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
// The above copyright notice and this permission notice shall be included in
// all copies or substantial portions of the Software.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
// IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
// FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
// AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
// LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
// OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
// SOFTWARE.
use crate::ui::GlobalSettings;
use crate::ui::Need::*;
use crate::ui::{self, MenuAction, MenuItem};
use crate::world::camera::{CameraAnchorRes, CameraParams};
use crate::world::ocean;
use crate::EntityTable;
use bevy::camera::visibility::RenderLayers;
use bevy::prelude::*;
// use log::info;

pub const MENU_LOCATION: Vec3 = Vec3::new(5.9, 0.01, -5.9);
pub const IMAGE_PATH: &'static str = "embedded://bevycube/media/menu_ocean.jpg";

pub const HITBOX_TABLE: &[MenuItem] = &[
  MenuItem {
    x: 354,
    y: 57,
    w: 74,
    h: 29,
    diamond: No,
    action: MenuAction::Back,
  },
  MenuItem {
    x: 111,
    y: 147,
    w: 97,
    h: 30,
    diamond: Yes,
    action: MenuAction::SetMeshMode(0),
  }, // Solid
  MenuItem {
    x: 214,
    y: 147,
    w: 93,
    h: 29,
    diamond: Yes,
    action: MenuAction::SetMeshMode(1),
  }, // Wire
  MenuItem {
    x: 316,
    y: 147,
    w: 107,
    h: 29,
    diamond: Yes,
    action: MenuAction::SetMeshMode(2),
  }, // Points
  MenuItem {
    x: 111,
    y: 237,
    w: 40,
    h: 29,
    diamond: Yes,
    action: MenuAction::SetMeshDimension(5),
  },
  MenuItem {
    x: 160,
    y: 237,
    w: 55,
    h: 29,
    diamond: Yes,
    action: MenuAction::SetMeshDimension(10),
  },
  MenuItem {
    x: 221,
    y: 237,
    w: 56,
    h: 29,
    diamond: Yes,
    action: MenuAction::SetMeshDimension(20),
  },
  MenuItem {
    x: 289,
    y: 237,
    w: 56,
    h: 29,
    diamond: Yes,
    action: MenuAction::SetMeshDimension(40),
  },
  MenuItem {
    x: 111,
    y: 282,
    w: 60,
    h: 29,
    diamond: Yes,
    action: MenuAction::SetMeshDimension(80),
  },
  MenuItem {
    x: 179,
    y: 282,
    w: 72,
    h: 29,
    diamond: Yes,
    action: MenuAction::SetMeshDimension(160),
  },
  MenuItem {
    x: 259,
    y: 282,
    w: 79,
    h: 29,
    diamond: Yes,
    action: MenuAction::SetMeshDimension(320),
  },
];

/// Request view of ocean settings menu from elsewhere
pub fn request_view(camera_res: &mut CameraAnchorRes) {
  camera_res.request_menu(CameraParams {
    anchor: MENU_LOCATION,
    zoom: 0.0,
    ..default()
  });
}

/// Spawn ocean settings menu plane
pub fn spawn_ocean_menu(
  commands: &mut Commands,
  meshes: &mut ResMut<Assets<Mesh>>,
  materials: &mut ResMut<Assets<StandardMaterial>>,
  asset_server: &Res<AssetServer>,
  et: &mut ResMut<EntityTable>,
) {
  let menu_id = ui::spawn_menu_plane(
    commands,
    meshes,
    materials,
    asset_server,
    "Ocean Menu",
    IMAGE_PATH,
    MENU_LOCATION,
    HITBOX_TABLE,
  );

  et.ocean_menu = Some(menu_id);

  if let Some(ground) = et.ground {
    commands.entity(ground).add_child(menu_id);
  }
}

/// Sync ocean settings with GlobalSettings
pub fn sync_ocean_menu_settings(
  settings: Res<GlobalSettings>,
  mut local: Local<Option<GlobalSettings>>,
  mut et: ResMut<EntityTable>,
  mut commands: Commands,
  mut meshes: ResMut<Assets<Mesh>>,
  mut materials: ResMut<Assets<StandardMaterial>>,
) {
  if local.is_none() {
    *local = Some(GlobalSettings::INVALID);
  }
  let l = local.as_mut().unwrap();

  if settings.mesh_dimension != l.mesh_dimension {
    ocean::spawn_ocean(
      &mut commands,
      &mut meshes,
      &mut materials,
      &mut et,
      settings.mesh_dimension,
    );
    *l = GlobalSettings::INVALID;
    l.mesh_dimension = settings.mesh_dimension;
  }

  if settings.mesh_mode != l.mesh_mode {
    let entities = [et.ocean, et.ocean_wire, et.ocean_point];
    for (i, opt_ent) in entities.iter().enumerate() {
      if let Some(entity) = *opt_ent {
        let is_active = i == settings.mesh_mode as usize;
        let layer = if is_active { 0 } else { 1 };
        commands
          .entity(entity)
          .insert_recursive::<Children>(RenderLayers::layer(layer));
        l.mesh_mode = settings.mesh_mode;
      }
    }
  }
}
