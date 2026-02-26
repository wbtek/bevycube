use crate::ui::Need::*;
use crate::ui::{self, main_ui, MenuAction, MenuItem};
use crate::world::camera::{CameraAnchorRes, CameraParams};
use crate::EntityTable;
use bevy::prelude::*;

pub const MENU_LOCATION: Vec3 = Vec3::new(0.0, 0.0, 0.0);
const IMAGE_PATH: &'static str = "embedded://bevycube/media/menu_overlay.jpg";

const HITBOX_TABLE: &[MenuItem] = &[MenuItem {
  x: 0,
  y: 0,
  w: 511,
  h: 511,
  diamond: No,
  action: MenuAction::Execute(main_ui::request_view),
}];

pub fn request_view(camera_res: &mut CameraAnchorRes) {
  camera_res.request_menu(CameraParams {
    anchor: MENU_LOCATION,
    zoom: 0.0,
    ..default()
  });
}

pub fn spawn_overlay_menu(
  commands: &mut Commands,
  meshes: &mut ResMut<Assets<Mesh>>,
  materials: &mut ResMut<Assets<StandardMaterial>>,
  asset_server: &Res<AssetServer>,
  et: &mut ResMut<EntityTable>,
) {
  let menu_id = ui::spawn_menu_plane(
    commands,
    meshes,
    materials,
    asset_server,
    "Overlay Menu",
    IMAGE_PATH,
    MENU_LOCATION,
    HITBOX_TABLE,
  );
  et.overlay_menu = Some(menu_id);

  commands.entity(menu_id).insert(Transform {
    translation: Vec3::new(-0.6, -0.375, -1.0),
    rotation: Quat::from_rotation_x(std::f32::consts::FRAC_PI_2),
    scale: Vec3::new(0.0450, 1.0, 0.00895),
    ..default()
  });

  if let Some(camera) = et.main_camera {
    commands.entity(camera).add_child(menu_id);
  }
}
