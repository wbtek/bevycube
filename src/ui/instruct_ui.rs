use crate::ui::Need::*;
use crate::ui::{self, MenuAction, MenuItem};
use crate::world::camera::{CameraAnchorRes, CameraParams};
use crate::EntityTable;
use bevy::prelude::*;

pub const MENU_LOCATION: Vec3 = Vec3::new(-5.9, 0.01, -5.9);
const IMAGE_PATH: &'static str = "embedded://bevycube/media/menu_instructions.jpg";

const HITBOX_TABLE: &[MenuItem] = &[MenuItem {
  x: 354,
  y: 57,
  w: 74,
  h: 29,
  diamond: No,
  action: MenuAction::Back,
}];

/// Used by other modules to navigate to the instructions
pub fn request_view(camera_res: &mut CameraAnchorRes) {
  camera_res.request_menu(CameraParams {
    anchor: MENU_LOCATION,
    zoom: 0.0,
    ..default()
  });
}

pub fn spawn_instruct_menu(
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
    "Instructions Menu",
    IMAGE_PATH,
    MENU_LOCATION,
    HITBOX_TABLE,
  );

  et.instruct_menu = Some(menu_id);

  if let Some(ground) = et.ground {
    commands.entity(ground).add_child(menu_id);
  }
}
