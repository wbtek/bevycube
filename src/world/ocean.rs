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

use crate::constants::*;
use crate::world::camera::CameraAnchorRes;
use crate::EntityTable;
use bevy::camera::visibility::RenderLayers;
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
          self.current[i] = (self.current[i] + magnitude * falloff).max(OCEAN_TO_GROUND - 0.01);
          self.next[i] = (self.next[i] + magnitude * falloff).max(OCEAN_TO_GROUND - 0.01);
        }
      }
    }
  }

  // relative, +-0
  pub fn get_height(&self, x: f32, z: f32) -> f32 {
    let size = self.size as f32;
    let side_len = self.side_length;

    let col = ((x + side_len / 2.0) / side_len * (size - 1.0))
      .round()
      .clamp(0.0, size - 1.0) as usize;
    let row = ((z + side_len / 2.0) / side_len * (size - 1.0))
      .round()
      .clamp(0.0, size - 1.0) as usize;

    self.current[row * self.size + col]
  }

  // positive depth to ground
  pub fn get_depth(&self, x: f32, z: f32) -> f32 {
    self.get_height(x, z) - GROUND_WORLD_Y + OCEAN_WORLD_Y
  }
}

#[derive(Debug, Component)]
#[require(Transform, Visibility)]
pub struct Ocean;

use bevy::asset::RenderAssetUsages;
use bevy::mesh::Indices;
use bevy::mesh::PrimitiveTopology;

pub fn generate_wireframe_from_mesh(source_mesh: &Mesh) -> Mesh {
  // create new LineList mesh
  let mut wireframe_mesh = Mesh::new(PrimitiveTopology::LineList, RenderAssetUsages::default());

  // copy same vertex positions
  if let Some(positions) = source_mesh.attribute(Mesh::ATTRIBUTE_POSITION) {
    wireframe_mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions.clone());
  }

  // convert triangle triplets [0, 1, 2] into line pairs [0, 1, 1, 2, 2, 0]
  if let Some(indices) = source_mesh.indices() {
    let mut line_indices = Vec::new();

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

  // copy positions like we did for lines
  if let Some(positions) = source_mesh.attribute(Mesh::ATTRIBUTE_POSITION) {
    point_mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions.clone());
  }
  // point lists don't technically need indices
  // keep so GPU knows what vertices to draw.
  if let Some(indices) = source_mesh.indices() {
    point_mesh.insert_indices(indices.clone());
  }

  point_mesh
}

pub fn spawn_ocean_fake(
  commands: &mut Commands,
  meshes: &mut ResMut<Assets<Mesh>>,
  materials: &mut ResMut<Assets<StandardMaterial>>,
  et: &mut ResMut<EntityTable>,
) -> Entity {
  let side_length = OCEAN_WORLD_SIDE_LEN;
  let world_y = OCEAN_WORLD_Y;

  let ocean_fake_mesh = Plane3d::default()
    .mesh()
    .size(side_length, side_length)
    .subdivisions(0)
    .build();

  let ocean_fake_id = commands
    .spawn((
      Ocean,
      Mesh3d(meshes.add(ocean_fake_mesh)),
      MeshMaterial3d(materials.add(StandardMaterial {
        base_color: Color::srgba(0.0, 0.0, 0.0, 0.0),
        alpha_mode: AlphaMode::Blend,
        ..default()
      })),
      bevy::picking::Pickable {
        is_hoverable: true,
        should_block_lower: false,
      },
      Transform::from_xyz(0.0, world_y, 0.0),
    ))
    .id();
  et.ocean_fake = Some(ocean_fake_id);

  ocean_fake_id
}

pub fn spawn_ocean(
  commands: &mut Commands,
  meshes: &mut ResMut<Assets<Mesh>>,
  materials: &mut ResMut<Assets<StandardMaterial>>,
  et: &mut ResMut<EntityTable>,
  dimension: u32,
) -> Entity {
  let mut clean = |id: &mut Option<Entity>| {
    if let Some(e) = id.take() {
      commands.entity(e).despawn();
    }
  };

  clean(&mut et.ocean);
  clean(&mut et.ocean_wire);
  clean(&mut et.ocean_point);

  let grid_size = 1 + dimension as usize;
  let side_length = OCEAN_WORLD_SIDE_LEN;
  let world_y = OCEAN_WORLD_Y;
  commands.insert_resource(OceanBuffer::new(grid_size, side_length, world_y));

  let ocean_mesh = Plane3d::default()
    .mesh()
    .size(side_length, side_length)
    .subdivisions((grid_size - 2) as u32)
    .build();

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
      bevy::picking::Pickable {
        is_hoverable: false,
        should_block_lower: false,
      },
      RenderLayers::layer(1),
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
      RenderLayers::layer(1),
      Transform::from_xyz(0.0, world_y, 0.0),
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
      RenderLayers::layer(1),
      Transform::from_xyz(0.0, world_y, 0.0),
    ))
    .id();
  et.ocean_point = Some(ocean_point_id);

  ocean_id
}

pub fn apply_camera_repulsion(
  mut water: ResMut<OceanBuffer>,
  anchor: ResMut<CameraAnchorRes>,
  et: Res<EntityTable>,
) {
  let entities = [et.ocean, et.ocean_wire, et.ocean_point];
  for e in entities.iter() {
    if e.is_none() {
      return;
    };
  }

  let dist = anchor.current.get_camera_effect();

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
        .distance_squared(anchor.current.anchor.xz())
        < r_sq
      {
        water.next[i] = push_depth.max(OCEAN_TO_GROUND - 0.01);
      }
    }
  }
}

pub fn simulate_waves(
  mut water: ResMut<OceanBuffer>,
  et: Res<EntityTable>,
  global_transforms: Query<&GlobalTransform>,
) {
  let entities = [et.ocean, et.ocean_wire, et.ocean_point];
  for e in entities.iter() {
    if e.is_none() {
      return;
    };
  }

  let Some(disk_id) = et.disk else { return };
  let Ok(disk_gtf) = global_transforms.get(disk_id) else {
    return;
  };
  let disk_xz = disk_gtf.translation().xz();

  let size = water.size;
  for z in 1..size - 1 {
    for x in 1..size - 1 {
      let i = z * size + x;

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
      // clamp between just below ground and just below disk if under disk
      let pre_clamp = (avg * 2.0 - water.next[i]) * 0.98;
      let low_clamp = OCEAN_TO_GROUND - 0.01;
      let mut high_clamp = f32::MAX;
      if water.get_world_pos(x, z).xz().distance_squared(disk_xz) < DISK_WORLD_R2 {
        high_clamp = DISK_WORLD_Y - OCEAN_WORLD_Y - 0.01;
      }
      water.next[i] = pre_clamp.clamp(low_clamp, high_clamp);
    }
  }
}

pub fn update_ocean_mesh(
  water: Res<OceanBuffer>,
  et: Res<EntityTable>,
  mut meshes: ResMut<Assets<Mesh>>,
  query: Query<&Mesh3d, With<Ocean>>,
) {
  let entities = [et.ocean, et.ocean_wire, et.ocean_point];
  for e in entities.iter() {
    if e.is_none() {
      return;
    };
  }

  for mesh_3d in &query {
    if let Some(mesh) = meshes.get_mut(&mesh_3d.0) {
      if let Some(VertexAttributeValues::Float32x3(pos)) =
        mesh.attribute_mut(Mesh::ATTRIBUTE_POSITION)
      {
        for (i, p) in pos.iter_mut().enumerate() {
          if i < water.current.len() {
            p[1] = water.current[i].max(OCEAN_TO_GROUND - 0.01);
          }
        }
      }
    }
  }
}

pub fn clamp_edges(mut water: ResMut<OceanBuffer>, et: Res<EntityTable>) {
  let entities = [et.ocean, et.ocean_wire, et.ocean_point];
  for e in entities.iter() {
    if e.is_none() {
      return;
    };
  }

  water.zap_edges();
}

pub fn swap_and_copy(mut water: ResMut<OceanBuffer>, et: Res<EntityTable>) {
  let entities = [et.ocean, et.ocean_wire, et.ocean_point];
  for e in entities.iter() {
    if e.is_none() {
      return;
    };
  }

  water.swap();
}
