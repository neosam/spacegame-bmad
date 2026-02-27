pub mod camera;
pub mod collision;
pub mod flight;
pub mod input;
pub mod weapons;

use bevy::prelude::*;

use self::camera::camera_follow_player;
use self::collision::{
    apply_damage, check_laser_collisions, check_projectile_collisions, despawn_destroyed,
    DamageQueue,
};
use self::flight::{apply_drag, apply_rotation, apply_thrust, apply_velocity, FlightConfig};
use self::input::{read_input, ActionState};
use self::weapons::{
    fire_weapon, move_spread_projectiles, regenerate_energy, switch_weapon, tick_fire_cooldown,
    tick_laser_pulses, tick_spread_projectiles, LaserFired, SpreadFired, WeaponConfig,
};

/// System ordering within FixedUpdate. Prevents race conditions.
/// Input and Physics are active in Story 0.1.
/// Collision, Damage, Events reserved for future stories.
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub enum CoreSet {
    /// Input reading and action mapping
    Input,
    /// Flight physics: thrust, drag, rotation, velocity
    Physics,
    /// Collision detection (Story 0.5+)
    Collision,
    /// Damage application (Story 0.5+)
    Damage,
    /// Event emission (Epic 1+)
    Events,
}

pub struct CorePlugin;

impl Plugin for CorePlugin {
    fn build(&self, app: &mut App) {
        // Load FlightConfig from RON file with graceful fallback to defaults
        let config_path = "assets/config/flight.ron";
        let config = match std::fs::read_to_string(config_path) {
            Ok(contents) => match FlightConfig::from_ron(&contents) {
                Ok(config) => config,
                Err(e) => {
                    warn!("Failed to parse {config_path}: {e}. Using defaults.");
                    FlightConfig::default()
                }
            },
            Err(e) => {
                warn!("Failed to read {config_path}: {e}. Using defaults.");
                FlightConfig::default()
            }
        };

        app.insert_resource(config);
        app.init_resource::<ActionState>();

        // Load WeaponConfig from RON file with graceful fallback to defaults
        let weapon_config_path = "assets/config/weapons.ron";
        let weapon_config = match std::fs::read_to_string(weapon_config_path) {
            Ok(contents) => match WeaponConfig::from_ron(&contents) {
                Ok(cfg) => cfg,
                Err(e) => {
                    warn!("Failed to parse {weapon_config_path}: {e}. Using defaults.");
                    WeaponConfig::default()
                }
            },
            Err(e) => {
                warn!("Failed to read {weapon_config_path}: {e}. Using defaults.");
                WeaponConfig::default()
            }
        };
        app.insert_resource(weapon_config);
        app.add_message::<LaserFired>();
        app.add_message::<SpreadFired>();

        // Configure system ordering in FixedUpdate
        app.configure_sets(
            FixedUpdate,
            (
                CoreSet::Input,
                CoreSet::Physics,
                CoreSet::Collision,
                CoreSet::Damage,
                CoreSet::Events,
            )
                .chain(),
        );

        // Input reading in PreUpdate
        app.add_systems(PreUpdate, read_input);

        // Fire cooldown, energy regen, and weapon switching in Input set
        app.add_systems(
            FixedUpdate,
            (tick_fire_cooldown, regenerate_energy, switch_weapon).in_set(CoreSet::Input),
        );

        // Flight physics in FixedUpdate
        app.add_systems(
            FixedUpdate,
            (apply_thrust, apply_rotation, apply_drag, apply_velocity)
                .chain()
                .in_set(CoreSet::Physics),
        );

        // Weapon systems after Physics, before Collision
        app.add_systems(
            FixedUpdate,
            (
                fire_weapon,
                tick_laser_pulses,
                move_spread_projectiles,
                tick_spread_projectiles,
            )
                .chain()
                .after(CoreSet::Physics)
                .before(CoreSet::Collision),
        );

        // Collision detection in Collision set
        app.init_resource::<DamageQueue>();
        app.add_systems(
            FixedUpdate,
            (check_laser_collisions, check_projectile_collisions).in_set(CoreSet::Collision),
        );

        // Damage application in Damage set
        app.add_systems(
            FixedUpdate,
            (apply_damage, despawn_destroyed)
                .chain()
                .in_set(CoreSet::Damage),
        );

        // Camera follow in PostUpdate
        app.add_systems(PostUpdate, camera_follow_player);
    }
}
