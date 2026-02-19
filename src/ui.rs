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
pub mod instructions_ui;
pub mod main_ui;
pub mod ocean_ui;
pub mod roundel_ui;
use crate::world::camera::CameraAnchorRes;
use crate::{EntityTable, ImageFilterMode, ImageSampler, ImageSamplerDescriptor, StitchedRoundel};
use bevy::prelude::*;
use log::info;

// ============================================================================
// NEW MENU LIBRARY (Bevy 0.18 Data-Driven UI)
// ============================================================================

#[derive(Resource, Debug, Reflect)]
#[reflect(Resource)]
pub struct GlobalSettings {
  pub anisotropy: u16,
  pub mipmaps: bool,
  pub resolution_level: u8,
  pub mesh_mode: u8,
  pub mesh_subdiv: u32,
}

impl Default for GlobalSettings {
  fn default() -> Self {
    Self {
      anisotropy: 4,
      mipmaps: true,
      resolution_level: 1, // Medium
      mesh_mode: 1,        // wire
      mesh_subdiv: 20,
    }
  }
}

/// Shared actions for all data-driven menus
pub enum MenuAction {
  Execute(fn(&mut CameraAnchorRes)),
  Back,
  SetAnisotropy(u16),
  SetMipmaps(bool),
  SetResolution(u8),
  SetMeshMode(u8),
  SetMeshSubdiv(u32),
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

            MenuAction::SetAnisotropy(val) => settings.anisotropy = val,
            MenuAction::SetMipmaps(val) => settings.mipmaps = val,
            MenuAction::SetResolution(val) => settings.resolution_level = val,
            MenuAction::SetMeshMode(val) => settings.mesh_mode = val,
            MenuAction::SetMeshSubdiv(val) => settings.mesh_subdiv = val,

            MenuAction::OpenUrl(url) => {
              info!("Open URL: {}", url);
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

/// Standard recipe for spawning a 5.0x5.0 menu plane
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
  let id = commands
    .spawn((
      Name::new(label.to_string()),
      Mesh3d(meshes.add(Plane3d::default().mesh().size(5.0, 5.0))),
      MeshMaterial3d(materials.add(StandardMaterial {
        base_color_texture: Some(asset_server.load(image_path)),
        alpha_mode: AlphaMode::Add,
        reflectance: 0.0,
        ..default()
      })),
      Transform::from_translation(location),
    ))
    .id();

  attach_menu_interaction(commands, id, hitbox_table);
  id
}

// ============================================================================
// END NEW LIBRARY / START LEGACY SETTINGS UI
// ============================================================================

pub struct SettingsUiPlugin;
impl Plugin for SettingsUiPlugin {
  fn build(&self, app: &mut App) {
    app.register_type::<Settings>();
  }
}

#[derive(Debug, Component, Reflect)]
#[reflect(Component)]
pub struct Settings {
  pub active: bool,
}

impl Default for Settings {
  fn default() -> Self {
    Self { active: true }
  }
}

#[derive(Debug, Component)]
#[require(Transform, Visibility)]
pub struct SetAnisotropic;
#[derive(Debug, Component)]
#[require(Transform, Visibility)]
pub struct SetMipmaps;
#[derive(Debug, Component)]
#[require(Transform, Visibility)]
pub struct SetResolution;
#[derive(Debug, Component)]
#[require(Transform, Visibility)]
pub struct SetFps;

pub fn to_local(pixel: f32) -> f32 {
  (pixel - 256.0) / 512.0 * 5.0
}
pub fn from_local(pixel: f32) -> f32 {
  (pixel / 5.0) * 512.0 + 256.0
}

pub fn spawn_settings_ui(
  commands: &mut Commands,
  meshes: &mut ResMut<Assets<Mesh>>,
  settings_mat: Handle<StandardMaterial>,
  diamond_mat: Handle<StandardMaterial>,
  ground_id: Entity,
  et: &mut EntityTable,
) {
  let settings_id = commands
    .spawn((
      Settings { active: true },
      Mesh3d(meshes.add(Plane3d::default().mesh().size(5.0, 5.0))),
      MeshMaterial3d(settings_mat),
      Transform::from_xyz(7.5, 0.01, 7.5),
    ))
    .id();
  et.settings = Some(settings_id);

  commands.entity(ground_id).add_child(settings_id);

  macro_rules! row {
        ($c:ident, $y1:expr, $y2:expr, [$($x:expr),*], [$($i:ident),*]) => {
            SettingsCategory { cat: SetCatType::$c, y_top: $y1, y_bot: $y2, x_bounds: vec![$($x),*], items: vec![$(SetItem::$i),*] }
        };
    }

  let settings_data = [
    row!(
      Anisotropic,
      140.,
      185.,
      [107., 166., 215., 263., 310., 387.],
      [An16, An8, An4, An2, AnOff]
    ),
    row!(
      Mipmaps, //
      230.,
      275.,
      [107., 177., 255.],
      [MMOn, MMOff]
    ),
    row!(
      AssetResolution,
      320.,
      365.,
      [107., 206., 350., 433.],
      [AResHi, AResMed, AResLow]
    ),
    row!(
      Mesh,
      410.,
      455.,
      [107., 179., 280., 433.],
      [MeshOff, MeshWire, MeshPoint]
    ),
  ];

  enum SetCatType {
    Anisotropic,
    Mipmaps,
    AssetResolution,
    Mesh,
  }
  enum SetItem {
    An16,
    An8,
    An4,
    An2,
    AnOff,
    MMOn,
    MMOff,
    AResHi,
    AResMed,
    AResLow,
    MeshOff,
    MeshWire,
    MeshPoint,
  }
  struct SettingsCategory {
    cat: SetCatType,
    y_top: f32,
    y_bot: f32,
    x_bounds: Vec<f32>,
    items: Vec<SetItem>,
  }

  let set_anisotropic_id = commands
    .spawn((
      SetAnisotropic,
      Mesh3d(meshes.add(Plane3d::default().mesh().size(5. / 16., 5. / 16.))),
      MeshMaterial3d(diamond_mat.clone()),
      Transform::from_xyz(to_local(107. + 14.), 0.01, to_local(140. + 22.)),
    ))
    .id();
  et.set_anisotropic = Some(set_anisotropic_id);
  commands.entity(settings_id).add_child(set_anisotropic_id);

  let set_mipmaps_id = commands
    .spawn((
      SetMipmaps,
      Mesh3d(meshes.add(Plane3d::default().mesh().size(5. / 16., 5. / 16.))),
      MeshMaterial3d(diamond_mat.clone()),
      Transform::from_xyz(to_local(107. + 14.), 0.01, to_local(230. + 22.)),
    ))
    .id();
  et.set_mipmaps = Some(set_mipmaps_id);
  commands.entity(settings_id).add_child(set_mipmaps_id);

  let set_resolution_id = commands
    .spawn((
      SetResolution,
      Mesh3d(meshes.add(Plane3d::default().mesh().size(5. / 16., 5. / 16.))),
      MeshMaterial3d(diamond_mat.clone()),
      Transform::from_xyz(to_local(107. + 14.), 0.01, to_local(320. + 22.)),
    ))
    .id();
  et.set_resolution = Some(set_resolution_id);
  commands.entity(settings_id).add_child(set_resolution_id);

  let set_fps_id = commands
    .spawn((
      SetFps,
      Mesh3d(meshes.add(Plane3d::default().mesh().size(5. / 16., 5. / 16.))),
      MeshMaterial3d(diamond_mat.clone()),
      Transform::from_xyz(to_local(107. + 14.), 0.01, to_local(410. + 22.)),
    ))
    .id();
  et.deadbeef = Some(set_fps_id);
  commands.entity(settings_id).add_child(set_fps_id);

  commands.entity(settings_id).observe(
    move |mut click: On<Pointer<Click>>,
          et: Res<EntityTable>,
          stitched: Option<Res<StitchedRoundel>>,
          mut commands: Commands,
          mut query: Query<(&mut Settings, &GlobalTransform)>,
          mut diamond_query: Query<&mut Transform, Without<Settings>>,
          mut images: ResMut<Assets<Image>>,
          mut materials: ResMut<Assets<StandardMaterial>>| {
      let Ok((settings, settings_global)) = query.get_mut(click.event_target()) else {
        return;
      };
      if !settings.active || click.duration.as_millis() > 250 {
        return;
      }
      let Some(hit_pos) = click.hit.position else {
        return;
      };
      let local_hit = settings_global.affine().inverse().transform_point3(hit_pos);
      let px = from_local(local_hit.x);
      let py = from_local(local_hit.z);

      let clicked_data = settings_data
        .iter()
        .find(|row| py >= row.y_top && py <= row.y_bot)
        .and_then(|row| {
          row
            .x_bounds
            .windows(2)
            .zip(row.items.iter())
            .find(|(bounds, _)| px >= bounds[0] && px < bounds[1])
            .map(|(bounds, item)| (&row.cat, item, row.y_top, bounds[0]))
        });

      if let Some((category, item, y_start, x_start)) = clicked_data {
        let Some(ref stitched_res) = stitched else {
          return;
        };
        let target_handle = &stitched_res.handle;
        match category {
          SetCatType::Anisotropic => {
            if let Ok(mut transform) = diamond_query.get_mut(et.set_anisotropic.unwrap()) {
              transform.translation =
                Vec3::new(to_local(x_start + 14.0), 0.01, to_local(y_start + 22.0));
            }
            if let Some(img) = images.get_mut(target_handle) {
              let mut is_desc = match img.sampler.clone() {
                ImageSampler::Descriptor(d) => d,
                _ => ImageSamplerDescriptor::default(),
              };
              is_desc.anisotropy_clamp = match item {
                SetItem::An16 => 16,
                SetItem::An8 => 8,
                SetItem::An4 => 4,
                SetItem::An2 => 2,
                _ => 1,
              };
              if is_desc.anisotropy_clamp > 1 {
                is_desc.mipmap_filter = ImageFilterMode::Linear;
                if let Ok(mut mip_transform) = diamond_query.get_mut(et.set_mipmaps.unwrap()) {
                  mip_transform.translation =
                    Vec3::new(to_local(107. + 14.0), 0.01, to_local(230. + 22.0));
                }
              }
              img.sampler = ImageSampler::Descriptor(is_desc);
              for (_, mat) in materials.iter_mut() {
                if mat
                  .base_color_texture
                  .as_ref()
                  .map(|h| h.id() == target_handle.id())
                  .unwrap_or(false)
                {
                  mat.base_color_texture = Some(target_handle.clone());
                }
              }
            }
          }
          SetCatType::Mipmaps => {
            if let Ok(mut transform) = diamond_query.get_mut(et.set_mipmaps.unwrap()) {
              transform.translation =
                Vec3::new(to_local(x_start + 14.0), 0.01, to_local(y_start + 22.0));
            }
            if let Some(img) = images.get_mut(target_handle) {
              let mut is_desc = match img.sampler.clone() {
                ImageSampler::Descriptor(d) => d,
                _ => ImageSamplerDescriptor::default(),
              };
              is_desc.mipmap_filter = match item {
                SetItem::MMOn => ImageFilterMode::Linear,
                SetItem::MMOff => {
                  is_desc.anisotropy_clamp = 1;
                  if let Ok(mut aniso_transform) =
                    diamond_query.get_mut(et.set_anisotropic.unwrap())
                  {
                    aniso_transform.translation =
                      Vec3::new(to_local(310. + 14.0), 0.01, to_local(140. + 22.0));
                  }
                  ImageFilterMode::Linear
                }
                _ => ImageFilterMode::Linear,
              };
              img.sampler = ImageSampler::Descriptor(is_desc);
              for (_, mat) in materials.iter_mut() {
                if mat
                  .base_color_texture
                  .as_ref()
                  .map(|h| h.id() == target_handle.id())
                  .unwrap_or(false)
                {
                  mat.base_color_texture = Some(target_handle.clone());
                }
              }
            }
          }
          SetCatType::AssetResolution => {
            if let Ok(mut transform) = diamond_query.get_mut(et.set_resolution.unwrap()) {
              transform.translation =
                Vec3::new(to_local(x_start + 14.0), 0.01, to_local(y_start + 22.0));
            }
            if let Some(img) = images.get_mut(target_handle) {
              let mut is_desc = match img.sampler.clone() {
                ImageSampler::Descriptor(d) => d,
                _ => ImageSamplerDescriptor::default(),
              };
              is_desc.lod_min_clamp = match item {
                SetItem::AResHi => 0.,
                SetItem::AResMed => 1.,
                SetItem::AResLow => 2.,
                _ => 3.,
              };
              img.sampler = ImageSampler::Descriptor(is_desc);
              for (_, mat) in materials.iter_mut() {
                if mat
                  .base_color_texture
                  .as_ref()
                  .map(|h| h.id() == target_handle.id())
                  .unwrap_or(false)
                {
                  mat.base_color_texture = Some(target_handle.clone());
                }
              }
            }
          }
          SetCatType::Mesh => {
            if let Ok(mut transform) = diamond_query.get_mut(et.deadbeef.unwrap()) {
              transform.translation =
                Vec3::new(to_local(x_start + 14.0), 0.01, to_local(y_start + 22.0));
            }
            match item {
              SetItem::MeshOff => {
                commands
                  .entity(et.ocean_wire.expect("No entity!"))
                  .insert(Visibility::Hidden);
                commands
                  .entity(et.ocean_point.expect("No entity!"))
                  .insert(Visibility::Hidden);
              }
              SetItem::MeshWire => {
                commands
                  .entity(et.ocean_wire.expect("No entity!"))
                  .insert(Visibility::Visible);
                commands
                  .entity(et.ocean_point.expect("No entity!"))
                  .insert(Visibility::Hidden);
              }
              SetItem::MeshPoint => {
                commands
                  .entity(et.ocean_wire.expect("No entity!"))
                  .insert(Visibility::Hidden);
                commands
                  .entity(et.ocean_point.expect("No entity!"))
                  .insert(Visibility::Visible);
              }
              _ => {}
            }
          }
        }
        click.propagate(false);
      }
    },
  );
}
