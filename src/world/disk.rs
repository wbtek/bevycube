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

#[derive(Debug, Resource)]
pub struct DiskParms {
    pub rotation_speed: f32,
}

#[derive(Debug, Component, Default, Reflect)]
#[reflect(Component)]
#[require(Transform, Visibility)]
pub struct RotatingDisk;

pub fn spawn_rotating_disk(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    roundel_mat: StandardMaterial,
    et: &mut ResMut<EntityTable>,
) -> Entity {
    let mesh = Circle::new(4.0)
        .mesh()
        .resolution(128)
        .build()
        .rotated_by(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2));

    let disk_id = commands
        .spawn((
            RotatingDisk,
            Mesh3d(meshes.add(mesh)),
            MeshMaterial3d(materials.add(roundel_mat)),
            Transform::IDENTITY,
        ))
        .id();

    et.disk = Some(disk_id);

    commands.entity(disk_id).observe(
        |mut drag: On<Pointer<Drag>>, mut settings: ResMut<DiskParms>| {
            drag.propagate(false);
            settings.rotation_speed += drag.delta.x * 0.001;
        },
    );

    disk_id
}

pub fn rotate_disk(
    et: Res<EntityTable>,
    mut query: Query<&mut Transform>,
    settings: Res<DiskParms>,
    time: Res<Time>,
) {
    if let Some(id) = et.disk {
        if let Ok(mut transform) = query.get_mut(id) {
            transform.rotate_local_y(settings.rotation_speed * time.delta_secs());
        }
    }
}
