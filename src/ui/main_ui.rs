use crate::EntityTable;
use bevy::prelude::*;

pub fn spawn_main_menu(
  commands: &mut Commands,
  meshes: &mut ResMut<Assets<Mesh>>,
  materials: &mut ResMut<Assets<StandardMaterial>>,
  asset_server: &Res<AssetServer>,
  et: &mut ResMut<EntityTable>,
) {
  // Load the texture directly for this test
  let texture_handle = asset_server.load("embedded://bevycube/media/menu_main.jpg");

  let main_menu_id = commands
    .spawn((
      Name::new("Main Menu"),
      Mesh3d(meshes.add(Plane3d::default().mesh().size(5.0, 5.0))),
      MeshMaterial3d(materials.add(StandardMaterial {
        base_color_texture: Some(texture_handle),
        alpha_mode: AlphaMode::Add, // Matches your existing aesthetic
        reflectance: 0.0,
        ..default()
      })),
      // Position: Upper Center (North)
      Transform::from_xyz(0.0, 0.01, -7.5),
    ))
    .id();
  et.main_menu = Some(main_menu_id);
  commands
    .entity(et.ground.expect("Ground not found!"))
    .add_child(main_menu_id);
}
