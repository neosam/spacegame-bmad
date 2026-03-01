#![deny(clippy::unwrap_used)]
//! Integration tests for chunk loading optimization (Story 1.5).

mod helpers;

use bevy::prelude::*;
use helpers::{run_until_loaded, spawn_player, test_app};
use void_drifter::infrastructure::events::EventSeverityConfig;
use void_drifter::infrastructure::logbook::Logbook;
use void_drifter::shared::events::GameEvent;
use void_drifter::infrastructure::save::delta::WorldDeltas;
use void_drifter::world::{
    ActiveChunks, BiomeConfig, ChunkEntity, ChunkEntityIndex, ChunkLoadState, ExploredChunks,
    PendingChunks, WorldConfig,
};
use void_drifter::world::chunk::ChunkCoord;

/// After update_chunks, ChunkEntityIndex contains entries for all active chunks
/// with correct entity counts (AC: #1).
#[test]
fn chunk_entity_index_populated_on_load() {
    let mut app = test_app();
    spawn_player(&mut app);

    // Run enough frames for all chunks to load
    run_until_loaded(&mut app);

    // Capture values before mutable borrow
    let active_coords: Vec<ChunkCoord> = app
        .world()
        .resource::<ActiveChunks>()
        .chunks
        .keys()
        .copied()
        .collect();
    let index_entity_count = app.world().resource::<ChunkEntityIndex>().entity_count();
    let index_has_coord = |coord: &ChunkCoord| -> bool {
        app.world()
            .resource::<ChunkEntityIndex>()
            .chunks
            .contains_key(coord)
    };

    // Every active chunk should have an entry in the index
    for coord in &active_coords {
        assert!(
            index_has_coord(coord),
            "ChunkEntityIndex should have entry for active chunk ({}, {})",
            coord.x,
            coord.y,
        );
    }

    // Entity count in index should match actual ChunkEntity query count
    let actual_chunk_entity_count = app
        .world_mut()
        .query_filtered::<Entity, With<ChunkEntity>>()
        .iter(app.world())
        .count();
    assert_eq!(
        index_entity_count, actual_chunk_entity_count,
        "ChunkEntityIndex entity_count should match actual ChunkEntity query count"
    );
}

/// After moving player and unloading, index entries for old chunks are gone,
/// no leaked entities (AC: #2).
#[test]
fn chunk_entity_index_cleared_on_unload() {
    let mut app = test_app();
    let player = spawn_player(&mut app);

    // Fully load initial chunks
    run_until_loaded(&mut app);

    // Record initial index keys
    let initial_coords: Vec<ChunkCoord> = app
        .world()
        .resource::<ChunkEntityIndex>()
        .chunks
        .keys()
        .copied()
        .collect();
    assert!(
        !initial_coords.is_empty(),
        "Should have index entries after initial load"
    );

    // Move player far away so all initial chunks are unloaded
    app.world_mut()
        .entity_mut(player)
        .get_mut::<Transform>()
        .expect("Player should have Transform")
        .translation = Vec3::new(20000.0, 20000.0, 0.0);

    // Unloading is immediate (one frame)
    app.update();

    let index = app.world().resource::<ChunkEntityIndex>();
    for coord in &initial_coords {
        assert!(
            !index.chunks.contains_key(coord),
            "ChunkEntityIndex should not have entry for unloaded chunk ({}, {})",
            coord.x,
            coord.y,
        );
    }

    // Verify no leaked ChunkEntity components for unloaded chunks
    let mut query = app.world_mut().query::<&ChunkEntity>();
    for chunk_ent in query.iter(app.world()) {
        assert!(
            !initial_coords.contains(&chunk_ent.coord),
            "Should not find leaked entity for unloaded chunk ({}, {})",
            chunk_ent.coord.x,
            chunk_ent.coord.y,
        );
    }
}

/// With max_chunks_per_frame: 2 and 25 desired chunks, only 2 load per frame.
/// After 13 frames all 25 loaded (AC: #3).
#[test]
fn staggered_loading_respects_max_per_frame() {
    use std::time::Duration;
    use bevy::time::TimeUpdateStrategy;

    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    let config = WorldConfig {
        max_chunks_per_frame: 2,
        load_radius: 2,
        ..Default::default()
    };
    app.insert_resource(config);
    app.insert_resource(BiomeConfig::default());
    app.init_resource::<ActiveChunks>();
    app.init_resource::<ExploredChunks>();
    app.init_resource::<ChunkEntityIndex>();
    app.init_resource::<PendingChunks>();
    app.init_resource::<ChunkLoadState>();
    app.add_message::<GameEvent>();
    app.insert_resource(EventSeverityConfig::default());
    app.init_resource::<Logbook>();
    app.init_resource::<WorldDeltas>();
    app.add_systems(FixedUpdate, void_drifter::world::update_chunks);
    app.insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_secs_f64(
        1.0 / 60.0,
    )));
    app.insert_resource(Time::<Fixed>::from_seconds(1.0 / 60.0));
    app.update(); // prime

    app.world_mut().spawn((
        void_drifter::core::flight::Player,
        Transform::default(),
    ));

    // After first frame: exactly 2 chunks should be loaded
    app.update();
    let after_first = app.world().resource::<ActiveChunks>().chunks.len();
    assert_eq!(
        after_first, 2,
        "With max_chunks_per_frame=2, exactly 2 chunks should load in first frame, got {after_first}"
    );

    // Run 12 more frames (total 13) to load all 25 chunks (ceil(25/2) = 13)
    for _ in 0..12 {
        app.update();
    }

    let after_all = app.world().resource::<ActiveChunks>().chunks.len();
    assert_eq!(
        after_all, 25,
        "After 13 frames with max_chunks_per_frame=2, all 25 chunks should be loaded, got {after_all}"
    );
}

/// With max_chunks_per_frame: 1, first loaded chunk is nearest to player (AC: #4).
#[test]
fn load_priority_nearest_first() {
    use std::time::Duration;
    use bevy::time::TimeUpdateStrategy;
    use void_drifter::world::chunk::manhattan_distance;

    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    let config = WorldConfig {
        max_chunks_per_frame: 1,
        load_radius: 2,
        ..Default::default()
    };
    app.insert_resource(config);
    app.insert_resource(BiomeConfig::default());
    app.init_resource::<ActiveChunks>();
    app.init_resource::<ExploredChunks>();
    app.init_resource::<ChunkEntityIndex>();
    app.init_resource::<PendingChunks>();
    app.init_resource::<ChunkLoadState>();
    app.add_message::<GameEvent>();
    app.insert_resource(EventSeverityConfig::default());
    app.init_resource::<Logbook>();
    app.init_resource::<WorldDeltas>();
    app.add_systems(FixedUpdate, void_drifter::world::update_chunks);
    app.insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_secs_f64(
        1.0 / 60.0,
    )));
    app.insert_resource(Time::<Fixed>::from_seconds(1.0 / 60.0));
    app.update(); // prime

    let player_chunk = ChunkCoord { x: 0, y: 0 };
    app.world_mut().spawn((
        void_drifter::core::flight::Player,
        Transform::default(),
    ));

    // After first frame, only 1 chunk loads — it should be the player's own chunk (distance 0)
    app.update();
    let active = app.world().resource::<ActiveChunks>();
    assert_eq!(active.chunks.len(), 1, "Exactly 1 chunk should be loaded");
    let first_loaded = *active.chunks.keys().next().expect("Should have one chunk");
    assert_eq!(
        manhattan_distance(first_loaded, player_chunk),
        0,
        "First loaded chunk should be the player's chunk (distance 0), got ({}, {})",
        first_loaded.x,
        first_loaded.y,
    );

    // Load second chunk — should be distance 1
    app.update();
    let active = app.world().resource::<ActiveChunks>();
    assert_eq!(active.chunks.len(), 2, "Exactly 2 chunks should be loaded");
    let second_loaded = active
        .chunks
        .keys()
        .find(|c| **c != first_loaded)
        .expect("Should have a second chunk");
    assert_eq!(
        manhattan_distance(*second_loaded, player_chunk),
        1,
        "Second loaded chunk should be at distance 1, got distance {}",
        manhattan_distance(*second_loaded, player_chunk),
    );
}

/// Simulate 100 chunk transitions. Entity count never exceeds budget.
/// No ChunkEntityIndex leak (AC: #7).
#[test]
fn extended_exploration_memory_bounded() {
    let mut app = test_app();
    let player = spawn_player(&mut app);
    let config = app.world().resource::<WorldConfig>().clone();

    // Fully load initial chunks
    run_until_loaded(&mut app);
    let total = (2 * config.load_radius + 1).pow(2) as usize;
    let load_frames = total.div_ceil(config.max_chunks_per_frame);

    // Simulate 100 chunk transitions: move player 1 chunk_size per step
    for step in 0..100 {
        let x = (step as f32 + 1.0) * config.chunk_size + config.chunk_size * 0.5;
        app.world_mut()
            .entity_mut(player)
            .get_mut::<Transform>()
            .expect("Player should have Transform")
            .translation = Vec3::new(x, 0.0, 0.0);

        // Run enough frames for staggered loading to complete
        for _ in 0..load_frames {
            app.update();
        }

        // Check entity budget
        let entity_count = app
            .world_mut()
            .query_filtered::<Entity, With<ChunkEntity>>()
            .iter(app.world())
            .count();
        assert!(
            entity_count <= config.entity_budget,
            "Entity count {entity_count} exceeds budget {} at step {step}",
            config.entity_budget,
        );

        // Check index consistency: index count should match actual entity count
        let index = app.world().resource::<ChunkEntityIndex>();
        let active = app.world().resource::<ActiveChunks>();
        assert_eq!(
            index.chunks.len(),
            active.chunks.len(),
            "ChunkEntityIndex chunk count should match ActiveChunks at step {step}"
        );
    }
}

/// Move player to new chunk while pending queue has entries.
/// Assert pending queue is recalculated with new distances (AC: #5).
#[test]
fn pending_queue_refreshed_on_chunk_change() {
    use std::time::Duration;
    use bevy::time::TimeUpdateStrategy;

    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    let config = WorldConfig {
        max_chunks_per_frame: 2,
        load_radius: 2,
        ..Default::default()
    };
    app.insert_resource(config.clone());
    app.insert_resource(BiomeConfig::default());
    app.init_resource::<ActiveChunks>();
    app.init_resource::<ExploredChunks>();
    app.init_resource::<ChunkEntityIndex>();
    app.init_resource::<PendingChunks>();
    app.init_resource::<ChunkLoadState>();
    app.add_message::<GameEvent>();
    app.insert_resource(EventSeverityConfig::default());
    app.init_resource::<Logbook>();
    app.init_resource::<WorldDeltas>();
    app.add_systems(FixedUpdate, void_drifter::world::update_chunks);
    app.insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_secs_f64(
        1.0 / 60.0,
    )));
    app.insert_resource(Time::<Fixed>::from_seconds(1.0 / 60.0));
    app.update(); // prime

    let player = app
        .world_mut()
        .spawn((
            void_drifter::core::flight::Player,
            Transform::default(),
        ))
        .id();

    // First frame: loads 2 of 25 chunks, 23 remain pending
    app.update();
    let pending_after_first = app.world().resource::<PendingChunks>().chunks.len();
    assert!(
        pending_after_first > 0,
        "Should have pending chunks after first frame"
    );

    // Move player to a different chunk while pending queue still has entries
    app.world_mut()
        .entity_mut(player)
        .get_mut::<Transform>()
        .expect("Player should have Transform")
        .translation = Vec3::new(config.chunk_size * 5.0 + 500.0, 0.0, 0.0);

    app.update();

    // The pending queue should have been recalculated for the new player position
    let new_pending: Vec<ChunkCoord> = app
        .world()
        .resource::<PendingChunks>()
        .chunks
        .iter()
        .copied()
        .collect();

    let new_player_chunk = void_drifter::world::chunk::world_to_chunk(
        bevy::math::Vec2::new(config.chunk_size * 5.0 + 500.0, 0.0),
        config.chunk_size,
    );

    // All pending chunks should be within load_radius of the NEW player position
    let desired = void_drifter::world::chunk::chunks_in_radius(new_player_chunk, config.load_radius);
    for coord in &new_pending {
        assert!(
            desired.contains(coord),
            "Pending chunk ({}, {}) should be within load_radius of new player chunk ({}, {})",
            coord.x,
            coord.y,
            new_player_chunk.x,
            new_player_chunk.y,
        );
    }

    // Pending chunks should be sorted by distance to new position
    for window in new_pending.windows(2) {
        let dist_a = void_drifter::world::chunk::manhattan_distance(window[0], new_player_chunk);
        let dist_b = void_drifter::world::chunk::manhattan_distance(window[1], new_player_chunk);
        assert!(
            dist_a <= dist_b,
            "Pending queue should be sorted by distance: ({},{}) dist={} before ({},{}) dist={}",
            window[0].x,
            window[0].y,
            dist_a,
            window[1].x,
            window[1].y,
            dist_b,
        );
    }
}
