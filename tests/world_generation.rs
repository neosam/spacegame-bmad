#![deny(clippy::unwrap_used)]

mod helpers;

use bevy::prelude::*;
use helpers::{spawn_player, test_app};
use void_drifter::core::collision::{Collider, Health};
use void_drifter::core::spawning::{Asteroid, ScoutDrone};
use void_drifter::shared::components::Velocity;
use void_drifter::world::{ActiveChunks, ChunkEntity, WorldConfig};

/// Player at origin → chunks within load_radius are populated with entities (AC: #1, #2)
#[test]
fn player_at_origin_loads_chunks_with_entities() {
    let mut app = test_app();
    spawn_player(&mut app);

    // Run a frame to trigger chunk loading
    app.update();

    let config = WorldConfig::default();
    let active = app.world().resource::<ActiveChunks>();
    let expected_chunks = (2 * config.load_radius + 1).pow(2) as usize;
    assert_eq!(
        active.chunks.len(),
        expected_chunks,
        "Should have {expected_chunks} active chunks for load_radius={}",
        config.load_radius
    );

    // Should have spawned some entities
    let chunk_entity_count = app
        .world_mut()
        .query_filtered::<Entity, With<ChunkEntity>>()
        .iter(app.world())
        .count();
    assert!(
        chunk_entity_count > 0,
        "Should have spawned chunk entities, got 0"
    );
}

/// Player moves to new chunk → new chunk loads, distant chunk unloads (AC: #2, #4)
#[test]
fn player_movement_triggers_chunk_load_unload() {
    let mut app = test_app();
    let player = spawn_player(&mut app);

    // Initial load at origin
    app.update();

    let initial_chunks = app.world().resource::<ActiveChunks>().chunks.clone();

    // Move player far enough to change chunk (chunk_size = 1000, move 3000 units)
    app.world_mut()
        .entity_mut(player)
        .get_mut::<Transform>()
        .expect("Player should have Transform")
        .translation = Vec3::new(3000.0, 0.0, 0.0);

    app.update();

    let new_chunks = app.world().resource::<ActiveChunks>().chunks.clone();
    assert_ne!(
        initial_chunks, new_chunks,
        "Chunks should change when player moves to a different chunk"
    );

    // Old chunks far from new position should be unloaded
    // Player is now at chunk (3, 0), old chunks at e.g. (-2, -2) should be gone
    let chunk_coord = void_drifter::world::chunk::ChunkCoord { x: -2, y: -2 };
    assert!(
        !new_chunks.contains(&chunk_coord),
        "Distant chunk should be unloaded"
    );
}

/// Entities from unloaded chunks are actually despawned from the ECS (AC: #2)
#[test]
fn chunk_unload_despawns_entities() {
    let mut app = test_app();
    let player = spawn_player(&mut app);

    // Initial load at origin
    app.update();

    // Record entities belonging to chunk (-2, -2)
    let distant_chunk = void_drifter::world::chunk::ChunkCoord { x: -2, y: -2 };
    let has_distant_entities = {
        let mut query = app.world_mut().query::<&ChunkEntity>();
        query
            .iter(app.world())
            .any(|ce| ce.coord == distant_chunk)
    };
    assert!(
        has_distant_entities,
        "Chunk (-2,-2) should have entities at origin load_radius=2"
    );

    // Move player far away so chunk (-2,-2) falls out of range
    app.world_mut()
        .entity_mut(player)
        .get_mut::<Transform>()
        .expect("Player should have Transform")
        .translation = Vec3::new(5000.0, 5000.0, 0.0);

    app.update();

    // Verify entities with ChunkEntity { coord: (-2,-2) } are actually gone
    let remaining = {
        let mut query = app.world_mut().query::<&ChunkEntity>();
        query
            .iter(app.world())
            .filter(|ce| ce.coord == distant_chunk)
            .count()
    };
    assert_eq!(
        remaining, 0,
        "Entities from unloaded chunk (-2,-2) should be despawned, found {remaining}"
    );
}

/// Entities have correct components (AC: #3, #5)
#[test]
fn chunk_entities_have_correct_components() {
    let mut app = test_app();
    spawn_player(&mut app);
    app.update();

    // Check asteroids
    let mut asteroid_query = app
        .world_mut()
        .query_filtered::<(&Health, &Collider, &Velocity, &ChunkEntity), With<Asteroid>>();
    let asteroid_count = asteroid_query.iter(app.world()).count();
    assert!(asteroid_count > 0, "Should have spawned asteroids");

    let config = WorldConfig::default();
    for (health, collider, _velocity, _chunk) in asteroid_query.iter(app.world()) {
        assert!(
            (health.max - config.asteroid_health).abs() < f32::EPSILON,
            "Asteroid health should match config"
        );
        assert!(
            (collider.radius - config.asteroid_radius).abs() < f32::EPSILON,
            "Asteroid radius should match config"
        );
    }

    // Check drones (may or may not exist depending on RNG, just verify components if present)
    let mut drone_query = app
        .world_mut()
        .query_filtered::<(&Health, &Collider, &Velocity, &ChunkEntity), With<ScoutDrone>>();
    for (health, collider, _velocity, _chunk) in drone_query.iter(app.world()) {
        assert!(
            (health.max - config.drone_health).abs() < f32::EPSILON,
            "Drone health should match config"
        );
        assert!(
            (collider.radius - config.drone_radius).abs() < f32::EPSILON,
            "Drone radius should match config"
        );
    }
}

/// Chunk determinism: reload same chunk → same entities at same positions (AC: #3)
#[test]
fn chunk_reload_produces_same_entities() {
    let mut app = test_app();
    let player = spawn_player(&mut app);
    app.update();

    // Record initial entity positions in chunk (0,0)
    let target_chunk = void_drifter::world::chunk::ChunkCoord { x: 0, y: 0 };
    let mut initial_positions: Vec<(f32, f32)> = Vec::new();
    {
        let mut query = app.world_mut().query::<(&Transform, &ChunkEntity)>();
        for (transform, chunk_ent) in query.iter(app.world()) {
            if chunk_ent.coord == target_chunk {
                initial_positions.push((transform.translation.x, transform.translation.y));
            }
        }
    }
    initial_positions.sort_by(|a, b| a.0.partial_cmp(&b.0).expect("Should compare f32"));

    // Move player far away to unload chunk (0,0)
    app.world_mut()
        .entity_mut(player)
        .get_mut::<Transform>()
        .expect("Player should have Transform")
        .translation = Vec3::new(10000.0, 10000.0, 0.0);
    app.update();

    // Verify chunk (0,0) is unloaded
    let active = app.world().resource::<ActiveChunks>();
    assert!(
        !active.chunks.contains(&target_chunk),
        "Chunk (0,0) should be unloaded"
    );

    // Move player back to origin to reload chunk (0,0)
    app.world_mut()
        .entity_mut(player)
        .get_mut::<Transform>()
        .expect("Player should have Transform")
        .translation = Vec3::ZERO;
    app.update();

    // Record reloaded positions
    let mut reloaded_positions: Vec<(f32, f32)> = Vec::new();
    {
        let mut query = app.world_mut().query::<(&Transform, &ChunkEntity)>();
        for (transform, chunk_ent) in query.iter(app.world()) {
            if chunk_ent.coord == target_chunk {
                reloaded_positions.push((transform.translation.x, transform.translation.y));
            }
        }
    }
    reloaded_positions.sort_by(|a, b| a.0.partial_cmp(&b.0).expect("Should compare f32"));

    assert_eq!(
        initial_positions.len(),
        reloaded_positions.len(),
        "Same number of entities should be spawned on reload"
    );
    for (initial, reloaded) in initial_positions.iter().zip(reloaded_positions.iter()) {
        assert!(
            (initial.0 - reloaded.0).abs() < f32::EPSILON
                && (initial.1 - reloaded.1).abs() < f32::EPSILON,
            "Entity positions should match on reload: initial={initial:?}, reloaded={reloaded:?}"
        );
    }
}

/// Total entity count stays within budget (AC: #6)
#[test]
fn entity_count_within_budget() {
    let mut app = test_app();
    spawn_player(&mut app);
    app.update();

    let config = WorldConfig::default();
    let count = app
        .world_mut()
        .query_filtered::<Entity, With<ChunkEntity>>()
        .iter(app.world())
        .count();
    assert!(
        count <= config.entity_budget,
        "Entity count {count} exceeds budget {}",
        config.entity_budget
    );
}

/// Entity budget is enforced even when many chunks load at once (AC: #6)
#[test]
fn entity_budget_enforced_across_multi_chunk_load() {
    use std::time::Duration;
    use bevy::time::TimeUpdateStrategy;

    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    // Use a very small budget to prove enforcement works
    let mut config = WorldConfig::default();
    config.entity_budget = 20;
    config.load_radius = 2; // 5x5 = 25 chunks, each with 3-10 entities → would be 75-250 without budget
    app.insert_resource(config.clone());
    app.init_resource::<ActiveChunks>();
    app.add_systems(FixedUpdate, void_drifter::world::update_chunks);
    app.insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_secs_f64(1.0 / 60.0)));
    app.insert_resource(Time::<Fixed>::from_seconds(1.0 / 60.0));
    app.update(); // prime

    // Spawn player at origin
    app.world_mut().spawn((
        void_drifter::core::flight::Player,
        Transform::default(),
    ));

    // Load all 25 chunks in one frame
    app.update();

    let count = app
        .world_mut()
        .query_filtered::<Entity, With<ChunkEntity>>()
        .iter(app.world())
        .count();
    assert!(
        count <= 20,
        "Entity budget of 20 should be enforced, but got {count} entities"
    );
}

/// Entity budget accounts for non-chunk collidable entities like player and manual spawns (AC: #6)
#[test]
fn entity_budget_accounts_for_non_chunk_collidable_entities() {
    use std::time::Duration;
    use bevy::time::TimeUpdateStrategy;
    use void_drifter::core::collision::Collider;

    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    // Small budget to prove non-chunk entities are counted
    let mut config = WorldConfig::default();
    config.entity_budget = 20;
    config.load_radius = 2; // 25 chunks, would produce 75-250 entities without budget
    app.insert_resource(config);
    app.init_resource::<ActiveChunks>();
    app.add_systems(FixedUpdate, void_drifter::world::update_chunks);
    app.insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_secs_f64(1.0 / 60.0)));
    app.insert_resource(Time::<Fixed>::from_seconds(1.0 / 60.0));
    app.update(); // prime

    // Spawn player with Collider (counts toward budget)
    app.world_mut().spawn((
        void_drifter::core::flight::Player,
        Collider { radius: 12.0 },
        Transform::default(),
    ));

    // Spawn 10 non-chunk asteroids with Collider (all count toward budget)
    for i in 0..10 {
        app.world_mut().spawn((
            Asteroid,
            Collider { radius: 20.0 },
            Transform::from_translation(Vec3::new(i as f32 * 100.0, 0.0, 0.0)),
        ));
    }

    // 11 pre-existing collidable entities → only 9 more chunk entities allowed
    app.update();

    let chunk_count = app
        .world_mut()
        .query_filtered::<Entity, With<ChunkEntity>>()
        .iter(app.world())
        .count();

    let total_collidable = app
        .world_mut()
        .query_filtered::<Entity, With<Collider>>()
        .iter(app.world())
        .count();

    assert!(
        total_collidable <= 20,
        "Total collidable entities should not exceed budget of 20, got {total_collidable} \
         (chunk: {chunk_count}, non-chunk: {})",
        total_collidable - chunk_count
    );
    assert!(
        chunk_count < 20,
        "Chunk entity count {chunk_count} should be less than budget because non-chunk entities consume budget"
    );
}

/// Chunk system does not panic when no player exists (AC: #8 — robustness)
#[test]
fn update_chunks_without_player_does_not_panic() {
    let mut app = test_app();
    // No player spawned — update_chunks should early-return gracefully

    app.update();

    let active = app.world().resource::<ActiveChunks>();
    assert!(
        active.chunks.is_empty(),
        "No chunks should load without a player, got {} chunks",
        active.chunks.len()
    );

    let chunk_entity_count = app
        .world_mut()
        .query_filtered::<Entity, With<ChunkEntity>>()
        .iter(app.world())
        .count();
    assert_eq!(
        chunk_entity_count, 0,
        "No chunk entities should exist without a player"
    );
}

/// Existing weapon/collision systems work with chunk-spawned entities (AC: #5)
#[test]
fn laser_hits_chunk_spawned_asteroid() {
    use void_drifter::core::input::ActionState;

    let mut app = test_app();
    let player = spawn_player(&mut app);

    // Run to get chunks loaded
    app.update();

    // Find first chunk-spawned asteroid and record its entity + health
    let (asteroid_entity, initial_health, asteroid_pos) = {
        let mut query = app.world_mut().query_filtered::<(
            Entity,
            &Health,
            &Transform,
        ), (With<Asteroid>, With<ChunkEntity>)>();
        let (entity, health, transform) = query
            .iter(app.world())
            .next()
            .expect("Should have a chunk-spawned asteroid");
        (entity, health.current, Vec2::new(transform.translation.x, transform.translation.y))
    };

    // Position player close to asteroid, facing it
    let direction = (asteroid_pos - Vec2::ZERO).normalize_or_zero();
    let player_pos = asteroid_pos - direction * 30.0;
    let angle = direction.y.atan2(direction.x) - std::f32::consts::FRAC_PI_2;
    {
        let mut entity_mut = app.world_mut().entity_mut(player);
        let mut transform = entity_mut
            .get_mut::<Transform>()
            .expect("Player should have Transform");
        transform.translation = player_pos.extend(0.0);
        transform.rotation = Quat::from_rotation_z(angle);
    }

    // Fire laser at the asteroid
    app.world_mut().resource_mut::<ActionState>().fire = true;
    app.update();
    app.world_mut().resource_mut::<ActionState>().fire = false;

    // Run several frames for laser travel and damage pipeline
    for _ in 0..5 {
        app.update();
    }

    // Verify: asteroid took damage OR was destroyed
    match app.world().get_entity(asteroid_entity) {
        Ok(entity_ref) => {
            let health = entity_ref
                .get::<Health>()
                .expect("Asteroid should have Health");
            assert!(
                health.current < initial_health,
                "Chunk-spawned asteroid should take laser damage: health {} should be < {}",
                health.current,
                initial_health
            );
        }
        Err(_) => {
            // Entity was destroyed — also valid, means damage was applied
        }
    }
}
