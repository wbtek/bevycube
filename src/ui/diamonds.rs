//! # Diamond Indicators
//!
//! Syncs diamond sprite position with selected menu option.
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
use crate::ui::{to_local, GlobalSettings, MenuAction};
use crate::EntityTable;
use bevy::prelude::*;

/// Component marking diamond sprite category
#[derive(Component)]
pub struct DiamondCategory(pub MenuAction);

/// Sync diamond positions with GlobalSettings
pub fn sync_diamonds(
  settings: Res<GlobalSettings>,
  et: Res<EntityTable>,
  mut query: Query<(&DiamondCategory, &mut Transform, &ChildOf)>,
) {
  if !settings.is_changed() {
    return;
  }

  for (category, mut transform, child_of) in query.iter_mut() {
    let parent_entity = child_of.0;

    let table = if Some(parent_entity) == et.ocean_menu {
      crate::ui::ocean_ui::HITBOX_TABLE
    } else if Some(parent_entity) == et.roundel_menu {
      crate::ui::roundel_ui::HITBOX_TABLE
    } else {
      continue;
    };

    let is_match = |action: &MenuAction| -> bool {
      match (action, &category.0) {
        (MenuAction::SetMeshMode(v), MenuAction::SetMeshMode(_)) => *v == settings.mesh_mode,
        (MenuAction::SetAnisotropy(v), MenuAction::SetAnisotropy(_)) => *v == settings.anisotropy,
        (MenuAction::SetMipmaps(v), MenuAction::SetMipmaps(_)) => *v == settings.mipmaps,
        (MenuAction::SetResolution(v), MenuAction::SetResolution(_)) => {
          *v == settings.asset_resolution
        }
        (MenuAction::SetMeshDimension(v), MenuAction::SetMeshDimension(_)) => {
          *v == settings.mesh_dimension
        }
        _ => false,
      }
    };

    if let Some(item) = table.iter().find(|i| is_match(&i.action)) {
      transform.translation.x = to_local(item.x as f32 + 10.);
      transform.translation.z = to_local(item.y as f32 + 15.);
      transform.translation.y = 0.01;
    }
  }
}
