use crate::EntityTable;
use bevy::prelude::*;

pub fn spawn_instructions_menu(
  commands: &mut Commands,
  meshes: &mut ResMut<Assets<Mesh>>,
  materials: &mut ResMut<Assets<StandardMaterial>>,
  asset_server: &Res<AssetServer>,
  et: &mut ResMut<EntityTable>,
) {
  // Load the texture directly for this test
  let texture_handle = asset_server.load("embedded://bevycube/media/menu_instructions.jpg");

  let instructions_menu_id = commands
    .spawn((
      Name::new("Instructions Menu"),
      Mesh3d(meshes.add(Plane3d::default().mesh().size(5.0, 5.0))),
      MeshMaterial3d(materials.add(StandardMaterial {
        base_color_texture: Some(texture_handle),
        alpha_mode: AlphaMode::Add, // Matches your existing aesthetic
        reflectance: 0.0,
        ..default()
      })),
      // Position: Left Center (NorthWest)
      Transform::from_xyz(-7.5, 0.01, -7.5),
    ))
    .id();
  et.instructions_menu = Some(instructions_menu_id);
  commands
    .entity(et.ground.expect("Ground not found!"))
    .add_child(instructions_menu_id);
}
