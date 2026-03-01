use bevy::prelude::*;
use bevy::ecs::message::MessageReader;
use bevy_kira_audio::prelude::*;

use crate::game_states::{GameState, PlayingSubState};
use crate::shared::events::{GameEvent, GameEventKind};

/// Aktueller Musik-Zustand.
#[derive(PartialEq, Clone, Debug, Default)]
pub enum MusicState {
    #[default]
    None,
    Exploration,
    Combat,
    Arena,
    Docked,
}

/// Tracks den aktuellen Musik-Zustand und Audio-Instance.
#[derive(Resource, Default)]
pub struct MusicTrack {
    pub current: MusicState,
    pub target: MusicState,
}

/// Gecachte Audio-Handles für SFX.
/// Alle Handles sind Optional — fehlendes Asset = kein Sound, kein Crash.
#[derive(Resource, Default)]
pub struct AudioAssets {
    pub laser_fire: Option<Handle<bevy_kira_audio::AudioSource>>,
    pub spread_fire: Option<Handle<bevy_kira_audio::AudioSource>>,
    pub explosion_small: Option<Handle<bevy_kira_audio::AudioSource>>,
    pub explosion_large: Option<Handle<bevy_kira_audio::AudioSource>>,
    pub player_death: Option<Handle<bevy_kira_audio::AudioSource>>,
    pub station_dock: Option<Handle<bevy_kira_audio::AudioSource>>,
    pub wormhole_enter: Option<Handle<bevy_kira_audio::AudioSource>>,
}

/// Plugin für Audio-Infrastruktur.
pub struct AudioInfrastructurePlugin;

impl Plugin for AudioInfrastructurePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(AudioPlugin); // bevy_kira_audio Plugin
        app.init_resource::<MusicTrack>();
        app.init_resource::<AudioAssets>();
        app.add_systems(Startup, load_audio_assets);
        app.add_systems(
            Update,
            detect_music_state.run_if(in_state(GameState::Playing)),
        );
        app.add_systems(
            Update,
            play_event_sfx.run_if(in_state(GameState::Playing)),
        );
    }
}

/// Erkennt den gewünschten Musik-Zustand basierend auf Game-Zustand.
pub fn detect_music_state(
    mut music_track: ResMut<MusicTrack>,
    playing_sub_state: Option<Res<State<PlayingSubState>>>,
) {
    let target = if let Some(sub_state) = playing_sub_state {
        match sub_state.get() {
            PlayingSubState::InWormhole => MusicState::Arena,
            PlayingSubState::Flying => MusicState::Exploration,
        }
    } else {
        MusicState::Exploration
    };

    if music_track.target != target {
        music_track.target = target;
    }
}

/// Lädt Audio-Assets beim Start. Graceful — kein Crash wenn Dateien fehlen.
pub fn load_audio_assets(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    let assets = AudioAssets {
        laser_fire: Some(asset_server.load::<bevy_kira_audio::AudioSource>("audio/sfx/laser_fire.ogg")),
        spread_fire: Some(asset_server.load::<bevy_kira_audio::AudioSource>("audio/sfx/spread_fire.ogg")),
        explosion_small: Some(asset_server.load::<bevy_kira_audio::AudioSource>("audio/sfx/explosion_small.ogg")),
        explosion_large: Some(asset_server.load::<bevy_kira_audio::AudioSource>("audio/sfx/explosion_large.ogg")),
        player_death: Some(asset_server.load::<bevy_kira_audio::AudioSource>("audio/sfx/player_death.ogg")),
        station_dock: Some(asset_server.load::<bevy_kira_audio::AudioSource>("audio/sfx/station_dock.ogg")),
        wormhole_enter: Some(asset_server.load::<bevy_kira_audio::AudioSource>("audio/sfx/wormhole_enter.ogg")),
    };
    commands.insert_resource(assets);
}

/// Spielt SFX basierend auf eingehenden GameEvents.
/// Kein Handle = kein Sound, kein Crash.
pub fn play_event_sfx(
    mut events: MessageReader<GameEvent>,
    audio_assets: Res<AudioAssets>,
    audio: Res<Audio>,
) {
    for event in events.read() {
        match &event.kind {
            GameEventKind::WeaponFired { .. } => {
                if let Some(handle) = &audio_assets.laser_fire {
                    audio.play(handle.clone());
                }
            }
            GameEventKind::EnemyDestroyed { .. } => {
                if let Some(handle) = &audio_assets.explosion_small {
                    audio.play(handle.clone());
                }
            }
            GameEventKind::PlayerDeath => {
                if let Some(handle) = &audio_assets.player_death {
                    audio.play(handle.clone());
                }
            }
            GameEventKind::StationDocked => {
                if let Some(handle) = &audio_assets.station_dock {
                    audio.play(handle.clone());
                }
            }
            GameEventKind::WormholeEntered { .. } => {
                if let Some(handle) = &audio_assets.wormhole_enter {
                    audio.play(handle.clone());
                }
            }
            _ => {}
        }
    }
}
