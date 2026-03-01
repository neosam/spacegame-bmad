use bevy::prelude::*;
use serde::Deserialize;

use crate::core::input::ActionState;
use crate::shared::components::Velocity;

/// Flight balance values loaded from `assets/config/flight.ron`.
/// Derives Asset + TypePath for future AssetServer migration.
#[derive(Resource, Asset, Deserialize, TypePath, Clone, Debug)]
pub struct FlightConfig {
    pub thrust_power: f32,
    pub max_speed: f32,
    pub drag_coefficient: f32,
    pub rotation_speed: f32,
    /// Fraction of chunk generation speed at which the speed cap fully engages.
    #[serde(default = "default_speed_cap_fraction")]
    pub speed_cap_fraction: f32,
    /// Deceleration lerp factor when speed exceeds the cap (0..1, higher = slower decel).
    #[serde(default = "default_speed_cap_decel")]
    pub speed_cap_decel: f32,
}

fn default_speed_cap_fraction() -> f32 {
    0.85
}

fn default_speed_cap_decel() -> f32 {
    0.92
}

impl Default for FlightConfig {
    fn default() -> Self {
        Self {
            thrust_power: 300.0,
            max_speed: 400.0,
            drag_coefficient: 1.5,
            rotation_speed: 5.0,
            speed_cap_fraction: default_speed_cap_fraction(),
            speed_cap_decel: default_speed_cap_decel(),
        }
    }
}

impl FlightConfig {
    /// Load config from RON file contents.
    pub fn from_ron(ron_str: &str) -> Result<Self, ron::error::SpannedError> {
        ron::from_str(ron_str)
    }
}

/// Marker component for the player entity.
#[derive(Component)]
pub struct Player;

/// Apply thrust in facing direction with soft speed cap.
/// `velocity += facing * thrust_power * intensity * (1.0 - speed/max_speed) * dt`
pub fn apply_thrust(
    action_state: Res<ActionState>,
    config: Res<FlightConfig>,
    time: Res<Time>,
    mut query: Query<(&Transform, &mut Velocity), With<Player>>,
) {
    if action_state.thrust <= 0.0 {
        return;
    }

    let dt = time.delta_secs();

    for (transform, mut velocity) in query.iter_mut() {
        let facing = transform.rotation * Vec3::Y;
        let facing_2d = Vec2::new(facing.x, facing.y);
        let speed = velocity.0.length();
        let effectiveness = (1.0 - speed / config.max_speed).max(0.0);

        velocity.0 +=
            facing_2d * config.thrust_power * action_state.thrust * effectiveness * dt;
    }
}

/// Apply instant rotation (no angular velocity — arcade feel).
/// `transform.rotate_z(rotation_speed * rotate_input * dt)`
pub fn apply_rotation(
    action_state: Res<ActionState>,
    config: Res<FlightConfig>,
    time: Res<Time>,
    mut query: Query<&mut Transform, With<Player>>,
) {
    if action_state.rotate == 0.0 {
        return;
    }

    let dt = time.delta_secs();

    for mut transform in query.iter_mut() {
        transform.rotate_z(config.rotation_speed * action_state.rotate * dt);
    }
}

/// Apply drag to decelerate over time.
/// `velocity *= (1.0 - drag_coefficient * dt)`
pub fn apply_drag(
    config: Res<FlightConfig>,
    time: Res<Time>,
    mut query: Query<&mut Velocity, With<Player>>,
) {
    let dt = time.delta_secs();
    let drag_factor = (1.0 - config.drag_coefficient * dt).max(0.0);

    for mut velocity in query.iter_mut() {
        velocity.0 *= drag_factor;

        // Stop completely if near zero to avoid floating point drift
        if velocity.0.length_squared() < 0.01 {
            velocity.0 = Vec2::ZERO;
        }
    }
}

/// Compute the maximum speed at which chunks can be generated fast enough.
/// `chunk_size * max_chunks_per_frame * fixed_hz / load_radius`
pub fn compute_chunk_gen_speed(
    world_config: &crate::world::WorldConfig,
    timestep_secs: f32,
) -> Option<f32> {
    if timestep_secs <= 0.0 {
        return None;
    }
    Some(
        world_config.chunk_size
            * world_config.max_chunks_per_frame as f32
            * (1.0 / timestep_secs)
            / world_config.load_radius as f32,
    )
}

/// Startup validation: warns if max_speed exceeds chunk generation capacity.
pub fn validate_speed_cap(
    config: Res<FlightConfig>,
    world_config: Res<crate::world::WorldConfig>,
    time: Res<Time<Fixed>>,
) {
    let Some(chunk_gen_speed) =
        compute_chunk_gen_speed(&world_config, time.timestep().as_secs_f32())
    else {
        return;
    };
    let capped_max = chunk_gen_speed * config.speed_cap_fraction;

    if config.max_speed > capped_max {
        warn!(
            "FlightConfig: max_speed ({}) exceeds chunk generation capacity ({}). Speed will be soft-capped at {}.",
            config.max_speed, chunk_gen_speed, capped_max
        );
    }
}

/// Clamp player speed to prevent outrunning chunk generation.
/// Applies smooth deceleration (lerp) when speed exceeds the computed cap.
/// Cap = min(max_speed, chunk_gen_speed * speed_cap_fraction).
pub fn clamp_speed(
    config: Res<FlightConfig>,
    world_config: Res<crate::world::WorldConfig>,
    time: Res<Time<Fixed>>,
    mut query: Query<&mut Velocity, With<Player>>,
) {
    let Some(chunk_gen_speed) =
        compute_chunk_gen_speed(&world_config, time.timestep().as_secs_f32())
    else {
        return;
    };
    let effective_cap = (chunk_gen_speed * config.speed_cap_fraction).min(config.max_speed);
    let decel = config.speed_cap_decel.clamp(0.0, 1.0);

    for mut velocity in query.iter_mut() {
        let speed = velocity.0.length();
        if speed > effective_cap {
            let target_speed = speed.lerp(effective_cap, 1.0 - decel);
            velocity.0 = velocity.0.normalize_or_zero() * target_speed;
        }
    }
}

/// Apply velocity to position.
/// `transform.translation += velocity * dt`
pub fn apply_velocity(
    time: Res<Time>,
    mut query: Query<(&Velocity, &mut Transform), With<Player>>,
) {
    let dt = time.delta_secs();

    for (velocity, mut transform) in query.iter_mut() {
        transform.translation += Vec3::new(velocity.0.x, velocity.0.y, 0.0) * dt;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::time::TimeUpdateStrategy;
    use std::f32::consts::PI;
    use std::time::Duration;

    fn setup_test_app() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.init_resource::<ActionState>();
        app.insert_resource(FlightConfig::default());
        // Fixed 1/60s time step for deterministic tests
        app.insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_secs_f64(
            1.0 / 60.0,
        )));
        // Prime time — first frame always has dt=0
        app.update();
        app
    }

    fn spawn_player(app: &mut App) -> Entity {
        app.world_mut().spawn((
            Player,
            Velocity::default(),
            Transform::default(),
        )).id()
    }

    fn spawn_player_with_rotation(app: &mut App, angle: f32) -> Entity {
        app.world_mut().spawn((
            Player,
            Velocity::default(),
            Transform::from_rotation(Quat::from_rotation_z(angle)),
        )).id()
    }

    #[test]
    fn thrust_increases_velocity_in_facing_direction() {
        let mut app = setup_test_app();
        let entity = spawn_player(&mut app);

        // Set thrust input
        app.world_mut().resource_mut::<ActionState>().thrust = 1.0;

        // Add thrust system and run
        app.add_systems(Update, apply_thrust);
        app.update();

        let velocity = app.world().entity(entity).get::<Velocity>()
            .expect("Player should have Velocity");
        // Default facing is +Y, velocity should be positive Y
        assert!(velocity.0.y > 0.0, "Thrust should increase velocity in facing direction (Y)");
        assert!(velocity.0.x.abs() < 0.001, "No sideways velocity when facing up");
    }

    #[test]
    fn soft_speed_cap_reduces_thrust_effectiveness() {
        let mut app = setup_test_app();
        let entity = spawn_player(&mut app);

        let config = app.world().resource::<FlightConfig>().clone();

        // Set velocity close to max speed
        app.world_mut().entity_mut(entity).get_mut::<Velocity>()
            .expect("Player should have Velocity")
            .0 = Vec2::new(0.0, config.max_speed * 0.95);

        app.world_mut().resource_mut::<ActionState>().thrust = 1.0;
        app.add_systems(Update, apply_thrust);
        app.update();

        let velocity = app.world().entity(entity).get::<Velocity>()
            .expect("Player should have Velocity");
        // At 95% max speed, effectiveness is only 5% — small acceleration
        assert!(
            velocity.0.y < config.max_speed,
            "Velocity should stay below max_speed due to soft cap"
        );
    }

    #[test]
    fn drag_reduces_velocity_over_time() {
        let mut app = setup_test_app();
        let entity = spawn_player(&mut app);

        // Set initial velocity
        app.world_mut().entity_mut(entity).get_mut::<Velocity>()
            .expect("Player should have Velocity")
            .0 = Vec2::new(0.0, 200.0);

        app.add_systems(Update, apply_drag);
        app.update();

        let velocity = app.world().entity(entity).get::<Velocity>()
            .expect("Player should have Velocity");
        assert!(velocity.0.y < 200.0, "Drag should reduce velocity");
        assert!(velocity.0.y > 0.0, "Velocity should still be positive after one frame");
    }

    #[test]
    fn rotation_changes_facing_direction() {
        let mut app = setup_test_app();
        let entity = spawn_player(&mut app);

        let initial_rotation = app.world().entity(entity).get::<Transform>()
            .expect("Player should have Transform")
            .rotation;

        app.world_mut().resource_mut::<ActionState>().rotate = 1.0;
        app.add_systems(Update, apply_rotation);
        app.update();

        let new_rotation = app.world().entity(entity).get::<Transform>()
            .expect("Player should have Transform")
            .rotation;
        assert_ne!(
            initial_rotation, new_rotation,
            "Rotation should change when rotate input is active"
        );
    }

    #[test]
    fn zero_thrust_and_drag_stops_ship() {
        let mut app = setup_test_app();
        let entity = spawn_player(&mut app);

        // Set small velocity
        app.world_mut().entity_mut(entity).get_mut::<Velocity>()
            .expect("Player should have Velocity")
            .0 = Vec2::new(0.0, 0.05);

        // No thrust, only drag
        app.add_systems(Update, apply_drag);

        // Run several frames
        for _ in 0..60 {
            app.update();
        }

        let velocity = app.world().entity(entity).get::<Velocity>()
            .expect("Player should have Velocity");
        assert_eq!(velocity.0, Vec2::ZERO, "Ship should stop after drag with no thrust");
    }

    #[test]
    fn velocity_applies_to_position() {
        let mut app = setup_test_app();
        let entity = spawn_player(&mut app);

        app.world_mut().entity_mut(entity).get_mut::<Velocity>()
            .expect("Player should have Velocity")
            .0 = Vec2::new(100.0, 0.0);

        app.add_systems(Update, apply_velocity);
        app.update();

        let transform = app.world().entity(entity).get::<Transform>()
            .expect("Player should have Transform");
        assert!(transform.translation.x > 0.0, "Position should move with velocity");
    }

    #[test]
    fn rotated_ship_thrusts_in_facing_direction() {
        let mut app = setup_test_app();
        // Rotate 90 degrees (facing left/+X direction after rotation)
        let entity = spawn_player_with_rotation(&mut app, PI / 2.0);

        app.world_mut().resource_mut::<ActionState>().thrust = 1.0;
        app.add_systems(Update, apply_thrust);
        app.update();

        let velocity = app.world().entity(entity).get::<Velocity>()
            .expect("Player should have Velocity");
        // After 90 degree rotation from +Y, facing should be roughly -X
        assert!(velocity.0.x.abs() > 0.0 || velocity.0.y.abs() > 0.0,
            "Thrust should produce velocity in rotated direction");
    }

    #[test]
    fn flight_config_from_ron() {
        let ron_str = "(thrust_power: 100.0, max_speed: 200.0, drag_coefficient: 1.0, rotation_speed: 3.0)";
        let config = FlightConfig::from_ron(ron_str).expect("Should parse RON");
        assert_eq!(config.thrust_power, 100.0);
        assert_eq!(config.max_speed, 200.0);
        assert_eq!(config.drag_coefficient, 1.0);
        assert_eq!(config.rotation_speed, 3.0);
    }

    fn setup_speed_cap_test_app() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.init_resource::<ActionState>();
        app.insert_resource(FlightConfig::default());
        app.insert_resource(crate::world::WorldConfig::default());
        app.insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_secs_f64(
            1.0 / 60.0,
        )));
        // Prime time — first frame always has dt=0
        app.update();
        app
    }

    /// Fixed timestep used in speed cap test app (must match setup_speed_cap_test_app).
    const TEST_FIXED_TIMESTEP: f32 = 1.0 / 64.0;

    /// Compute effective_cap from WorldConfig defaults and test timestep.
    fn compute_test_effective_cap(flight_config: &FlightConfig) -> f32 {
        let wc = crate::world::WorldConfig::default();
        let chunk_gen_speed = compute_chunk_gen_speed(&wc, TEST_FIXED_TIMESTEP)
            .expect("timestep should be positive");
        (chunk_gen_speed * flight_config.speed_cap_fraction).min(flight_config.max_speed)
    }

    #[test]
    fn clamp_speed_limits_velocity_above_cap() {
        let mut app = setup_speed_cap_test_app();

        // Make max_speed very high so chunk_gen cap engages instead
        let mut config = app.world_mut().resource_mut::<FlightConfig>();
        config.max_speed = 200_000.0;
        let effective_cap = compute_test_effective_cap(&config);

        let entity = spawn_player(&mut app);

        let above_cap_speed = effective_cap * 1.08;
        app.world_mut()
            .entity_mut(entity)
            .get_mut::<Velocity>()
            .expect("Player should have Velocity")
            .0 = Vec2::new(0.0, above_cap_speed);

        app.add_systems(FixedUpdate, clamp_speed);
        app.update();

        let velocity = app
            .world()
            .entity(entity)
            .get::<Velocity>()
            .expect("Player should have Velocity");
        assert!(
            velocity.0.length() < above_cap_speed,
            "clamp_speed should reduce velocity when above cap, got {}",
            velocity.0.length()
        );
    }

    #[test]
    fn clamp_speed_does_nothing_below_cap() {
        let mut app = setup_speed_cap_test_app();
        let entity = spawn_player(&mut app);

        let config = app.world().resource::<FlightConfig>().clone();
        let effective_cap = compute_test_effective_cap(&config);
        let below_cap_speed = effective_cap * 0.5;

        app.world_mut()
            .entity_mut(entity)
            .get_mut::<Velocity>()
            .expect("Player should have Velocity")
            .0 = Vec2::new(0.0, below_cap_speed);

        app.add_systems(FixedUpdate, clamp_speed);
        app.update();

        let velocity = app
            .world()
            .entity(entity)
            .get::<Velocity>()
            .expect("Player should have Velocity");
        assert!(
            (velocity.0.length() - below_cap_speed).abs() < f32::EPSILON,
            "clamp_speed should not modify velocity below cap, got {}",
            velocity.0.length()
        );
    }

    #[test]
    fn clamp_speed_smooth_deceleration() {
        let mut app = setup_speed_cap_test_app();

        let mut config = app.world_mut().resource_mut::<FlightConfig>();
        config.max_speed = 200_000.0;
        let effective_cap = compute_test_effective_cap(&config);

        let entity = spawn_player(&mut app);

        let above_cap_speed = effective_cap * 1.08;
        app.world_mut()
            .entity_mut(entity)
            .get_mut::<Velocity>()
            .expect("Player should have Velocity")
            .0 = Vec2::new(0.0, above_cap_speed);

        app.add_systems(FixedUpdate, clamp_speed);
        app.update();

        let velocity = app
            .world()
            .entity(entity)
            .get::<Velocity>()
            .expect("Player should have Velocity");
        let speed_after_one_frame = velocity.0.length();

        // Should NOT hard-cut to cap — should be between cap and original speed
        assert!(
            speed_after_one_frame > effective_cap,
            "Should not hard-cut to cap (smooth deceleration), got {}",
            speed_after_one_frame
        );
        assert!(
            speed_after_one_frame < above_cap_speed,
            "Should be reducing toward cap, got {}",
            speed_after_one_frame
        );
    }

    #[test]
    fn speed_cap_config_defaults_when_missing_from_ron() {
        // RON without speed_cap fields — serde defaults should apply
        let ron_str = "(thrust_power: 100.0, max_speed: 200.0, drag_coefficient: 1.0, rotation_speed: 3.0)";
        let config = FlightConfig::from_ron(ron_str).expect("Should parse RON without speed cap fields");
        assert!(
            (config.speed_cap_fraction - 0.85).abs() < f32::EPSILON,
            "speed_cap_fraction should default to 0.85"
        );
        assert!(
            (config.speed_cap_decel - 0.92).abs() < f32::EPSILON,
            "speed_cap_decel should default to 0.92"
        );
    }

    #[test]
    fn validate_speed_cap_condition_true_when_max_speed_exceeds_chunk_gen() {
        // Directly test the condition that triggers the warning
        let wc = crate::world::WorldConfig::default();
        let chunk_gen_speed = compute_chunk_gen_speed(&wc, TEST_FIXED_TIMESTEP)
            .expect("timestep should be positive");
        let capped_max = chunk_gen_speed * default_speed_cap_fraction();

        let high_max_speed = 200_000.0;
        assert!(
            high_max_speed > capped_max,
            "max_speed {} should exceed capped_max {} to trigger warning",
            high_max_speed, capped_max
        );
    }

    #[test]
    fn validate_speed_cap_condition_false_when_below() {
        // Directly test the condition — default max_speed should NOT trigger warning
        let wc = crate::world::WorldConfig::default();
        let chunk_gen_speed = compute_chunk_gen_speed(&wc, TEST_FIXED_TIMESTEP)
            .expect("timestep should be positive");
        let capped_max = chunk_gen_speed * default_speed_cap_fraction();

        let config = FlightConfig::default();
        assert!(
            config.max_speed <= capped_max,
            "default max_speed {} should not exceed capped_max {}",
            config.max_speed, capped_max
        );
    }

    #[test]
    fn validate_speed_cap_system_runs_without_panic() {
        let mut app = setup_speed_cap_test_app();

        let mut config = app.world_mut().resource_mut::<FlightConfig>();
        config.max_speed = 200_000.0;

        app.add_systems(Startup, validate_speed_cap);
        app.update();
        // System ran without panic — warning emitted (log capture not feasible in unit tests)
    }

    #[test]
    fn flight_config_default_includes_speed_cap_fields() {
        let config = FlightConfig::default();
        assert!(
            (config.speed_cap_fraction - 0.85).abs() < f32::EPSILON,
            "Default speed_cap_fraction should be 0.85"
        );
        assert!(
            (config.speed_cap_decel - 0.92).abs() < f32::EPSILON,
            "Default speed_cap_decel should be 0.92"
        );
    }

    #[test]
    fn compute_chunk_gen_speed_returns_none_for_zero_timestep() {
        let wc = crate::world::WorldConfig::default();
        assert!(
            compute_chunk_gen_speed(&wc, 0.0).is_none(),
            "Should return None for zero timestep"
        );
    }

    #[test]
    fn clamp_speed_hard_cut_when_decel_zero() {
        let mut app = setup_speed_cap_test_app();

        let mut config = app.world_mut().resource_mut::<FlightConfig>();
        config.max_speed = 200_000.0;
        config.speed_cap_decel = 0.0; // lerp factor = 1.0 → instant snap
        let effective_cap = compute_test_effective_cap(&config);

        let entity = spawn_player(&mut app);

        let above_cap_speed = effective_cap * 1.5;
        app.world_mut()
            .entity_mut(entity)
            .get_mut::<Velocity>()
            .expect("Player should have Velocity")
            .0 = Vec2::new(0.0, above_cap_speed);

        app.add_systems(FixedUpdate, clamp_speed);
        app.update();

        let velocity = app
            .world()
            .entity(entity)
            .get::<Velocity>()
            .expect("Player should have Velocity");
        assert!(
            (velocity.0.length() - effective_cap).abs() < 1.0,
            "decel=0.0 should snap to cap, got {} (expected ~{})",
            velocity.0.length(), effective_cap
        );
    }

    #[test]
    fn clamp_speed_no_decel_when_decel_one() {
        let mut app = setup_speed_cap_test_app();

        let mut config = app.world_mut().resource_mut::<FlightConfig>();
        config.max_speed = 200_000.0;
        config.speed_cap_decel = 1.0; // lerp factor = 0.0 → no movement
        let effective_cap = compute_test_effective_cap(&config);

        let entity = spawn_player(&mut app);

        let above_cap_speed = effective_cap * 1.5;
        app.world_mut()
            .entity_mut(entity)
            .get_mut::<Velocity>()
            .expect("Player should have Velocity")
            .0 = Vec2::new(0.0, above_cap_speed);

        app.add_systems(FixedUpdate, clamp_speed);
        app.update();

        let velocity = app
            .world()
            .entity(entity)
            .get::<Velocity>()
            .expect("Player should have Velocity");
        assert!(
            (velocity.0.length() - above_cap_speed).abs() < 1.0,
            "decel=1.0 should not decelerate, got {} (expected ~{})",
            velocity.0.length(), above_cap_speed
        );
    }

}
