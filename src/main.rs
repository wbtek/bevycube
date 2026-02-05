use bevy::prelude::*;
use bevy::asset::AssetMetaCheck;

use bevycube::*;

fn main() {
    #[cfg(target_arch = "wasm32")]
    console_log::init_with_level(log::Level::Debug).ok();

    App::new()
        .init_resource::<EntityTable>()
        .insert_resource(DiskParms { rotation_speed: 0.2 })
        .insert_resource(CubeParms { rotation_speed: -1.0 })
        .add_plugins(DefaultPlugins.set(AssetPlugin {
            meta_check: AssetMetaCheck::Never,
            ..default()
        }))
        .add_plugins(MeshPickingPlugin)
        .add_plugins((
            DemoAssetsPlugin,
            PlayerPlugin,
            EnvironmentPlugin,
            CameraPlugin,
            SettingsUiPlugin,
        ))
        .run();
}
