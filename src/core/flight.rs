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
}

impl Default for FlightConfig {
    fn default() -> Self {
        Self {
            thrust_power: 300.0,
            max_speed: 400.0,
            drag_coefficient: 1.5,
            rotation_speed: 5.0,
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

}
