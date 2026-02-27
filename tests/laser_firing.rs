mod helpers;

use bevy::ecs::message::MessageReader;
use bevy::prelude::*;
use helpers::{spawn_player, test_app};
use void_drifter::core::input::ActionState;
use void_drifter::core::weapons::{
    fire_weapon, FireCooldown, LaserFired, LaserPulse, WeaponConfig,
};

// Note: Gamepad input tests (GamepadButton::South, right trigger threshold) are omitted
// because Bevy's Gamepad component requires internal engine setup that is not feasible
// with MinimalPlugins. Keyboard fire mapping is tested; gamepad paths rely on manual QA.

#[test]
fn laser_spawns_when_fire_input_active() {
    let mut app = test_app();
    spawn_player(&mut app);

    // Set fire input
    app.world_mut().resource_mut::<ActionState>().fire = true;
    app.update();

    // A LaserPulse entity should exist
    let pulse_count = app
        .world_mut()
        .query::<&LaserPulse>()
        .iter(app.world())
        .count();
    assert!(
        pulse_count > 0,
        "Laser pulse should spawn when fire input is active"
    );
}

#[test]
fn laser_does_not_spawn_when_cooldown_active() {
    let mut app = test_app();
    let entity = spawn_player(&mut app);

    // Set a high cooldown
    app.world_mut()
        .entity_mut(entity)
        .get_mut::<FireCooldown>()
        .expect("Player should have FireCooldown")
        .timer = 1.0;

    // Set fire input
    app.world_mut().resource_mut::<ActionState>().fire = true;
    app.update();

    let pulse_count = app
        .world_mut()
        .query::<&LaserPulse>()
        .iter(app.world())
        .count();
    assert_eq!(
        pulse_count, 0,
        "Laser should NOT spawn when cooldown is active"
    );
}

#[test]
fn laser_spawns_at_correct_position() {
    let mut app = test_app();
    spawn_player(&mut app);

    app.world_mut().resource_mut::<ActionState>().fire = true;
    app.update();

    let config = app.world().resource::<WeaponConfig>().clone();
    let expected_local_y = 20.0 + config.laser_range / 2.0;

    let transform = app
        .world_mut()
        .query::<(&LaserPulse, &Transform)>()
        .iter(app.world())
        .next()
        .expect("Should have a laser pulse")
        .1
        .clone();

    // LaserPulse is a child of the player — Transform is local-space
    assert!(
        transform.translation.x.abs() < 0.01,
        "Laser local X should be near 0, got {}",
        transform.translation.x
    );
    assert!(
        (transform.translation.y - expected_local_y).abs() < 0.01,
        "Laser local Y should be nose_offset + range/2 (~{}), got {}",
        expected_local_y,
        transform.translation.y
    );
}

#[test]
fn laser_has_correct_direction_matching_player_rotation() {
    let mut app = test_app();
    let entity = spawn_player(&mut app);

    // Rotate player 90 degrees (facing -X after rotation from +Y)
    let angle = std::f32::consts::FRAC_PI_2;
    app.world_mut()
        .entity_mut(entity)
        .get_mut::<Transform>()
        .expect("Player should have Transform")
        .rotation = Quat::from_rotation_z(angle);

    app.world_mut().resource_mut::<ActionState>().fire = true;
    app.update();

    let pulse = app
        .world_mut()
        .query::<&LaserPulse>()
        .iter(app.world())
        .next()
        .expect("Should have a laser pulse");

    // LaserPulse.direction stores world-space firing direction
    // After 90-degree rotation from +Y, facing should be approximately -X
    assert!(
        pulse.direction.x < -0.5,
        "Laser direction X should be negative after 90-degree rotation, got {}",
        pulse.direction.x
    );

    // Child entity should have identity rotation (parent propagates rotation)
    let transform = app
        .world_mut()
        .query::<(&LaserPulse, &Transform)>()
        .iter(app.world())
        .next()
        .expect("Should have a laser pulse")
        .1
        .clone();
    let angle_diff = transform.rotation.angle_between(Quat::IDENTITY);
    assert!(
        angle_diff < 0.01,
        "Child laser Transform should have identity rotation, got angle diff {}",
        angle_diff
    );
}

#[test]
fn laser_despawns_after_pulse_duration() {
    let mut app = test_app();
    spawn_player(&mut app);

    app.world_mut().resource_mut::<ActionState>().fire = true;
    app.update();

    // Verify pulse exists
    let pulse_count = app
        .world_mut()
        .query::<&LaserPulse>()
        .iter(app.world())
        .count();
    assert!(pulse_count > 0, "Pulse should exist after firing");

    // Stop firing and run enough frames for pulse to expire
    // Default pulse duration is 0.08s, at 1/60s per frame that's ~5 frames
    app.world_mut().resource_mut::<ActionState>().fire = false;
    for _ in 0..10 {
        app.update();
    }

    let pulse_count = app
        .world_mut()
        .query::<&LaserPulse>()
        .iter(app.world())
        .count();
    assert_eq!(
        pulse_count, 0,
        "Laser pulse should despawn after duration expires"
    );
}

#[test]
fn fire_rate_limits_pulses() {
    let mut app = test_app();
    spawn_player(&mut app);

    // Fire once — should spawn exactly 1 pulse
    app.world_mut().resource_mut::<ActionState>().fire = true;
    app.update();

    let count_after_first = app
        .world_mut()
        .query::<&LaserPulse>()
        .iter(app.world())
        .count();
    assert_eq!(
        count_after_first, 1,
        "Should spawn exactly 1 pulse on first fire"
    );

    // Immediately fire again — cooldown should block second pulse
    app.world_mut().resource_mut::<ActionState>().fire = true;
    app.update();

    let count_after_second = app
        .world_mut()
        .query::<&LaserPulse>()
        .iter(app.world())
        .count();
    assert!(
        count_after_second <= 1,
        "Cooldown should prevent second pulse on immediate next frame, got {}",
        count_after_second
    );

    // Wait for cooldown to expire, then fire again
    let fire_rate = app.world().resource::<WeaponConfig>().laser_fire_rate;
    let cooldown_frames = (60.0 / fire_rate).ceil() as usize;
    app.world_mut().resource_mut::<ActionState>().fire = false;
    for _ in 0..cooldown_frames {
        app.update();
    }

    // Fire again after cooldown — should spawn a new pulse
    app.world_mut().resource_mut::<ActionState>().fire = true;
    app.update();

    let count_after_cooldown = app
        .world_mut()
        .query::<&LaserPulse>()
        .iter(app.world())
        .count();
    assert!(
        count_after_cooldown >= 1,
        "Should be able to fire again after cooldown expires"
    );
}

/// Helper resource to count LaserFired messages in tests.
#[derive(Resource, Default)]
struct LaserFiredCount(usize);

/// System that reads LaserFired messages and increments the counter.
fn count_laser_fired(
    mut reader: MessageReader<LaserFired>,
    mut count: ResMut<LaserFiredCount>,
) {
    for _ in reader.read() {
        count.0 += 1;
    }
}

#[test]
fn laser_fired_message_emitted_on_fire() {
    let mut app = test_app();
    app.init_resource::<LaserFiredCount>();
    app.add_systems(FixedUpdate, count_laser_fired.after(fire_weapon));
    spawn_player(&mut app);

    app.world_mut().resource_mut::<ActionState>().fire = true;
    app.update();

    let count = app.world().resource::<LaserFiredCount>().0;
    assert_eq!(
        count, 1,
        "LaserFired message should be emitted exactly once when firing"
    );
}

#[test]
fn laser_pulse_is_child_of_player() {
    let mut app = test_app();
    let player = spawn_player(&mut app);

    app.world_mut().resource_mut::<ActionState>().fire = true;
    app.update();

    // Find the LaserPulse entity
    let laser_entity = app
        .world_mut()
        .query::<(Entity, &LaserPulse)>()
        .iter(app.world())
        .next()
        .expect("Should have a laser pulse entity")
        .0;

    // Verify ChildOf relationship points to the player
    let child_of = app
        .world()
        .entity(laser_entity)
        .get::<ChildOf>()
        .expect("LaserPulse should have ChildOf component (must be child of player)");
    assert_eq!(
        child_of.parent(),
        player,
        "LaserPulse parent should be the player entity"
    );

    // Verify player's Children contains the laser
    let children = app
        .world()
        .entity(player)
        .get::<Children>()
        .expect("Player should have Children component after spawning laser");
    assert!(
        children.iter().any(|c| c == laser_entity),
        "Player's Children should contain the laser pulse entity"
    );
}

#[test]
fn weapon_config_ron_parses_correctly() {
    let ron_str =
        std::fs::read_to_string("assets/config/weapons.ron").expect("weapons.ron should exist");
    let config = WeaponConfig::from_ron(&ron_str).expect("weapons.ron should parse correctly");
    assert_eq!(config.laser_fire_rate, 4.0);
    assert_eq!(config.laser_range, 500.0);
}
