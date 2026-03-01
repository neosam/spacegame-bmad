use std::time::Duration;

use bevy::prelude::*;
use bevy::time::TimeUpdateStrategy;
use void_drifter::core::collision::{
    apply_damage, check_contact_collisions, check_laser_collisions, check_projectile_collisions,
    despawn_destroyed, handle_player_death, tick_contact_cooldown, tick_invincibility, Collider,
    DamageQueue, DestroyedPositions, Health, LaserHitPositions,
};
use void_drifter::core::spawning::{
    drift_entities, spawn_respawn_timers, tick_respawn_timers, Asteroid, ScoutDrone, SpawningConfig,
};
use void_drifter::core::flight::{
    apply_drag, apply_rotation, apply_thrust, apply_velocity, FlightConfig, Player,
};
use void_drifter::core::input::ActionState;
use void_drifter::core::weapons::{
    fire_weapon, move_spread_projectiles, regenerate_energy, switch_weapon, tick_fire_cooldown,
    tick_laser_pulses, tick_spread_projectiles, ActiveWeapon, Energy, FireCooldown, LaserFired,
    SpreadFired, WeaponConfig,
};
use void_drifter::infrastructure::events::{record_game_events, EventSeverityConfig};
use void_drifter::infrastructure::logbook::Logbook;
use void_drifter::shared::components::Velocity;
use void_drifter::shared::events::GameEvent;
use void_drifter::rendering::minimap::{MinimapConfig, MinimapState};
use void_drifter::rendering::world_map::{WorldMapConfig, WorldMapOpen, WorldMapState};
use void_drifter::core::economy::{Credits, DiscoveredChunks};
use void_drifter::infrastructure::save::delta::{track_destroyed_entities, WorldDeltas};
use void_drifter::core::tutorial::{TutorialConfig, TutorialPhase};
use void_drifter::world::{
    update_chunks, ActiveChunks, BiomeConfig, ChunkEntityIndex, ChunkLoadState, ExploredChunks,
    PendingChunks, WorldConfig,
};

/// Create a minimal test App with flight, weapon, collision, and damage systems.
/// Systems run in FixedUpdate to match production scheduling.
/// Uses fixed 1/60s time step for deterministic tests.
pub fn test_app() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(bevy::state::app::StatesPlugin);
    app.init_resource::<ActionState>();
    app.insert_resource(FlightConfig::default());
    app.insert_resource(WeaponConfig::default());
    app.init_resource::<DamageQueue>();
    app.init_resource::<DestroyedPositions>();
    app.init_resource::<LaserHitPositions>();
    app.insert_resource(SpawningConfig::default());
    app.insert_resource(WorldConfig::default());
    app.insert_resource(BiomeConfig::default());
    app.insert_resource(MinimapConfig::default());
    app.init_resource::<MinimapState>();
    app.insert_resource(WorldMapConfig::default());
    app.init_resource::<WorldMapOpen>();
    app.init_resource::<WorldMapState>();
    app.init_resource::<ExploredChunks>();
    app.init_resource::<ChunkEntityIndex>();
    app.init_resource::<PendingChunks>();
    app.init_resource::<ChunkLoadState>();
    app.init_resource::<ButtonInput<KeyCode>>();
    app.init_resource::<ActiveChunks>();
    app.init_resource::<WorldDeltas>();
    app.init_resource::<Credits>();
    app.init_resource::<DiscoveredChunks>();
    app.insert_resource(TutorialConfig::default());
    app.init_state::<TutorialPhase>();
    app.add_message::<LaserFired>();
    app.add_message::<SpreadFired>();
    app.add_message::<GameEvent>();
    app.insert_resource(EventSeverityConfig::default());
    app.init_resource::<Logbook>();
    // Match production ordering: update_chunks runs before collision/damage chain
    app.add_systems(FixedUpdate, update_chunks);
    app.add_systems(
        FixedUpdate,
        (apply_thrust, apply_rotation, apply_drag, apply_velocity).chain().after(update_chunks),
    );
    // Weapon systems: cooldown tick, energy regen, switch, fire, pulse/projectile tick
    // Then collision detection, then damage application (with respawn timer creation)
    app.add_systems(
        FixedUpdate,
        (
            tick_fire_cooldown,
            regenerate_energy,
            switch_weapon,
            fire_weapon,
            tick_laser_pulses,
            move_spread_projectiles,
            tick_spread_projectiles,
            check_laser_collisions,
            check_projectile_collisions,
            check_contact_collisions,
            apply_damage,
            track_destroyed_entities,
            handle_player_death,
            spawn_respawn_timers,
            despawn_destroyed,
            tick_contact_cooldown,
            tick_invincibility,
            tick_respawn_timers,
            drift_entities,
            record_game_events,
        )
            .chain()
            .after(update_chunks),
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

/// Spawn an asteroid entity at the given position with a collider and health.
#[allow(dead_code)]
pub fn spawn_asteroid(app: &mut App, position: Vec2, radius: f32, health: f32) -> Entity {
    app.world_mut()
        .spawn((
            Asteroid,
            Collider { radius },
            Health {
                current: health,
                max: health,
            },
            Velocity::default(),
            Transform::from_translation(position.extend(0.0)),
        ))
        .id()
}

/// Spawn a Scout Drone entity at the given position with a collider and health.
#[allow(dead_code)]
pub fn spawn_drone(app: &mut App, position: Vec2, radius: f32, health: f32) -> Entity {
    app.world_mut()
        .spawn((
            ScoutDrone,
            Collider { radius },
            Health {
                current: health,
                max: health,
            },
            Velocity::default(),
            Transform::from_translation(position.extend(0.0)),
        ))
        .id()
}

/// Run enough frames to fully load all chunks for a given config (staggered loading).
#[allow(dead_code)]
pub fn run_until_loaded(app: &mut App) {
    let config = app.world().resource::<WorldConfig>().clone();
    let total_chunks = (2 * config.load_radius + 1).pow(2) as usize;
    let frames = total_chunks.div_ceil(config.max_chunks_per_frame);
    for _ in 0..frames {
        app.update();
    }
}

/// Spawn a player entity at the origin facing +Y with FireCooldown, Energy, ActiveWeapon, Health, and Collider.
pub fn spawn_player(app: &mut App) -> Entity {
    app.world_mut()
        .spawn((
            Player,
            Velocity::default(),
            Health {
                current: 100.0,
                max: 100.0,
            },
            Collider { radius: 12.0 },
            FireCooldown::default(),
            Energy::default(),
            ActiveWeapon::default(),
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
            Health {
                current: 100.0,
                max: 100.0,
            },
            Collider { radius: 12.0 },
            FireCooldown::default(),
            Energy::default(),
            ActiveWeapon::default(),
            Transform::default(),
        ))
        .id()
}

/// Set a player entity's active weapon to Spread (also inserts SpreadUnlocked).
#[allow(dead_code)]
pub fn set_active_weapon_spread(app: &mut App, entity: Entity) {
    app.world_mut()
        .entity_mut(entity)
        .insert(void_drifter::core::tutorial::SpreadUnlocked);
    *app.world_mut()
        .entity_mut(entity)
        .get_mut::<ActiveWeapon>()
        .expect("Player should have ActiveWeapon") = ActiveWeapon::Spread;
}
