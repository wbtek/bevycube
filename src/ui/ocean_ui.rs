use crate::ui::{self, MenuAction, MenuItem};
use crate::world::camera::{CameraAnchorRes, CameraParams};
use crate::EntityTable;
use bevy::prelude::*;

pub const MENU_LOCATION: Vec3 = Vec3::new(7.5, 0.01, -7.5);
const IMAGE_PATH: &'static str = "embedded://bevycube/media/menu_ocean.jpg";

const HITBOX_TABLE: &[MenuItem] = &[
  MenuItem {
    x: 354,
    y: 57,
    w: 74,
    h: 29,
    action: MenuAction::Back,
  },
  MenuItem {
    x: 111,
    y: 147,
    w: 97,
    h: 30,
    action: MenuAction::SetMeshMode(0),
  }, // Solid
  MenuItem {
    x: 214,
    y: 147,
    w: 93,
    h: 29,
    action: MenuAction::SetMeshMode(1),
  }, // Wire
  MenuItem {
    x: 316,
    y: 147,
    w: 107,
    h: 29,
    action: MenuAction::SetMeshMode(2),
  }, // Points
  MenuItem {
    x: 111,
    y: 237,
    w: 55,
    h: 29,
    action: MenuAction::SetMeshSubdiv(5),
  },
  MenuItem {
    x: 172,
    y: 237,
    w: 55,
    h: 29,
    action: MenuAction::SetMeshSubdiv(10),
  },
  MenuItem {
    x: 233,
    y: 237,
    w: 56,
    h: 29,
    action: MenuAction::SetMeshSubdiv(20),
  },
  MenuItem {
    x: 295,
    y: 237,
    w: 56,
    h: 29,
    action: MenuAction::SetMeshSubdiv(40),
  },
  MenuItem {
    x: 357,
    y: 237,
    w: 60,
    h: 29,
    action: MenuAction::SetMeshSubdiv(80),
  },
];

/// Used by other modules to navigate to the Ocean menu
pub fn request_view(camera_res: &mut CameraAnchorRes) {
  camera_res.request_menu(CameraParams {
    anchor: MENU_LOCATION,
    zoom: 0.0,
    ..default()
  });
}

pub fn spawn_ocean_menu(
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
    "Ocean Menu",
    IMAGE_PATH,
    MENU_LOCATION,
    HITBOX_TABLE,
  );

  et.ocean_menu = Some(menu_id);

  if let Some(ground) = et.ground {
    commands.entity(ground).add_child(menu_id);
  }
}
