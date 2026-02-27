use std::time::Duration;

use bevy::prelude::*;
use bevy::time::TimeUpdateStrategy;
use void_drifter::core::flight::{
    apply_drag, apply_rotation, apply_thrust, apply_velocity, FlightConfig, Player,
};
use void_drifter::core::input::ActionState;
use void_drifter::core::weapons::{
    fire_laser, tick_fire_cooldown, tick_laser_pulses, FireCooldown, LaserFired, WeaponConfig,
};
use void_drifter::shared::components::Velocity;

/// Create a minimal test App with flight and weapon systems but no windowing/rendering.
/// Systems run in FixedUpdate to match production scheduling.
/// Uses fixed 1/60s time step for deterministic tests.
pub fn test_app() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.init_resource::<ActionState>();
    app.insert_resource(FlightConfig::default());
    app.insert_resource(WeaponConfig::default());
    app.add_message::<LaserFired>();
    // Match production: flight systems in FixedUpdate
    app.add_systems(
        FixedUpdate,
        (apply_thrust, apply_rotation, apply_drag, apply_velocity).chain(),
    );
    // Weapon systems: cooldown tick, fire, pulse tick
    app.add_systems(
        FixedUpdate,
        (tick_fire_cooldown, fire_laser, tick_laser_pulses).chain(),
    );
    // Fixed 1/60s time step for deterministic tests
    app.insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_secs_f64(
        1.0 / 60.0,
    )));
    app.insert_resource(Time::<Fixed>::from_seconds(1.0 / 60.0));
    // Prime time — first frame always has dt=0
    app.update();
    app
}

/// Spawn a player entity at the origin facing +Y with FireCooldown.
pub fn spawn_player(app: &mut App) -> Entity {
    app.world_mut()
        .spawn((
            Player,
            Velocity::default(),
            FireCooldown::default(),
            Transform::default(),
        ))
        .id()
}

/// Spawn a player entity with an initial velocity.
#[allow(dead_code)]
pub fn spawn_player_with_velocity(app: &mut App, velocity: Vec2) -> Entity {
    app.world_mut()
        .spawn((
            Player,
            Velocity(velocity),
            FireCooldown::default(),
            Transform::default(),
        ))
        .id()
}
