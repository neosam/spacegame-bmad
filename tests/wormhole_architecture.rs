#![deny(clippy::unwrap_used)]
//! Integration tests for Story 9-5: Isolated Scene Architecture.
//!
//! Tests cover: PlayingSubState::InWormhole exists, ArenaState default values,
//! and WormholeEntrance has correct fields.

mod helpers;

use bevy::prelude::*;
use void_drifter::core::wormhole::{ArenaState, WormholeEntrance};
use void_drifter::game_states::PlayingSubState;

// ── Tests ───────────────────────────────────────────────────────────────────

/// Verify that PlayingSubState::InWormhole variant exists and is distinguishable from Flying.
#[test]
fn playing_sub_state_in_wormhole_exists() {
    let flying = PlayingSubState::Flying;
    let in_wormhole = PlayingSubState::InWormhole;
    assert_ne!(
        flying, in_wormhole,
        "PlayingSubState::InWormhole must be a distinct variant from Flying"
    );
}

/// Verify ArenaState has correct Default values: wave=0, total_waves=0, enemies_remaining=0, cleared=false.
#[test]
fn arena_state_default_values() {
    let state = ArenaState::default();
    assert_eq!(state.wave, 0, "ArenaState::wave should default to 0");
    assert_eq!(
        state.total_waves, 0,
        "ArenaState::total_waves should default to 0"
    );
    assert_eq!(
        state.enemies_remaining, 0,
        "ArenaState::enemies_remaining should default to 0"
    );
    assert!(
        !state.cleared,
        "ArenaState::cleared should default to false"
    );
}

/// Verify WormholeEntrance can be constructed and fields are accessible.
#[test]
fn wormhole_entrance_has_correct_fields() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);

    // Spawn a placeholder entity to use as wormhole_entity
    let dummy_entity = app.world_mut().spawn_empty().id();

    let entrance = WormholeEntrance {
        world_position: Vec2::new(500.0, -300.0),
        wormhole_entity: dummy_entity,
    };

    assert!(
        (entrance.world_position.x - 500.0).abs() < f32::EPSILON,
        "WormholeEntrance::world_position.x should be 500.0, got {}",
        entrance.world_position.x
    );
    assert!(
        (entrance.world_position.y - (-300.0)).abs() < f32::EPSILON,
        "WormholeEntrance::world_position.y should be -300.0, got {}",
        entrance.world_position.y
    );
    assert_eq!(
        entrance.wormhole_entity, dummy_entity,
        "WormholeEntrance::wormhole_entity should match the spawned entity"
    );
}
