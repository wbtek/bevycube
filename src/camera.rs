use bevy::prelude::*;
use crate::EntityTable;

pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (update_camera_zoom, update_mobile_zoom));
    }
}

#[derive(Component)] #[require(Transform, Visibility)] pub struct CameraAnchor;
#[derive(Component)] pub struct MainCamera;

pub fn update_camera_zoom(
    mut mouse_wheel: MessageReader<bevy::input::mouse::MouseWheel>, 
    et: Res<EntityTable>, 
    mut query: Query<&mut Transform>
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
    mut query: Query<&mut Transform>
) {
    let active: Vec<_> = touches.iter().collect();
    if active.len() != 2 { return; }
    if let Some(mut transform) = et.main_camera.and_then(|id| query.get_mut(id).ok()) {
        let pinch_delta = active[0].position().distance(active[1].position()) - active[0].previous_position().distance(active[1].previous_position());
        if pinch_delta.abs() > 0.1 {
            transform.translation.z = (transform.translation.z - pinch_delta * 0.05).clamp(0.01, 40.0);
            transform.look_at(Vec3::ZERO, Vec3::Y);
        }
    }
}
