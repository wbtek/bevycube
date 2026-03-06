//! # Camera System
//!
//! Dolly camera with anchor-based navigation and history stack.
//! Supports mouse wheel zoom, pinch zoom (mobile), orbit, and pan.

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
use bevy::prelude::EaseFunction::*;
use bevy::prelude::*;
use std::f32::consts::PI;

/// Plugin for initializing and updating camera systems
pub struct CameraPlugin;

impl Plugin for CameraPlugin {
  fn build(&self, app: &mut App) {
    app.add_systems(
      Update,
      (update_camera_zoom, update_mobile_zoom, update_camera_motion),
    );
  }
}

/// Component marking the camera anchor entity
#[derive(Component)]
#[require(Transform, Visibility)]
pub struct CameraAnchor;

/// Component marking the main 3D camera
#[derive(Component)]
pub struct MainCamera;

/// Configuration for camera position and orientation
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CameraParams {
  /// Anchor position in world space
  pub anchor: Vec3,
  /// y distance from anchor to ground
  pub track_near_end_y: f32,
  /// Camera direction (0.0 = facing north)
  pub direction: f32,
  /// Track slope (0.0 = flat, 1.5 = almost vertical)
  pub slope: f32,
  /// Camera distance from anchor end of track
  pub zoom: f32,
}

impl Default for CameraParams {
  fn default() -> Self {
    Self {
      anchor: Vec3::ZERO,
      track_near_end_y: 4.0,
      direction: 0.0,
      slope: PI * 0.5 * 0.2,
      zoom: 15.0,
    }
  }
}

impl CameraParams {
  /// Update pan position based on delta
  pub fn update_pan(&mut self, delta_x: f32, delta_y: f32) {
    let (sin, cos) = self.direction.sin_cos();
    let rotated_x = delta_x * cos + delta_y * sin;
    let rotated_z = -delta_x * sin + delta_y * cos;
    self.anchor += Vec3::new(rotated_x, 0.0, rotated_z);
  }

  /// Update orbit direction and slope
  pub fn update_orbit(&mut self, delta_x: f32, delta_y: f32) {
    self.slope = (self.slope + delta_y).clamp(0.0, 1.5);
    self.direction = (self.direction + delta_x).rem_euclid(PI * 2.);
  }

  /// Update zoom level
  pub fn update_zoom(&mut self, delta: f32) {
    self.zoom = (self.zoom - delta).clamp(0.01, 40.0);
  }

  /// Calculates where the camera should be relative to the anchor
  pub fn get_camera_offset(&self) -> Vec3 {
    // direction 0 is North (towards -Z).
    // We calculate horizontal X and Z using direction, then scale by the slope's cosine.
    let vertical_scale = self.slope.sin();
    let horizontal_scale = self.slope.cos();
    let zoom = self.zoom.clamp(0.01, 40.0);

    let x = self.direction.sin() * horizontal_scale * zoom;
    let y = self.track_near_end_y + (vertical_scale * zoom);
    let z = self.direction.cos() * horizontal_scale * zoom;

    Vec3::new(x, y, z)
  }

  /// Return where anchor is
  pub fn get_anchor_xyz(&self) -> Vec3 {
    self.anchor
  }

  /// Camera effect for camera (used for water repulsion)
  pub fn get_camera_effect(&self) -> f32 {
    self.track_near_end_y + self.zoom.clamp(0.01, 40.0)
  }

  /// Camera effect for arbitrary location (used for menu activation)
  pub fn get_camera_effect_xyz(&self, location: Vec3) -> f32 {
    self.track_near_end_y + self.zoom.clamp(0.01, 40.0) + self.get_anchor_xyz().distance(location)
  }
}

/// Active camera motion state
#[derive(Debug)]
pub struct CameraMotion {
  pub from: CameraParams,
  pub target: CameraParams,
  pub timer: Timer,
  pub peak_zoom: f32,
}

/// Resource tracking camera anchor and navigation history
#[derive(Debug, Resource)]
pub struct CameraAnchorRes {
  pub current: CameraParams,
  pub in_motion: Option<CameraMotion>,
  pub camera_id: Option<Entity>,
  pub history: Vec<CameraParams>,
}

impl Default for CameraAnchorRes {
  fn default() -> Self {
    Self {
      current: CameraParams::default(),
      in_motion: None,
      camera_id: None,
      history: Vec::new(),
    }
  }
}

impl CameraAnchorRes {
  fn set_motion(&mut self, from: CameraParams, to: CameraParams) {
    let mut f = from;
    let mut t = to;

    f.direction = (f.direction + PI).rem_euclid(2. * PI) - PI;
    t.direction = (t.direction + PI).rem_euclid(2. * PI) - PI;

    t.zoom = t.zoom.clamp(0.01, 40.0);
    t.slope = t.slope.clamp(0.0, 1.5);

    let dist = from.anchor.distance(to.anchor);
    self.in_motion = Some(CameraMotion {
      from: f,
      target: t,
      timer: Timer::from_seconds(1.5, TimerMode::Once),
      peak_zoom: (self.current.zoom.max(to.zoom) + dist).clamp(0.01, 40.0),
    });
  }

  /// Push current state and request menu navigation
  pub fn request_menu(&mut self, target: CameraParams) {
    self.current.zoom = self.current.zoom.clamp(0.01, 40.0);

    let mut clamped_target = target;
    clamped_target.zoom = target.zoom.clamp(0.01, 40.0);

    if self.current != clamped_target {
      self.history.push(self.current);
    }
    self.set_motion(self.current, clamped_target);
  }

  /// Pop from history stack and return to previous view
  pub fn request_back(&mut self) {
    let target = self.history.pop().unwrap_or_else(CameraParams::default);
    self.set_motion(self.current, target);
  }
}

/// Spawns camera anchor and main camera entities
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
  et.main_camera = Some(camera_id);

  commands.entity(anchor_id).add_child(camera_id);
}

/// Desktop Zoom (MouseWheel)
pub fn update_camera_zoom(
  mut mouse_wheel: MessageReader<bevy::input::mouse::MouseWheel>,
  mut res: ResMut<CameraAnchorRes>,
) {
  for event in mouse_wheel.read() {
    let zoom_amount = event.y * 0.005;
    res.current.update_zoom(zoom_amount);
  }
}

/// Mobile Zoom (pinch) and Orbit (2 fingers same direction)
pub fn update_mobile_zoom(
  touches: Res<bevy::input::touch::Touches>,
  mut res: ResMut<CameraAnchorRes>,
) {
  let active: Vec<_> = touches.iter().collect();
  if active.len() != 2 {
    return;
  }

  // Zoom
  let curr_dist = active[0].position().distance(active[1].position());
  let prev_dist = active[0]
    .previous_position()
    .distance(active[1].previous_position());
  let pinch_delta = curr_dist - prev_dist;

  if pinch_delta.abs() > 0.1 {
    res.current.update_zoom(pinch_delta * 0.05); //
  }

  // Orbit
  // "Right Drag" if both fingers go similar direction (Dot prod positive)
  let delta_0 = active[0].position() - active[0].previous_position();
  let delta_1 = active[1].position() - active[1].previous_position();

  if delta_0.dot(delta_1) > 0.0 {
    let avg_delta = (delta_0 + delta_1) / 2.0;
    res
      .current
      .update_orbit(-avg_delta.x * 0.005, avg_delta.y * 0.005);
  }
}

/// Move anchor and camera to already calculated position each frame
pub fn sync_camera_transforms(
  res: Res<CameraAnchorRes>,
  et: Res<EntityTable>,
  mut query: Query<&mut Transform>,
) {
  if let (Some(anchor_id), Some(camera_id)) = (et.main_anchor, res.camera_id) {
    if let Ok(mut transform) = query.get_mut(anchor_id) {
      transform.translation = res.current.anchor;
    }
    if let Ok(mut transform) = query.get_mut(camera_id) {
      let offset = res.current.get_camera_offset();
      transform.translation = offset;
      transform.look_at(Vec3::ZERO, Vec3::Y);
    }
  }
}

/// Calculate camera / anchor animation if in progress
pub fn update_camera_motion(time: Res<Time>, mut res: ResMut<CameraAnchorRes>) {
  // If in motion, take it as protection
  if let Some(mut motion) = res.in_motion.take() {
    motion.timer.tick(time.delta());
    let t = motion.timer.fraction().clamp(0., 1.);

    // Easing curves
    let elastic_t = ElasticIn.sample_unchecked(t);
    let bounce_t = BounceOut.sample_unchecked(t);

    // Anchor: blended Elastic and Bounce slide
    res.current.anchor = motion.from.anchor.lerp(
      motion.target.anchor,
      (elastic_t + bounce_t + bounce_t + t + t) / 5.,
    );

    // Zoom: Bell curve wave effect with some Bounce
    let zoom_t = 1.0 - (2.0 * t - 1.0).powi(2);
    res.current.zoom = (motion
      .from
      .zoom
      .lerp(motion.target.zoom, (bounce_t + t) / 2.)
      + (motion.peak_zoom - motion.from.zoom.max(motion.target.zoom)) * zoom_t)
      .clamp(0.01, 40.0);

    // Direction: Linear rotation
    res.current.direction = motion.from.direction.lerp(motion.target.direction, t);

    // Slope: Linear
    res.current.slope = motion.from.slope.lerp(motion.target.slope, t);

    if motion.timer.just_finished() {
      res.current = motion.target;
      res.in_motion = None;
    } else {
      // Put it back to continue the motion next frame
      res.in_motion = Some(motion);
    }
  }
}
