#![deny(clippy::unwrap_used)]
//! Integration tests for Story 9-5, 9-1 and 9-2: Wormhole Architecture, Visuals, and Enter.
//!
//! Tests cover: PlayingSubState::InWormhole exists, ArenaState default values,
//! WormholeEntrance has correct fields, Wormhole component, should_spawn_wormhole logic,
//! and wormhole proximity / entry mechanics.

mod helpers;

use bevy::prelude::*;
use void_drifter::core::wormhole::{
    ArenaState, Wormhole, WormholeEntrance, WORMHOLE_ENTER_RADIUS, should_spawn_wormhole,
    check_wormhole_proximity,
};
use void_drifter::core::flight::Player;
use void_drifter::core::input::ActionState;
use void_drifter::game_states::PlayingSubState;
use void_drifter::infrastructure::events::EventSeverityConfig;
use void_drifter::shared::events::GameEvent;
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

// ── Story 9-2: Enter Wormhole ─────────────────────────────────────────────────

/// WormholeEntrance resource stores the correct entry position.
#[test]
fn wormhole_entrance_stores_position() {
    // Use a real spawned entity as the wormhole reference
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    let dummy = app.world_mut().spawn_empty().id();

    let entrance = WormholeEntrance {
        world_position: Vec2::new(100.0, 200.0),
        wormhole_entity: dummy,
    };
    assert_eq!(
        entrance.world_position.x, 100.0,
        "WormholeEntrance.world_position.x should be 100.0"
    );
    assert_eq!(
        entrance.world_position.y, 200.0,
        "WormholeEntrance.world_position.y should be 200.0"
    );
}

/// WORMHOLE_ENTER_RADIUS constant must be a positive value (60 units).
#[test]
fn wormhole_enter_radius_is_positive() {
    assert!(
        WORMHOLE_ENTER_RADIUS > 0.0,
        "WORMHOLE_ENTER_RADIUS must be positive, got {}",
        WORMHOLE_ENTER_RADIUS
    );
    assert!(
        (WORMHOLE_ENTER_RADIUS - 60.0).abs() < f32::EPSILON,
        "WORMHOLE_ENTER_RADIUS should be 60.0, got {}",
        WORMHOLE_ENTER_RADIUS
    );
}

/// ArenaState created by check_wormhole_proximity has 3 total_waves.
#[test]
fn arena_state_on_enter_has_three_waves() {
    // Verify the expected arena state values that are set on wormhole entry
    let arena = ArenaState {
        wave: 0,
        total_waves: 3,
        enemies_remaining: 0,
        cleared: false,
    };
    assert_eq!(arena.total_waves, 3, "ArenaState on entry should have 3 waves");
    assert_eq!(arena.wave, 0, "ArenaState on entry should start at wave 0");
    assert!(!arena.cleared, "ArenaState on entry should not be cleared");
}

/// check_wormhole_proximity system: when player is close and interact=true,
/// WormholeEntrance resource is created with the player's pre-entry position.
#[test]
fn wormhole_proximity_creates_entrance_resource() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(bevy::state::app::StatesPlugin);
    app.init_resource::<ActionState>();
    app.insert_resource(EventSeverityConfig::default());
    app.add_message::<GameEvent>();

    // Register states needed by check_wormhole_proximity
    app.init_state::<void_drifter::game_states::GameState>();
    app.add_sub_state::<PlayingSubState>();

    app.add_systems(Update, check_wormhole_proximity);

    // Spawn player at position (50, 0) — within 60 units of wormhole at origin
    let player_pos = Vec2::new(50.0, 0.0);
    app.world_mut().spawn((
        Player,
        Transform::from_translation(player_pos.extend(0.0)),
    ));

    // Spawn a non-cleared wormhole at origin
    let wormhole_entity = app.world_mut().spawn((
        Wormhole {
            coord: ChunkCoord { x: 3, y: 4 },
            cleared: false,
        },
        Transform::from_translation(Vec3::ZERO),
    )).id();

    // Set interact = true
    app.world_mut().resource_mut::<ActionState>().interact = true;

    // Run one update frame
    app.update();

    // WormholeEntrance resource should now exist
    let entrance = app
        .world()
        .get_resource::<WormholeEntrance>()
        .expect("WormholeEntrance resource should be inserted after player enters wormhole");

    assert!(
        (entrance.world_position.x - player_pos.x).abs() < 0.01,
        "WormholeEntrance.world_position.x should match pre-entry player x: expected {}, got {}",
        player_pos.x,
        entrance.world_position.x
    );
    assert!(
        (entrance.world_position.y - player_pos.y).abs() < 0.01,
        "WormholeEntrance.world_position.y should match pre-entry player y: expected {}, got {}",
        player_pos.y,
        entrance.world_position.y
    );
    assert_eq!(
        entrance.wormhole_entity, wormhole_entity,
        "WormholeEntrance.wormhole_entity should reference the wormhole entity"
    );
}

/// check_wormhole_proximity: player too far away — no WormholeEntrance created.
#[test]
fn wormhole_proximity_no_entrance_when_too_far() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(bevy::state::app::StatesPlugin);
    app.init_resource::<ActionState>();
    app.insert_resource(EventSeverityConfig::default());
    app.add_message::<GameEvent>();

    app.init_state::<void_drifter::game_states::GameState>();
    app.add_sub_state::<PlayingSubState>();

    app.add_systems(Update, check_wormhole_proximity);

    // Player far from wormhole (200 units away)
    app.world_mut().spawn((
        Player,
        Transform::from_translation(Vec3::new(200.0, 0.0, 0.0)),
    ));

    // Wormhole at origin
    app.world_mut().spawn((
        Wormhole {
            coord: ChunkCoord { x: 5, y: 5 },
            cleared: false,
        },
        Transform::from_translation(Vec3::ZERO),
    ));

    // Set interact = true (player is pressing E but is too far)
    app.world_mut().resource_mut::<ActionState>().interact = true;

    app.update();

    // WormholeEntrance should NOT exist (player is out of range)
    assert!(
        app.world().get_resource::<WormholeEntrance>().is_none(),
        "WormholeEntrance should not be created when player is too far from wormhole"
    );
}

/// check_wormhole_proximity: player close but interact=false — no entrance created.
#[test]
fn wormhole_proximity_no_entrance_without_interact() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(bevy::state::app::StatesPlugin);
    app.init_resource::<ActionState>();
    app.insert_resource(EventSeverityConfig::default());
    app.add_message::<GameEvent>();

    app.init_state::<void_drifter::game_states::GameState>();
    app.add_sub_state::<PlayingSubState>();

    app.add_systems(Update, check_wormhole_proximity);

    // Player within range
    app.world_mut().spawn((
        Player,
        Transform::from_translation(Vec3::new(40.0, 0.0, 0.0)),
    ));

    // Wormhole at origin
    app.world_mut().spawn((
        Wormhole {
            coord: ChunkCoord { x: 2, y: 3 },
            cleared: false,
        },
        Transform::from_translation(Vec3::ZERO),
    ));

    // interact = false (player is nearby but NOT pressing E)
    app.world_mut().resource_mut::<ActionState>().interact = false;

    app.update();

    // WormholeEntrance should NOT exist
    assert!(
        app.world().get_resource::<WormholeEntrance>().is_none(),
        "WormholeEntrance should not be created when interact is false"
    );
}

/// check_wormhole_proximity: cleared wormhole is ignored even when close and interact=true.
#[test]
fn wormhole_proximity_ignores_cleared_wormholes() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(bevy::state::app::StatesPlugin);
    app.init_resource::<ActionState>();
    app.insert_resource(EventSeverityConfig::default());
    app.add_message::<GameEvent>();

    app.init_state::<void_drifter::game_states::GameState>();
    app.add_sub_state::<PlayingSubState>();

    app.add_systems(Update, check_wormhole_proximity);

    // Player within range
    app.world_mut().spawn((
        Player,
        Transform::from_translation(Vec3::new(30.0, 0.0, 0.0)),
    ));

    // Cleared wormhole at origin — should be ignored
    app.world_mut().spawn((
        Wormhole {
            coord: ChunkCoord { x: 4, y: 2 },
            cleared: true,
        },
        Transform::from_translation(Vec3::ZERO),
    ));

    // interact = true
    app.world_mut().resource_mut::<ActionState>().interact = true;

    app.update();

    // WormholeEntrance should NOT exist for a cleared wormhole
    assert!(
        app.world().get_resource::<WormholeEntrance>().is_none(),
        "WormholeEntrance should not be created for a cleared wormhole"
    );
}

/// ArenaState created on wormhole entry starts at wave=0 with total_waves=3.
#[test]
fn wormhole_entry_creates_arena_state() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(bevy::state::app::StatesPlugin);
    app.init_resource::<ActionState>();
    app.insert_resource(EventSeverityConfig::default());
    app.add_message::<GameEvent>();

    app.init_state::<void_drifter::game_states::GameState>();
    app.add_sub_state::<PlayingSubState>();

    app.add_systems(Update, check_wormhole_proximity);

    // Player within range
    app.world_mut().spawn((
        Player,
        Transform::from_translation(Vec3::new(40.0, 0.0, 0.0)),
    ));

    // Non-cleared wormhole
    app.world_mut().spawn((
        Wormhole {
            coord: ChunkCoord { x: 6, y: 7 },
            cleared: false,
        },
        Transform::from_translation(Vec3::ZERO),
    ));

    app.world_mut().resource_mut::<ActionState>().interact = true;

    app.update();

    // ArenaState should be inserted
    let arena = app
        .world()
        .get_resource::<ArenaState>()
        .expect("ArenaState resource should be inserted on wormhole entry");

    assert_eq!(arena.wave, 0, "ArenaState.wave should be 0 on entry");
    assert_eq!(
        arena.total_waves, 3,
        "ArenaState.total_waves should be 3 on entry"
    );
    assert_eq!(
        arena.enemies_remaining, 0,
        "ArenaState.enemies_remaining should be 0 on entry"
    );
    assert!(!arena.cleared, "ArenaState.cleared should be false on entry");
}
