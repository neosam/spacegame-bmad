#![deny(clippy::unwrap_used)]

mod helpers;

use bevy::prelude::*;
use helpers::{run_until_loaded, spawn_player, test_app};
use void_drifter::core::collision::{Collider, Health};
use void_drifter::core::spawning::Asteroid;
use void_drifter::shared::components::Velocity;
use void_drifter::world::{
    ActiveChunks, BiomeConfig, BiomeType, ChunkEntity, ChunkEntityIndex, ChunkLoadState,
    ExploredChunks, PendingChunks, WorldConfig,
};

/// Player at origin -> chunks within load_radius are populated with entities (AC: #1, #2)
#[test]
fn player_at_origin_loads_chunks_with_entities() {
    let mut app = test_app();
    spawn_player(&mut app);

    // Run enough frames for staggered loading to complete
    run_until_loaded(&mut app);

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

/// Player moves to new chunk -> new chunk loads, distant chunk unloads (AC: #2, #4)
#[test]
fn player_movement_triggers_chunk_load_unload() {
    let mut app = test_app();
    let player = spawn_player(&mut app);

    // Initial load at origin
    run_until_loaded(&mut app);

    let initial_chunks: std::collections::HashSet<_> =
        app.world().resource::<ActiveChunks>().chunks.keys().copied().collect();

    // Move player far enough to change chunk (chunk_size = 1000, move 3000 units)
    app.world_mut()
        .entity_mut(player)
        .get_mut::<Transform>()
        .expect("Player should have Transform")
        .translation = Vec3::new(3000.0, 0.0, 0.0);

    run_until_loaded(&mut app);

    let new_chunks: std::collections::HashSet<_> =
        app.world().resource::<ActiveChunks>().chunks.keys().copied().collect();
    assert_ne!(
        initial_chunks, new_chunks,
        "Chunks should change when player moves to a different chunk"
    );

    // Old chunks far from new position should be unloaded
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
    run_until_loaded(&mut app);

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

    // Unloading is immediate (one frame), loading is staggered
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

/// Entities have correct components including BiomeType (AC: #3, #4, #5)
#[test]
fn chunk_entities_have_correct_components() {
    let mut app = test_app();
    spawn_player(&mut app);
    run_until_loaded(&mut app);

    let biome_config = BiomeConfig::default();

    // Check that all chunk entities have BiomeType component
    let mut biome_query = app
        .world_mut()
        .query_filtered::<(&Health, &Collider, &Velocity, &ChunkEntity, &BiomeType), With<Asteroid>>();
    let asteroid_count = biome_query.iter(app.world()).count();
    assert!(asteroid_count > 0, "Should have spawned asteroids");

    for (health, collider, _velocity, _chunk, biome) in biome_query.iter(app.world()) {
        let params = biome_config.params_for(*biome);
        assert!(
            (health.max - params.asteroid_health).abs() < f32::EPSILON,
            "Asteroid health {} should match biome {:?} config {}",
            health.max, biome, params.asteroid_health
        );
        assert!(
            (collider.radius - params.asteroid_radius).abs() < f32::EPSILON,
            "Asteroid radius {} should match biome {:?} config {}",
            collider.radius, biome, params.asteroid_radius
        );
    }
}

/// Chunk determinism: reload same chunk -> same entities at same positions (AC: #3, #8)
#[test]
fn chunk_reload_produces_same_entities() {
    let mut app = test_app();
    let player = spawn_player(&mut app);
    run_until_loaded(&mut app);

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
    run_until_loaded(&mut app);

    // Verify chunk (0,0) is unloaded
    let active = app.world().resource::<ActiveChunks>();
    assert!(
        !active.chunks.contains_key(&target_chunk),
        "Chunk (0,0) should be unloaded"
    );

    // Move player back to origin to reload chunk (0,0)
    app.world_mut()
        .entity_mut(player)
        .get_mut::<Transform>()
        .expect("Player should have Transform")
        .translation = Vec3::ZERO;
    run_until_loaded(&mut app);

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

/// Total entity count stays within budget (AC: #7)
#[test]
fn entity_count_within_budget() {
    let mut app = test_app();
    spawn_player(&mut app);
    run_until_loaded(&mut app);

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

/// Entity budget is enforced even when many chunks load at once (AC: #7)
#[test]
fn entity_budget_enforced_across_multi_chunk_load() {
    use std::time::Duration;
    use bevy::time::TimeUpdateStrategy;

    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    // Use a very small budget to prove enforcement works
    let config = WorldConfig { entity_budget: 20, load_radius: 2, ..Default::default() };
    app.insert_resource(config);
    app.insert_resource(BiomeConfig::default());
    app.init_resource::<ActiveChunks>();
    app.init_resource::<ExploredChunks>();
    app.init_resource::<ChunkEntityIndex>();
    app.init_resource::<PendingChunks>();
    app.init_resource::<ChunkLoadState>();
    app.add_systems(FixedUpdate, void_drifter::world::update_chunks);
    app.insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_secs_f64(1.0 / 60.0)));
    app.insert_resource(Time::<Fixed>::from_seconds(1.0 / 60.0));
    app.update(); // prime

    // Spawn player at origin
    app.world_mut().spawn((
        void_drifter::core::flight::Player,
        Transform::default(),
    ));

    // Run enough frames for staggered loading to complete (25 chunks / 4 per frame = 7 frames)
    for _ in 0..7 {
        app.update();
    }

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

/// Entity budget accounts for non-chunk collidable entities (AC: #7)
#[test]
fn entity_budget_accounts_for_non_chunk_collidable_entities() {
    use std::time::Duration;
    use bevy::time::TimeUpdateStrategy;
    use void_drifter::core::collision::Collider;

    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    let config = WorldConfig { entity_budget: 20, load_radius: 2, ..Default::default() };
    app.insert_resource(config);
    app.insert_resource(BiomeConfig::default());
    app.init_resource::<ActiveChunks>();
    app.init_resource::<ExploredChunks>();
    app.init_resource::<ChunkEntityIndex>();
    app.init_resource::<PendingChunks>();
    app.init_resource::<ChunkLoadState>();
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

    // 11 pre-existing collidable entities -> only 9 more chunk entities allowed
    // Run enough frames for staggered loading to complete
    for _ in 0..7 {
        app.update();
    }

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

/// Chunk system does not panic when no player exists (robustness)
#[test]
fn update_chunks_without_player_does_not_panic() {
    let mut app = test_app();

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
    run_until_loaded(&mut app);

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
            // Entity was destroyed -- also valid, means damage was applied
        }
    }
}

// ── Biome-specific integration tests (Story 1.2) ────────────────────────

/// Chunk spawned entities have BiomeType component matching chunk's biome (AC: #4, #5)
#[test]
fn chunk_entities_have_biome_component() {
    let mut app = test_app();
    spawn_player(&mut app);
    run_until_loaded(&mut app);

    // All chunk entities should have a BiomeType component
    let mut query = app
        .world_mut()
        .query_filtered::<(&ChunkEntity, &BiomeType), With<Collider>>();
    let count = query.iter(app.world()).count();
    assert!(
        count > 0,
        "Should have chunk entities with BiomeType component"
    );

    // The BiomeType on each entity should match the chunk's biome in ActiveChunks
    let active = app.world().resource::<ActiveChunks>();
    for (chunk_ent, biome) in query.iter(app.world()) {
        let expected_biome = active
            .chunks
            .get(&chunk_ent.coord)
            .expect("Chunk should be tracked in ActiveChunks");
        assert_eq!(
            biome, expected_biome,
            "Entity biome {:?} should match chunk ({},{}) biome {:?}",
            biome, chunk_ent.coord.x, chunk_ent.coord.y, expected_biome
        );
    }
}

/// Chunk reload produces same biome type (AC: #2, #8)
#[test]
fn chunk_reload_produces_same_biome() {
    let mut app = test_app();
    let player = spawn_player(&mut app);
    run_until_loaded(&mut app);

    let target_chunk = void_drifter::world::chunk::ChunkCoord { x: 0, y: 0 };

    // Record initial biome for chunk (0,0)
    let initial_biome = *app
        .world()
        .resource::<ActiveChunks>()
        .chunks
        .get(&target_chunk)
        .expect("Chunk (0,0) should be loaded");

    // Move player far away to unload
    app.world_mut()
        .entity_mut(player)
        .get_mut::<Transform>()
        .expect("Player should have Transform")
        .translation = Vec3::new(10000.0, 10000.0, 0.0);
    run_until_loaded(&mut app);

    // Move back to reload
    app.world_mut()
        .entity_mut(player)
        .get_mut::<Transform>()
        .expect("Player should have Transform")
        .translation = Vec3::ZERO;
    run_until_loaded(&mut app);

    let reloaded_biome = *app
        .world()
        .resource::<ActiveChunks>()
        .chunks
        .get(&target_chunk)
        .expect("Chunk (0,0) should be reloaded");

    assert_eq!(
        initial_biome, reloaded_biome,
        "Chunk (0,0) biome should be the same after reload"
    );
}

/// Multiple biome types appear across loaded chunks (AC: #6)
#[test]
fn multiple_biomes_appear_across_chunks() {
    use std::time::Duration;
    use bevy::time::TimeUpdateStrategy;

    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    // Use large load_radius to get many chunks (11x11 = 121 chunks)
    let config = WorldConfig { load_radius: 5, ..Default::default() };
    app.insert_resource(config);
    app.insert_resource(BiomeConfig::default());
    app.init_resource::<ActiveChunks>();
    app.init_resource::<ExploredChunks>();
    app.init_resource::<ChunkEntityIndex>();
    app.init_resource::<PendingChunks>();
    app.init_resource::<ChunkLoadState>();
    app.add_systems(FixedUpdate, void_drifter::world::update_chunks);
    app.insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_secs_f64(1.0 / 60.0)));
    app.insert_resource(Time::<Fixed>::from_seconds(1.0 / 60.0));
    app.update(); // prime

    app.world_mut().spawn((
        void_drifter::core::flight::Player,
        Transform::default(),
    ));

    // 121 chunks / 4 per frame = 31 frames
    let config = app.world().resource::<WorldConfig>().clone();
    let total = (2 * config.load_radius + 1).pow(2) as usize;
    let frames = total.div_ceil(config.max_chunks_per_frame);
    for _ in 0..frames {
        app.update();
    }

    let active = app.world().resource::<ActiveChunks>();
    let biome_types: std::collections::HashSet<BiomeType> =
        active.chunks.values().copied().collect();
    let total = active.chunks.len() as f32;

    assert!(
        biome_types.len() >= 3,
        "Should have all 3 biome types across 121 chunks, got {biome_types:?}"
    );

    // AC6: No single biome dominates >60%
    let mut counts = std::collections::HashMap::<BiomeType, usize>::new();
    for biome in active.chunks.values() {
        *counts.entry(*biome).or_insert(0) += 1;
    }
    for (biome, count) in &counts {
        let pct = (*count as f32 / total) * 100.0;
        assert!(
            pct <= 60.0,
            "Biome {biome:?} dominates {pct:.1}% ({count}/{}), exceeds 60% limit",
            active.chunks.len()
        );
    }
}

/// Entity budget still enforced with high-density biome chunks (AC: #7)
#[test]
fn entity_budget_with_high_density_biome() {
    use std::time::Duration;
    use bevy::time::TimeUpdateStrategy;

    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    let config = WorldConfig { entity_budget: 30, load_radius: 2, ..Default::default() };
    app.insert_resource(config);
    // Force all chunks to be AsteroidField (high density) by setting threshold to 0.0
    let biome_config = BiomeConfig { deep_space_threshold: 0.0, asteroid_field_threshold: 1.0, ..Default::default() };
    app.insert_resource(biome_config);
    app.init_resource::<ActiveChunks>();
    app.init_resource::<ExploredChunks>();
    app.init_resource::<ChunkEntityIndex>();
    app.init_resource::<PendingChunks>();
    app.init_resource::<ChunkLoadState>();
    app.add_systems(FixedUpdate, void_drifter::world::update_chunks);
    app.insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_secs_f64(1.0 / 60.0)));
    app.insert_resource(Time::<Fixed>::from_seconds(1.0 / 60.0));
    app.update(); // prime

    app.world_mut().spawn((
        void_drifter::core::flight::Player,
        Transform::default(),
    ));

    // 25 chunks / 4 per frame = 7 frames
    for _ in 0..7 {
        app.update();
    }

    let count = app
        .world_mut()
        .query_filtered::<Entity, With<ChunkEntity>>()
        .iter(app.world())
        .count();
    assert!(
        count <= 30,
        "Entity budget of 30 should be enforced with all AsteroidField biomes, but got {count} entities"
    );
}

/// Gameplay still works with biome-tagged asteroid (AC: #4)
#[test]
fn laser_hits_biome_tagged_asteroid() {
    use void_drifter::core::input::ActionState;

    let mut app = test_app();
    let player = spawn_player(&mut app);
    run_until_loaded(&mut app);

    // Find first chunk-spawned asteroid WITH BiomeType and record it
    let (asteroid_entity, initial_health, asteroid_pos) = {
        let mut query = app.world_mut().query_filtered::<(
            Entity,
            &Health,
            &Transform,
            &BiomeType,
        ), (With<Asteroid>, With<ChunkEntity>)>();
        let (entity, health, transform, _biome) = query
            .iter(app.world())
            .next()
            .expect("Should have a biome-tagged asteroid");
        (entity, health.current, Vec2::new(transform.translation.x, transform.translation.y))
    };

    // Position player close and fire
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

    app.world_mut().resource_mut::<ActionState>().fire = true;
    app.update();
    app.world_mut().resource_mut::<ActionState>().fire = false;

    for _ in 0..5 {
        app.update();
    }

    match app.world().get_entity(asteroid_entity) {
        Ok(entity_ref) => {
            let health = entity_ref
                .get::<Health>()
                .expect("Asteroid should have Health");
            assert!(
                health.current < initial_health,
                "Biome-tagged asteroid should take damage: {} should be < {}",
                health.current,
                initial_health
            );
        }
        Err(_) => {
            // Destroyed = valid
        }
    }
}
