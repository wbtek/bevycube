use crate::ui::{to_local, GlobalSettings, MenuAction};
use crate::EntityTable;
use bevy::prelude::*;

#[derive(Component)]
pub struct DiamondCategory(pub MenuAction);

pub fn sync_diamonds(
  settings: Res<GlobalSettings>,
  et: Res<EntityTable>,
  mut query: Query<(&DiamondCategory, &mut Transform, &ChildOf)>,
) {
  if !settings.is_changed() {
    return;
  }

  for (category, mut transform, child_of) in query.iter_mut() {
    let parent_entity = child_of.0;

    let table = if Some(parent_entity) == et.ocean_menu {
      crate::ui::ocean_ui::HITBOX_TABLE
    } else if Some(parent_entity) == et.roundel_menu {
      crate::ui::roundel_ui::HITBOX_TABLE
    } else {
      continue;
    };

    let is_match = |action: &MenuAction| -> bool {
      match (action, &category.0) {
        (MenuAction::SetMeshMode(v), MenuAction::SetMeshMode(_)) => *v == settings.mesh_mode,
        (MenuAction::SetAnisotropy(v), MenuAction::SetAnisotropy(_)) => *v == settings.anisotropy,
        (MenuAction::SetMipmaps(v), MenuAction::SetMipmaps(_)) => *v == settings.mipmaps,
        (MenuAction::SetResolution(v), MenuAction::SetResolution(_)) => {
          *v == settings.resolution_level
        }
        (MenuAction::SetMeshSubdiv(v), MenuAction::SetMeshSubdiv(_)) => *v == settings.mesh_subdiv,
        _ => false,
      }
    };

    if let Some(item) = table.iter().find(|i| is_match(&i.action)) {
      transform.translation.x = to_local(item.x as f32 + 10.);
      transform.translation.z = to_local(item.y as f32 + 15.);
      transform.translation.y = 0.01;
    }
  }
}
