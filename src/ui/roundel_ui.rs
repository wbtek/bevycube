//! # Roundel Settings UI
//!
/// Texture settings (anisotropy, mipmaps, resolution).
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
use crate::roundel::StitchedRoundel;
use crate::ui::{self, GlobalSettings, MenuAction, MenuItem, Need::*};
use crate::world::camera::{CameraAnchorRes, CameraParams};
use crate::EntityTable;
use bevy::image::{ImageFilterMode, ImageSampler, ImageSamplerDescriptor};
use bevy::prelude::*;

pub const MENU_LOCATION: Vec3 = Vec3::new(5.9, 0.01, 0.0);
pub const IMAGE_PATH: &'static str = "embedded://bevycube/media/menu_roundel.jpg";

pub const HITBOX_TABLE: &[MenuItem] = &[
  MenuItem {
    x: 354,
    y: 57,
    w: 73,
    h: 29,
    diamond: No,
    action: MenuAction::Back,
  },
  MenuItem {
    x: 111,
    y: 147,
    w: 52,
    h: 29,
    diamond: Yes,
    action: MenuAction::SetAnisotropy(16),
  },
  MenuItem {
    x: 171,
    y: 147,
    w: 40,
    h: 29,
    diamond: Yes,
    action: MenuAction::SetAnisotropy(8),
  },
  MenuItem {
    x: 219,
    y: 147,
    w: 39,
    h: 29,
    diamond: Yes,
    action: MenuAction::SetAnisotropy(4),
  },
  MenuItem {
    x: 266,
    y: 147,
    w: 39,
    h: 29,
    diamond: Yes,
    action: MenuAction::SetAnisotropy(2),
  },
  MenuItem {
    x: 314,
    y: 147,
    w: 69,
    h: 38,
    diamond: Yes,
    action: MenuAction::SetAnisotropy(1),
  },
  MenuItem {
    x: 111,
    y: 237,
    w: 62,
    h: 29,
    diamond: Yes,
    action: MenuAction::SetMipmaps(1),
  },
  MenuItem {
    x: 182,
    y: 237,
    w: 68,
    h: 38,
    diamond: Yes,
    action: MenuAction::SetMipmaps(0),
  },
  MenuItem {
    x: 111,
    y: 327,
    w: 90,
    h: 38,
    diamond: Yes,
    action: MenuAction::SetResolution(0),
  },
  MenuItem {
    x: 210,
    y: 327,
    w: 136,
    h: 29,
    diamond: Yes,
    action: MenuAction::SetResolution(1),
  },
  MenuItem {
    x: 354,
    y: 328,
    w: 75,
    h: 37,
    diamond: Yes,
    action: MenuAction::SetResolution(2),
  },
];

/// Request view of roundel settings menu from elsewhere
pub fn request_view(camera_res: &mut CameraAnchorRes) {
  camera_res.request_menu(CameraParams {
    anchor: MENU_LOCATION,
    zoom: 0.0,
    ..default()
  });
}

/// Spawn roundel settings menu plane
pub fn spawn_roundel_menu(
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
    "Roundel Menu",
    IMAGE_PATH,
    MENU_LOCATION,
    HITBOX_TABLE,
  );

  et.roundel_menu = Some(menu_id);

  if let Some(ground) = et.ground {
    commands.entity(ground).add_child(menu_id);
  }
}

/// Sync roundel settings with GlobalSettings
pub fn sync_roundel_menu_settings(
  settings: Res<GlobalSettings>,
  mut local: Local<Option<GlobalSettings>>,
  stitched: Option<Res<StitchedRoundel>>,
  mut images: ResMut<Assets<Image>>,
  mut materials: ResMut<Assets<StandardMaterial>>,
) {
  if local.is_none() {
    *local = Some(GlobalSettings::INVALID);
  }
  let l = local.as_mut().unwrap();

  if let Some(ref stitched_res) = stitched {
    let handle = &stitched_res.handle;
    if let Some(img) = images.get_mut(handle) {
      let mut img_samp = match img.sampler.clone() {
        ImageSampler::Descriptor(d) => d,
        _ => ImageSamplerDescriptor::default(),
      };

      if settings.anisotropy != l.anisotropy {
        img_samp.anisotropy_clamp = settings.anisotropy as u16;
        l.anisotropy = settings.anisotropy;
      }

      if settings.mipmaps != l.mipmaps {
        img_samp.mipmap_filter = match settings.mipmaps {
          1 => ImageFilterMode::Linear,
          _ => ImageFilterMode::Nearest,
        };
        l.mipmaps = settings.mipmaps;
      }

      if settings.asset_resolution != l.asset_resolution {
        img_samp.lod_min_clamp = settings.asset_resolution as f32;
        l.asset_resolution = settings.asset_resolution;
      }

      img.sampler = ImageSampler::Descriptor(img_samp);
      for (_, mat) in materials.iter_mut() {
        if mat
          .base_color_texture
          .as_ref()
          .map(|h| h.id() == handle.id())
          .unwrap_or(false)
        {
          mat.base_color_texture = Some(handle.clone());
        }
      }
    }
  }
}
