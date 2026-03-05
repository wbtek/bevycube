//! # BevyCube
//!
//! A 3D real-time rendered graphics demo written in Rust,
//! compiles to WASM for web browsers. Uses Bevy 0.18 game engine.
//!
//! ## Architecture Overview
//!
//! - **World Plugin**: 3D objects (cube, disk, ocean, ground)
//! - **Camera Plugin**: Dolly camera with anchor system and history stack
//! - **UI Plugin**: Data-driven menu system with hitbox tables
//! - **Roundel Plugin**: Runtime mipmap stitching for textures
//!
//! ## Navigation
//!
//! Menus are navigated using a stack-based camera system.
//! Clicking a menu pushes the current camera state; "Back" pops it.
//!
//! ## Ocean Physics
//!
//! Grid-based wave simulation with camera repulsion and buoyancy.
//! Supports Solid, Wire, and Points render modes.
//!
//! ## License
//!
//! MIT License - See [Cargo.toml](Cargo.toml) for full license text.

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

/// Core data structures and plugins for the application
pub mod constants;
/// Runtime mipmap stitching for roundel textures
pub mod roundel;
/// Data-driven menu system and UI handling
pub mod ui;
/// 3D world objects, physics, and rendering
pub mod world;

use bevy::asset::embedded_asset;
use bevy::prelude::*;

/// Tracks entity references for quick access across systems
/// Used to avoid querying for entities repeatedly
#[derive(Debug, Resource, Default)]
pub struct EntityTable {
  pub cube: Option<Entity>,
  pub disk: Option<Entity>,
  pub ground: Option<Entity>,
  pub main_menu: Option<Entity>,
  pub ocean_menu: Option<Entity>,
  pub roundel_menu: Option<Entity>,
  pub instruct_menu: Option<Entity>,
  pub about_menu: Option<Entity>,
  pub overlay_menu: Option<Entity>,
  pub main_anchor: Option<Entity>,
  pub main_camera: Option<Entity>,
  pub ocean: Option<Entity>,
  pub ocean_wire: Option<Entity>,
  pub ocean_point: Option<Entity>,
  pub ocean_fake: Option<Entity>,
}

/// Embeds media assets into the binary at compile time
/// Optimizes WASM builds by removing external file dependencies
pub struct EmbeddedAssetsPlugin;
impl Plugin for EmbeddedAssetsPlugin {
  fn build(&self, app: &mut App) {
    embedded_asset!(app, "media/wbtekbg2b512.jpg");
    embedded_asset!(app, "media/diamond_sprite.jpg");
    embedded_asset!(app, "media/menu_main.jpg");
    embedded_asset!(app, "media/menu_roundel.jpg");
    embedded_asset!(app, "media/menu_ocean.jpg");
    embedded_asset!(app, "media/menu_instructions.jpg");
    embedded_asset!(app, "media/menu_about.jpg");
    embedded_asset!(app, "media/menu_overlay.jpg");
  }
}
