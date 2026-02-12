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

pub mod roundel;
pub mod ui;
pub mod world;

use crate::roundel::*;

use bevy::asset::embedded_asset;
use bevy::image::*;
use bevy::prelude::*;

#[derive(Debug, Resource, Default)]
pub struct EntityTable {
  pub cube: Option<Entity>,
  pub disk: Option<Entity>,
  pub ground: Option<Entity>,
  pub main_menu: Option<Entity>,
  pub instructions_menu: Option<Entity>,
  pub settings: Option<Entity>,
  pub set_anisotropic: Option<Entity>,
  pub set_mipmaps: Option<Entity>,
  pub set_resolution: Option<Entity>,
  pub set_fps: Option<Entity>,
  pub safety_disk: Option<Entity>,
  pub safety_disk_hidden: Option<Entity>,
  pub main_anchor: Option<Entity>,
  pub main_camera: Option<Entity>,
  pub ocean: Option<Entity>,
  pub ocean_wire: Option<Entity>,
  pub ocean_point: Option<Entity>,
}

pub struct EmbeddedAssetsPlugin;
impl Plugin for EmbeddedAssetsPlugin {
  fn build(&self, app: &mut App) {
    embedded_asset!(app, "media/wbtekbg2b512.jpg");
    embedded_asset!(app, "media/settings.jpg");
    embedded_asset!(app, "media/diamond_sprite.jpg");
    embedded_asset!(app, "media/menu_main.jpg");
    embedded_asset!(app, "media/menu_roundel.jpg");
    embedded_asset!(app, "media/menu_ocean.jpg");
    embedded_asset!(app, "media/menu_instructions.jpg");
    embedded_asset!(app, "media/menu_about.jpg");
  }
}
