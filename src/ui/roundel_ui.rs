use crate::roundel::StitchedRoundel;
use crate::ui::{self, GlobalSettings, MenuAction, MenuItem, Need::*};
use crate::world::camera::{CameraAnchorRes, CameraParams};
use crate::EntityTable;
use bevy::image::{ImageFilterMode, ImageSampler, ImageSamplerDescriptor};
use bevy::prelude::*;

pub const MENU_LOCATION: Vec3 = Vec3::new(5.9, 0.01, 0.0);
pub const IMAGE_PATH: &'static str = "embedded://bevycube/media/menu_roundel.jpg";

pub const HITBOX_TABLE: &[MenuItem] = &[
  MenuItem {
    x: 354,
    y: 57,
    w: 73,
    h: 29,
    diamond: No,
    action: MenuAction::Back,
  },
  MenuItem {
    x: 111,
    y: 147,
    w: 52,
    h: 29,
    diamond: Yes,
    action: MenuAction::SetAnisotropy(16),
  },
  MenuItem {
    x: 171,
    y: 147,
    w: 40,
    h: 29,
    diamond: Yes,
    action: MenuAction::SetAnisotropy(8),
  },
  MenuItem {
    x: 219,
    y: 147,
    w: 39,
    h: 29,
    diamond: Yes,
    action: MenuAction::SetAnisotropy(4),
  },
  MenuItem {
    x: 266,
    y: 147,
    w: 39,
    h: 29,
    diamond: Yes,
    action: MenuAction::SetAnisotropy(2),
  },
  MenuItem {
    x: 314,
    y: 147,
    w: 69,
    h: 38,
    diamond: Yes,
    action: MenuAction::SetAnisotropy(1),
  },
  MenuItem {
    x: 111,
    y: 237,
    w: 62,
    h: 29,
    diamond: Yes,
    action: MenuAction::SetMipmaps(1),
  },
  MenuItem {
    x: 182,
    y: 237,
    w: 68,
    h: 38,
    diamond: Yes,
    action: MenuAction::SetMipmaps(0),
  },
  MenuItem {
    x: 111,
    y: 327,
    w: 90,
    h: 38,
    diamond: Yes,
    action: MenuAction::SetResolution(0),
  },
  MenuItem {
    x: 210,
    y: 327,
    w: 136,
    h: 29,
    diamond: Yes,
    action: MenuAction::SetResolution(1),
  },
  MenuItem {
    x: 354,
    y: 328,
    w: 75,
    h: 37,
    diamond: Yes,
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

  et.roundel_menu = Some(menu_id);

  if let Some(ground) = et.ground {
    commands.entity(ground).add_child(menu_id);
  }
}

pub fn sync_roundel_menu_settings(
  settings: Res<GlobalSettings>,
  mut local: Local<Option<GlobalSettings>>,
  stitched: Option<Res<StitchedRoundel>>,
  mut images: ResMut<Assets<Image>>,
  mut materials: ResMut<Assets<StandardMaterial>>,
) {
  if !settings.is_changed() {
    return;
  }

  let first = local.is_none();
  let l = local.get_or_insert_default();

  let Some(ref stitched_res) = stitched else {
    return;
  };
  let handle = &stitched_res.handle;
  let Some(img) = images.get_mut(handle) else {
    return;
  };
  let mut isamp = match img.sampler.clone() {
    ImageSampler::Descriptor(d) => d,
    _ => ImageSamplerDescriptor::default(),
  };

  if first || settings.anisotropy != l.anisotropy {
    isamp.anisotropy_clamp = settings.anisotropy as u16;
    l.anisotropy = settings.anisotropy;
  }

  if first || settings.mipmaps != l.mipmaps {
    isamp.mipmap_filter = match settings.mipmaps {
      1 => ImageFilterMode::Linear,
      _ => ImageFilterMode::Nearest,
    };
    l.mipmaps = settings.mipmaps;
  }

  if first || settings.asset_resolution != l.asset_resolution {
    isamp.lod_min_clamp = settings.asset_resolution as f32;
    l.asset_resolution = settings.asset_resolution;
  }

  img.sampler = ImageSampler::Descriptor(isamp);
  for (_, mat) in materials.iter_mut() {
    if mat
      .base_color_texture
      .as_ref()
      .map(|h| h.id() == handle.id())
      .unwrap_or(false)
    {
      mat.base_color_texture = Some(handle.clone());
    }
  }
}
