use crate::world::camera::{CameraAnchorRes, CameraParams};
// use crate::ui::{instructions_ui, ocean_ui, roundel_ui, about_ui};
use crate::EntityTable;
use bevy::prelude::*;

// --- CONSTANTS & DATA TABLES ---

pub const MENU_LOCATION: Vec3 = Vec3::new(7.5, 0.01, 0.0);
const IMAGE_PATH: &str = "embedded://bevycube/media/menu_roundel.jpg";

pub enum MenuAction {
  Execute(fn(&mut CameraAnchorRes)),
  ToggleAnisotropy(u16), // Example setting action
  Back,
}

const HITBOX_TABLE: &[MenuItem] = &[
  MenuItem {
    x: 354.,
    y: 57.,
    w: 74.,
    h: 29.,
    action: MenuAction::Back,
  },
  // Coordinates for your anisotropy/mipmap buttons go here
];

pub fn request_view(camera_res: &mut CameraAnchorRes) {
  camera_res.request_menu(CameraParams {
    anchor: MENU_LOCATION,
    zoom: 0.0,
    ..default()
  });
}

pub struct MenuItem {
  pub x: f32,
  pub y: f32,
  pub w: f32,
  pub h: f32,
  pub action: MenuAction,
}

// --- LOGIC ---

/// Maps local -2.5..2.5 back to 0..512
fn to_pixel(local_coord: f32) -> f32 {
  (local_coord * 512.0 / 5.0) + 256.0
}

// --- SPAWNING ---

pub fn spawn_roundel_menu(
  commands: &mut Commands,
  meshes: &mut ResMut<Assets<Mesh>>,
  materials: &mut ResMut<Assets<StandardMaterial>>,
  asset_server: &Res<AssetServer>,
  et: &mut ResMut<EntityTable>,
) {
  let menu_id = commands
    .spawn((
      Name::new("Roundel Menu"),
      Mesh3d(meshes.add(Plane3d::default().mesh().size(5.0, 5.0))),
      MeshMaterial3d(materials.add(StandardMaterial {
        base_color_texture: Some(asset_server.load(IMAGE_PATH)),
        alpha_mode: AlphaMode::Add,
        reflectance: 0.0,
        ..default()
      })),
      Transform::from_translation(MENU_LOCATION),
    ))
    .id();

  et.main_menu = Some(menu_id);
  if let Some(ground) = et.ground {
    commands.entity(ground).add_child(menu_id);
  }

  commands.entity(menu_id).observe(
    |mut ev: On<Pointer<Click>>,
     mut camera_res: ResMut<CameraAnchorRes>,
     query: Query<&GlobalTransform>| {
      let Some(hit_world_pos) = ev.hit.position else {
        return;
      };

      // Use .event_target() to get the entity ID in Bevy 0.18
      let target_entity = ev.event_target();
      let Ok(menu_gt) = query.get(target_entity) else {
        return;
      };

      let local_pos = menu_gt.affine().inverse().transform_point3(hit_world_pos);

      let px = to_pixel(local_pos.x);
      let py = to_pixel(local_pos.z);

      for item in HITBOX_TABLE {
        if px >= item.x && px <= (item.x + item.w) && py >= item.y && py <= (item.y + item.h) {
          match item.action {
            MenuAction::Back => {
              ev.propagate(false);
              camera_res.request_back()
            }
            MenuAction::Execute(func) => func(&mut camera_res),
            _ => {}
          }
          break;
        }
      }
    },
  );
}
