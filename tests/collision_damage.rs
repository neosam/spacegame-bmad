mod helpers;

use bevy::prelude::*;
use helpers::{spawn_asteroid, spawn_drone, spawn_player, test_app};
use void_drifter::core::collision::Health;
use void_drifter::core::weapons::{SpreadProjectile, WeaponConfig};

/// Helper: position a player at `pos` facing direction `dir`.
fn position_player_facing(app: &mut App, player: Entity, pos: Vec2, dir: Vec2) {
    let angle = dir.y.atan2(dir.x) - std::f32::consts::FRAC_PI_2;
    let mut binding = app.world_mut().entity_mut(player);
    let mut transform = binding
        .get_mut::<Transform>()
        .expect("Player should have Transform");
    transform.translation = pos.extend(0.0);
    transform.rotation = Quat::from_rotation_z(angle);
}

// ── Laser collision tests ───────────────────────────────────────────────

#[test]
fn laser_hits_asteroid_and_deals_damage() {
    let mut app = test_app();
    let player = spawn_player(&mut app);
    let config = app.world().resource::<WeaponConfig>().clone();

    // Place player at origin facing +Y, asteroid at (0, 100) with radius 20, health 50
    position_player_facing(&mut app, player, Vec2::ZERO, Vec2::Y);
    let asteroid = spawn_asteroid(&mut app, Vec2::new(0.0, 100.0), 20.0, 50.0);

    // Fire laser
    app.world_mut()
        .resource_mut::<void_drifter::core::input::ActionState>()
        .fire = true;

    // Run one frame: fire_weapon emits LaserFired, then collision checks run
    app.update();

    let health = app
        .world()
        .entity(asteroid)
        .get::<Health>()
        .expect("Asteroid should have Health");
    assert!(
        health.current < 50.0,
        "Asteroid should have taken damage, got {}",
        health.current
    );
    let expected = 50.0 - config.laser_damage;
    assert!(
        (health.current - expected).abs() < 0.01,
        "Expected health {expected}, got {}",
        health.current
    );
}

#[test]
fn laser_hits_drone_and_deals_damage() {
    let mut app = test_app();
    let player = spawn_player(&mut app);
    let config = app.world().resource::<WeaponConfig>().clone();

    // Place player at origin facing +Y, drone at (0, 80) with radius 10, health 20
    position_player_facing(&mut app, player, Vec2::ZERO, Vec2::Y);
    let drone = spawn_drone(&mut app, Vec2::new(0.0, 80.0), 10.0, 20.0);

    // Fire laser
    app.world_mut()
        .resource_mut::<void_drifter::core::input::ActionState>()
        .fire = true;

    app.update();

    let health = app
        .world()
        .entity(drone)
        .get::<Health>()
        .expect("Drone should have Health");
    let expected = 20.0 - config.laser_damage;
    assert!(
        (health.current - expected).abs() < 0.01,
        "Expected drone health {expected}, got {}",
        health.current
    );
}

// ── Spread projectile collision tests ───────────────────────────────────

#[test]
fn spread_projectile_hits_asteroid_and_deals_damage() {
    let mut app = test_app();
    let config = app.world().resource::<WeaponConfig>().clone();

    // Spawn a spread projectile directly at (0, 0) heading +Y
    app.world_mut().spawn((
        SpreadProjectile {
            origin: Vec2::ZERO,
            direction: Vec2::Y,
            speed: config.spread_projectile_speed,
            damage: config.spread_damage,
            timer: config.spread_projectile_lifetime,
        },
        Transform::from_translation(Vec3::new(0.0, 0.0, 0.0)),
    ));

    // Asteroid very close to the projectile spawn — will overlap immediately
    let asteroid = spawn_asteroid(&mut app, Vec2::new(0.0, 5.0), 10.0, 30.0);

    app.update();

    let health = app
        .world()
        .entity(asteroid)
        .get::<Health>()
        .expect("Asteroid should have Health");
    let expected = 30.0 - config.spread_damage;
    assert!(
        (health.current - expected).abs() < 0.01,
        "Expected asteroid health {expected}, got {}",
        health.current
    );
}

#[test]
fn spread_projectile_hits_drone_and_deals_damage() {
    let mut app = test_app();
    let config = app.world().resource::<WeaponConfig>().clone();

    // Spawn a spread projectile directly at (0, 0) heading +Y
    app.world_mut().spawn((
        SpreadProjectile {
            origin: Vec2::ZERO,
            direction: Vec2::Y,
            speed: config.spread_projectile_speed,
            damage: config.spread_damage,
            timer: config.spread_projectile_lifetime,
        },
        Transform::from_translation(Vec3::new(0.0, 0.0, 0.0)),
    ));

    // Drone very close to the projectile spawn
    let drone = spawn_drone(&mut app, Vec2::new(0.0, 5.0), 10.0, 15.0);

    app.update();

    let health = app
        .world()
        .entity(drone)
        .get::<Health>()
        .expect("Drone should have Health");
    let expected = 15.0 - config.spread_damage;
    assert!(
        (health.current - expected).abs() < 0.01,
        "Expected drone health {expected}, got {}",
        health.current
    );
}

// ── Despawn test ────────────────────────────────────────────────────────

#[test]
fn entity_despawns_when_health_reaches_zero() {
    let mut app = test_app();
    let config = app.world().resource::<WeaponConfig>().clone();

    // Spawn a projectile at (0, 0) heading +Y
    app.world_mut().spawn((
        SpreadProjectile {
            origin: Vec2::ZERO,
            direction: Vec2::Y,
            speed: config.spread_projectile_speed,
            damage: config.spread_damage,
            timer: config.spread_projectile_lifetime,
        },
        Transform::from_translation(Vec3::ZERO),
    ));

    // Asteroid with health <= spread_damage so it will be destroyed
    let asteroid = spawn_asteroid(&mut app, Vec2::new(0.0, 1.0), 10.0, config.spread_damage);

    app.update();

    // Entity should have been despawned
    assert!(
        app.world().get_entity(asteroid).is_err(),
        "Asteroid should be despawned when health reaches zero"
    );
}

// ── Multiple hits test ──────────────────────────────────────────────────

#[test]
fn multiple_projectiles_can_hit_same_target() {
    let mut app = test_app();
    let config = app.world().resource::<WeaponConfig>().clone();

    // Spawn 3 projectiles all at the same position
    for _ in 0..3 {
        app.world_mut().spawn((
            SpreadProjectile {
                origin: Vec2::ZERO,
                direction: Vec2::Y,
                speed: config.spread_projectile_speed,
                damage: config.spread_damage,
                timer: config.spread_projectile_lifetime,
            },
            Transform::from_translation(Vec3::ZERO),
        ));
    }

    // Asteroid with enough health to survive multiple hits
    let asteroid = spawn_asteroid(&mut app, Vec2::new(0.0, 1.0), 10.0, 100.0);

    app.update();

    let health = app
        .world()
        .entity(asteroid)
        .get::<Health>()
        .expect("Asteroid should still exist");

    // All 3 projectiles should have hit the same target (AC #7: no invincibility frames)
    let expected = 100.0 - 3.0 * config.spread_damage;
    assert!(
        (health.current - expected).abs() < 0.01,
        "All 3 projectiles should hit same target: expected health {expected}, got {}",
        health.current
    );
}

// ── Damage value verification tests ─────────────────────────────────────

#[test]
fn laser_damage_uses_weapon_config_laser_damage() {
    let mut app = test_app();
    let config = app.world().resource::<WeaponConfig>().clone();

    let player = spawn_player(&mut app);
    position_player_facing(&mut app, player, Vec2::ZERO, Vec2::Y);

    // Asteroid in laser path
    let asteroid = spawn_asteroid(&mut app, Vec2::new(0.0, 100.0), 20.0, 100.0);

    app.world_mut()
        .resource_mut::<void_drifter::core::input::ActionState>()
        .fire = true;

    app.update();

    let health = app
        .world()
        .entity(asteroid)
        .get::<Health>()
        .expect("Asteroid should have Health");
    let expected = 100.0 - config.laser_damage;
    assert!(
        (health.current - expected).abs() < 0.01,
        "Laser should deal config.laser_damage ({}) per hit, got health {}",
        config.laser_damage,
        health.current
    );
}

#[test]
fn spread_damage_uses_projectile_damage() {
    let mut app = test_app();

    let custom_damage = 42.0;

    // Spawn projectile with custom damage
    app.world_mut().spawn((
        SpreadProjectile {
            origin: Vec2::ZERO,
            direction: Vec2::Y,
            speed: 600.0,
            damage: custom_damage,
            timer: 1.0,
        },
        Transform::from_translation(Vec3::ZERO),
    ));

    // Asteroid overlapping the projectile
    let asteroid = spawn_asteroid(&mut app, Vec2::new(0.0, 1.0), 10.0, 100.0);

    app.update();

    let health = app
        .world()
        .entity(asteroid)
        .get::<Health>()
        .expect("Asteroid should have Health");
    let expected = 100.0 - custom_damage;
    assert!(
        (health.current - expected).abs() < 0.01,
        "Spread should deal projectile.damage ({custom_damage}), got health {}",
        health.current
    );
}

