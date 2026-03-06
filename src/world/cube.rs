//! # Cube System
//!
//! Rotating cube with animations (Slide, Spin, Jump, Flip).
//! Handles buoyancy, drag interaction, and jump targeting.

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
use crate::world::camera::*;
use crate::{world::ocean::OceanBuffer, EntityTable};
use bevy::ecs::relationship::Relationship;
use bevy::prelude::EaseFunction::{BounceInOut, ElasticInOut};
use bevy::prelude::*;

/// Cube rotation speed resource
#[derive(Debug, Resource)]
pub struct CubeParms {
  pub rotation_speed: f32,
}

/// Component marking the rotating cube
#[derive(Debug, Component, Default, Reflect)]
#[reflect(Component)]
#[require(Transform, Visibility)]
pub struct RotatingCube;

/// Animation type for cube movement
#[derive(Debug, Clone, Copy, PartialEq, Reflect)]
pub enum AnimationType {
  Jump,
  Slide,
  Spin,
  Flip,
}

/// Data for active jump animation
#[derive(Debug, Component)]
pub struct JumpData {
  pub world_start: Vec3,
  pub start_rotation: Quat,
  pub local_target: Vec3,
  pub timer: f32,
  pub duration: f32,
  pub target_entity: Entity,
  pub animation: Option<AnimationType>,
}

/// Spawn rotating cube with roundel faces
pub fn spawn_rotating_cube(
  commands: &mut Commands,
  meshes: &mut ResMut<Assets<Mesh>>,
  materials: &mut ResMut<Assets<StandardMaterial>>,
  roundel_mat: StandardMaterial,
  et: &mut ResMut<EntityTable>,
) -> Entity {
  let half = CUBE_WORLD_SIDE_LEN * 0.5;
  let shy = half - 0.01;
  let cube_id = commands
    .spawn((
      RotatingCube,
      Transform::from_xyz(0.0, half + DISK_WORLD_Y, 0.0),
    ))
    .id();
  et.cube = Some(cube_id);

  let face_data = [
    (
      Vec3::new(0.0, 0.0, shy), //
      Quat::IDENTITY,
    ),
    (
      Vec3::new(0.0, 0.0, -shy),
      Quat::from_rotation_y(std::f32::consts::PI),
    ),
    (
      Vec3::new(shy, 0.0, 0.0),
      Quat::from_rotation_y(std::f32::consts::FRAC_PI_2),
    ),
    (
      Vec3::new(-shy, 0.0, 0.0),
      Quat::from_rotation_y(-std::f32::consts::FRAC_PI_2),
    ),
    (
      Vec3::new(0.0, shy, 0.0),
      Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2),
    ),
    (
      Vec3::new(0.0, -shy, 0.0),
      Quat::from_rotation_x(std::f32::consts::FRAC_PI_2),
    ),
  ];

  commands.entity(cube_id).with_children(|parent| {
    parent.spawn((
      Mesh3d(meshes.add(Sphere::new(0.1).mesh().uv(32, 18))),
      MeshMaterial3d(materials.add(StandardMaterial {
        base_color: Color::srgb(0.75, 0.25, 1.0),
        unlit: true,
        ..default()
      })),
    ));
    for (offset, rotation) in face_data {
      parent.spawn((
        Mesh3d(meshes.add(Circle::new(0.90).mesh().resolution(128))),
        MeshMaterial3d(materials.add(roundel_mat.clone())),
        Transform::from_translation(offset).with_rotation(rotation),
      ));
      parent.spawn((
        Mesh3d(meshes.add(Circle::new(0.90).mesh().resolution(128))),
        MeshMaterial3d(materials.add(StandardMaterial {
          emissive: LinearRgba::from(Color::srgb(0.75, 0.25, 1.0)) * 0.03,
          ..roundel_mat.clone()
        })),
        Transform {
          translation: offset * 0.99,
          rotation: rotation * Quat::from_rotation_y(std::f32::consts::PI),
          scale: Vec3::splat(0.995),
        },
      ));
    }
  });

  commands.entity(cube_id).observe(
    |mut click: On<Pointer<Click>>, mut camera_res: ResMut<CameraAnchorRes>| {
      crate::ui::main_ui::request_view(&mut camera_res);
      click.propagate(false);
    },
  );

  commands.entity(cube_id).observe(
    |mut drag: On<Pointer<Drag>>, mut settings: ResMut<CubeParms>| {
      drag.propagate(false);
      settings.rotation_speed += drag.delta.x * 0.005;
    },
  );

  cube_id
}

/// Handle jump request from common click event
// Over evolved muck, needs demucking
pub fn handle_jump_request(
  mut click: On<Pointer<Click>>,
  mut commands: Commands,
  et: Res<EntityTable>,
  jump_check: Query<&JumpData>,
  global_query: Query<&GlobalTransform>,
  mut camera_res: ResMut<CameraAnchorRes>,
  water: Res<OceanBuffer>,
) {
  // All oceans must be present
  let entities = [et.ocean, et.ocean_wire, et.ocean_point, et.ocean_fake];
  for e in entities.iter() {
    if e.is_none() {
      return;
    };
  }
  // It's a drag if > 1/4sec, we want clicks
  if click.duration.as_millis() > 250 {
    return;
  }
  // Go Main Menu if right button
  if click.button == PointerButton::Secondary {
    crate::ui::main_ui::request_view(&mut camera_res);
    click.propagate(false);
    return;
  }
  // Pop Menu stack if middle button
  if click.button == PointerButton::Middle {
    camera_res.request_back();
    click.propagate(false);
    return;
  }
  // Eat the click if we got this far (don't pass it along)
  click.propagate(false);
  // Must have hit something
  let Some(hit_pos) = click.hit.position else {
    return;
  };
  // Cube must exist
  let Some(cube_entity) = et.cube else { return };
  // Target must not be cube
  if jump_check.contains(cube_entity) {
    return;
  }

  // Get the target
  let target_entity = click.event_target();
  // Get the world cube
  let Ok(cube_global) = global_query.get(cube_entity) else {
    return;
  };
  // Get the world target
  let Ok(target_global) = global_query.get(target_entity) else {
    return;
  };

  //
  let local_target = target_global.affine().inverse().transform_point3(hit_pos);
  let mut l = local_target;

  let mut final_entity = target_entity;

  // Adjust cube position and depth depending on target
  let half = CUBE_WORLD_SIDE_LEN * 0.5;
  if Some(target_entity) == et.disk {
    // Disk target, raise enuf to rest on it
    l.y += half;
  } else {
    // Ground target, float on water unless too shallow
    final_entity = et.ground.unwrap();
    let cube_bottom_offset = half;
    let cube_float_offset = cube_bottom_offset - 2. / 3.;
    l.y = (water.get_depth(l.x, l.z) + cube_float_offset).max(cube_bottom_offset);
    // Move target outside disk if colliding
    let min_distance_to_disk_center =
      DISK_WORLD_RADIUS + ((CUBE_WORLD_SIDE_LEN * 0.5).powf(2.0) * 2.0).sqrt();
    l = l
      .xz()
      .clamp_length_min(min_distance_to_disk_center)
      .extend(l.y)
      .xzy();
  }

  // Unparent to animate in world coordinates
  commands.entity(cube_entity).remove_parent_in_place();
  // Pass enuf info for cube to get to local coords of entity clicked on.
  // Cube takes shortest path to future place of clicked disk spot.
  commands.entity(cube_entity).insert(JumpData {
    world_start: cube_global.translation(),
    start_rotation: cube_global.compute_transform().rotation,
    local_target: l,
    timer: 0.0,
    duration: 3.0,
    target_entity: final_entity,
    animation: None,
  });
}

/// Update cube rotation based on current rotation speed
pub fn rotate_cube(
  et: Res<EntityTable>,
  mut query: Query<&mut Transform>,
  settings: Res<CubeParms>,
  time: Res<Time>,
) {
  if let Some(id) = et.cube {
    if let Ok(mut transform) = query.get_mut(id) {
      transform.rotate_local_y(settings.rotation_speed * time.delta_secs());
    }
  }
}

/// Update jump animation per frame
pub fn update_jump(
  mut commands: Commands,
  time: Res<Time>,
  et: Res<EntityTable>,
  mut water: ResMut<OceanBuffer>,
  target_query: Query<&GlobalTransform>,
  mut cube_query: Query<(&mut Transform, &mut JumpData)>,
) {
  // Only if able to get cube, transform, and JumpData
  if let Some(cube_entity) = et.cube {
    if let Ok((mut transform, mut data)) = cube_query.get_mut(cube_entity) {
      // oceans must exist
      let entities = [et.ocean, et.ocean_wire, et.ocean_point, et.ocean_fake];
      for e in entities.iter() {
        if e.is_none() {
          return;
        };
      }

      // Get target
      if let Ok(target_global) = target_query.get(data.target_entity) {
        // Get start and finish
        let local_start = target_global
          .affine()
          .inverse()
          .transform_point3(data.world_start);
        let target_pos = data.local_target;
        // Set animation based on distance to travel
        let anim_type = *data.animation.get_or_insert_with(|| {
          let d = local_start.distance(target_pos);
          if d < 1.5 {
            AnimationType::Slide
          } else if d < 2.5 {
            AnimationType::Spin
          } else if d < 4.5 {
            AnimationType::Jump
          } else {
            AnimationType::Flip
          }
        });
        // Scale time from 0.0 to 1.0 to animate from start to finish
        data.timer += time.delta_secs();
        let t = (data.timer / data.duration).clamp(0.0, 1.0);
        // Convert t to elastic and bounce, all three used in various ways.
        // Use elastic only when it can't cause collision, bounce can't.
        // Use t for smooth motion like rotation or to tone down ease functions.
        let (elastic_t, bounce_t) = (
          ElasticInOut.sample_unchecked(t),
          BounceInOut.sample_unchecked(t),
        );
        let mut bounce_height =
          (DISK_WORLD_Y - GROUND_WORLD_Y + 4.) * (0.5 - (bounce_t - 0.5).abs());
        match anim_type {
          AnimationType::Slide | AnimationType::Spin => {
            // Don't bounce for these two
            bounce_height = 0.;
            transform.rotation = data.start_rotation
              * Quat::from_rotation_y(
                t * std::f32::consts::PI
                  * if anim_type == AnimationType::Slide {
                    // exactly one rotation for Slide
                    2.
                  } else {
                    // rotate twice as much for Spin
                    4.
                  },
              );
          }
          AnimationType::Jump => {
            transform.rotation =
              // three rotations for Jump
              data.start_rotation * Quat::from_rotation_y(t * std::f32::consts::PI * 6.0);
          }
          AnimationType::Flip => {
            let yaw = 6. * std::f32::consts::PI * t;
            if t > 0.0909 && t < 0.9090 {
              // Only tumble when we are high/small enuf to not collide with hard surface
              transform.rotation = data.start_rotation
                * Quat::from_rotation_y(yaw)
                // two tumbles
                * Quat::from_rotation_x(4. * std::f32::consts::PI * (t - 0.0909) * 1.2222);
              let world_pos = transform.translation;
              water.splash(world_pos.x, world_pos.z, 0.5, 3.0);
            } else {
              // three rotations for Flip
              transform.rotation = data.start_rotation * Quat::from_rotation_y(yaw);
            }
          }
        }
        // Shrink to half size in all dimensions then grow back
        transform.scale = Vec3::splat((2. * (0.5 - t).abs()).clamp(0.5, 1.));
        // Elastic overshoot at start and finish, bounce to height if not zero'd
        transform.translation = target_global
          .transform_point(local_start.lerp(data.local_target, elastic_t))
          + Vec3::new(0., bounce_height, 0.);

        if t >= 1. {
          // time is up, make sure scale is reset
          transform.scale = Vec3::splat(1.);

          // splash
          let world_pos = transform.translation;
          water.splash(world_pos.x, world_pos.z, 1.0, 3.0);

          // parent to target. We stick to spinning disk, etc.
          commands
            .entity(cube_entity)
            .set_parent_in_place(data.target_entity);
          // remove JumpData so code stops until next move
          commands.entity(cube_entity).remove::<JumpData>();
        }
      }
    }
  }
}

/// Apply buoyancy based on ocean height
pub fn apply_buoyancy(
  et: Res<EntityTable>,
  water: Res<OceanBuffer>,
  mut query: Query<&mut Transform, (With<RotatingCube>, Without<JumpData>)>,
  parent_query: Query<&ChildOf>,
) {
  let entities = [et.ocean, et.ocean_wire, et.ocean_point];
  for e in entities.iter() {
    if e.is_none() {
      return;
    };
  }

  if let (Some(cube_id), Some(disk_id)) = (et.cube, et.disk) {
    if let Ok(mut transform) = query.get_mut(cube_id) {
      if let Ok(parent) = parent_query.get(cube_id) {
        if parent.get() != disk_id {
          let cur = transform.translation;
          let mut next = cur;

          let water_depth = water.get_depth(cur.x, cur.z);
          let cube_bottom_offset = CUBE_WORLD_SIDE_LEN * 0.5;
          let cube_float_offset = cube_bottom_offset - 2. / 3.;
          let cube_next_float_immediate = water_depth + cube_float_offset;
          let total_distance = cube_next_float_immediate - cur.y;
          let fractional_distance = total_distance / 3.;
          next.y = (cur.y + fractional_distance).max(cube_bottom_offset);

          let slope_x =
            (water.get_height(cur.x - 1., cur.z) - water.get_height(cur.x + 1., cur.z)) / 3.;
          let slope_z =
            (water.get_height(cur.x, cur.z - 1.) - water.get_height(cur.x, cur.z + 1.)) / 3.;

          if slope_x.abs() + slope_z.abs() > 0.002 {
            next.x += slope_x.clamp(-0.02, 0.02);
            next.z += slope_z.clamp(-0.02, 0.02);
          }

          let min_distance_to_disk_center =
            DISK_WORLD_RADIUS + ((CUBE_WORLD_SIDE_LEN * 0.5).powf(2.0) * 2.0).sqrt();
          next = next
            .xz()
            .clamp_length_min(min_distance_to_disk_center)
            .extend(next.y)
            .xzy();

          transform.translation = next;
        }
      }
    }
  }
}
