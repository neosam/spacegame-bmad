#![deny(clippy::unwrap_used)]

use void_drifter::infrastructure::audio::AudioAssets;

#[test]
fn audio_assets_default_has_no_handles() {
    let assets = AudioAssets::default();
    assert!(assets.laser_fire.is_none());
    assert!(assets.spread_fire.is_none());
    assert!(assets.explosion_small.is_none());
    assert!(assets.explosion_large.is_none());
    assert!(assets.player_death.is_none());
    assert!(assets.station_dock.is_none());
    assert!(assets.wormhole_enter.is_none());
}

#[test]
fn sfx_does_not_require_all_assets() {
    // AudioAssets kann mit partiell befüllten Handles existieren
    let assets = AudioAssets {
        laser_fire: None,
        explosion_small: None,
        ..Default::default()
    };
    // Kompiliert = Test bestanden
    assert!(assets.laser_fire.is_none());
    assert!(assets.explosion_small.is_none());
}

#[test]
fn music_state_default_is_none() {
    use void_drifter::infrastructure::audio::{MusicState, MusicTrack};
    let track = MusicTrack::default();
    assert_eq!(track.current, MusicState::None);
    assert_eq!(track.target, MusicState::None);
}

#[test]
fn music_states_are_distinct() {
    use void_drifter::infrastructure::audio::MusicState;
    assert_ne!(MusicState::Exploration, MusicState::Combat);
    assert_ne!(MusicState::Arena, MusicState::Docked);
}

#[test]
fn music_track_target_can_be_set() {
    use void_drifter::infrastructure::audio::{MusicState, MusicTrack};
    let mut track = MusicTrack::default();
    track.target = MusicState::Arena;
    assert_eq!(track.target, MusicState::Arena);
}

#[test]
fn music_track_current_and_target_are_independent() {
    use void_drifter::infrastructure::audio::{MusicState, MusicTrack};
    let mut track = MusicTrack::default();
    track.current = MusicState::Exploration;
    track.target = MusicState::Combat;
    assert_ne!(track.current, track.target);
    assert_eq!(track.current, MusicState::Exploration);
    assert_eq!(track.target, MusicState::Combat);
}

#[test]
fn music_state_clone_works() {
    use void_drifter::infrastructure::audio::MusicState;
    let state = MusicState::Arena;
    let cloned = state.clone();
    assert_eq!(state, cloned);
}
