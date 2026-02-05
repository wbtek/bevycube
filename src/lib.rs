pub mod ui;
pub mod camera;
pub mod roundel;
pub mod world;

use crate::roundel::*;

use bevy::prelude::*;
use bevy::asset::embedded_asset;
use bevy::image::*;

#[derive(Debug, Resource)] pub struct DiskParms { pub rotation_speed: f32 }
#[derive(Debug, Resource)] pub struct CubeParms { pub rotation_speed: f32 }

#[derive(Debug, Resource, Default)]
pub struct EntityTable {
    pub cube: Option<Entity>,
    pub disk: Option<Entity>,
    pub ground: Option<Entity>,
    pub settings: Option<Entity>,
    pub set_anisotropic: Option<Entity>,
    pub set_mipmaps: Option<Entity>,
    pub set_resolution: Option<Entity>,
    pub set_fps: Option<Entity>,
    pub safety_disk: Option<Entity>,
    pub main_anchor: Option<Entity>,
    pub main_camera: Option<Entity>,
}

pub struct DemoAssetsPlugin;
impl Plugin for DemoAssetsPlugin {
    fn build(&self, app: &mut App) {
        embedded_asset!(app, "media/wbtekbg2b512.jpg");
        embedded_asset!(app, "media/settings.jpg");
        embedded_asset!(app, "media/diamond_sprite.jpg");
    }
}

