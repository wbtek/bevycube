//! # Roundel Plugin
//!
/// Runtime mipmap stitching for roundel textures.
/// Combines 5 texture levels into one for performance.
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
use bevy::asset::embedded_asset;
use bevy::image::*;
use bevy::math::Affine2;
use bevy::prelude::*;
use bevy::render::render_resource::*;

/// Plugin for mipmap stitching
pub struct RoundelPlugin;

impl Plugin for RoundelPlugin {
  fn build(&self, app: &mut App) {
    embedded_asset!(app, "media/WhiteBearCrab512.jpg");
    embedded_asset!(app, "media/WhiteBearCrab256.jpg");
    embedded_asset!(app, "media/WhiteBearCrab128.jpg");
    embedded_asset!(app, "media/WhiteBearCrab64.jpg");
    embedded_asset!(app, "media/WhiteBearCrab32.jpg");

    app.add_systems(Update, stitch_roundel_system);
  }
}

/// Resource for stitched roundel texture
#[derive(Resource)]
pub struct StitchedRoundel {
  pub handle: Handle<Image>,
}

/// Resource for tracking mipmap loading
#[derive(Debug, Resource)]
pub struct RoundelMipmapLoading {
  pub handles: [Handle<Image>; 5],
  pub target_handle: Handle<Image>,
}

/// Get roundel material configuration
pub fn get_roundel_material(handle: Handle<Image>) -> StandardMaterial {
  StandardMaterial {
    base_color_texture: Some(handle),
    alpha_mode: AlphaMode::Opaque,
    uv_transform: Affine2::from_translation(Vec2::splat(0.5))
      * Affine2::from_scale(Vec2::splat(0.98))
      * Affine2::from_translation(Vec2::splat(-0.5)),
    cull_mode: Some(Face::Back),
    ..default()
  }
}

/// Stitch mipmap levels together at runtime
fn stitch_roundel_system(
  mut commands: Commands,
  loading: Option<Res<RoundelMipmapLoading>>,
  mut images: ResMut<Assets<Image>>,
  mut materials: ResMut<Assets<StandardMaterial>>,
) {
  let Some(loading) = loading else { return };
  if loading.handles.iter().all(|h| images.get(h).is_some()) {
    let mut combined_data = Vec::new();
    let format = images
      .get(&loading.handles[0])
      .unwrap()
      .texture_descriptor
      .format;

    for h in &loading.handles {
      if let Some(ref data) = images.get(h).unwrap().data {
        combined_data.extend_from_slice(data);
      } else {
        return;
      }
    }

    let final_handle = images.add(Image {
      data: Some(combined_data),
      texture_descriptor: TextureDescriptor {
        label: Some("stitched_roundel"),
        size: Extent3d {
          width: 512,
          height: 512,
          depth_or_array_layers: 1,
        },
        mip_level_count: loading.handles.len() as u32,
        sample_count: 1,
        dimension: TextureDimension::D2,
        format,
        usage: TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST,
        view_formats: &[],
      },
      sampler: ImageSampler::Descriptor(ImageSamplerDescriptor {
        mipmap_filter: ImageFilterMode::Linear,
        mag_filter: ImageFilterMode::Linear,
        min_filter: ImageFilterMode::Linear,
        anisotropy_clamp: 16,
        ..default()
      }),
      ..default()
    });

    // Update existing materials that were using the placeholder
    for (_, mat) in materials.iter_mut() {
      if let Some(ref tex) = mat.base_color_texture {
        if tex.id() == loading.target_handle.id() {
          mat.base_color_texture = Some(final_handle.clone());
        }
      }
    }

    commands.insert_resource(StitchedRoundel {
      handle: final_handle,
    });
    commands.remove_resource::<RoundelMipmapLoading>();
  }
}
