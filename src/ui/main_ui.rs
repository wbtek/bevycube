use crate::ui::Need::*;
use crate::ui::{self, about_ui, instructions_ui, ocean_ui, roundel_ui, MenuAction, MenuItem};
use crate::world::camera::{CameraAnchorRes, CameraParams};
use crate::EntityTable;
use bevy::prelude::*;

pub const MENU_LOCATION: Vec3 = Vec3::new(0.0, 0.01, -7.5);
const IMAGE_PATH: &'static str = "embedded://bevycube/media/menu_main.jpg";

const HITBOX_TABLE: &[MenuItem] = &[
  MenuItem {
    x: 354,
    y: 57,
    w: 74,
    h: 29,
    diamond: No,
    action: MenuAction::Back,
  },
  MenuItem {
    x: 79,
    y: 102,
    w: 235,
    h: 38,
    diamond: No,
    action: MenuAction::Execute(roundel_ui::request_view),
  },
  MenuItem {
    x: 80,
    y: 147,
    w: 207,
    h: 38,
    diamond: No,
    action: MenuAction::Execute(ocean_ui::request_view),
  },
  MenuItem {
    x: 79,
    y: 192,
    w: 167,
    h: 29,
    diamond: No,
    action: MenuAction::Execute(instructions_ui::request_view),
  },
  MenuItem {
    x: 80,
    y: 237,
    w: 85,
    h: 33,
    diamond: No,
    action: MenuAction::Execute(about_ui::request_view),
  },
];

pub fn request_view(camera_res: &mut CameraAnchorRes) {
  camera_res.request_menu(CameraParams {
    anchor: MENU_LOCATION,
    zoom: 0.0,
    ..default()
  });
}

pub fn spawn_main_menu(
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
    "Main Menu",
    IMAGE_PATH,
    MENU_LOCATION,
    HITBOX_TABLE,
  );
  et.main_menu = Some(menu_id);

  if let Some(ground) = et.ground {
    commands.entity(ground).add_child(menu_id);
  }
}
