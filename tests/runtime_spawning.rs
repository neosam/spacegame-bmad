mod helpers;

use bevy::prelude::*;
use void_drifter::core::collision::Health;
use void_drifter::core::input::ActionState;
use void_drifter::core::spawning::{Asteroid, RespawnTimer, ScoutDrone, SpawningConfig};
use void_drifter::core::weapons::ActiveWeapon;
use void_drifter::world::ChunkEntity;

use helpers::{spawn_asteroid, spawn_drone, spawn_player, test_app};

// ── Asteroid destruction by laser ───────────────────────────────────────

#[test]
fn asteroid_destroyed_by_laser() {
    let mut app = test_app();
    let _player = spawn_player(&mut app);

    // Spawn asteroid directly in front of player (facing +Y)
    let asteroid = spawn_asteroid(&mut app, Vec2::new(0.0, 50.0), 20.0, 10.0);

    // Fire laser (player faces +Y by default)
    app.world_mut().resource_mut::<ActionState>().fire = true;
    app.update();

    // Clear fire to avoid continuous firing
    app.world_mut().resource_mut::<ActionState>().fire = false;

    // Run several frames for damage pipeline
    for _ in 0..5 {
        app.update();
    }

    // Asteroid should be despawned (health was 10, laser does 10 damage)
    let entity_ref = app.world().get_entity(asteroid);
    assert!(
        entity_ref.is_err(),
        "Asteroid should be despawned after laser hit"
    );
}

// ── Drone destruction by spread projectiles ─────────────────────────────

#[test]
fn drone_destroyed_by_spread_projectiles() {
    let mut app = test_app();
    let player = spawn_player(&mut app);

    // Switch to spread weapon
    *app.world_mut()
        .entity_mut(player)
        .get_mut::<ActiveWeapon>()
        .expect("Player should have ActiveWeapon") = ActiveWeapon::Spread;

    // Spawn drone directly in front of player
    let drone = spawn_drone(&mut app, Vec2::new(0.0, 40.0), 10.0, 5.0);

    // Fire spread
    app.world_mut().resource_mut::<ActionState>().fire = true;
    app.update();
    app.world_mut().resource_mut::<ActionState>().fire = false;

    // Run frames for projectiles to travel and collide
    for _ in 0..10 {
        app.update();
    }

    let entity_ref = app.world().get_entity(drone);
    assert!(
        entity_ref.is_err(),
        "Drone should be despawned after spread projectile hit"
    );
}

// ── Destroyed asteroid respawns ─────────────────────────────────────────

#[test]
fn destroyed_asteroid_creates_respawn_timer() {
    let mut app = test_app();
    let _player = spawn_player(&mut app);

    // Use short respawn delay for test
    app.world_mut().resource_mut::<SpawningConfig>().respawn_delay = 0.5;

    // Spawn asteroid with 10 HP right in front of player
    spawn_asteroid(&mut app, Vec2::new(0.0, 50.0), 20.0, 10.0);

    // Fire laser to destroy it
    app.world_mut().resource_mut::<ActionState>().fire = true;
    app.update();
    app.world_mut().resource_mut::<ActionState>().fire = false;

    // Run frames for damage + despawn
    for _ in 0..5 {
        app.update();
    }

    // A RespawnTimer should have been created
    let timer_count = app
        .world_mut()
        .query_filtered::<Entity, With<RespawnTimer>>()
        .iter(app.world())
        .count();
    assert!(
        timer_count >= 1,
        "RespawnTimer should be created when asteroid is destroyed, found {timer_count}"
    );
}

#[test]
fn destroyed_asteroid_respawns_after_delay() {
    let mut app = test_app();
    let _player = spawn_player(&mut app);

    // Use very short respawn delay for test
    app.world_mut().resource_mut::<SpawningConfig>().respawn_delay = 0.2;

    // Spawn asteroid with 10 HP right in front of player (no ChunkEntity → manual spawn)
    let asteroid = spawn_asteroid(&mut app, Vec2::new(0.0, 50.0), 20.0, 10.0);

    // Count initial non-chunk asteroids
    let initial_count = app
        .world_mut()
        .query_filtered::<Entity, (With<Asteroid>, Without<ChunkEntity>)>()
        .iter(app.world())
        .count();
    assert_eq!(initial_count, 1, "Should start with 1 non-chunk asteroid");

    // Fire laser to destroy it
    app.world_mut().resource_mut::<ActionState>().fire = true;
    app.update();
    app.world_mut().resource_mut::<ActionState>().fire = false;

    // Run enough frames for destruction
    for _ in 0..5 {
        app.update();
    }

    // Manually spawned asteroid should be gone
    let entity_ref = app.world().get_entity(asteroid);
    assert!(
        entity_ref.is_err(),
        "Manually spawned asteroid should be destroyed"
    );

    // Wait for respawn (0.2s = ~12 frames at 60fps, add margin)
    for _ in 0..30 {
        app.update();
    }

    // New non-chunk asteroid should have spawned from respawn timer
    let final_count = app
        .world_mut()
        .query_filtered::<Entity, (With<Asteroid>, Without<ChunkEntity>)>()
        .iter(app.world())
        .count();
    assert_eq!(
        final_count, 1,
        "Asteroid should respawn after delay, found {final_count}"
    );
}

// ── Contact damage from spawned entity ──────────────────────────────────

#[test]
fn spawned_asteroid_deals_contact_damage_to_player() {
    let mut app = test_app();
    let player = spawn_player(&mut app);

    // Spawn asteroid overlapping with player at origin
    spawn_asteroid(&mut app, Vec2::new(10.0, 0.0), 20.0, 50.0);

    // Run a few frames for contact collision to trigger
    for _ in 0..3 {
        app.update();
    }

    let health = app
        .world()
        .entity(player)
        .get::<Health>()
        .expect("Player should have Health");
    assert!(
        health.current < 100.0,
        "Player should take contact damage from asteroid, health: {}",
        health.current
    );
}

// ── Spawned entities interact with full pipeline ────────────────────────

#[test]
fn scout_drone_has_correct_marker() {
    let mut app = test_app();

    let drone = spawn_drone(&mut app, Vec2::new(100.0, 100.0), 10.0, 30.0);

    let has_marker = app.world().entity(drone).contains::<ScoutDrone>();
    assert!(has_marker, "Spawned drone should have ScoutDrone marker");
}

#[test]
fn asteroid_has_correct_marker() {
    let mut app = test_app();

    let asteroid = spawn_asteroid(&mut app, Vec2::new(100.0, 100.0), 20.0, 50.0);

    let has_marker = app.world().entity(asteroid).contains::<Asteroid>();
    assert!(has_marker, "Spawned asteroid should have Asteroid marker");
}
