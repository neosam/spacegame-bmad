#![deny(clippy::unwrap_used)]
//! Integration tests for world map overlay (Story 1.4).

mod helpers;

use bevy::prelude::*;
use bevy::input::ButtonInput;
use helpers::{run_until_loaded, spawn_player, test_app};
use void_drifter::rendering::world_map::{
    toggle_world_map, update_world_map, WorldMapOpen, WorldMapPlayerMarker,
    WorldMapRoot, WorldMapState, WorldMapTile,
};
use void_drifter::world::ExploredChunks;

/// Create a test app with toggle and update world map systems in Update schedule.
fn world_map_test_app() -> App {
    let mut app = test_app();
    app.add_systems(Update, (toggle_world_map, update_world_map).chain());
    app
}

/// Simulate pressing the M key for one frame.
/// Must clear() after update because MinimalPlugins has no InputPlugin to reset just_pressed.
fn press_m_key(app: &mut App) {
    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .press(KeyCode::KeyM);
    app.update();
    let mut input = app.world_mut().resource_mut::<ButtonInput<KeyCode>>();
    input.release(KeyCode::KeyM);
    input.clear();
}

#[test]
fn world_map_overlay_spawns_on_m_key_press() {
    let mut app = world_map_test_app();
    let _player = spawn_player(&mut app);
    // Run a frame so player exists and chunks load
    app.update();

    // Verify no WorldMapRoot exists initially
    let roots_before = app
        .world_mut()
        .query_filtered::<Entity, With<WorldMapRoot>>()
        .iter(app.world())
        .count();
    assert_eq!(roots_before, 0, "No WorldMapRoot should exist before toggle");

    // Press M to open map
    press_m_key(&mut app);

    // Verify WorldMapRoot entity spawned
    let roots_after = app
        .world_mut()
        .query_filtered::<Entity, With<WorldMapRoot>>()
        .iter(app.world())
        .count();
    assert_eq!(roots_after, 1, "WorldMapRoot should exist after pressing M");

    let map_open = app.world().resource::<WorldMapOpen>();
    assert!(map_open.0, "WorldMapOpen should be true");
}

#[test]
fn world_map_overlay_despawns_on_second_m_key_press() {
    let mut app = world_map_test_app();
    let _player = spawn_player(&mut app);
    app.update();

    // Open the map
    press_m_key(&mut app);

    let roots = app
        .world_mut()
        .query_filtered::<Entity, With<WorldMapRoot>>()
        .iter(app.world())
        .count();
    assert_eq!(roots, 1, "Map should be open");

    // Close the map
    press_m_key(&mut app);

    let roots_after = app
        .world_mut()
        .query_filtered::<Entity, With<WorldMapRoot>>()
        .iter(app.world())
        .count();
    assert_eq!(roots_after, 0, "WorldMapRoot should be despawned after second M press");

    let map_open = app.world().resource::<WorldMapOpen>();
    assert!(!map_open.0, "WorldMapOpen should be false");

    let map_state = app.world().resource::<WorldMapState>();
    assert!(
        map_state.rendered_chunks.is_empty(),
        "WorldMapState should be cleared on close"
    );
    assert!(
        map_state.map_container.is_none(),
        "Map container should be None after close"
    );
}

#[test]
fn explored_chunks_populated_after_chunk_loading() {
    let mut app = world_map_test_app();
    let _player = spawn_player(&mut app);

    // Run a few frames so update_chunks loads chunks around origin
    for _ in 0..3 {
        app.update();
    }

    let explored = app.world().resource::<ExploredChunks>();
    assert!(
        !explored.chunks.is_empty(),
        "ExploredChunks should have entries after player spawns at origin and chunks load"
    );

    // With default load_radius=2, we expect (2*2+1)^2 = 25 chunks
    assert!(
        explored.chunks.len() >= 9,
        "At least 9 chunks should be explored (3x3 minimum), got {}",
        explored.chunks.len()
    );
}

#[test]
fn tile_count_matches_visible_explored_chunks_when_map_opened() {
    let mut app = world_map_test_app();
    let _player = spawn_player(&mut app);

    // Run enough frames for all chunks to load (staggered loading)
    run_until_loaded(&mut app);

    let explored_count = app.world().resource::<ExploredChunks>().chunks.len();
    assert!(explored_count > 0, "Should have explored chunks");

    // Open the map
    press_m_key(&mut app);

    // Count WorldMapTile entities
    let tile_count = app
        .world_mut()
        .query_filtered::<Entity, With<WorldMapTile>>()
        .iter(app.world())
        .count();

    let state = app.world().resource::<WorldMapState>();
    assert_eq!(
        tile_count,
        state.rendered_chunks.len(),
        "Tile entity count should match rendered_chunks count"
    );

    // All tiles should be <= explored count (some may be culled for visibility)
    assert!(
        tile_count <= explored_count,
        "Tile count ({tile_count}) should be <= explored count ({explored_count})"
    );

    // With default config (800x600 map, 12px tiles) and player at origin,
    // all 25 explored chunks should fit
    assert!(
        tile_count > 0,
        "At least some tiles should be spawned"
    );
}

#[test]
fn player_marker_exists_when_map_is_open() {
    let mut app = world_map_test_app();
    let _player = spawn_player(&mut app);
    app.update();

    // Open map
    press_m_key(&mut app);

    let markers = app
        .world_mut()
        .query_filtered::<Entity, With<WorldMapPlayerMarker>>()
        .iter(app.world())
        .count();
    assert_eq!(markers, 1, "Exactly one WorldMapPlayerMarker should exist when map is open");

    // Close map
    press_m_key(&mut app);

    // Extra updates for deferred despawn propagation
    app.update();
    app.update();

    let markers_after = app
        .world_mut()
        .query_filtered::<Entity, With<WorldMapPlayerMarker>>()
        .iter(app.world())
        .count();
    assert_eq!(markers_after, 0, "No player marker should exist after map is closed");
}

#[test]
fn world_map_tiles_reposition_when_player_moves() {
    let mut app = world_map_test_app();
    let player = spawn_player(&mut app);

    // Let chunks load at origin
    for _ in 0..3 {
        app.update();
    }

    // Open the map
    press_m_key(&mut app);

    let initial_center = app.world().resource::<WorldMapState>().center_chunk;
    let initial_rendered = app.world().resource::<WorldMapState>().rendered_chunks.len();
    assert!(initial_rendered > 0, "Should have rendered tiles after opening map");

    // Move player to next chunk (chunk_size=1000, move 1500 units → chunk 1)
    app.world_mut()
        .entity_mut(player)
        .get_mut::<Transform>()
        .expect("Player should have Transform")
        .translation = Vec3::new(1500.0, 0.0, 0.0);

    // Run updates: FixedUpdate loads new chunks, Update repositions tiles
    app.update();

    // Check if map is still open
    assert!(
        app.world().resource::<WorldMapOpen>().0,
        "Map should still be open after player moves"
    );

    // Check center_chunk updated
    let new_center = app.world().resource::<WorldMapState>().center_chunk;
    assert_ne!(
        new_center, initial_center,
        "Center chunk should have changed after player moved"
    );

    // rendered_chunks should still have entries
    let rendered_after = app.world().resource::<WorldMapState>().rendered_chunks.len();
    assert!(
        rendered_after > 0,
        "Should still have rendered chunks after repositioning"
    );
}
