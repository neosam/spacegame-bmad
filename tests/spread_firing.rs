mod helpers;

use bevy::ecs::message::MessageReader;
use bevy::prelude::*;
use helpers::{set_active_weapon_spread, spawn_player, test_app};
use void_drifter::core::input::ActionState;
use void_drifter::core::weapons::{
    fire_weapon, Energy, SpreadFired, SpreadProjectile, WeaponConfig,
};

#[test]
fn spread_spawns_projectiles_when_fire_active() {
    let mut app = test_app();
    let entity = spawn_player(&mut app);
    set_active_weapon_spread(&mut app, entity);

    app.world_mut().resource_mut::<ActionState>().fire = true;
    app.update();

    let count = app
        .world_mut()
        .query::<&SpreadProjectile>()
        .iter(app.world())
        .count();
    let expected = app
        .world()
        .resource::<WeaponConfig>()
        .spread_projectile_count as usize;
    assert_eq!(
        count, expected,
        "Should spawn {expected} spread projectiles, got {count}"
    );
}

#[test]
fn spread_does_not_fire_when_energy_insufficient() {
    let mut app = test_app();
    let entity = spawn_player(&mut app);
    set_active_weapon_spread(&mut app, entity);

    // Drain energy below cost
    app.world_mut()
        .entity_mut(entity)
        .get_mut::<Energy>()
        .expect("Player should have Energy")
        .current = 1.0;

    app.world_mut().resource_mut::<ActionState>().fire = true;
    app.update();

    let count = app
        .world_mut()
        .query::<&SpreadProjectile>()
        .iter(app.world())
        .count();
    assert_eq!(
        count, 0,
        "Spread should NOT fire when energy is insufficient"
    );
}

#[test]
fn spread_does_not_fire_when_active_weapon_is_laser() {
    let mut app = test_app();
    spawn_player(&mut app); // defaults to ActiveWeapon::Laser

    app.world_mut().resource_mut::<ActionState>().fire = true;
    app.update();

    let count = app
        .world_mut()
        .query::<&SpreadProjectile>()
        .iter(app.world())
        .count();
    assert_eq!(
        count, 0,
        "Spread projectiles should NOT spawn when ActiveWeapon is Laser"
    );
}

#[test]
fn correct_number_of_projectiles_spawned() {
    let mut app = test_app();
    let entity = spawn_player(&mut app);
    set_active_weapon_spread(&mut app, entity);

    app.world_mut().resource_mut::<ActionState>().fire = true;
    app.update();

    let count = app
        .world_mut()
        .query::<&SpreadProjectile>()
        .iter(app.world())
        .count();
    assert_eq!(
        count, 5,
        "Default config should spawn 5 spread projectiles"
    );
}

#[test]
fn projectiles_move_over_frames() {
    let mut app = test_app();
    let entity = spawn_player(&mut app);
    set_active_weapon_spread(&mut app, entity);

    app.world_mut().resource_mut::<ActionState>().fire = true;
    app.update();

    // Record initial positions keyed by Entity for stable comparison
    let initial_positions: Vec<(Entity, Vec3)> = app
        .world_mut()
        .query::<(Entity, &SpreadProjectile, &Transform)>()
        .iter(app.world())
        .map(|(e, _, t)| (e, t.translation))
        .collect();

    assert!(
        !initial_positions.is_empty(),
        "Should have projectiles to track"
    );

    // Stop firing and advance a frame
    app.world_mut().resource_mut::<ActionState>().fire = false;
    app.update();

    // Compare by matching Entity IDs for stable ordering
    for (entity_id, old_pos) in &initial_positions {
        let new_pos = app
            .world()
            .entity(*entity_id)
            .get::<Transform>()
            .expect("Projectile should still exist")
            .translation;
        assert!(
            old_pos.distance(new_pos) > 0.1,
            "Projectile {:?} should have moved between frames",
            entity_id
        );
    }
}

#[test]
fn projectiles_despawn_after_lifetime() {
    let mut app = test_app();
    let entity = spawn_player(&mut app);
    set_active_weapon_spread(&mut app, entity);

    app.world_mut().resource_mut::<ActionState>().fire = true;
    app.update();

    // Verify projectiles exist
    let count = app
        .world_mut()
        .query::<&SpreadProjectile>()
        .iter(app.world())
        .count();
    assert!(count > 0, "Projectiles should exist after firing");

    // Default lifetime is 0.8s, at 1/60s per frame that's ~48 frames. Run 60 to be safe.
    app.world_mut().resource_mut::<ActionState>().fire = false;
    for _ in 0..60 {
        app.update();
    }

    let count = app
        .world_mut()
        .query::<&SpreadProjectile>()
        .iter(app.world())
        .count();
    assert_eq!(
        count, 0,
        "All spread projectiles should despawn after lifetime expires"
    );
}

#[test]
fn energy_deducted_on_spread_fire() {
    let mut app = test_app();
    let entity = spawn_player(&mut app);
    set_active_weapon_spread(&mut app, entity);

    let energy_before = app
        .world()
        .entity(entity)
        .get::<Energy>()
        .expect("Player should have Energy")
        .current;

    app.world_mut().resource_mut::<ActionState>().fire = true;
    app.update();

    let energy_after = app
        .world()
        .entity(entity)
        .get::<Energy>()
        .expect("Player should have Energy")
        .current;

    let cost = app
        .world()
        .resource::<WeaponConfig>()
        .spread_energy_cost;

    assert!(
        energy_after < energy_before,
        "Energy should decrease after spread fire"
    );
    // Allow small float tolerance due to regen within the same frame
    let expected_deduction = energy_before - cost;
    assert!(
        (energy_after - expected_deduction).abs() < 1.0,
        "Energy should decrease by approximately {cost}, got {} → {}",
        energy_before,
        energy_after
    );
}

#[test]
fn energy_regenerates_over_time() {
    let mut app = test_app();
    let entity = spawn_player(&mut app);

    // Set energy to 50
    app.world_mut()
        .entity_mut(entity)
        .get_mut::<Energy>()
        .expect("Player should have Energy")
        .current = 50.0;

    app.update();

    let energy = app
        .world()
        .entity(entity)
        .get::<Energy>()
        .expect("Player should have Energy")
        .current;
    assert!(
        energy > 50.0,
        "Energy should regenerate over time, got {}",
        energy
    );
}

#[test]
fn spread_arc_distributes_projectiles_correctly() {
    let mut app = test_app();
    let entity = spawn_player(&mut app);
    set_active_weapon_spread(&mut app, entity);

    app.world_mut().resource_mut::<ActionState>().fire = true;
    app.update();

    // Player faces +Y, so spread should be centered on +Y direction
    let directions: Vec<Vec2> = app
        .world_mut()
        .query::<&SpreadProjectile>()
        .iter(app.world())
        .map(|p| p.direction)
        .collect();

    assert_eq!(directions.len(), 5, "Should have 5 projectile directions");

    // All directions should be roughly facing +Y (y > 0)
    for (i, dir) in directions.iter().enumerate() {
        assert!(
            dir.y > 0.0,
            "Projectile {i} direction should face +Y, got {dir:?}"
        );
    }

    // The center projectile (index 2) should be closest to pure +Y
    let center_angle = directions[2].y.atan2(directions[2].x);
    let pure_up_angle = std::f32::consts::FRAC_PI_2;
    assert!(
        (center_angle - pure_up_angle).abs() < 0.01,
        "Center projectile should face straight +Y, angle diff: {}",
        (center_angle - pure_up_angle).abs()
    );

    // Edge projectiles should be spread across ~30 degrees (0.5236 rad)
    let left_angle = directions[0].y.atan2(directions[0].x);
    let right_angle = directions[4].y.atan2(directions[4].x);
    let total_arc = (left_angle - right_angle).abs();
    let expected_arc = 30.0_f32.to_radians();
    assert!(
        (total_arc - expected_arc).abs() < 0.01,
        "Total arc should be ~30 degrees ({expected_arc:.4} rad), got {total_arc:.4} rad"
    );
}

/// Helper resource to count SpreadFired messages in tests.
#[derive(Resource, Default)]
struct SpreadFiredCount(usize);

/// System that reads SpreadFired messages and increments the counter.
fn count_spread_fired(
    mut reader: MessageReader<SpreadFired>,
    mut count: ResMut<SpreadFiredCount>,
) {
    for _ in reader.read() {
        count.0 += 1;
    }
}

#[test]
fn spread_fired_message_emitted_on_fire() {
    let mut app = test_app();
    app.init_resource::<SpreadFiredCount>();
    app.add_systems(FixedUpdate, count_spread_fired.after(fire_weapon));
    let entity = spawn_player(&mut app);
    set_active_weapon_spread(&mut app, entity);

    app.world_mut().resource_mut::<ActionState>().fire = true;
    app.update();

    let count = app.world().resource::<SpreadFiredCount>().0;
    assert_eq!(
        count, 1,
        "SpreadFired message should be emitted exactly once when firing spread"
    );
}

#[test]
fn fire_rate_limits_spread_fire() {
    let mut app = test_app();
    let entity = spawn_player(&mut app);
    set_active_weapon_spread(&mut app, entity);

    // Fire once
    app.world_mut().resource_mut::<ActionState>().fire = true;
    app.update();

    let count_after_first = app
        .world_mut()
        .query::<&SpreadProjectile>()
        .iter(app.world())
        .count();
    assert_eq!(
        count_after_first, 5,
        "First spread should spawn 5 projectiles"
    );

    // Immediately fire again — cooldown should block
    app.update();

    let count_after_second = app
        .world_mut()
        .query::<&SpreadProjectile>()
        .iter(app.world())
        .count();
    assert_eq!(
        count_after_second, 5,
        "Cooldown should prevent second spread fire, still only 5 projectiles"
    );

    // Wait for cooldown to expire, then fire again
    let fire_rate = app.world().resource::<WeaponConfig>().spread_fire_rate;
    let cooldown_frames = (60.0 / fire_rate).ceil() as usize;
    app.world_mut().resource_mut::<ActionState>().fire = false;
    for _ in 0..cooldown_frames {
        app.update();
    }

    // Fire again after cooldown — should spawn new projectiles
    app.world_mut().resource_mut::<ActionState>().fire = true;
    app.update();

    let count_after_cooldown = app
        .world_mut()
        .query::<&SpreadProjectile>()
        .iter(app.world())
        .count();
    assert!(
        count_after_cooldown > count_after_second,
        "Should fire again after cooldown expires, got {}",
        count_after_cooldown
    );
}
