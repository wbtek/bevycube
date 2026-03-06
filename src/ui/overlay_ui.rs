//! # Overlay UI (Main Menu button)
//!
//! Click-through overlay to reach main menu.
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
use crate::ui::{self, main_ui, MenuAction, MenuItem};
use crate::EntityTable;
use bevy::prelude::*;

/// Location placeholder for this "menu", differentiate by crate
pub const MENU_LOCATION: Vec3 = Vec3::new(0.0, 0.0, 0.0);
const IMAGE_PATH: &'static str = "embedded://bevycube/media/menu_overlay.jpg";

/// HITBOX_TABLE for menu interactions
pub const HITBOX_TABLE: &[MenuItem] = &[MenuItem {
  x: 0,
  y: 0,
  w: 511,
  h: 511,
  diamond: No,
  action: MenuAction::Execute(main_ui::request_view),
}];

/// Spawn overlay menu plane
pub fn spawn_overlay_menu(
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
    "Overlay Menu",
    IMAGE_PATH,
    MENU_LOCATION,
    HITBOX_TABLE,
  );
  et.overlay_menu = Some(menu_id);

  commands.entity(menu_id).insert(Transform {
    translation: Vec3::new(-0.6, -0.375, -1.0),
    rotation: Quat::from_rotation_x(std::f32::consts::FRAC_PI_2),
    scale: Vec3::new(0.0450, 1.0, 0.00895),
    ..default()
  });

  if let Some(camera) = et.main_camera {
    commands.entity(camera).add_child(menu_id);
  }
}
