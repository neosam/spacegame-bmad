use std::time::Duration;

use bevy::prelude::*;
use bevy::time::TimeUpdateStrategy;
use void_drifter::core::collision::{
    apply_damage, check_laser_collisions, check_projectile_collisions, despawn_destroyed,
    Collider, DamageQueue, Health,
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
use void_drifter::shared::components::Velocity;

/// Create a minimal test App with flight, weapon, collision, and damage systems.
/// Systems run in FixedUpdate to match production scheduling.
/// Uses fixed 1/60s time step for deterministic tests.
pub fn test_app() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.init_resource::<ActionState>();
    app.insert_resource(FlightConfig::default());
    app.insert_resource(WeaponConfig::default());
    app.init_resource::<DamageQueue>();
    app.add_message::<LaserFired>();
    app.add_message::<SpreadFired>();
    // Match production: flight systems in FixedUpdate
    app.add_systems(
        FixedUpdate,
        (apply_thrust, apply_rotation, apply_drag, apply_velocity).chain(),
    );
    // Weapon systems: cooldown tick, energy regen, switch, fire, pulse/projectile tick
    // Then collision detection, then damage application
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
            apply_damage,
            despawn_destroyed,
        )
            .chain(),
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
            Collider { radius },
            Health {
                current: health,
                max: health,
            },
            Transform::from_translation(position.extend(0.0)),
        ))
        .id()
}

/// Spawn a Scout Drone entity at the given position with a collider and health.
#[allow(dead_code)]
pub fn spawn_drone(app: &mut App, position: Vec2, radius: f32, health: f32) -> Entity {
    app.world_mut()
        .spawn((
            Collider { radius },
            Health {
                current: health,
                max: health,
            },
            Transform::from_translation(position.extend(0.0)),
        ))
        .id()
}

/// Spawn a player entity at the origin facing +Y with FireCooldown, Energy, and ActiveWeapon.
pub fn spawn_player(app: &mut App) -> Entity {
    app.world_mut()
        .spawn((
            Player,
            Velocity::default(),
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
            FireCooldown::default(),
            Energy::default(),
            ActiveWeapon::default(),
            Transform::default(),
        ))
        .id()
}

/// Set a player entity's active weapon to Spread.
#[allow(dead_code)]
pub fn set_active_weapon_spread(app: &mut App, entity: Entity) {
    *app.world_mut()
        .entity_mut(entity)
        .get_mut::<ActiveWeapon>()
        .expect("Player should have ActiveWeapon") = ActiveWeapon::Spread;
}
