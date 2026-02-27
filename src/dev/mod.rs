use bevy::prelude::*;

pub struct DevPlugin;

impl Plugin for DevPlugin {
    fn build(&self, _app: &mut App) {
        #[cfg(feature = "dev")]
        {
            use bevy::diagnostic::{
                EntityCountDiagnosticsPlugin, FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin,
            };

            _app.add_plugins((
                FrameTimeDiagnosticsPlugin::default(),
                EntityCountDiagnosticsPlugin::default(),
                LogDiagnosticsPlugin::default(),
                bevy_inspector_egui::quick::WorldInspectorPlugin::new(),
            ));
        }
    }
}
