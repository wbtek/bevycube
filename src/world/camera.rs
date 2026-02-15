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
use std::f32::consts::PI;

pub struct CameraPlugin;

impl Plugin for CameraPlugin {
  fn build(&self, app: &mut App) {
    app.add_systems(Update, (update_camera_zoom, update_mobile_zoom));
  }
}

#[derive(Component)]
#[require(Transform, Visibility)]
pub struct CameraAnchor;
#[derive(Component)]
pub struct MainCamera;

#[derive(Debug, Clone, Copy)]
pub struct CameraParams {
  pub anchor: Vec3,
  pub track_near_end_y: f32,
  pub direction: f32,
  pub slope: f32,
  pub zoom: f32,
}

impl Default for CameraParams {
  fn default() -> Self {
    Self {
      anchor: Vec3::ZERO,
      track_near_end_y: 4.0,
      direction: 0.0,
      slope: 0.0,
      zoom: 15.0,
    }
  }
}

impl CameraParams {
  pub fn update_pan(&mut self, delta_x: f32, delta_y: f32) {
    let (sin, cos) = self.direction.sin_cos();
    let rotated_x = delta_x * cos + delta_y * sin;
    let rotated_z = -delta_x * sin + delta_y * cos;
    self.anchor += Vec3::new(rotated_x, 0.0, rotated_z);
  }

  pub fn update_orbit(&mut self, delta_x: f32, delta_y: f32) {
    self.slope = (self.slope + delta_y).clamp(0.0, 1.5);
    self.direction = (self.direction + delta_x).rem_euclid(PI * 2.);
  }

  pub fn update_zoom(&mut self, delta: f32) {
    self.zoom = (self.zoom - delta).clamp(0.01, 40.0);
  }

  // Calculates where the camera should be relative to the anchor
  pub fn get_camera_offset(&self) -> Vec3 {
    // direction 0 is North (towards -Z).
    // We calculate horizontal X and Z using direction, then scale by the slope's cosine.
    let vertical_scale = self.slope.sin();
    let horizontal_scale = self.slope.cos();

    let x = self.direction.sin() * horizontal_scale * self.zoom;
    let y = self.track_near_end_y + (vertical_scale * self.zoom);
    let z = self.direction.cos() * horizontal_scale * self.zoom;

    Vec3::new(x, y, z)
  }

  pub fn get_anchor_xyz(&self) -> Vec3 {
    self.anchor
  }

  pub fn get_camera_effect(&self) -> f32 {
    self.track_near_end_y + self.zoom
  }
}

pub struct CameraMotion {
  pub from: CameraParams,
  pub target: CameraParams,
  pub timer: Timer,
}

#[derive(Resource)]
pub struct CameraAnchorRes {
  pub current: CameraParams,
  pub in_motion: Option<CameraMotion>,
  pub camera_id: Option<Entity>,
}

impl Default for CameraAnchorRes {
  fn default() -> Self {
    Self {
      current: CameraParams::default(),
      in_motion: None,
      camera_id: None,
    }
  }
}

pub fn spawn_camera(
  commands: &mut Commands,
  et: &mut ResMut<EntityTable>,
  anchor: &mut ResMut<CameraAnchorRes>,
) {
  let anchor_id = commands
    .spawn((CameraAnchor, Transform::IDENTITY, Visibility::default()))
    .id();
  et.main_anchor = Some(anchor_id);

  let camera_id = commands
    .spawn((
      MainCamera,
      Camera3d::default(),
      Projection::Perspective(PerspectiveProjection::default()),
      Transform::from_xyz(0.0, 7.5, 15.0).looking_at(Vec3::ZERO, Vec3::Y),
    ))
    .id();
  anchor.camera_id = Some(camera_id);

  commands.entity(anchor_id).add_child(camera_id);
}

pub fn update_camera_zoom(
  mut mouse_wheel: MessageReader<bevy::input::mouse::MouseWheel>,
  mut res: ResMut<CameraAnchorRes>,
) {
  for event in mouse_wheel.read() {
    let zoom_amount = event.y * 0.005;
    res.current.update_zoom(zoom_amount);
  }
}

pub fn update_mobile_zoom(
  touches: Res<bevy::input::touch::Touches>,
  mut res: ResMut<CameraAnchorRes>,
) {
  let active: Vec<_> = touches.iter().collect();
  if active.len() != 2 {
    return;
  }

  // --- 1. ZOOM LOGIC ---
  let curr_dist = active[0].position().distance(active[1].position());
  let prev_dist = active[0]
    .previous_position()
    .distance(active[1].previous_position());
  let pinch_delta = curr_dist - prev_dist;

  if pinch_delta.abs() > 0.1 {
    res.current.update_zoom(pinch_delta * 0.05); //
  }

  // --- 2. ORBIT LOGIC (The "Right Drag" substitute) ---
  // Calculate the average delta of both fingers
  let delta_0 = active[0].position() - active[0].previous_position();
  let delta_1 = active[1].position() - active[1].previous_position();

  // If both fingers are moving in a similar direction (Dot product is positive)
  if delta_0.dot(delta_1) > 0.0 {
    let avg_delta = (delta_0 + delta_1) / 2.0;
    res
      .current
      .update_orbit(-avg_delta.x * 0.005, avg_delta.y * 0.005);
  }
}

pub fn sync_camera_transforms(
  res: Res<CameraAnchorRes>,
  et: Res<EntityTable>,
  mut query: Query<&mut Transform>,
) {
  let Some(anchor_id) = et.main_anchor else {
    return;
  };
  let Some(camera_id) = res.camera_id else {
    return;
  };

  if let Ok(mut transform) = query.get_mut(anchor_id) {
    transform.translation = res.current.anchor;
  }

  if let Ok(mut transform) = query.get_mut(camera_id) {
    let offset = res.current.get_camera_offset();
    transform.translation = offset;
    transform.look_at(Vec3::ZERO, Vec3::Y);
  }
}
