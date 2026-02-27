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
    /// Damage per laser pulse
    pub laser_damage: f32,
    /// Visual flash duration in seconds
    pub laser_pulse_duration: f32,
    /// Visual width of the laser line
    pub laser_width: f32,
    /// Maximum energy capacity
    pub energy_max: f32,
    /// Energy regenerated per second
    pub energy_regen_rate: f32,
    /// Energy consumed per spread shot
    pub spread_energy_cost: f32,
    /// Number of projectiles per spread shot
    pub spread_projectile_count: u32,
    /// Total arc width in degrees
    pub spread_arc_degrees: f32,
    /// Projectile speed in world units/sec
    pub spread_projectile_speed: f32,
    /// Projectile lifetime in seconds
    pub spread_projectile_lifetime: f32,
    /// Damage per spread projectile
    pub spread_damage: f32,
    /// Spread fire rate in shots per second
    pub spread_fire_rate: f32,
}

impl Default for WeaponConfig {
    fn default() -> Self {
        Self {
            laser_fire_rate: 4.0,
            laser_range: 500.0,
            laser_damage: 10.0,
            laser_pulse_duration: 0.08,
            laser_width: 2.0,
            energy_max: 100.0,
            energy_regen_rate: 15.0,
            spread_energy_cost: 20.0,
            spread_projectile_count: 5,
            spread_arc_degrees: 30.0,
            spread_projectile_speed: 600.0,
            spread_projectile_lifetime: 0.8,
            spread_damage: 5.0,
            spread_fire_rate: 2.0,
        }
    }
}

impl WeaponConfig {
    /// Load config from RON string.
    pub fn from_ron(ron_str: &str) -> Result<Self, ron::error::SpannedError> {
        ron::from_str(ron_str)
    }
}

/// Player energy bar for powering spread weapon.
#[derive(Component)]
pub struct Energy {
    pub current: f32,
    pub max_capacity: f32,
}

impl Default for Energy {
    fn default() -> Self {
        Self {
            current: 100.0,
            max_capacity: 100.0,
        }
    }
}

/// Which weapon the player currently fires on input.
#[derive(Component, Default, Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActiveWeapon {
    #[default]
    Laser,
    Spread,
}

/// Hitscan laser pulse — purely visual flash along the firing line.
/// Origin, direction, and range used by collision detection.
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

/// Spread projectile — physical entity that moves through space.
/// Checked for circle-circle collision each frame.
#[derive(Component)]
pub struct SpreadProjectile {
    pub origin: Vec2,
    pub direction: Vec2,
    pub speed: f32,
    pub damage: f32,
    /// Remaining lifetime in seconds
    pub timer: f32,
}

/// Marker for projectile entities that need their visual mesh spawned.
#[derive(Component)]
pub struct NeedsProjectileVisual;

/// Fire cooldown on the player entity. Prevents firing faster than configured rate.
#[derive(Component, Default)]
pub struct FireCooldown {
    /// Seconds remaining until next fire allowed
    pub timer: f32,
}

/// Emitted when a laser pulse is fired. Read by collision detection.
#[derive(Message)]
pub struct LaserFired {
    pub origin: Vec2,
    pub direction: Vec2,
    pub range: f32,
}

/// Emitted when spread projectiles are fired.
#[derive(Message)]
pub struct SpreadFired {
    pub origin: Vec2,
    pub direction: Vec2,
    pub count: u32,
}

/// Decrements the fire cooldown timer each tick.
pub fn tick_fire_cooldown(time: Res<Time>, mut query: Query<&mut FireCooldown>) {
    let dt = time.delta_secs();
    for mut cooldown in query.iter_mut() {
        cooldown.timer = (cooldown.timer - dt).max(0.0);
    }
}

/// Regenerates player energy over time, clamped at max capacity.
pub fn regenerate_energy(
    time: Res<Time>,
    config: Res<WeaponConfig>,
    mut query: Query<&mut Energy>,
) {
    let dt = time.delta_secs();
    for mut energy in query.iter_mut() {
        energy.current = (energy.current + config.energy_regen_rate * dt).min(energy.max_capacity);
    }
}

/// Fires the active weapon when fire input is active and cooldown is ready.
/// Branches on ActiveWeapon: Laser (hitscan) or Spread (projectiles with energy cost).
pub fn fire_weapon(
    action_state: Res<ActionState>,
    config: Res<WeaponConfig>,
    mut player_query: Query<
        (
            &Transform,
            &mut FireCooldown,
            &mut Energy,
            &ActiveWeapon,
        ),
        With<Player>,
    >,
    mut commands: Commands,
    mut laser_events: MessageWriter<LaserFired>,
    mut spread_events: MessageWriter<SpreadFired>,
) {
    if !action_state.fire {
        return;
    }

    for (transform, mut cooldown, mut energy, active_weapon) in player_query.iter_mut() {
        if cooldown.timer > 0.0 {
            continue;
        }

        let facing = transform.rotation * Vec3::Y;
        let direction = Vec2::new(facing.x, facing.y).normalize_or_zero();
        let nose_offset = direction * 20.0;
        let origin =
            Vec2::new(transform.translation.x, transform.translation.y) + nose_offset;

        match active_weapon {
            ActiveWeapon::Laser => {
                let midpoint = origin + direction * (config.laser_range / 2.0);
                let angle =
                    direction.y.atan2(direction.x) - std::f32::consts::FRAC_PI_2;

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

                laser_events.write(LaserFired {
                    origin,
                    direction,
                    range: config.laser_range,
                });

                cooldown.timer = 1.0 / config.laser_fire_rate;
            }
            ActiveWeapon::Spread => {
                if energy.current < config.spread_energy_cost {
                    continue;
                }
                energy.current -= config.spread_energy_cost;

                let arc_rad = config.spread_arc_degrees.to_radians();
                let count = config.spread_projectile_count;
                let step = if count > 1 {
                    arc_rad / (count - 1) as f32
                } else {
                    0.0
                };
                let start_angle = -arc_rad / 2.0;

                for i in 0..count {
                    let offset_angle = start_angle + step * i as f32;
                    let cos = offset_angle.cos();
                    let sin = offset_angle.sin();
                    let proj_direction = Vec2::new(
                        direction.x * cos - direction.y * sin,
                        direction.x * sin + direction.y * cos,
                    );

                    commands.spawn((
                        SpreadProjectile {
                            origin,
                            direction: proj_direction,
                            speed: config.spread_projectile_speed,
                            damage: config.spread_damage,
                            timer: config.spread_projectile_lifetime,
                        },
                        NeedsProjectileVisual,
                        Transform::from_translation(origin.extend(0.0)),
                    ));
                }

                spread_events.write(SpreadFired {
                    origin,
                    direction,
                    count,
                });

                cooldown.timer = 1.0 / config.spread_fire_rate;
            }
        }
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

/// Moves spread projectiles along their direction each tick.
pub fn move_spread_projectiles(
    time: Res<Time>,
    mut query: Query<(&SpreadProjectile, &mut Transform)>,
) {
    let dt = time.delta_secs();
    for (projectile, mut transform) in query.iter_mut() {
        transform.translation.x += projectile.direction.x * projectile.speed * dt;
        transform.translation.y += projectile.direction.y * projectile.speed * dt;
    }
}

/// Decrements spread projectile timers and despawns expired projectiles.
pub fn tick_spread_projectiles(
    time: Res<Time>,
    mut commands: Commands,
    mut query: Query<(Entity, &mut SpreadProjectile)>,
) {
    let dt = time.delta_secs();
    for (entity, mut projectile) in query.iter_mut() {
        projectile.timer -= dt;
        if projectile.timer <= 0.0 {
            commands.entity(entity).despawn();
        }
    }
}

/// Toggles the player's active weapon when switch input is active.
pub fn switch_weapon(
    action_state: Res<ActionState>,
    mut query: Query<&mut ActiveWeapon, With<Player>>,
) {
    if !action_state.switch_weapon {
        return;
    }
    for mut weapon in query.iter_mut() {
        *weapon = match *weapon {
            ActiveWeapon::Laser => ActiveWeapon::Spread,
            ActiveWeapon::Spread => ActiveWeapon::Laser,
        };
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
        assert!(config.energy_max > 0.0);
        assert!(config.energy_regen_rate > 0.0);
        assert!(config.spread_energy_cost > 0.0);
        assert!(config.spread_projectile_count > 0);
        assert!(config.spread_arc_degrees > 0.0);
        assert!(config.spread_projectile_speed > 0.0);
        assert!(config.spread_projectile_lifetime > 0.0);
        assert!(config.spread_damage > 0.0);
        assert!(config.spread_fire_rate > 0.0);
    }

    #[test]
    fn weapon_config_from_ron() {
        let ron_str = "(laser_fire_rate: 5.0, laser_range: 600.0, laser_damage: 15.0, laser_pulse_duration: 0.1, laser_width: 3.0, energy_max: 200.0, energy_regen_rate: 20.0, spread_energy_cost: 25.0, spread_projectile_count: 7, spread_arc_degrees: 45.0, spread_projectile_speed: 700.0, spread_projectile_lifetime: 1.0, spread_damage: 8.0, spread_fire_rate: 3.0)";
        let config = WeaponConfig::from_ron(ron_str).expect("Should parse RON");
        assert_eq!(config.laser_fire_rate, 5.0);
        assert_eq!(config.laser_range, 600.0);
        assert_eq!(config.energy_max, 200.0);
        assert_eq!(config.spread_projectile_count, 7);
        assert_eq!(config.spread_arc_degrees, 45.0);
        assert_eq!(config.spread_fire_rate, 3.0);
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

    #[test]
    fn energy_regenerates() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.insert_resource(WeaponConfig::default());
        app.insert_resource(bevy::time::TimeUpdateStrategy::ManualDuration(
            std::time::Duration::from_secs_f64(1.0 / 60.0),
        ));
        app.add_systems(Update, regenerate_energy);
        app.update(); // prime

        let entity = app
            .world_mut()
            .spawn(Energy {
                current: 50.0,
                max_capacity: 100.0,
            })
            .id();

        app.update();

        let energy = app
            .world()
            .entity(entity)
            .get::<Energy>()
            .expect("Should have Energy");
        assert!(
            energy.current > 50.0,
            "Energy should regenerate, got {}",
            energy.current
        );
    }

    #[test]
    fn switch_weapon_toggles_laser_to_spread() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.init_resource::<ActionState>();
        app.add_systems(Update, switch_weapon);

        let entity = app
            .world_mut()
            .spawn((Player, ActiveWeapon::Laser))
            .id();

        // Set switch_weapon input
        app.world_mut().resource_mut::<ActionState>().switch_weapon = true;

        app.update();

        let weapon = app
            .world()
            .entity(entity)
            .get::<ActiveWeapon>()
            .expect("Should have ActiveWeapon");
        assert_eq!(
            *weapon,
            ActiveWeapon::Spread,
            "Should toggle from Laser to Spread"
        );
    }

    #[test]
    fn switch_weapon_toggles_spread_to_laser() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.init_resource::<ActionState>();
        app.add_systems(Update, switch_weapon);

        let entity = app
            .world_mut()
            .spawn((Player, ActiveWeapon::Spread))
            .id();

        // Set switch_weapon input
        app.world_mut().resource_mut::<ActionState>().switch_weapon = true;

        app.update();

        let weapon = app
            .world()
            .entity(entity)
            .get::<ActiveWeapon>()
            .expect("Should have ActiveWeapon");
        assert_eq!(
            *weapon,
            ActiveWeapon::Laser,
            "Should toggle from Spread to Laser"
        );
    }

    #[test]
    fn switch_weapon_no_input_keeps_current() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.init_resource::<ActionState>();
        app.add_systems(Update, switch_weapon);

        let entity = app
            .world_mut()
            .spawn((Player, ActiveWeapon::Laser))
            .id();

        // switch_weapon defaults to false — no toggle
        app.update();

        let weapon = app
            .world()
            .entity(entity)
            .get::<ActiveWeapon>()
            .expect("Should have ActiveWeapon");
        assert_eq!(
            *weapon,
            ActiveWeapon::Laser,
            "Should remain Laser when no switch input"
        );
    }

    #[test]
    fn energy_clamps_at_max() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.insert_resource(WeaponConfig::default());
        app.insert_resource(bevy::time::TimeUpdateStrategy::ManualDuration(
            std::time::Duration::from_secs_f64(1.0 / 60.0),
        ));
        app.add_systems(Update, regenerate_energy);
        app.update(); // prime

        let entity = app
            .world_mut()
            .spawn(Energy {
                current: 100.0,
                max_capacity: 100.0,
            })
            .id();

        app.update();

        let energy = app
            .world()
            .entity(entity)
            .get::<Energy>()
            .expect("Should have Energy");
        assert_eq!(
            energy.current, 100.0,
            "Energy should clamp at max capacity"
        );
    }
}
