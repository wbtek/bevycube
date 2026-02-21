use crate::ui::GlobalSettings;
use crate::ui::Need::*;
use crate::ui::{self, MenuAction, MenuItem};
use crate::world::camera::{CameraAnchorRes, CameraParams};
use crate::EntityTable;
use bevy::camera::visibility::RenderLayers;
use bevy::prelude::*;
use log::info;

pub const MENU_LOCATION: Vec3 = Vec3::new(7.5, 0.01, -7.5);
pub const IMAGE_PATH: &'static str = "embedded://bevycube/media/menu_ocean.jpg";

pub const HITBOX_TABLE: &[MenuItem] = &[
  MenuItem {
    x: 354,
    y: 57,
    w: 74,
    h: 29,
    diamond: No,
    action: MenuAction::Back,
  },
  MenuItem {
    x: 111,
    y: 147,
    w: 97,
    h: 30,
    diamond: Yes,
    action: MenuAction::SetMeshMode(0),
  }, // Solid
  MenuItem {
    x: 214,
    y: 147,
    w: 93,
    h: 29,
    diamond: Yes,
    action: MenuAction::SetMeshMode(1),
  }, // Wire
  MenuItem {
    x: 316,
    y: 147,
    w: 107,
    h: 29,
    diamond: Yes,
    action: MenuAction::SetMeshMode(2),
  }, // Points
  MenuItem {
    x: 111,
    y: 237,
    w: 40,
    h: 29,
    diamond: Yes,
    action: MenuAction::SetMeshDimension(5),
  },
  MenuItem {
    x: 160,
    y: 237,
    w: 55,
    h: 29,
    diamond: Yes,
    action: MenuAction::SetMeshDimension(10),
  },
  MenuItem {
    x: 221,
    y: 237,
    w: 56,
    h: 29,
    diamond: Yes,
    action: MenuAction::SetMeshDimension(20),
  },
  MenuItem {
    x: 289,
    y: 237,
    w: 56,
    h: 29,
    diamond: Yes,
    action: MenuAction::SetMeshDimension(40),
  },
  MenuItem {
    x: 357,
    y: 237,
    w: 60,
    h: 29,
    diamond: Yes,
    action: MenuAction::SetMeshDimension(80),
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

pub fn sync_ocean_menu_settings(
  settings: Res<GlobalSettings>,
  mut local: Local<Option<GlobalSettings>>,
  et: Res<EntityTable>,
  mut query: Query<Option<&mut RenderLayers>>,
) {
  if !settings.is_changed() {
    return;
  }

  let first = local.is_none();
  let l = local.get_or_insert_default();

  if first || settings.mesh_mode != l.mesh_mode {
    let entities = [et.ocean, et.ocean_wire, et.ocean_point];

    for (i, opt_ent) in entities.iter().enumerate() {
      if let Some(entity) = *opt_ent {
        if let Ok(opt_layers) = query.get_mut(entity) {
          let is_active_mode = i == settings.mesh_mode as usize;
          if let Some(mut layers) = opt_layers {
            *layers = if is_active_mode {
              RenderLayers::layer(0) // Seen by Camera
            } else {
              RenderLayers::layer(1) // Hidden from Camera
            };
          }
        }
      }
    }
    l.mesh_mode = settings.mesh_mode;
  }
  if first || settings.mesh_dimension != l.mesh_dimension {
    info!(
      "dimension change: {:?} to {:?}",
      l.mesh_dimension, settings.mesh_dimension
    );
    l.mesh_dimension = settings.mesh_dimension;
  }
}
