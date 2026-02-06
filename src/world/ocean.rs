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

use crate::{EntityTable, OceanBuffer};
use bevy::mesh::VertexAttributeValues;
use bevy::prelude::*;

#[derive(Debug, Component)]
#[require(Transform, Visibility)]
pub struct Ocean;

pub fn spawn_ocean(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    et: &mut ResMut<EntityTable>,
) -> Entity {
    let grid_size = 12;
    commands.insert_resource(OceanBuffer::new(grid_size));

    let ocean_mesh = Plane3d::default().mesh().size(23.0, 23.0).subdivisions(10);

    let ocean_id = commands
        .spawn((
            Ocean,
            Mesh3d(meshes.add(ocean_mesh)),
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color: Color::srgba(0.0, 0.3, 0.6, 0.4),
                alpha_mode: AlphaMode::Blend,
                perceptual_roughness: 0.08,
                metallic: 0.2,
                ..default()
            })),
            Transform::from_xyz(0.0, -0.25, 0.0),
        ))
        .id();
    et.ocean = Some(ocean_id);

    // Ocean Drag Observer for Camera Anchor
    commands.entity(ocean_id).observe(
        |mut drag: On<Pointer<Drag>>, et: Res<EntityTable>, mut query: Query<&mut Transform>| {
            drag.propagate(false);
            if let Some(mut transform) = et.main_anchor.and_then(|id| query.get_mut(id).ok()) {
                transform.translation.x -= drag.delta.x * 0.015;
                transform.translation.z -= drag.delta.y * 0.015;
            }
        },
    );

    ocean_id
}

pub fn apply_camera_repulsion(
    mut water: ResMut<OceanBuffer>,
    et: Res<EntityTable>,
    global_transforms: Query<&GlobalTransform>,
) {
    let (Some(cam_id), Some(anchor_id)) = (et.main_camera, et.main_anchor) else {
        return;
    };

    let Ok(cam_gtf) = global_transforms.get(cam_id) else {
        return;
    };
    let Ok(anchor_gtf) = global_transforms.get(anchor_id) else {
        return;
    };

    let cam_pos = cam_gtf.translation();
    let anchor_pos = anchor_gtf.translation();
    let dist = cam_pos.distance(anchor_pos);

    let repulsion_radius = ((15.0 - dist) / 15.0 * 6.0).max(0.0);
    let r_sq = repulsion_radius * repulsion_radius;
    let push_depth = ((15.0 - dist) / 15.0 * -2.0).min(0.0);

    let size = water.size;
    let step = 2.0;
    let anchor_xz = anchor_pos.xz();

    for z in 1..size - 1 {
        for x in 1..size - 1 {
            let i = z * size + x;
            let w_pos = Vec2::new((x as f32 * step) - 10.0, (z as f32 * step) - 10.0);

            if w_pos.distance_squared(anchor_xz) < r_sq {
                water.current[i] = push_depth;
            }
        }
    }
}

pub fn simulate_waves(
    mut water: ResMut<OceanBuffer>,
    et: Res<EntityTable>,
    global_transforms: Query<&GlobalTransform>,
) {
    let Some(disk_id) = et.disk else { return };
    let Ok(disk_gtf) = global_transforms.get(disk_id) else {
        return;
    };
    let disk_xz = disk_gtf.translation().xz();

    let size = water.size;
    for z in 1..size - 1 {
        for x in 1..size - 1 {
            let i = z * size + x;
            let w_pos = Vec2::new((x as f32 * 2.0) - 10.0, (z as f32 * 2.0) - 10.0);

            if w_pos.distance_squared(disk_xz) < 16.5 {
                water.previous[i] = 0.0;
                continue;
            }

            let avg = (water.current[i - size - 1]
                + water.current[i - size]
                + water.current[i - size + 1]
                + water.current[i - 1]
                + water.current[i]
                + water.current[i + 1]
                + water.current[i + size - 1]
                + water.current[i + size]
                + water.current[i + size + 1])
                / 9.0;

            water.previous[i] = (avg * 2.0 - water.previous[i]) * 0.98;
        }
    }
    water.swap();
}

pub fn update_ocean_mesh(
    water: Res<OceanBuffer>,
    mut meshes: ResMut<Assets<Mesh>>,
    query: Query<&Mesh3d, With<Ocean>>,
) {
    for mesh_3d in &query {
        if let Some(mesh) = meshes.get_mut(&mesh_3d.0) {
            if let Some(VertexAttributeValues::Float32x3(pos)) =
                mesh.attribute_mut(Mesh::ATTRIBUTE_POSITION)
            {
                for (i, p) in pos.iter_mut().enumerate() {
                    if i < water.current.len() {
                        p[1] = water.current[i];
                    }
                }
            }
        }
    }
}
