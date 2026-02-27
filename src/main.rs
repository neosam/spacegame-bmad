use bevy::prelude::*;
use void_drifter::game_plugins;
use void_drifter::game_states::{GameState, PlayingSubState};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Void Drifter".to_string(),
                resolution: (1280u32, 720u32).into(),
                ..default()
            }),
            ..default()
        }))
        .init_state::<GameState>()
        .add_sub_state::<PlayingSubState>()
        .add_plugins(game_plugins())
        .add_systems(Startup, setup_camera)
        .run();
}

fn setup_camera(mut commands: Commands) {
    commands.spawn(Camera2d);
}
