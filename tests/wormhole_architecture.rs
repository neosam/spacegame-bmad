#![deny(clippy::unwrap_used)]
//! Integration tests for Story 9-5 and Story 9-1: Wormhole Architecture and Visuals.
//!
//! Tests cover: PlayingSubState::InWormhole exists, ArenaState default values,
//! WormholeEntrance has correct fields, Wormhole component, and should_spawn_wormhole logic.

mod helpers;

use bevy::prelude::*;
use void_drifter::core::wormhole::{ArenaState, Wormhole, WormholeEntrance, should_spawn_wormhole};
use void_drifter::game_states::PlayingSubState;
use void_drifter::world::ChunkCoord;

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

// ── Story 9-1: Wormhole Visuals ──────────────────────────────────────────────

/// Chunks near the origin (Euclidean distance < 2) must never spawn wormholes.
#[test]
fn should_spawn_wormhole_respects_min_distance() {
    let origin = ChunkCoord { x: 0, y: 0 };
    assert!(
        !should_spawn_wormhole(origin, 12345),
        "Chunk (0,0) is at distance 0 — must not spawn wormhole"
    );

    let near_x = ChunkCoord { x: 1, y: 0 };
    assert!(
        !should_spawn_wormhole(near_x, 12345),
        "Chunk (1,0) is at distance 1 — must not spawn wormhole"
    );

    let near_y = ChunkCoord { x: 0, y: 1 };
    assert!(
        !should_spawn_wormhole(near_y, 12345),
        "Chunk (0,1) is at distance 1 — must not spawn wormhole"
    );
}

/// Same coordinate and seed must always produce the same result (deterministic).
#[test]
fn should_spawn_wormhole_deterministic() {
    let coord = ChunkCoord { x: 5, y: 3 };
    let result1 = should_spawn_wormhole(coord, 99999);
    let result2 = should_spawn_wormhole(coord, 99999);
    assert_eq!(
        result1, result2,
        "should_spawn_wormhole must be deterministic for the same coord and seed"
    );
}

/// Different seeds should not always produce the same result.
#[test]
fn should_spawn_wormhole_seed_changes_result() {
    // Find a far-away coord where at least one seed returns true and one returns false.
    // We try multiple coords until we find a pair of seeds with different outcomes.
    let far_coords: Vec<ChunkCoord> = (3..20i32)
        .flat_map(|x| (3..20i32).map(move |y| ChunkCoord { x, y }))
        .collect();

    let mut found_difference = false;
    for coord in &far_coords {
        let r1 = should_spawn_wormhole(*coord, 1);
        let r2 = should_spawn_wormhole(*coord, 2);
        if r1 != r2 {
            found_difference = true;
            break;
        }
    }
    assert!(
        found_difference,
        "Different seeds should sometimes produce different wormhole spawn results"
    );
}

/// Wormhole component fields are accessible and hold correct values.
#[test]
fn wormhole_component_fields() {
    let w = Wormhole {
        coord: ChunkCoord { x: 3, y: 4 },
        cleared: false,
    };
    assert!(!w.cleared, "Wormhole.cleared should be false");
    assert_eq!(w.coord.x, 3, "Wormhole.coord.x should be 3");
    assert_eq!(w.coord.y, 4, "Wormhole.coord.y should be 4");
}

/// A cleared wormhole reports cleared = true.
#[test]
fn wormhole_component_cleared_flag() {
    let w = Wormhole {
        coord: ChunkCoord { x: 7, y: -2 },
        cleared: true,
    };
    assert!(w.cleared, "Wormhole.cleared should be true when set");
}

/// At least some distant chunks should spawn wormholes with a fixed seed.
#[test]
fn should_spawn_wormhole_spawns_some_at_distance() {
    let seed = 42u64;
    let spawned_count = (-20i32..=20)
        .flat_map(|x| (-20i32..=20).map(move |y| ChunkCoord { x, y }))
        .filter(|c| should_spawn_wormhole(*c, seed))
        .count();
    assert!(
        spawned_count > 0,
        "At least one wormhole should spawn across a 41×41 grid at distance ≥ 2 with seed {}",
        seed
    );
}
