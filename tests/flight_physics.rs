mod helpers;

use bevy::prelude::*;
use helpers::{spawn_player, spawn_player_with_velocity, test_app};
use void_drifter::core::flight::FlightConfig;
use void_drifter::core::input::ActionState;
use void_drifter::shared::components::Velocity;
use std::f32::consts::PI;

#[test]
fn thrust_accelerates_player_over_multiple_frames() {
    let mut app = test_app();
    let entity = spawn_player(&mut app);

    app.world_mut().resource_mut::<ActionState>().thrust = 1.0;

    // Run 10 frames
    for _ in 0..10 {
        app.update();
    }

    let velocity = app
        .world()
        .entity(entity)
        .get::<Velocity>()
        .expect("Player should have Velocity");
    assert!(
        velocity.0.length() > 0.0,
        "Velocity should increase after multiple frames of thrust"
    );
}

#[test]
fn soft_cap_prevents_exceeding_max_speed() {
    let mut app = test_app();
    let entity = spawn_player(&mut app);

    let max_speed = app.world().resource::<FlightConfig>().max_speed;

    app.world_mut().resource_mut::<ActionState>().thrust = 1.0;

    // Run many frames to reach near max speed
    for _ in 0..1000 {
        app.update();
    }

    let velocity = app
        .world()
        .entity(entity)
        .get::<Velocity>()
        .expect("Player should have Velocity");
    assert!(
        velocity.0.length() <= max_speed + 1.0,
        "Velocity ({}) should not significantly exceed max_speed ({})",
        velocity.0.length(),
        max_speed
    );
}

#[test]
fn drift_decelerates_to_stop() {
    let mut app = test_app();
    let entity = spawn_player_with_velocity(&mut app, Vec2::new(0.0, 200.0));

    // No thrust — only drag
    app.world_mut().resource_mut::<ActionState>().thrust = 0.0;

    // Run enough frames for complete stop (drag_factor=0.975/frame, snap at speed<0.1)
    for _ in 0..350 {
        app.update();
    }

    let velocity = app
        .world()
        .entity(entity)
        .get::<Velocity>()
        .expect("Player should have Velocity");
    assert_eq!(
        velocity.0,
        Vec2::ZERO,
        "Ship should come to complete stop after sufficient drag time"
    );
}

#[test]
fn position_changes_with_velocity() {
    let mut app = test_app();
    let entity = spawn_player_with_velocity(&mut app, Vec2::new(100.0, 0.0));

    let initial_x = app
        .world()
        .entity(entity)
        .get::<Transform>()
        .expect("Player should have Transform")
        .translation
        .x;

    app.update();

    let new_x = app
        .world()
        .entity(entity)
        .get::<Transform>()
        .expect("Player should have Transform")
        .translation
        .x;

    assert!(
        new_x > initial_x,
        "Position should change in direction of velocity"
    );
}

#[test]
fn rotation_then_thrust_produces_lateral_movement() {
    let mut app = test_app();
    let entity = spawn_player(&mut app);

    // Rotate 90 degrees over several frames
    app.world_mut().resource_mut::<ActionState>().rotate = 1.0;
    let rotation_speed = app.world().resource::<FlightConfig>().rotation_speed;
    let frames_for_90_deg = ((PI / 2.0) / (rotation_speed / 60.0)).ceil() as u32;
    for _ in 0..frames_for_90_deg {
        app.update();
    }

    // Stop rotating, start thrusting
    app.world_mut().resource_mut::<ActionState>().rotate = 0.0;
    app.world_mut().resource_mut::<ActionState>().thrust = 1.0;
    for _ in 0..30 {
        app.update();
    }

    let velocity = app
        .world()
        .entity(entity)
        .get::<Velocity>()
        .expect("Player should have Velocity");

    // After ~90 degree rotation, thrust should produce significant lateral velocity
    assert!(
        velocity.0.x.abs() > 1.0,
        "Rotated ship should thrust laterally (x velocity: {})",
        velocity.0.x
    );
}
