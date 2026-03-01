pub mod camera;
pub mod collision;
pub mod flight;
pub mod input;
pub mod spawning;
pub mod tutorial;
pub mod weapons;

use bevy::prelude::*;

use self::camera::camera_follow_player;
use self::collision::{
    apply_damage, check_contact_collisions, check_laser_collisions, check_projectile_collisions,
    despawn_destroyed, handle_player_death, tick_contact_cooldown, tick_invincibility, DamageQueue,
    DestroyedPositions, LaserHitPositions,
};
use self::flight::{apply_drag, apply_rotation, apply_thrust, apply_velocity, clamp_speed, validate_speed_cap, FlightConfig};
use self::input::{read_input, ActionState};
use self::spawning::{
    drift_entities, spawn_respawn_timers, tick_respawn_timers,
    SpawningConfig,
};
use self::tutorial::{advance_phase_on_wreck_shot, apply_gravity_well, spawn_tutorial_zone, update_weapons_lock, TutorialConfig, TutorialPhase};
use self::weapons::{
    fire_weapon, move_spread_projectiles, regenerate_energy, switch_weapon, tick_fire_cooldown,
    tick_laser_pulses, tick_spread_projectiles, LaserFired, SpreadFired, WeaponConfig,
};

/// System ordering within FixedUpdate. Prevents race conditions.
/// Input → Physics → Collision → Damage → Events (chained).
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

        // Load SpawningConfig from RON file with graceful fallback to defaults
        let spawning_config_path = "assets/config/spawning.ron";
        let spawning_config = match std::fs::read_to_string(spawning_config_path) {
            Ok(contents) => match SpawningConfig::from_ron(&contents) {
                Ok(cfg) => cfg,
                Err(e) => {
                    warn!("Failed to parse {spawning_config_path}: {e}. Using defaults.");
                    SpawningConfig::default()
                }
            },
            Err(e) => {
                warn!("Failed to read {spawning_config_path}: {e}. Using defaults.");
                SpawningConfig::default()
            }
        };
        app.insert_resource(spawning_config);

        // Load TutorialConfig from RON file with graceful fallback to defaults
        let tutorial_config_path = "assets/config/tutorial.ron";
        let tutorial_config = match std::fs::read_to_string(tutorial_config_path) {
            Ok(contents) => match TutorialConfig::from_ron(&contents) {
                Ok(cfg) => cfg,
                Err(e) => {
                    warn!("Failed to parse {tutorial_config_path}: {e}. Using defaults.");
                    TutorialConfig::default()
                }
            },
            Err(e) => {
                warn!("Failed to read {tutorial_config_path}: {e}. Using defaults.");
                TutorialConfig::default()
            }
        };
        app.insert_resource(tutorial_config);

        // Tutorial phase state machine
        app.init_state::<TutorialPhase>();

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

        // Note: spawn_initial_entities removed — chunk-based spawning via WorldPlugin replaces it

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

        // Tutorial zone spawn
        app.add_systems(Startup, spawn_tutorial_zone);

        // Startup validation: warn if max_speed exceeds chunk generation capacity
        app.add_systems(Startup, validate_speed_cap);

        // Gravity well pull after Physics, before Collision
        app.add_systems(
            FixedUpdate,
            apply_gravity_well
                .after(CoreSet::Physics)
                .before(CoreSet::Collision),
        );

        // Speed clamping after Physics, before Collision
        app.add_systems(
            FixedUpdate,
            clamp_speed
                .after(CoreSet::Physics)
                .before(CoreSet::Collision),
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
        app.init_resource::<DestroyedPositions>();
        app.init_resource::<LaserHitPositions>();
        app.add_systems(
            FixedUpdate,
            (
                check_laser_collisions,
                check_projectile_collisions,
                check_contact_collisions,
            )
                .in_set(CoreSet::Collision),
        );

        // Damage application in Damage set (chain: apply → wreck phase → player death → respawn timers → despawn)
        app.add_systems(
            FixedUpdate,
            (
                apply_damage,
                advance_phase_on_wreck_shot,
                handle_player_death,
                spawn_respawn_timers,
                despawn_destroyed,
            )
                .chain()
                .in_set(CoreSet::Damage),
        );

        // Post-damage systems: cooldown tick, invincibility tick, respawn tick, drift
        app.add_systems(
            FixedUpdate,
            (
                tick_contact_cooldown,
                tick_invincibility,
                tick_respawn_timers,
                drift_entities,
            )
                .after(CoreSet::Damage),
        );

        // Tutorial weapon lock system
        app.add_systems(FixedUpdate, update_weapons_lock.before(CoreSet::Input));

        // Camera follow in PostUpdate
        app.add_systems(PostUpdate, camera_follow_player);
    }
}
