use crate::ui::{self, MenuAction, MenuItem};
use crate::world::camera::{CameraAnchorRes, CameraParams};
use crate::EntityTable;
use bevy::prelude::*;

pub const MENU_LOCATION: Vec3 = Vec3::new(-7.5, 0.01, 0.0);
const IMAGE_PATH: &'static str = "embedded://bevycube/media/menu_about.jpg";

const HITBOX_TABLE: &[MenuItem] = &[
  MenuItem {
    x: 354,
    y: 57,
    w: 74,
    h: 29,
    action: MenuAction::Back,
  },
  MenuItem {
    x: 79,
    y: 252,
    w: 176,
    h: 19,
    action: MenuAction::OpenUrl("https://wbtek.github.io/"),
  },
  MenuItem {
    x: 79,
    y: 314,
    w: 133,
    h: 19,
    action: MenuAction::OpenUrl("https://wbtek.net/"),
  },
];

/// Used by other modules to navigate to the About menu
pub fn request_view(camera_res: &mut CameraAnchorRes) {
  camera_res.request_menu(CameraParams {
    anchor: MENU_LOCATION,
    zoom: 0.0,
    ..default()
  });
}

pub fn spawn_about_menu(
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
    "About Menu",
    IMAGE_PATH,
    MENU_LOCATION,
    HITBOX_TABLE,
  );

  et.about_menu = Some(menu_id);

  if let Some(ground) = et.ground {
    commands.entity(ground).add_child(menu_id);
  }
}
