use crate::{EntityTable, ImageFilterMode, ImageSampler, ImageSamplerDescriptor, StitchedRoundel};
use bevy::prelude::*;

pub struct SettingsUiPlugin;
impl Plugin for SettingsUiPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<Settings>();
    }
}

#[derive(Debug, Component, Reflect)]
#[reflect(Component)]
pub struct Settings {
    pub active: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Self { active: true }
    }
}

#[derive(Debug, Component)]
#[require(Transform, Visibility)]
pub struct SetAnisotropic;
#[derive(Debug, Component)]
#[require(Transform, Visibility)]
pub struct SetMipmaps;
#[derive(Debug, Component)]
#[require(Transform, Visibility)]
pub struct SetResolution;
#[derive(Debug, Component)]
#[require(Transform, Visibility)]
pub struct SetFps;

pub fn to_local(pixel: f32) -> f32 {
    (pixel - 256.0) / 512.0 * 5.0
}
pub fn from_local(pixel: f32) -> f32 {
    (pixel / 5.0) * 512.0 + 256.0
}

pub fn spawn_settings_ui(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    settings_mat: Handle<StandardMaterial>,
    diamond_mat: Handle<StandardMaterial>,
    ground_id: Entity,
    et: &mut EntityTable,
) {
    let settings_id = commands
        .spawn((
            Settings { active: true },
            Mesh3d(meshes.add(Plane3d::default().mesh().size(5.0, 5.0))),
            MeshMaterial3d(settings_mat),
            Transform::from_xyz(7.5, 0.01, 7.5),
        ))
        .id();
    et.settings = Some(settings_id);

    commands.entity(ground_id).add_child(settings_id);

    macro_rules! row {
        ($c:ident, $y1:expr, $y2:expr, [$($x:expr),*], [$($i:ident),*]) => {
            SettingsCategory { cat: SetCatType::$c, y_top: $y1, y_bot: $y2, x_bounds: vec![$($x),*], items: vec![$(SetItem::$i),*] }
        };
    }

    let settings_data = [
        row!(
            Anisotropic,
            140.,
            185.,
            [107., 166., 215., 263., 310., 387.],
            [An16, An8, An4, An2, AnOff]
        ),
        row!(
            Mipmaps, /* rustfmt */
            230.,
            275.,
            [107., 177., 255.],
            [MMOn, MMOff]
        ),
        row!(
            AssetResolution,
            320.,
            365.,
            [107., 206., 350., 433.],
            [AResHi, AResMed, AResLow]
        ),
        row!(
            FPSDisplay,
            410.,
            455.,
            [107., 177., 255.],
            [FPSDispOn, FPSDispOff]
        ),
    ];

    enum SetCatType {
        Anisotropic,
        Mipmaps,
        AssetResolution,
        FPSDisplay,
    }
    enum SetItem {
        An16,
        An8,
        An4,
        An2,
        AnOff,
        MMOn,
        MMOff,
        AResHi,
        AResMed,
        AResLow,
        FPSDispOn,
        FPSDispOff,
    }
    struct SettingsCategory {
        cat: SetCatType,
        y_top: f32,
        y_bot: f32,
        x_bounds: Vec<f32>,
        items: Vec<SetItem>,
    }

    let set_anisotropic_id = commands
        .spawn((
            SetAnisotropic,
            Mesh3d(meshes.add(Plane3d::default().mesh().size(5. / 16., 5. / 16.))),
            MeshMaterial3d(diamond_mat.clone()),
            Transform::from_xyz(to_local(107. + 14.), 0.01, to_local(140. + 22.)),
        ))
        .id();
    et.set_anisotropic = Some(set_anisotropic_id);
    commands.entity(settings_id).add_child(set_anisotropic_id);

    let set_mipmaps_id = commands
        .spawn((
            SetMipmaps,
            Mesh3d(meshes.add(Plane3d::default().mesh().size(5. / 16., 5. / 16.))),
            MeshMaterial3d(diamond_mat.clone()),
            Transform::from_xyz(to_local(107. + 14.), 0.01, to_local(230. + 22.)),
        ))
        .id();
    et.set_mipmaps = Some(set_mipmaps_id);
    commands.entity(settings_id).add_child(set_mipmaps_id);

    let set_resolution_id = commands
        .spawn((
            SetResolution,
            Mesh3d(meshes.add(Plane3d::default().mesh().size(5. / 16., 5. / 16.))),
            MeshMaterial3d(diamond_mat.clone()),
            Transform::from_xyz(to_local(107. + 14.), 0.01, to_local(320. + 22.)),
        ))
        .id();
    et.set_resolution = Some(set_resolution_id);
    commands.entity(settings_id).add_child(set_resolution_id);

    let set_fps_id = commands
        .spawn((
            SetFps,
            Mesh3d(meshes.add(Plane3d::default().mesh().size(5. / 16., 5. / 16.))),
            MeshMaterial3d(diamond_mat.clone()),
            Transform::from_xyz(to_local(107. + 14.), 0.01, to_local(410. + 22.)),
        ))
        .id();
    et.set_fps = Some(set_fps_id);
    commands.entity(settings_id).add_child(set_fps_id);

    commands.entity(settings_id).observe(
        move |mut click: On<Pointer<Click>>,
              et: Res<EntityTable>,
              stitched: Option<Res<StitchedRoundel>>,
              mut query: Query<(&mut Settings, &GlobalTransform)>,
              mut diamond_query: Query<&mut Transform, Without<Settings>>,
              mut images: ResMut<Assets<Image>>,
              mut materials: ResMut<Assets<StandardMaterial>>| {
            let Ok((settings, settings_global)) = query.get_mut(click.event_target()) else {
                return;
            };
            if !settings.active || click.duration.as_millis() > 250 {
                return;
            }
            let Some(hit_pos) = click.hit.position else {
                return;
            };
            let local_hit = settings_global.affine().inverse().transform_point3(hit_pos);
            let px = from_local(local_hit.x);
            let py = from_local(local_hit.z);

            let clicked_data = settings_data
                .iter()
                .find(|row| py >= row.y_top && py <= row.y_bot)
                .and_then(|row| {
                    row.x_bounds
                        .windows(2)
                        .zip(row.items.iter())
                        .find(|(bounds, _)| px >= bounds[0] && px < bounds[1])
                        .map(|(bounds, item)| (&row.cat, item, row.y_top, bounds[0]))
                });

            if let Some((category, item, y_start, x_start)) = clicked_data {
                let Some(ref stitched_res) = stitched else {
                    return;
                };
                let target_handle = &stitched_res.handle;
                match category {
                    SetCatType::Anisotropic => {
                        if let Ok(mut transform) =
                            diamond_query.get_mut(et.set_anisotropic.unwrap())
                        {
                            transform.translation =
                                Vec3::new(to_local(x_start + 14.0), 0.01, to_local(y_start + 22.0));
                        }
                        if let Some(img) = images.get_mut(target_handle) {
                            let mut is_desc = match img.sampler.clone() {
                                ImageSampler::Descriptor(d) => d,
                                _ => ImageSamplerDescriptor::default(),
                            };
                            is_desc.anisotropy_clamp = match item {
                                SetItem::An16 => 16,
                                SetItem::An8 => 8,
                                SetItem::An4 => 4,
                                SetItem::An2 => 2,
                                _ => 1,
                            };
                            if is_desc.anisotropy_clamp > 1 {
                                is_desc.mipmap_filter = ImageFilterMode::Linear;
                                if let Ok(mut mip_transform) =
                                    diamond_query.get_mut(et.set_mipmaps.unwrap())
                                {
                                    mip_transform.translation = Vec3::new(
                                        to_local(107. + 14.0),
                                        0.01,
                                        to_local(230. + 22.0),
                                    );
                                }
                            }
                            img.sampler = ImageSampler::Descriptor(is_desc);
                            for (_, mat) in materials.iter_mut() {
                                if mat
                                    .base_color_texture
                                    .as_ref()
                                    .map(|h| h.id() == target_handle.id())
                                    .unwrap_or(false)
                                {
                                    mat.base_color_texture = Some(target_handle.clone());
                                }
                            }
                        }
                    }
                    SetCatType::Mipmaps => {
                        if let Ok(mut transform) = diamond_query.get_mut(et.set_mipmaps.unwrap()) {
                            transform.translation =
                                Vec3::new(to_local(x_start + 14.0), 0.01, to_local(y_start + 22.0));
                        }
                        if let Some(img) = images.get_mut(target_handle) {
                            let mut is_desc = match img.sampler.clone() {
                                ImageSampler::Descriptor(d) => d,
                                _ => ImageSamplerDescriptor::default(),
                            };
                            is_desc.mipmap_filter = match item {
                                SetItem::MMOn => ImageFilterMode::Linear,
                                SetItem::MMOff => {
                                    is_desc.anisotropy_clamp = 1;
                                    if let Ok(mut aniso_transform) =
                                        diamond_query.get_mut(et.set_anisotropic.unwrap())
                                    {
                                        aniso_transform.translation = Vec3::new(
                                            to_local(310. + 14.0),
                                            0.01,
                                            to_local(140. + 22.0),
                                        );
                                    }
                                    ImageFilterMode::Linear
                                }
                                _ => ImageFilterMode::Linear,
                            };
                            img.sampler = ImageSampler::Descriptor(is_desc);
                            for (_, mat) in materials.iter_mut() {
                                if mat
                                    .base_color_texture
                                    .as_ref()
                                    .map(|h| h.id() == target_handle.id())
                                    .unwrap_or(false)
                                {
                                    mat.base_color_texture = Some(target_handle.clone());
                                }
                            }
                        }
                    }
                    SetCatType::AssetResolution => {
                        if let Ok(mut transform) = diamond_query.get_mut(et.set_resolution.unwrap())
                        {
                            transform.translation =
                                Vec3::new(to_local(x_start + 14.0), 0.01, to_local(y_start + 22.0));
                        }
                        if let Some(img) = images.get_mut(target_handle) {
                            let mut is_desc = match img.sampler.clone() {
                                ImageSampler::Descriptor(d) => d,
                                _ => ImageSamplerDescriptor::default(),
                            };
                            is_desc.lod_min_clamp = match item {
                                SetItem::AResHi => 0.,
                                SetItem::AResMed => 1.,
                                SetItem::AResLow => 2.,
                                _ => 3.,
                            };
                            img.sampler = ImageSampler::Descriptor(is_desc);
                            for (_, mat) in materials.iter_mut() {
                                if mat
                                    .base_color_texture
                                    .as_ref()
                                    .map(|h| h.id() == target_handle.id())
                                    .unwrap_or(false)
                                {
                                    mat.base_color_texture = Some(target_handle.clone());
                                }
                            }
                        }
                    }
                    SetCatType::FPSDisplay => {
                        if let Ok(mut transform) = diamond_query.get_mut(et.set_fps.unwrap()) {
                            transform.translation =
                                Vec3::new(to_local(x_start + 14.0), 0.01, to_local(y_start + 22.0));
                        }
                    }
                }
                click.propagate(false);
            }
        },
    );
}
