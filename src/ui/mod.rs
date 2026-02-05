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

#[derive(Debug, Component)] #[require(Transform, Visibility)] pub struct SetAnisotropic;
#[derive(Debug, Component)] #[require(Transform, Visibility)] pub struct SetMipmaps;
#[derive(Debug, Component)] #[require(Transform, Visibility)] pub struct SetResolution;
#[derive(Debug, Component)] #[require(Transform, Visibility)] pub struct SetFps;

pub fn to_local(pixel: f32) -> f32 { (pixel - 256.0) / 512.0 * 5.0 }
pub fn from_local(pixel: f32) -> f32 { (pixel / 5.0) * 512.0 + 256.0 }

