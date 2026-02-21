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

use bevy::asset::AssetMetaCheck;
use bevy::prelude::*;
use bevycube::*;

fn main() {
  #[cfg(target_arch = "wasm32")]
  console_log::init_with_level(log::Level::Info).ok();

  App::new()
    .init_resource::<EntityTable>()
    .insert_resource(world::disk::DiskParms {
      rotation_speed: 0.2,
    })
    .insert_resource(world::cube::CubeParms {
      rotation_speed: -1.0,
    })
    .add_plugins(DefaultPlugins.set(AssetPlugin {
      meta_check: AssetMetaCheck::Never,
      ..default()
    }))
    .add_plugins(MeshPickingPlugin)
    .add_plugins((
      EmbeddedAssetsPlugin,
      roundel::RoundelPlugin,
      world::WorldPlugin,
      world::camera::CameraPlugin,
    ))
    .run();
}
