//! # Main Menu UI
//!
/// Main menu with navigation to other menus.
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
use crate::ui::Need::*;
use crate::ui::{self, about_ui, instruct_ui, ocean_ui, roundel_ui, MenuAction, MenuItem};
use crate::world::camera::{CameraAnchorRes, CameraParams};
use crate::EntityTable;
use bevy::prelude::*;

pub const MENU_LOCATION: Vec3 = Vec3::new(0.0, 0.01, -5.9);
const IMAGE_PATH: &'static str = "embedded://bevycube/media/menu_main.jpg";

/// HITBOX_TABLE for main menu interactions
const HITBOX_TABLE: &[MenuItem] = &[
  MenuItem {
    x: 354,
    y: 57,
    w: 74,
    h: 29,
    diamond: No,
    action: MenuAction::Back,
  },
  MenuItem {
    x: 79,
    y: 102,
    w: 167,
    h: 29,
    diamond: No,
    action: MenuAction::Execute(instruct_ui::request_view),
  },
  MenuItem {
    x: 80,
    y: 147,
    w: 207,
    h: 38,
    diamond: No,
    action: MenuAction::Execute(ocean_ui::request_view),
  },
  MenuItem {
    x: 79,
    y: 192,
    w: 235,
    h: 38,
    diamond: No,
    action: MenuAction::Execute(roundel_ui::request_view),
  },
  MenuItem {
    x: 80,
    y: 237,
    w: 85,
    h: 33,
    diamond: No,
    action: MenuAction::Execute(about_ui::request_view),
  },
];

/// Request view of main menu
pub fn request_view(camera_res: &mut CameraAnchorRes) {
  camera_res.request_menu(CameraParams {
    anchor: MENU_LOCATION,
    zoom: 0.0,
    ..default()
  });
}

/// Spawn main menu plane
pub fn spawn_main_menu(
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
    "Main Menu",
    IMAGE_PATH,
    MENU_LOCATION,
    HITBOX_TABLE,
  );
  et.main_menu = Some(menu_id);

  if let Some(ground) = et.ground {
    commands.entity(ground).add_child(menu_id);
  }
}
