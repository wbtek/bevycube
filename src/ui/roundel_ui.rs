use crate::ui::{self, MenuAction, MenuItem};
use crate::world::camera::{CameraAnchorRes, CameraParams};
use crate::EntityTable;
use bevy::prelude::*;

pub const MENU_LOCATION: Vec3 = Vec3::new(7.5, 0.01, 0.0);
const IMAGE_PATH: &'static str = "embedded://bevycube/media/menu_roundel.jpg";

const HITBOX_TABLE: &[MenuItem] = &[
  MenuItem {
    x: 354,
    y: 57,
    w: 73,
    h: 29,
    action: MenuAction::Back,
  },
  MenuItem {
    x: 111,
    y: 147,
    w: 52,
    h: 29,
    action: MenuAction::ToggleAnisotropy(16),
  },
  MenuItem {
    x: 171,
    y: 147,
    w: 40,
    h: 29,
    action: MenuAction::ToggleAnisotropy(8),
  },
  MenuItem {
    x: 219,
    y: 147,
    w: 39,
    h: 29,
    action: MenuAction::ToggleAnisotropy(4),
  },
  MenuItem {
    x: 266,
    y: 147,
    w: 39,
    h: 29,
    action: MenuAction::ToggleAnisotropy(2),
  },
  MenuItem {
    x: 314,
    y: 147,
    w: 69,
    h: 38,
    action: MenuAction::ToggleAnisotropy(1),
  },
  MenuItem {
    x: 111,
    y: 237,
    w: 62,
    h: 29,
    action: MenuAction::SetMipmaps(true),
  },
  MenuItem {
    x: 182,
    y: 237,
    w: 68,
    h: 38,
    action: MenuAction::SetMipmaps(false),
  },
  // Using 0, 1, 2 for High, Med, Low resolution levels
  MenuItem {
    x: 111,
    y: 327,
    w: 90,
    h: 38,
    action: MenuAction::SetResolution(0),
  },
  MenuItem {
    x: 210,
    y: 327,
    w: 136,
    h: 29,
    action: MenuAction::SetResolution(1),
  },
  MenuItem {
    x: 354,
    y: 328,
    w: 75,
    h: 37,
    action: MenuAction::SetResolution(2),
  },
];

/// Used by main_ui or other modules to navigate to this menu
pub fn request_view(camera_res: &mut CameraAnchorRes) {
  camera_res.request_menu(CameraParams {
    anchor: MENU_LOCATION,
    zoom: 0.0,
    ..default()
  });
}

pub fn spawn_roundel_menu(
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
    "Roundel Menu",
    IMAGE_PATH,
    MENU_LOCATION,
    HITBOX_TABLE,
  );

  et.main_menu = Some(menu_id);

  if let Some(ground) = et.ground {
    commands.entity(ground).add_child(menu_id);
  }
}
