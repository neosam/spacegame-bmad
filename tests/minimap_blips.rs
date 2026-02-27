//! Integration tests for minimap blip rendering (Story 1.3).

mod helpers;

use bevy::prelude::*;
use bevy::ui::BackgroundColor;
use helpers::{spawn_asteroid, spawn_drone, spawn_player, test_app};
use void_drifter::rendering::minimap::{
    blip_color, update_minimap_blips, BlipType, MinimapBlip, MinimapConfig, MinimapRoot,
    MinimapState,
};

/// Create a test app with the minimap update system added to Update schedule.
/// Also spawns a MinimapRoot entity that the system requires.
fn minimap_test_app() -> App {
    let mut app = test_app();
    app.add_systems(Update, update_minimap_blips);
    // Spawn a MinimapRoot node for the system to attach blips to
    app.world_mut().spawn((MinimapRoot, Node::default()));
    app
}

#[test]
fn minimap_blips_appear_for_nearby_asteroids() {
    let mut app = minimap_test_app();
    let _player = spawn_player(&mut app);
    // Spawn asteroid within default scanner range (2000.0)
    let asteroid = spawn_asteroid(&mut app, Vec2::new(500.0, 300.0), 20.0, 50.0);

    // Run two frames: first creates blip, second ensures it persists
    app.update();
    app.update();

    let state = app.world().resource::<MinimapState>();
    assert!(
        state.blips.contains_key(&asteroid),
        "Asteroid within scanner range should have a minimap blip"
    );

    // Verify the blip entity exists and has MinimapBlip component
    let blip_entity = state.blips[&asteroid];
    let blip = app
        .world()
        .entity(blip_entity)
        .get::<MinimapBlip>()
        .expect("Blip entity should have MinimapBlip component");
    assert_eq!(blip.source_entity, asteroid);
}

#[test]
fn minimap_blips_disappear_when_entity_despawned() {
    let mut app = minimap_test_app();
    let _player = spawn_player(&mut app);
    let asteroid = spawn_asteroid(&mut app, Vec2::new(500.0, 0.0), 20.0, 50.0);

    // Create blip
    app.update();
    app.update();

    let state = app.world().resource::<MinimapState>();
    assert!(
        state.blips.contains_key(&asteroid),
        "Blip should exist before despawn"
    );
    let blip_entity = state.blips[&asteroid];

    // Despawn the source entity
    app.world_mut().despawn(asteroid);

    // Run update to process removal
    app.update();
    app.update();

    let state = app.world().resource::<MinimapState>();
    assert!(
        !state.blips.contains_key(&asteroid),
        "Blip should be removed after source entity despawned"
    );
    // Verify the blip entity itself is despawned
    assert!(
        app.world().get_entity(blip_entity).is_err(),
        "Blip UI entity should be despawned"
    );
}

#[test]
fn minimap_blip_position_updates_when_player_moves() {
    let mut app = minimap_test_app();
    let player = spawn_player(&mut app);
    let asteroid = spawn_asteroid(&mut app, Vec2::new(500.0, 0.0), 20.0, 50.0);

    // Create blip
    app.update();
    app.update();

    let config = app.world().resource::<MinimapConfig>().clone();
    let state = app.world().resource::<MinimapState>();
    let blip_entity = state.blips[&asteroid];

    // Record initial blip position
    let initial_left = app
        .world()
        .entity(blip_entity)
        .get::<Node>()
        .expect("Blip should have Node")
        .left;

    // Move player closer to asteroid
    app.world_mut()
        .entity_mut(player)
        .get_mut::<Transform>()
        .expect("Player should have Transform")
        .translation = Vec3::new(250.0, 0.0, 0.0);

    // Update to recalculate positions
    app.update();

    let updated_left = app
        .world()
        .entity(blip_entity)
        .get::<Node>()
        .expect("Blip should have Node")
        .left;

    // After moving player closer to asteroid, the world offset shrinks,
    // so the blip should move closer to center (smaller left offset from center)
    let half_blip = config.blip_size / 2.0;
    let center_left = Val::Px(config.minimap_radius - half_blip);

    // The blip should have moved — initial was farther from center, updated is closer
    assert_ne!(
        initial_left, updated_left,
        "Blip position should change when player moves"
    );

    // Updated blip should be closer to center than initial
    if let (Val::Px(initial), Val::Px(updated), Val::Px(center)) =
        (initial_left, updated_left, center_left)
    {
        let initial_dist = (initial - center).abs();
        let updated_dist = (updated - center).abs();
        assert!(
            updated_dist < initial_dist,
            "Blip should be closer to center after player moved toward it: initial_dist={initial_dist}, updated_dist={updated_dist}"
        );
    } else {
        panic!("Expected Val::Px values for blip positions");
    }
}

#[test]
fn no_minimap_blips_for_entities_outside_scanner_range() {
    let mut app = minimap_test_app();
    let _player = spawn_player(&mut app);

    let config = app.world().resource::<MinimapConfig>().clone();
    let far_distance = config.scanner_range + 500.0;

    // Spawn asteroid and drone far outside scanner range
    let far_asteroid = spawn_asteroid(&mut app, Vec2::new(far_distance, 0.0), 20.0, 50.0);
    let far_drone = spawn_drone(&mut app, Vec2::new(0.0, far_distance), 10.0, 30.0);

    // Spawn one entity within range for comparison
    let near_asteroid = spawn_asteroid(&mut app, Vec2::new(100.0, 0.0), 20.0, 50.0);

    app.update();
    app.update();

    let state = app.world().resource::<MinimapState>();
    assert!(
        !state.blips.contains_key(&far_asteroid),
        "Asteroid outside scanner range should not have a blip"
    );
    assert!(
        !state.blips.contains_key(&far_drone),
        "Drone outside scanner range should not have a blip"
    );
    assert!(
        state.blips.contains_key(&near_asteroid),
        "Asteroid inside scanner range should have a blip"
    );
}

#[test]
fn minimap_shows_blips_for_both_asteroid_and_drone_types() {
    let mut app = minimap_test_app();
    let _player = spawn_player(&mut app);

    // Spawn multiple asteroids and drones within range
    let a1 = spawn_asteroid(&mut app, Vec2::new(100.0, 0.0), 20.0, 50.0);
    let a2 = spawn_asteroid(&mut app, Vec2::new(-200.0, 100.0), 20.0, 50.0);
    let a3 = spawn_asteroid(&mut app, Vec2::new(0.0, -300.0), 20.0, 50.0);
    let d1 = spawn_drone(&mut app, Vec2::new(400.0, 400.0), 10.0, 30.0);
    let d2 = spawn_drone(&mut app, Vec2::new(-500.0, 0.0), 10.0, 30.0);

    app.update();
    app.update();

    let state = app.world().resource::<MinimapState>();
    // Note: chunk system also spawns entities, so total blip count > 5.
    // Verify our manually placed entities all have blips.
    assert!(
        state.blips.len() >= 5,
        "At least 5 entities should have blips, got {}",
        state.blips.len()
    );
    assert!(state.blips.contains_key(&a1), "Asteroid 1 should have blip");
    assert!(state.blips.contains_key(&a2), "Asteroid 2 should have blip");
    assert!(state.blips.contains_key(&a3), "Asteroid 3 should have blip");
    assert!(state.blips.contains_key(&d1), "Drone 1 should have blip");
    assert!(state.blips.contains_key(&d2), "Drone 2 should have blip");
}

#[test]
fn minimap_blip_colors_match_entity_type() {
    let mut app = minimap_test_app();
    let _player = spawn_player(&mut app);

    let asteroid = spawn_asteroid(&mut app, Vec2::new(100.0, 0.0), 20.0, 50.0);
    let drone = spawn_drone(&mut app, Vec2::new(-100.0, 0.0), 10.0, 30.0);

    app.update();
    app.update();

    let config = app.world().resource::<MinimapConfig>().clone();
    let state = app.world().resource::<MinimapState>();

    // Verify asteroid blip has correct gray color
    let asteroid_blip = state
        .blips
        .get(&asteroid)
        .expect("Asteroid should have a blip");
    let asteroid_bg = app
        .world()
        .entity(*asteroid_blip)
        .get::<BackgroundColor>()
        .expect("Asteroid blip should have BackgroundColor");
    let expected_asteroid = blip_color(BlipType::Asteroid, &config);
    assert_eq!(
        asteroid_bg.0, expected_asteroid,
        "Asteroid blip should use asteroid color from config"
    );

    // Verify drone blip has correct red color
    let drone_blip = state
        .blips
        .get(&drone)
        .expect("Drone should have a blip");
    let drone_bg = app
        .world()
        .entity(*drone_blip)
        .get::<BackgroundColor>()
        .expect("Drone blip should have BackgroundColor");
    let expected_drone = blip_color(BlipType::ScoutDrone, &config);
    assert_eq!(
        drone_bg.0, expected_drone,
        "Drone blip should use drone color from config"
    );
}

#[test]
fn minimap_state_cleared_when_root_missing() {
    let mut app = minimap_test_app();
    let _player = spawn_player(&mut app);
    let _asteroid = spawn_asteroid(&mut app, Vec2::new(100.0, 0.0), 20.0, 50.0);

    // Create blips
    app.update();
    app.update();

    let state = app.world().resource::<MinimapState>();
    assert!(
        !state.blips.is_empty(),
        "Should have blips before root removal"
    );

    // Remove the MinimapRoot entity
    let root = app
        .world_mut()
        .query_filtered::<Entity, With<MinimapRoot>>()
        .single(app.world())
        .expect("MinimapRoot should exist");
    app.world_mut().despawn(root);

    // Run update — should clean up stale blips
    app.update();

    let state = app.world().resource::<MinimapState>();
    assert!(
        state.blips.is_empty(),
        "MinimapState should be cleared after MinimapRoot removal"
    );
}
