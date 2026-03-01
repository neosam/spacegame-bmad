pub mod camera;
pub mod collision;
pub mod economy;
pub mod flight;
pub mod input;
pub mod spawning;
pub mod station;
pub mod tutorial;
pub mod upgrades;
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
    drift_entities, spawn_respawn_timers, tick_respawn_timers, update_trader_ships,
    SpawningConfig,
};
use self::economy::{
    award_credits_on_discovery, award_credits_on_kill, emit_credit_events,
    on_player_death_deduct_credits,
    collect_material_drops, emit_pickup_events, queue_material_drops, spawn_material_drops,
    spawn_boss_loot,
    Credits, DiscoveredChunks, PendingCreditEvents,
    PlayerInventory, PendingDropSpawns, PendingPickupEvents,
};
use self::station::{record_discovered_stations, update_docking, update_undocking, DiscoveredStations, LastDockedStation};
use self::upgrades::{
    apply_upgrade_effects, emit_craft_events, handle_craft_input, init_base_stats,
    mark_player_needs_upgrade_visual, navigate_station_ui, process_crafting_request, CraftingRequest,
    DiscoveredRecipes, InstalledUpgrades, PendingCraftEvents, StationUiState,
};
use self::tutorial::{
    advance_phase_on_wreck_shot, apply_gravity_well, check_generator_destroyed,
    check_tutorial_wave_complete, dock_at_station, spawn_tutorial_enemies, spawn_tutorial_zone,
    start_destruction_cascade, tick_cascade_timer, unlock_laser_at_wreck, update_weapons_lock,
    validate_tutorial_config, TutorialConfig, TutorialPhase,
};
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

        // Startup validation: warn if any TutorialConfig constraint is violated
        app.add_systems(Startup, validate_tutorial_config);

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

        // Damage application in Damage set (chain: apply → wreck phase → player death → respawn timers → despawn → wave check)
        app.add_systems(
            FixedUpdate,
            (
                apply_damage,
                advance_phase_on_wreck_shot,
                handle_player_death,
                spawn_respawn_timers,
                despawn_destroyed,
                check_tutorial_wave_complete,
            )
                .chain()
                .in_set(CoreSet::Damage),
        );

        // Tutorial enemy wave spawn: fires once when SpreadUnlocked phase is entered
        app.add_systems(OnEnter(TutorialPhase::SpreadUnlocked), spawn_tutorial_enemies);

        // Destruction cascade: fires once on entering GeneratorDestroyed phase
        app.add_systems(
            OnEnter(TutorialPhase::GeneratorDestroyed),
            start_destruction_cascade,
        );

        // Post-damage systems: cooldown tick, invincibility tick, respawn tick, drift, traders
        app.add_systems(
            FixedUpdate,
            (
                tick_contact_cooldown,
                tick_invincibility,
                tick_respawn_timers,
                drift_entities,
                update_trader_ships,
            )
                .after(CoreSet::Damage),
        );

        // Station docking/undocking: runs in CoreSet::Events (update_docking before update_undocking)
        app.add_systems(
            FixedUpdate,
            (update_docking, update_undocking)
                .chain()
                .in_set(CoreSet::Events),
        );

        // Record station positions as they are spawned (persists across chunk unloads)
        app.init_resource::<DiscoveredStations>();
        app.init_resource::<LastDockedStation>();
        app.add_systems(Update, record_discovered_stations);

        // Flying → Shooting trigger: runs in CoreSet::Events, fires once when player reaches wreck
        app.add_systems(FixedUpdate, unlock_laser_at_wreck.in_set(CoreSet::Events));

        // Station docking: runs in CoreSet::Events, after Damage has settled
        app.add_systems(FixedUpdate, dock_at_station.in_set(CoreSet::Events));

        // Generator destruction detection: runs in CoreSet::Events after dock_at_station
        app.add_systems(
            FixedUpdate,
            check_generator_destroyed
                .in_set(CoreSet::Events)
                .after(dock_at_station),
        );

        // Cascade timer tick: runs in CoreSet::Events after check_generator_destroyed
        app.add_systems(
            FixedUpdate,
            tick_cascade_timer
                .in_set(CoreSet::Events)
                .after(check_generator_destroyed),
        );

        // Tutorial weapon lock system
        app.add_systems(FixedUpdate, update_weapons_lock.before(CoreSet::Input));

        // Credits economy: track kills and chunk discoveries
        app.init_resource::<Credits>();
        app.init_resource::<DiscoveredChunks>();
        app.init_resource::<PendingCreditEvents>();
        app.add_systems(
            FixedUpdate,
            (award_credits_on_kill, award_credits_on_discovery, emit_credit_events)
                .chain()
                .after(CoreSet::Events),
        );
        // Death penalty: deduct 10% of credits on PlayerDeath
        app.add_systems(
            FixedUpdate,
            on_player_death_deduct_credits.after(CoreSet::Events),
        );

        // Material drops: spawn on kill, collect on proximity
        app.init_resource::<PlayerInventory>();
        app.init_resource::<PendingDropSpawns>();
        app.init_resource::<PendingPickupEvents>();
        app.add_systems(
            FixedUpdate,
            (queue_material_drops, spawn_boss_loot, spawn_material_drops, collect_material_drops, emit_pickup_events)
                .chain()
                .after(CoreSet::Events),
        );

        // Upgrade system: resources and base-stat initialization
        app.init_resource::<InstalledUpgrades>();
        app.init_resource::<DiscoveredRecipes>();
        app.init_resource::<CraftingRequest>();
        app.init_resource::<PendingCraftEvents>();
        app.init_resource::<StationUiState>();
        // BaseStats must be initialized AFTER FlightConfig + WeaponConfig are inserted
        app.add_systems(Startup, init_base_stats);
        // Station UI navigation and craft input: runs in Input set so craft request is ready before processing
        app.add_systems(
            FixedUpdate,
            (navigate_station_ui, handle_craft_input)
                .chain()
                .in_set(CoreSet::Input),
        );
        // Upgrade effect application and event emission run after all economy events
        app.add_systems(
            FixedUpdate,
            (
                process_crafting_request,
                apply_upgrade_effects,
                mark_player_needs_upgrade_visual,
                emit_craft_events,
            )
                .chain()
                .after(CoreSet::Events),
        );

        // Camera follow in PostUpdate
        app.add_systems(PostUpdate, camera_follow_player);
    }
}
