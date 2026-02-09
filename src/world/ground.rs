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

use crate::EntityTable;
use bevy::prelude::*;

#[derive(Debug, Component)]
#[require(Transform, Visibility)]
pub struct Ground;

pub fn spawn_ground(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    ocean_floor_handle: Handle<Image>,
    et: &mut ResMut<EntityTable>,
) -> Entity {
    let ground_id = commands
        .spawn((
            Ground,
            Mesh3d(meshes.add(Plane3d::default().mesh().size(20., 20.))),
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color_texture: Some(ocean_floor_handle),
                ..default()
            })),
            Transform::from_xyz(0.0, -2.0, 0.0),
        ))
        .id();
    et.ground = Some(ground_id);

    // Camera Anchor Drag Observer
    commands.entity(ground_id).observe(
        |mut drag: On<Pointer<Drag>>, et: Res<EntityTable>, mut query: Query<&mut Transform>| {
            drag.propagate(false);
            if let Some(mut transform) = et.main_anchor.and_then(|id| query.get_mut(id).ok()) {
                transform.translation.x -= drag.delta.x * 0.015;
                transform.translation.z -= drag.delta.y * 0.015;
            }
        },
    );

    ground_id
}
