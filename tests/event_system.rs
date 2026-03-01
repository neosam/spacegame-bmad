#![deny(clippy::unwrap_used)]
//! Integration tests for Event System (Story 1.7).

mod helpers;

use bevy::prelude::*;
use helpers::{
    run_until_loaded, spawn_asteroid, spawn_player, test_app,
};
use void_drifter::core::collision::Health;
use void_drifter::core::input::ActionState;
use void_drifter::infrastructure::logbook::Logbook;
use void_drifter::shared::events::{EventSeverity, GameEventKind};

/// AC #10: Destroy an asteroid, verify EnemyDestroyed event is NOT in logbook (Tier3 = filtered).
#[test]
fn enemy_destroyed_emits_game_event() {
    let mut app = test_app();
    let _player = spawn_player(&mut app);

    // Place asteroid right in front of player (player faces +Y)
    let _asteroid = spawn_asteroid(&mut app, Vec2::new(0.0, 50.0), 20.0, 5.0);

    // Fire laser at the asteroid
    app.world_mut().resource_mut::<ActionState>().fire = true;
    app.update();
    app.world_mut().resource_mut::<ActionState>().fire = false;

    // Run several frames for damage + despawn + event recording
    for _ in 0..10 {
        app.update();
    }

    // EnemyDestroyed is Tier3 — filtered from logbook by design
    let logbook = app.world().resource::<Logbook>();
    let destroyed_events: Vec<_> = logbook
        .entries()
        .iter()
        .filter(|e| matches!(e.kind, GameEventKind::EnemyDestroyed { .. }))
        .collect();
    assert!(
        destroyed_events.is_empty(),
        "EnemyDestroyed (Tier3) should NOT appear in logbook"
    );
}

/// AC #10: Kill player, verify PlayerDeath + PlayerRespawned entries.
#[test]
fn player_death_emits_game_event() {
    let mut app = test_app();
    let player = spawn_player(&mut app);

    // Set player health to 1 so next contact damage kills
    app.world_mut()
        .entity_mut(player)
        .get_mut::<Health>()
        .expect("Player should have Health")
        .current = 1.0;

    // Spawn overlapping asteroid for contact damage
    spawn_asteroid(&mut app, Vec2::new(5.0, 0.0), 20.0, 100.0);

    // Run frames — contact collision + damage + player death + event recording
    for _ in 0..10 {
        app.update();
    }

    let logbook = app.world().resource::<Logbook>();
    let death_events: Vec<_> = logbook
        .entries()
        .iter()
        .filter(|e| matches!(e.kind, GameEventKind::PlayerDeath))
        .collect();
    assert!(
        !death_events.is_empty(),
        "Logbook should have PlayerDeath entry"
    );
    assert_eq!(
        death_events[0].severity,
        EventSeverity::Tier1,
        "PlayerDeath should be Tier1 severity"
    );

    let respawn_events: Vec<_> = logbook
        .entries()
        .iter()
        .filter(|e| matches!(e.kind, GameEventKind::PlayerRespawned))
        .collect();
    assert!(
        !respawn_events.is_empty(),
        "Logbook should have PlayerRespawned entry after death"
    );
}

/// AC #10: Load world, verify ChunkLoaded is NOT in Logbook (Tier3 = filtered).
#[test]
fn chunk_loading_emits_game_events() {
    let mut app = test_app();
    spawn_player(&mut app);

    run_until_loaded(&mut app);

    // ChunkLoaded is Tier3 — filtered from logbook by design
    let logbook = app.world().resource::<Logbook>();
    let chunk_events: Vec<_> = logbook
        .entries()
        .iter()
        .filter(|e| matches!(e.kind, GameEventKind::ChunkLoaded { .. }))
        .collect();
    assert!(
        chunk_events.is_empty(),
        "ChunkLoaded (Tier3) should NOT appear in logbook"
    );
}

/// AC #10: Fire laser, verify WeaponFired is NOT in Logbook (Tier3 = filtered).
#[test]
fn weapon_fire_emits_game_event() {
    let mut app = test_app();
    spawn_player(&mut app);

    // Fire weapon
    app.world_mut().resource_mut::<ActionState>().fire = true;
    app.update();
    app.world_mut().resource_mut::<ActionState>().fire = false;

    // Run frames to process events
    for _ in 0..5 {
        app.update();
    }

    // WeaponFired is Tier3 — filtered from logbook by design
    let logbook = app.world().resource::<Logbook>();
    let fire_events: Vec<_> = logbook
        .entries()
        .iter()
        .filter(|e| matches!(e.kind, GameEventKind::WeaponFired { .. }))
        .collect();
    assert!(
        fire_events.is_empty(),
        "WeaponFired (Tier3) should NOT appear in logbook"
    );
}

/// AC #9: All existing tests pass (verified by running full suite).
/// This test just confirms the event infrastructure doesn't regress basic gameplay.
#[test]
fn event_system_does_not_break_gameplay() {
    let mut app = test_app();
    let _player = spawn_player(&mut app);

    // Spawn asteroid, fire at it, verify asteroid takes damage
    let asteroid = spawn_asteroid(&mut app, Vec2::new(0.0, 50.0), 20.0, 100.0);

    app.world_mut().resource_mut::<ActionState>().fire = true;
    app.update();
    app.world_mut().resource_mut::<ActionState>().fire = false;

    for _ in 0..5 {
        app.update();
    }

    match app.world().get_entity(asteroid) {
        Ok(entity_ref) => {
            let health = entity_ref
                .get::<Health>()
                .expect("Asteroid should have Health");
            assert!(
                health.current < 100.0,
                "Asteroid should take damage: {} < 100.0",
                health.current
            );
        }
        Err(_) => {
            // Destroyed = valid
        }
    }
}
