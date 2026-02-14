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

use crate::world::camera;
use crate::EntityTable;
use bevy::prelude::*;

#[derive(Debug, Component)]
#[require(Transform, Visibility)]
pub struct Ground;

#[derive(Resource, Debug)]
pub struct GroundConfig {
  pub world_y: f32,
}

pub fn spawn_ground(
  commands: &mut Commands,
  meshes: &mut ResMut<Assets<Mesh>>,
  materials: &mut ResMut<Assets<StandardMaterial>>,
  ground_config: Res<GroundConfig>,
  ocean_floor_handle: Handle<Image>,
  et: &mut ResMut<EntityTable>,
) -> Entity {
  let world_y = ground_config.world_y;
  let ground_id = commands
    .spawn((
      Ground,
      Mesh3d(meshes.add(Plane3d::default().mesh().size(20., 20.))),
      MeshMaterial3d(materials.add(StandardMaterial {
        base_color_texture: Some(ocean_floor_handle),
        ..default()
      })),
      Transform::from_xyz(0.0, world_y, 0.0),
    ))
    .id();
  et.ground = Some(ground_id);

  commands.entity(ground_id).observe(
    |mut drag: On<Pointer<Drag>>, mut res: ResMut<camera::CameraAnchorRes>| {
      drag.propagate(false);
      res
        .current
        .update_pan(-drag.delta.x * 0.015, -drag.delta.y * 0.015);
    },
  );

  ground_id
}
