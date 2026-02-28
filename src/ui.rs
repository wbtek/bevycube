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

pub mod about_ui;
pub mod diamonds;
pub mod instruct_ui;
pub mod main_ui;
pub mod ocean_ui;
pub mod overlay_ui;
pub mod roundel_ui;
pub mod show_ui;
use crate::world::camera::CameraAnchorRes;
use bevy::light::NotShadowCaster;
use bevy::prelude::*;

#[derive(Resource, Debug, Clone, Copy, Reflect)]
#[reflect(Resource)]
pub struct GlobalSettings {
  pub anisotropy: u32,       // 1, 2, 4, 8, 16
  pub mipmaps: u32,          // 0: Off, 1: On
  pub asset_resolution: u32, // 0: High, 1: Med, 2: Low
  pub mesh_mode: u32,        // 0: Solid, 1: Wire, 2: Points
  pub mesh_dimension: u32,   // 40, 80, 160, etc
}

impl Default for GlobalSettings {
  fn default() -> Self {
    Self {
      anisotropy: 2,
      mipmaps: 1,
      asset_resolution: 1, // Medium
      mesh_mode: 0,        // Solid
      mesh_dimension: 20,
    }
  }
}

impl GlobalSettings {
  const INVALID: Self = Self {
    anisotropy: 1111111111,
    mipmaps: 1111111111,
    asset_resolution: 1111111111,
    mesh_mode: 1111111111,
    mesh_dimension: 1111111111,
  };
}

/// Shared actions for all data-driven menus
#[derive(Debug, Clone, Copy)]
pub enum MenuAction {
  Execute(fn(&mut CameraAnchorRes)),
  Back,
  SetAnisotropy(u32),
  SetMipmaps(u32),
  SetResolution(u32),
  SetMeshMode(u32),
  SetMeshDimension(u32),
  OpenUrl(&'static str),
}

pub enum Need {
  No,
  Yes,
}

/// Hitbox definition using pixel coordinates (0-512)
pub struct MenuItem {
  pub x: u32,
  pub y: u32,
  pub w: u32,
  pub h: u32,
  pub diamond: Need,
  pub action: MenuAction,
}

/// Maps local plane coordinate (-2.5..2.5) to pixel space (0..512)
pub fn to_pixel(local_coord: f32) -> f32 {
  (local_coord * 512.0 / 5.0) + 256.0
}

pub fn to_local(pixel: f32) -> f32 {
  (pixel - 256.0) / 512.0 * 5.0
}

/// Attaches a standard observer to a menu entity to handle table-based hits
pub fn attach_menu_interaction(
  commands: &mut Commands,
  entity: Entity,
  hitbox_table: &'static [MenuItem],
) {
  commands.entity(entity).observe(
    move |mut ev: On<Pointer<Click>>,
          mut camera_res: ResMut<CameraAnchorRes>,
          mut settings: ResMut<GlobalSettings>,
          query: Query<&GlobalTransform>| {
      let Some(hit_world_pos) = ev.hit.position else {
        return;
      };
      let Ok(menu_gt) = query.get(ev.event_target()) else {
        return;
      };

      // Convert world hit to local plane space, then to pixel coordinates
      let local_pos = menu_gt.affine().inverse().transform_point3(hit_world_pos);
      let px = to_pixel(local_pos.x) as u32;
      let py = to_pixel(local_pos.z) as u32;

      for item in hitbox_table {
        if px >= item.x && px <= (item.x + item.w) && py >= item.y && py <= (item.y + item.h) {
          ev.propagate(false);
          match item.action {
            MenuAction::Back => camera_res.request_back(),
            MenuAction::Execute(func) => func(&mut camera_res),

            MenuAction::SetAnisotropy(val) => {
              settings.anisotropy = val;
              if val > 1 {
                settings.mipmaps = 1;
              }
            }
            MenuAction::SetMipmaps(val) => {
              settings.mipmaps = val;
              if val == 0 {
                settings.anisotropy = 1;
              }
            }
            MenuAction::SetResolution(val) => settings.asset_resolution = val,
            MenuAction::SetMeshMode(val) => settings.mesh_mode = val,
            MenuAction::SetMeshDimension(val) => settings.mesh_dimension = val,

            MenuAction::OpenUrl(url) => {
              #[cfg(target_arch = "wasm32")]
              {
                if let Some(window) = web_sys::window() {
                  let _ = window.open_with_url_and_target(url, "_blank");
                }
              }
            }
          }
          break;
        }
      }
    },
  );
}

pub fn spawn_menu_plane(
  commands: &mut Commands,
  meshes: &mut ResMut<Assets<Mesh>>,
  materials: &mut ResMut<Assets<StandardMaterial>>,
  asset_server: &Res<AssetServer>,
  label: &str,
  image_path: &'static str,
  location: Vec3,
  hitbox_table: &'static [MenuItem],
) -> Entity {
  let menu_id = commands
    .spawn((
      Name::new(label.to_string()),
      Mesh3d(meshes.add(Plane3d::default().mesh().size(5.0, 5.0))),
      MeshMaterial3d(materials.add(StandardMaterial {
        base_color_texture: Some(asset_server.load(image_path)),
        alpha_mode: AlphaMode::Add,
        unlit: true,
        reflectance: 0.0,
        ..default()
      })),
      NotShadowCaster,
      Transform::from_translation(location),
    ))
    .id();

  // Create the diamond material
  let diamond_mat = materials.add(StandardMaterial {
    base_color_texture: Some(asset_server.load("embedded://bevycube/media/diamond_sprite.jpg")),
    alpha_mode: AlphaMode::Add,
    unlit: true,
    reflectance: 0.0,
    ..default()
  });

  // Spawn one diamond for each category that needs one
  // Category set from first MenuItem to request diamond
  let mut categories_spawned = Vec::new();

  for item in hitbox_table {
    if let Need::Yes = item.diamond {
      let cat_id = match item.action {
        MenuAction::SetAnisotropy(_) => 1,
        MenuAction::SetMipmaps(_) => 2,
        MenuAction::SetResolution(_) => 3,
        MenuAction::SetMeshMode(_) => 4,
        MenuAction::SetMeshDimension(_) => 5,
        _ => 0,
      };

      if cat_id > 0 && !categories_spawned.contains(&cat_id) {
        let diamond_id = commands
          .spawn((
            Name::new(format!("Diamond {}", cat_id)),
            crate::ui::diamonds::DiamondCategory(item.action),
            Mesh3d(meshes.add(Plane3d::default().mesh().size(5.0 / 16.0, 5.0 / 16.0))),
            MeshMaterial3d(diamond_mat.clone()),
            NotShadowCaster,
            // Initial position will be snapped by sync_diamonds
            Transform::from_xyz(0.0, 0.01, 0.0),
          ))
          .id();

        commands.entity(menu_id).add_child(diamond_id);
        categories_spawned.push(cat_id);
      }
    }
  }

  attach_menu_interaction(commands, menu_id, hitbox_table);
  menu_id
}
