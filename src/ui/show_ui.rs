use crate::world::camera::*;
use crate::EntityTable;
use bevy::camera::visibility::RenderLayers;
use bevy::prelude::*;
// use log::info;
//
const THRESHOLD: f32 = 7.0;

pub fn show_menus_system(
  mut commands: Commands,
  car: ResMut<CameraAnchorRes>,
  et: Res<EntityTable>,
) {
  let c = car.current;

  let hide = c.anchor == crate::ui::main_ui::MENU_LOCATION && c.direction == 0.0 && c.zoom <= 0.01;
  let layer = if hide { 1 } else { 0 };
  if let Some(id) = et.overlay_menu {
    commands
      .entity(id)
      .insert_recursive::<Children>(RenderLayers::layer(layer));
  }

  let pairs = [
    (crate::ui::main_ui::MENU_LOCATION, et.main_menu),
    (crate::ui::instruct_ui::MENU_LOCATION, et.instruct_menu),
    (crate::ui::ocean_ui::MENU_LOCATION, et.ocean_menu),
    (crate::ui::roundel_ui::MENU_LOCATION, et.roundel_menu),
    (crate::ui::about_ui::MENU_LOCATION, et.about_menu),
  ];

  // if true it receives clicks
  let mut hover_fake_ocean = true;

  for p in pairs.iter() {
    let hide = c.get_camera_effect_xyz(p.0) > THRESHOLD;
    let layer = if hide {
      1
    } else {
      hover_fake_ocean = false;
      0
    };
    if let Some(id) = p.1 {
      commands
        .entity(id)
        .insert_recursive::<Children>(RenderLayers::layer(layer));
    }
  }
  if let Some(fake_ocean_id) = et.ocean_fake {
    commands
      .entity(fake_ocean_id)
      .insert(bevy::picking::Pickable {
        is_hoverable: hover_fake_ocean,
        should_block_lower: false,
      });
  }
}
