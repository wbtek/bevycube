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

use crate::world::ground::GroundConfig;
use crate::EntityTable;
use bevy::mesh::VertexAttributeValues;
use bevy::prelude::*;

#[derive(Resource)]
pub struct OceanBuffer {
  pub current: Vec<f32>,
  pub next: Vec<f32>,
  pub size: usize,
  pub side_length: f32,
  pub world_y: f32,
}

impl OceanBuffer {
  pub fn new(size: usize, side_length: f32, world_y: f32) -> Self {
    let count = size * size;
    Self {
      current: vec![0.0; count],
      next: vec![0.0; count],
      size,
      side_length,
      world_y,
    }
  }

  pub fn spacing(&self) -> f32 {
    self.side_length / (self.size - 1) as f32
  }

  pub fn get_world_pos(&self, x_idx: usize, z_idx: usize) -> Vec3 {
    let offset = self.side_length / 2.;
    let x = (x_idx as f32 * self.spacing()) - offset;
    let z = (z_idx as f32 * self.spacing()) - offset;
    Vec3::new(x, 0., z)
  }

  pub fn swap(&mut self) {
    std::mem::swap(&mut self.current, &mut self.next);
  }

  pub fn zap_edges(&mut self) {
    let left_column = 0;
    let right_column = self.size - 1;
    let top_row = 0;
    let bottom_row = self.size - 1;

    for n in 1..self.size - 1 {
      // not the corners

      let row = n;
      self.current[row * self.size + left_column] = 0.;
      self.current[row * self.size + right_column] = 0.;

      let col = n;
      self.current[top_row * self.size + col] = 0.;
      self.current[bottom_row * self.size + col] = 0.;
    }

    // the corners
    self.current[top_row * self.size + left_column] = 0.;
    self.current[top_row * self.size + right_column] = 0.;
    self.current[bottom_row * self.size + left_column] = 0.;
    self.current[bottom_row * self.size + right_column] = 0.;
  }

  /// Injects a vertical displacement at a specific world coordinate.
  pub fn splash(&mut self, x: f32, z: f32, magnitude: f32, diameter: f32) {
    // let size = self.size as f32;
    // let spacing = 20.0 / (size - 1.0);
    let spacing = self.spacing();
    let side_length = self.side_length;
    let r_sq = (diameter / 2.0).powi(2);

    for row in 0..self.size {
      for col in 0..self.size {
        let i = row * self.size + col;
        let vx = (col as f32 * spacing) - side_length / 2.;
        let vz = (row as f32 * spacing) - side_length / 2.;

        let dist_sq = (vx - x).powi(2) + (vz - z).powi(2);
        if dist_sq < r_sq {
          let falloff = 1.0 - (dist_sq / r_sq).sqrt();
          self.current[i] += magnitude * falloff;
          self.next[i] += magnitude * falloff;
        }
      }
    }
  }

  pub fn get_height(&self, x: f32, z: f32) -> f32 {
    let size = self.size as f32;

    let col = ((x + 10.0) / 20.0 * (size - 1.0))
      .round()
      .clamp(0.0, size - 1.0) as usize;
    let row = ((z + 10.0) / 20.0 * (size - 1.0))
      .round()
      .clamp(0.0, size - 1.0) as usize;

    self.current[row * self.size + col]
  }
}

#[derive(Debug, Component)]
#[require(Transform, Visibility)]
pub struct Ocean;

use bevy::asset::RenderAssetUsages;
use bevy::mesh::Indices;
use bevy::mesh::PrimitiveTopology;

/*
// Works: Functionally equivalent, modifyable
fn create_foo_mesh(x_units: f32, z_units: f32, subdivisions: u32) -> Mesh {
    let grid_res = subdivisions + 2;
    let total_vertices = (grid_res * grid_res) as usize;
    let x_spacing = x_units / (grid_res as f32 - 1.0);
    let z_spacing = z_units / (grid_res as f32 - 1.0);

    let mut positions = Vec::with_capacity(total_vertices);
    let mut indices = Vec::new();

    // 1. Position mapping (foo = 144 vertices)
    for i in 0..total_vertices as u32 {
        let row = i / grid_res;
        let col = i % grid_res;
        let x = (col as f32 * x_spacing) - (x_units / 2.0);
        let z = (row as f32 * z_spacing) - (z_units / 2.0);
        positions.push([x, 0.0, z]);
    }

    // 2. Index mapping (Standard CCW winding)
    for row in 0..grid_res - 1 {
        for col in 0..grid_res - 1 {
            let i = row * grid_res + col;
            indices.extend_from_slice(&[i, i + grid_res, i + 1]);
            indices.extend_from_slice(&[i + 1, i + grid_res, i + grid_res + 1]);
        }
    }

    // 3. Bevy 0.18 Constructor
    let mut mesh = Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::default() // Required for dynamic updates in 0.18
    );

    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    mesh.insert_indices(Indices::U32(indices));
    // Required for PBR and Solari lighting in 0.18
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, vec![[0.0, 1.0, 0.0]; total_vertices]);

    mesh
}
*/

/*
// Works: proves we can accept the parameters and return the right value.
fn create_foo_mesh(x_units: f32, z_units: f32, subdivisions: u32) -> Mesh {
    Plane3d::default().mesh().size(x_units, z_units).subdivisions(subdivisions).build()
}
*/

pub fn generate_wireframe_from_mesh(source_mesh: &Mesh) -> Mesh {
  // 1. Create a new mesh using LineList topology
  let mut wireframe_mesh = Mesh::new(PrimitiveTopology::LineList, RenderAssetUsages::default());

  // 2. Copy the exact same vertex positions (No new generation needed)
  if let Some(positions) = source_mesh.attribute(Mesh::ATTRIBUTE_POSITION) {
    wireframe_mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions.clone());
  }

  // 3. Convert Triangle Triplets [0, 1, 2] into Line Pairs [0, 1, 1, 2, 2, 0]
  if let Some(indices) = source_mesh.indices() {
    let mut line_indices = Vec::new();

    // This iterator works for both u16 and u32 indices
    let mut iter = indices.iter();
    while let (Some(a), Some(b), Some(c)) = (iter.next(), iter.next(), iter.next()) {
      // Triangle edges: A-B, B-C, C-A
      line_indices.extend_from_slice(&[a as u32, b as u32, b as u32, c as u32, c as u32, a as u32]);
    }

    wireframe_mesh.insert_indices(Indices::U32(line_indices));
  }

  wireframe_mesh
}

pub fn generate_points_from_mesh(source_mesh: &Mesh) -> Mesh {
  let mut point_mesh = Mesh::new(
    PrimitiveTopology::PointList, // Tell the GPU: "One vertex = One point"
    RenderAssetUsages::default(),
  );

  // Copy the positions exactly like we did for lines
  if let Some(positions) = source_mesh.attribute(Mesh::ATTRIBUTE_POSITION) {
    point_mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions.clone());
  }

  // Point lists don't technically need indices, but keeping them
  // ensures the GPU knows which vertices to draw.
  if let Some(indices) = source_mesh.indices() {
    point_mesh.insert_indices(indices.clone());
  }

  point_mesh
}

pub fn spawn_ocean(
  commands: &mut Commands,
  meshes: &mut ResMut<Assets<Mesh>>,
  materials: &mut ResMut<Assets<StandardMaterial>>,
  et: &mut ResMut<EntityTable>,
) -> Entity {
  // let grid_size = 12;
  // let side_length = 23.0;
  let grid_size = 40;
  let side_length = 20.0;
  let world_y = -0.25;
  commands.insert_resource(OceanBuffer::new(grid_size, side_length, world_y));

  // let ocean_mesh = Plane3d::default().mesh().size(23.0, 23.0).subdivisions(10).build();
  let ocean_mesh = Plane3d::default()
    .mesh()
    .size(side_length, side_length)
    .subdivisions((grid_size - 2) as u32)
    .build();
  // let ocean_mesh = create_foo_mesh(23., 23., 10);

  let wire_mesh = generate_wireframe_from_mesh(&ocean_mesh);
  let point_mesh = generate_points_from_mesh(&ocean_mesh);

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
      Transform::from_xyz(0.0, world_y, 0.0),
    ))
    .id();
  et.ocean = Some(ocean_id);

  let ocean_wire_id = commands
    .spawn((
      Ocean,
      Mesh3d(meshes.add(wire_mesh)),
      MeshMaterial3d(materials.add(StandardMaterial {
        base_color: Color::WHITE,
        unlit: false,
        ..default()
      })),
      Visibility::Hidden,
    ))
    .id();
  et.ocean_wire = Some(ocean_wire_id);

  let ocean_point_id = commands
    .spawn((
      Ocean,
      Mesh3d(meshes.add(point_mesh)),
      MeshMaterial3d(materials.add(StandardMaterial {
        base_color: Color::WHITE,
        unlit: true,
        ..default()
      })),
      Visibility::Hidden,
    ))
    .id();
  et.ocean_point = Some(ocean_point_id);

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
  let push_depth = ((15.0 - dist) / 15.0 * -5.0).min(0.0);

  let size = water.size;

  for z in 1..size - 1 {
    for x in 1..size - 1 {
      let i = z * size + x;

      if water
        .get_world_pos(x, z)
        .xz()
        .distance_squared(anchor_pos.xz())
        < r_sq
      {
        water.next[i] = push_depth;
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

      if water.get_world_pos(x, z).xz().distance_squared(disk_xz) < 16.5 {
        water.next[i] = -0.5;
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
      water.next[i] = (avg * 2.0 - water.next[i]) * 0.98;
    }
  }
}

pub fn update_ocean_mesh(
  water: Res<OceanBuffer>,
  ground_config: Res<GroundConfig>,
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
            p[1] = water.current[i].max(ground_config.world_y);
          }
        }
      }
    }
  }
}

pub fn clamp_edges(mut water: ResMut<OceanBuffer>) {
  water.zap_edges();
}

pub fn swap_and_copy(mut water: ResMut<OceanBuffer>) {
  water.swap();
}
