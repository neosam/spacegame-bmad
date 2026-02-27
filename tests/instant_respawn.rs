mod helpers;

use bevy::prelude::*;
use helpers::{spawn_asteroid, spawn_player, test_app};
use void_drifter::core::collision::{
    DestroyedPositions, Health, CONTACT_DAMAGE, INVINCIBILITY_DURATION,
};
use void_drifter::shared::components::{Invincible, Velocity};

#[test]
fn player_takes_contact_damage_from_asteroid() {
    let mut app = test_app();
    let player = spawn_player(&mut app);

    // Spawn asteroid overlapping player at origin
    spawn_asteroid(&mut app, Vec2::new(10.0, 0.0), 10.0, 50.0);

    app.update(); // Run collision + damage systems

    let health = app
        .world()
        .entity(player)
        .get::<Health>()
        .expect("Player should have Health");
    assert!(
        (health.current - (100.0 - CONTACT_DAMAGE)).abs() < f32::EPSILON,
        "Player health should be reduced by CONTACT_DAMAGE, got {}",
        health.current
    );
}

#[test]
fn player_dies_and_respawns_at_origin_with_full_health() {
    let mut app = test_app();
    let player = spawn_player(&mut app);

    // Set player health low so contact damage kills them
    app.world_mut()
        .entity_mut(player)
        .get_mut::<Health>()
        .expect("Player should have Health")
        .current = 5.0;

    // Move player away from origin
    app.world_mut()
        .entity_mut(player)
        .get_mut::<Transform>()
        .expect("Player should have Transform")
        .translation = Vec3::new(200.0, 300.0, 0.0);

    // Give player velocity
    app.world_mut()
        .entity_mut(player)
        .get_mut::<Velocity>()
        .expect("Player should have Velocity")
        .0 = Vec2::new(50.0, 60.0);

    // Spawn overlapping asteroid at player position
    spawn_asteroid(&mut app, Vec2::new(200.0, 300.0), 10.0, 50.0);

    app.update(); // Contact damage → death → respawn

    // Player should still exist (not despawned)
    assert!(
        app.world().get_entity(player).is_ok(),
        "Player entity should NOT be despawned"
    );

    let health = app
        .world()
        .entity(player)
        .get::<Health>()
        .expect("Player should have Health");
    assert!(
        (health.current - health.max).abs() < f32::EPSILON,
        "Health should be reset to max after death"
    );

    let transform = app
        .world()
        .entity(player)
        .get::<Transform>()
        .expect("Player should have Transform");
    assert!(
        transform.translation.distance(Vec3::ZERO) < f32::EPSILON,
        "Player should respawn at origin"
    );

    let velocity = app
        .world()
        .entity(player)
        .get::<Velocity>()
        .expect("Player should have Velocity");
    assert!(
        velocity.0.length() < f32::EPSILON,
        "Player velocity should be zero after respawn"
    );
}

#[test]
fn player_death_records_destroyed_position() {
    let mut app = test_app();
    let player = spawn_player(&mut app);

    let death_pos = Vec3::new(150.0, 250.0, 0.0);

    app.world_mut()
        .entity_mut(player)
        .get_mut::<Health>()
        .expect("Player should have Health")
        .current = 5.0;

    app.world_mut()
        .entity_mut(player)
        .get_mut::<Transform>()
        .expect("Player should have Transform")
        .translation = death_pos;

    // Spawn overlapping asteroid at player position
    spawn_asteroid(&mut app, Vec2::new(150.0, 250.0), 10.0, 50.0);

    app.update();

    // DestroyedPositions should contain the player's death position
    // Note: rendering systems drain this, but test_app doesn't include rendering
    let destroyed = app.world().resource::<DestroyedPositions>();
    assert!(
        destroyed
            .positions
            .iter()
            .any(|p| (p.x - death_pos.x).abs() < 0.1 && (p.y - death_pos.y).abs() < 0.1),
        "DestroyedPositions should contain player's death position"
    );
}

#[test]
fn invincible_player_does_not_take_contact_damage() {
    let mut app = test_app();
    let player = spawn_player(&mut app);

    // Make player invincible
    app.world_mut()
        .entity_mut(player)
        .insert(Invincible {
            timer: INVINCIBILITY_DURATION,
        });

    // Spawn overlapping asteroid
    spawn_asteroid(&mut app, Vec2::new(10.0, 0.0), 10.0, 50.0);

    app.update();

    let health = app
        .world()
        .entity(player)
        .get::<Health>()
        .expect("Player should have Health");
    assert!(
        (health.current - 100.0).abs() < f32::EPSILON,
        "Invincible player should not take contact damage, health = {}",
        health.current
    );
}

#[test]
fn player_laser_does_not_hit_self() {
    let mut app = test_app();
    let player = spawn_player(&mut app);

    // Player has Health and Collider now — verify laser doesn't hit self
    let health_before = app
        .world()
        .entity(player)
        .get::<Health>()
        .expect("Player should have Health")
        .current;

    // Simulate firing by pressing fire key
    app.world_mut()
        .resource_mut::<void_drifter::core::input::ActionState>()
        .fire = true;

    app.update();

    let health_after = app
        .world()
        .entity(player)
        .get::<Health>()
        .expect("Player should have Health")
        .current;
    assert!(
        (health_after - health_before).abs() < f32::EPSILON,
        "Player's own laser should not damage self"
    );
}

#[test]
fn player_projectiles_do_not_hit_self() {
    let mut app = test_app();
    let player = spawn_player(&mut app);

    // Switch to spread weapon
    helpers::set_active_weapon_spread(&mut app, player);

    let health_before = app
        .world()
        .entity(player)
        .get::<Health>()
        .expect("Player should have Health")
        .current;

    // Fire spread projectiles
    app.world_mut()
        .resource_mut::<void_drifter::core::input::ActionState>()
        .fire = true;

    // Run several frames to let projectiles travel
    for _ in 0..5 {
        app.update();
    }

    let health_after = app
        .world()
        .entity(player)
        .get::<Health>()
        .expect("Player should have Health")
        .current;
    assert!(
        (health_after - health_before).abs() < f32::EPSILON,
        "Player's own projectiles should not damage self"
    );
}

#[test]
fn player_gets_invincibility_after_death() {
    let mut app = test_app();
    let player = spawn_player(&mut app);

    // Kill player via low health + contact damage
    app.world_mut()
        .entity_mut(player)
        .get_mut::<Health>()
        .expect("Player should have Health")
        .current = 5.0;

    spawn_asteroid(&mut app, Vec2::new(10.0, 0.0), 10.0, 50.0);

    app.update();

    let invincible = app.world().entity(player).get::<Invincible>();
    assert!(
        invincible.is_some(),
        "Player should have Invincible component after death"
    );
}
