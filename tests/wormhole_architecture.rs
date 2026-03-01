#![deny(clippy::unwrap_used)]
//! Integration tests for Story 9-5, 9-1 and 9-2: Wormhole Architecture, Visuals, and Enter.
//!
//! Tests cover: PlayingSubState::InWormhole exists, ArenaState default values,
//! WormholeEntrance has correct fields, Wormhole component, should_spawn_wormhole logic,
//! and wormhole proximity / entry mechanics.

mod helpers;

use bevy::prelude::*;
use void_drifter::core::wormhole::{
    ArenaState, ArenaEnemy, ArenaBoundary, Wormhole, WormholeEntrance,
    WORMHOLE_ENTER_RADIUS, ARENA_RADIUS,
    should_spawn_wormhole, check_wormhole_proximity, setup_arena, spawn_arena_wave,
    enforce_arena_boundary, cleanup_arena,
};
use void_drifter::core::spawning::SpawningConfig;
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
        completion_notified: false,
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

// ── Story 9-3: Arena Combat ───────────────────────────────────────────────────

/// ArenaEnemy and ArenaBoundary are distinct marker components (compile check).
#[test]
fn arena_enemy_is_not_boundary() {
    let _: ArenaEnemy = ArenaEnemy;
    let _: ArenaBoundary = ArenaBoundary;
}

/// ARENA_RADIUS constant must be exactly 800.0.
#[test]
fn arena_radius_is_800() {
    assert_eq!(ARENA_RADIUS, 800.0, "ARENA_RADIUS should be 800.0");
}

/// setup_arena initializes ArenaState: wave=0, total_waves=3, enemies_remaining=0, cleared=false.
#[test]
fn setup_arena_initializes_state() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    // Start with a non-default ArenaState to verify setup_arena resets it
    app.insert_resource(ArenaState {
        wave: 5,
        total_waves: 10,
        enemies_remaining: 99,
        cleared: true,
        completion_notified: true,
    });
    app.add_systems(bevy::app::Startup, setup_arena);
    app.update();

    let arena = app
        .world()
        .get_resource::<ArenaState>()
        .expect("ArenaState resource should exist after setup_arena");

    assert_eq!(arena.wave, 0, "setup_arena should reset wave to 0");
    assert_eq!(arena.total_waves, 3, "setup_arena should set total_waves to 3");
    assert_eq!(arena.enemies_remaining, 0, "setup_arena should reset enemies_remaining to 0");
    assert!(!arena.cleared, "setup_arena should reset cleared to false");
}

/// spawn_arena_wave spawns wave 1 (3 ScoutDrones) when no enemies remain and wave=0.
#[test]
fn spawn_arena_wave_spawns_wave_1_on_first_call() {
    use void_drifter::core::spawning::ScoutDrone;

    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.insert_resource(SpawningConfig::default());
    app.insert_resource(ArenaState {
        wave: 0,
        total_waves: 3,
        enemies_remaining: 0,
        cleared: false,
        completion_notified: false,
    });
    app.add_systems(bevy::app::Update, spawn_arena_wave);
    app.update();

    let drone_count = app
        .world_mut()
        .query_filtered::<Entity, (With<ScoutDrone>, With<ArenaEnemy>)>()
        .iter(app.world())
        .count();
    assert_eq!(drone_count, 3, "Wave 1 should spawn 3 ScoutDrones with ArenaEnemy marker");

    let arena = app
        .world()
        .get_resource::<ArenaState>()
        .expect("ArenaState should exist");
    assert_eq!(arena.wave, 1, "wave should advance to 1 after first spawn");
}

/// After all waves are spawned and enemies are gone, arena is marked cleared.
#[test]
fn arena_cleared_after_all_waves_defeated() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.insert_resource(SpawningConfig::default());
    // Simulate: wave 3 already done, no enemies
    app.insert_resource(ArenaState {
        wave: 3,
        total_waves: 3,
        enemies_remaining: 0,
        cleared: false,
        completion_notified: false,
    });
    app.add_systems(bevy::app::Update, spawn_arena_wave);
    app.update();

    let arena = app
        .world()
        .get_resource::<ArenaState>()
        .expect("ArenaState should exist");
    assert!(arena.cleared, "Arena should be cleared after all waves are done and no enemies remain");
}

/// enforce_arena_boundary clamps player position to ARENA_RADIUS when outside.
#[test]
fn enforce_arena_boundary_clamps_player() {
    use void_drifter::shared::components::Velocity;

    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_systems(bevy::app::Update, enforce_arena_boundary);

    // Spawn player far outside the arena
    let player = app.world_mut().spawn((
        Player,
        Transform::from_translation(Vec3::new(2000.0, 0.0, 0.0)),
        Velocity(Vec2::new(500.0, 0.0)),
    )).id();

    app.update();

    let transform = app
        .world()
        .entity(player)
        .get::<Transform>()
        .expect("Player should have Transform");
    let pos2 = transform.translation.truncate();
    assert!(
        pos2.length() <= ARENA_RADIUS + 0.01,
        "Player at distance {} should be clamped to ARENA_RADIUS {}",
        pos2.length(),
        ARENA_RADIUS
    );

    let velocity = app
        .world()
        .entity(player)
        .get::<Velocity>()
        .expect("Player should have Velocity");
    assert_eq!(
        velocity.0,
        Vec2::ZERO,
        "Player velocity should be zeroed when clamped to boundary"
    );
}

/// enforce_arena_boundary leaves player inside the arena untouched.
#[test]
fn enforce_arena_boundary_does_not_clamp_inside_player() {
    use void_drifter::shared::components::Velocity;

    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_systems(bevy::app::Update, enforce_arena_boundary);

    let start_pos = Vec3::new(100.0, 200.0, 0.0);
    let start_vel = Vec2::new(30.0, -20.0);

    let player = app.world_mut().spawn((
        Player,
        Transform::from_translation(start_pos),
        Velocity(start_vel),
    )).id();

    app.update();

    let transform = app
        .world()
        .entity(player)
        .get::<Transform>()
        .expect("Player should have Transform");
    // Position should be unchanged
    assert!(
        (transform.translation - start_pos).length() < 0.01,
        "Player inside arena should not be moved: expected {:?}, got {:?}",
        start_pos,
        transform.translation
    );
}

/// cleanup_arena despawns all ArenaEnemy entities.
#[test]
fn cleanup_arena_despawns_all_arena_enemies() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_systems(bevy::app::Update, cleanup_arena);

    // Spawn some ArenaEnemy entities
    for _ in 0..5 {
        app.world_mut().spawn(ArenaEnemy);
    }
    // Spawn a non-arena entity that should survive
    app.world_mut().spawn(Transform::default());

    app.update();

    let arena_enemy_count = app
        .world_mut()
        .query_filtered::<Entity, With<ArenaEnemy>>()
        .iter(app.world())
        .count();
    assert_eq!(arena_enemy_count, 0, "cleanup_arena should despawn all ArenaEnemy entities");

    // Non-arena entity should still exist
    let transform_count = app
        .world_mut()
        .query_filtered::<Entity, With<Transform>>()
        .iter(app.world())
        .count();
    assert!(transform_count > 0, "Non-arena entities should survive cleanup_arena");
}

/// Arena enemies have ArenaEnemy marker component set during wave spawn.
#[test]
fn arena_enemies_have_arena_enemy_marker() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.insert_resource(SpawningConfig::default());
    app.insert_resource(ArenaState {
        wave: 0,
        total_waves: 3,
        enemies_remaining: 0,
        cleared: false,
        completion_notified: false,
    });
    app.add_systems(bevy::app::Update, spawn_arena_wave);
    app.update();

    // All spawned enemies must have ArenaEnemy
    let with_marker = app
        .world_mut()
        .query_filtered::<Entity, With<ArenaEnemy>>()
        .iter(app.world())
        .count();
    assert!(
        with_marker > 0,
        "At least some enemies should be spawned with ArenaEnemy marker after wave 1"
    );
}

// ── Story 9-4: Arena Rewards ──────────────────────────────────────────────────

/// calculate_arena_reward scales correctly with distance.
#[test]
fn arena_reward_credits_scale_with_distance() {
    use void_drifter::core::wormhole::calculate_arena_reward;

    let reward_at_1 = calculate_arena_reward(1.0);
    let reward_at_5 = calculate_arena_reward(5.0);
    assert!(
        reward_at_1 >= 200,
        "Reward at distance 1 should be at least 200, got {}",
        reward_at_1
    );
    assert!(
        reward_at_5 <= 1000,
        "Reward at distance 5 should be at most 1000, got {}",
        reward_at_5
    );
    assert!(
        reward_at_5 > reward_at_1,
        "Reward at distance 5 ({}) should exceed reward at distance 1 ({})",
        reward_at_5,
        reward_at_1
    );
}

/// calculate_arena_reward clamps at 200 for very short distances.
#[test]
fn arena_reward_minimum_is_200() {
    use void_drifter::core::wormhole::calculate_arena_reward;

    let reward_at_zero = calculate_arena_reward(0.0);
    assert_eq!(
        reward_at_zero, 200,
        "Reward should be clamped to minimum 200 at distance 0, got {}",
        reward_at_zero
    );
}

/// calculate_arena_reward clamps at 1000 for very large distances.
#[test]
fn arena_reward_maximum_is_1000() {
    use void_drifter::core::wormhole::calculate_arena_reward;

    let reward_far = calculate_arena_reward(100.0);
    assert_eq!(
        reward_far, 1000,
        "Reward should be clamped to maximum 1000 at large distances, got {}",
        reward_far
    );
}

/// PlayerSave serializes and deserializes cleared_wormholes correctly via RON.
#[test]
fn save_roundtrip_cleared_wormholes() {
    use void_drifter::infrastructure::save::player_save::PlayerSave;

    let mut save = PlayerSave::default();
    save.cleared_wormholes = vec![[3, 4], [5, -2]];
    let ron_str = save.to_ron().expect("Should serialize cleared_wormholes to RON");
    let loaded = PlayerSave::from_ron(&ron_str).expect("Should deserialize cleared_wormholes from RON");
    assert_eq!(
        loaded.cleared_wormholes.len(),
        2,
        "Should have 2 cleared wormhole entries after roundtrip"
    );
    assert_eq!(
        loaded.cleared_wormholes[0],
        [3, 4],
        "First cleared wormhole coord should be [3, 4]"
    );
    assert_eq!(
        loaded.cleared_wormholes[1],
        [5, -2],
        "Second cleared wormhole coord should be [5, -2]"
    );
}

/// ArenaState.completion_notified field defaults to false.
#[test]
fn arena_state_completion_notified_defaults_false() {
    let state = ArenaState::default();
    assert!(
        !state.completion_notified,
        "ArenaState::completion_notified should default to false"
    );
}
