use bevy::ecs::message::{Message, MessageWriter};
use bevy::prelude::*;
use serde::Deserialize;

use crate::core::flight::Player;
use crate::core::input::ActionState;

/// Weapon balance values loaded from `assets/config/weapons.ron`.
/// Derives Asset + TypePath for future AssetServer migration.
#[derive(Resource, Asset, Deserialize, TypePath, Clone, Debug)]
pub struct WeaponConfig {
    /// Laser pulses per second
    pub laser_fire_rate: f32,
    /// Max range in world units
    pub laser_range: f32,
    /// Damage per pulse (unused until Story 0.5)
    pub laser_damage: f32,
    /// Visual flash duration in seconds
    pub laser_pulse_duration: f32,
    /// Visual width of the laser line
    pub laser_width: f32,
}

impl Default for WeaponConfig {
    fn default() -> Self {
        Self {
            laser_fire_rate: 4.0,
            laser_range: 500.0,
            laser_damage: 10.0,
            laser_pulse_duration: 0.08,
            laser_width: 2.0,
        }
    }
}

impl WeaponConfig {
    /// Load config from RON string.
    pub fn from_ron(ron_str: &str) -> Result<Self, ron::error::SpannedError> {
        ron::from_str(ron_str)
    }
}

/// Hitscan laser pulse — purely visual flash along the firing line.
/// Origin, direction, and range stored for future collision detection (Story 0.5).
#[derive(Component)]
pub struct LaserPulse {
    pub origin: Vec2,
    pub direction: Vec2,
    pub range: f32,
    /// Remaining visual lifetime in seconds
    pub timer: f32,
}

/// Marker for laser entities that need their visual mesh spawned.
#[derive(Component)]
pub struct NeedsLaserVisual;

/// Fire cooldown on the player entity. Prevents firing faster than configured rate.
#[derive(Component, Default)]
pub struct FireCooldown {
    /// Seconds remaining until next fire allowed
    pub timer: f32,
}

/// Emitted when a laser pulse is fired. For future collision system subscription.
#[derive(Message)]
pub struct LaserFired {
    pub origin: Vec2,
    pub direction: Vec2,
    pub range: f32,
}

/// Decrements the fire cooldown timer each tick.
pub fn tick_fire_cooldown(time: Res<Time>, mut query: Query<&mut FireCooldown>) {
    let dt = time.delta_secs();
    for mut cooldown in query.iter_mut() {
        cooldown.timer = (cooldown.timer - dt).max(0.0);
    }
}

/// Fires a laser pulse when fire input is active and cooldown is ready.
/// Spawns a LaserPulse entity with Transform (no mesh — rendering handled separately).
pub fn fire_laser(
    action_state: Res<ActionState>,
    config: Res<WeaponConfig>,
    mut player_query: Query<(&Transform, &mut FireCooldown), With<Player>>,
    mut commands: Commands,
    mut laser_events: MessageWriter<LaserFired>,
) {
    if !action_state.fire {
        return;
    }

    for (transform, mut cooldown) in player_query.iter_mut() {
        if cooldown.timer > 0.0 {
            continue;
        }

        let facing = transform.rotation * Vec3::Y;
        let direction = Vec2::new(facing.x, facing.y).normalize_or_zero();
        let nose_offset = direction * 20.0;
        let origin = Vec2::new(transform.translation.x, transform.translation.y) + nose_offset;

        let midpoint = origin + direction * (config.laser_range / 2.0);
        let angle = direction.y.atan2(direction.x) - std::f32::consts::FRAC_PI_2;

        commands.spawn((
            LaserPulse {
                origin,
                direction,
                range: config.laser_range,
                timer: config.laser_pulse_duration,
            },
            NeedsLaserVisual,
            Transform::from_translation(midpoint.extend(0.0))
                .with_rotation(Quat::from_rotation_z(angle)),
        ));

        // Emit message for future collision system
        laser_events.write(LaserFired {
            origin,
            direction,
            range: config.laser_range,
        });

        // Reset cooldown
        cooldown.timer = 1.0 / config.laser_fire_rate;
    }
}

/// Decrements laser pulse timers and despawns expired pulses.
pub fn tick_laser_pulses(
    time: Res<Time>,
    mut commands: Commands,
    mut query: Query<(Entity, &mut LaserPulse)>,
) {
    let dt = time.delta_secs();
    for (entity, mut pulse) in query.iter_mut() {
        pulse.timer -= dt;
        if pulse.timer <= 0.0 {
            commands.entity(entity).despawn();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn weapon_config_default_has_valid_values() {
        let config = WeaponConfig::default();
        assert!(config.laser_fire_rate > 0.0);
        assert!(config.laser_range > 0.0);
        assert!(config.laser_damage > 0.0);
        assert!(config.laser_pulse_duration > 0.0);
        assert!(config.laser_width > 0.0);
    }

    #[test]
    fn weapon_config_from_ron() {
        let ron_str = "(laser_fire_rate: 5.0, laser_range: 600.0, laser_damage: 15.0, laser_pulse_duration: 0.1, laser_width: 3.0)";
        let config = WeaponConfig::from_ron(ron_str).expect("Should parse RON");
        assert_eq!(config.laser_fire_rate, 5.0);
        assert_eq!(config.laser_range, 600.0);
        assert_eq!(config.laser_damage, 15.0);
        assert_eq!(config.laser_pulse_duration, 0.1);
        assert_eq!(config.laser_width, 3.0);
    }

    #[test]
    fn fire_cooldown_decrements() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.insert_resource(bevy::time::TimeUpdateStrategy::ManualDuration(
            std::time::Duration::from_secs_f64(1.0 / 60.0),
        ));
        app.add_systems(Update, tick_fire_cooldown);
        app.update(); // prime

        let entity = app
            .world_mut()
            .spawn(FireCooldown { timer: 0.5 })
            .id();

        app.update();

        let cooldown = app
            .world()
            .entity(entity)
            .get::<FireCooldown>()
            .expect("Should have FireCooldown");
        assert!(
            cooldown.timer < 0.5,
            "Cooldown timer should decrease after update"
        );
        assert!(cooldown.timer > 0.0, "Cooldown should not be zero after one frame");
    }

    #[test]
    fn fire_cooldown_clamps_at_zero() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.insert_resource(bevy::time::TimeUpdateStrategy::ManualDuration(
            std::time::Duration::from_secs_f64(1.0 / 60.0),
        ));
        app.add_systems(Update, tick_fire_cooldown);
        app.update(); // prime

        let entity = app
            .world_mut()
            .spawn(FireCooldown { timer: 0.001 })
            .id();

        app.update();

        let cooldown = app
            .world()
            .entity(entity)
            .get::<FireCooldown>()
            .expect("Should have FireCooldown");
        assert_eq!(
            cooldown.timer, 0.0,
            "Cooldown timer should clamp to zero"
        );
    }
}
