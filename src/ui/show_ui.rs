//! # UI Visibility System
//!
//! Controls which menus are visible based on camera / anchor position.
//! Hides fake ocean when it would interfere with menus.
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
use crate::world::camera::*;
use crate::EntityTable;
use bevy::camera::visibility::RenderLayers;
use bevy::prelude::*;
// use log::info;
//
const THRESHOLD: f32 = 7.0;

/// Show/hide menus and fake ocean based on anchor/camera position
pub fn show_menus_system(
  mut commands: Commands,
  car: ResMut<CameraAnchorRes>,
  et: Res<EntityTable>,
) {
  let c = car.current;

  let hide = c.anchor == crate::ui::main_ui::MENU_LOCATION && c.direction == 0.0 && c.zoom <= 0.01;
  let layer = if hide { 1 } else { 0 };
  if let Some(id) = et.overlay_menu {
    commands
      .entity(id)
      .insert_recursive::<Children>(RenderLayers::layer(layer));
  }

  let pairs = [
    (crate::ui::main_ui::MENU_LOCATION, et.main_menu),
    (crate::ui::instruct_ui::MENU_LOCATION, et.instruct_menu),
    (crate::ui::ocean_ui::MENU_LOCATION, et.ocean_menu),
    (crate::ui::roundel_ui::MENU_LOCATION, et.roundel_menu),
    (crate::ui::about_ui::MENU_LOCATION, et.about_menu),
  ];

  // visible on layer 0
  let mut fake_ocean_layer = 0;

  for p in pairs.iter() {
    let hide = c.get_camera_effect_xyz(p.0) > THRESHOLD;
    let layer = if hide {
      1
    } else {
      // not seen when layer 1, no clicks
      fake_ocean_layer = 1;
      0
    };
    if let Some(id) = p.1 {
      commands
        .entity(id)
        .insert_recursive::<Children>(RenderLayers::layer(layer));
    }
  }
  if let Some(fake_ocean_id) = et.ocean_fake {
    commands
      .entity(fake_ocean_id)
      .insert_recursive::<Children>(RenderLayers::layer(fake_ocean_layer));
  }
}
