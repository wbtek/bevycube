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
  pub direction: f32,
  pub slope: f32,
  pub zoom: f32,
}

impl Default for CameraParams {
  fn default() -> Self {
    Self {
      anchor: Vec3::ZERO,
      direction: 0.0,
      slope: 0.0,
      zoom: 15.0,
    }
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

pub fn spawn_camera(commands: &mut Commands, et: &mut ResMut<EntityTable>) {
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
  et.main_camera = Some(camera_id);

  commands.entity(anchor_id).add_child(camera_id);
}

pub fn update_camera_zoom(
  mut mouse_wheel: MessageReader<bevy::input::mouse::MouseWheel>,
  et: Res<EntityTable>,
  mut query: Query<&mut Transform>,
) {
  if let Some(mut transform) = et.main_camera.and_then(|id| query.get_mut(id).ok()) {
    for event in mouse_wheel.read() {
      let zoom_amount = event.y * 0.005;
      transform.translation.z = (transform.translation.z - zoom_amount).clamp(0.01, 40.0);
      transform.look_at(Vec3::ZERO, Vec3::Y);
    }
  }
}

pub fn update_mobile_zoom(
  touches: Res<bevy::input::touch::Touches>,
  et: Res<EntityTable>,
  mut query: Query<&mut Transform>,
) {
  let active: Vec<_> = touches.iter().collect();
  if active.len() != 2 {
    return;
  }
  if let Some(mut transform) = et.main_camera.and_then(|id| query.get_mut(id).ok()) {
    let pinch_delta = active[0].position().distance(active[1].position())
      - active[0]
        .previous_position()
        .distance(active[1].previous_position());
    if pinch_delta.abs() > 0.1 {
      transform.translation.z = (transform.translation.z - pinch_delta * 0.05).clamp(0.01, 40.0);
      transform.look_at(Vec3::ZERO, Vec3::Y);
    }
  }
}
