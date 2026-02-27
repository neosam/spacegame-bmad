use bevy::prelude::*;
use serde::Deserialize;

use crate::core::collision::{Collider, Health};
use crate::core::flight::Player;
use crate::shared::components::Velocity;

// ── Marker Components ───────────────────────────────────────────────────

/// Marker component for asteroid entities.
#[derive(Component)]
pub struct Asteroid;

/// Marker component for Scout Drone entities.
#[derive(Component)]
pub struct ScoutDrone;

/// Marker for asteroid entities that need their visual mesh attached by RenderingPlugin.
#[derive(Component)]
pub struct NeedsAsteroidVisual;

/// Marker for drone entities that need their visual mesh attached by RenderingPlugin.
#[derive(Component)]
pub struct NeedsDroneVisual;

// ── Respawn ─────────────────────────────────────────────────────────────

/// Which type of entity to respawn.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpawnType {
    Asteroid,
    ScoutDrone,
}

/// Timer entity that counts down and then spawns a replacement entity.
#[derive(Component, Debug)]
pub struct RespawnTimer {
    pub timer: f32,
    pub spawn_type: SpawnType,
    pub position: Vec2,
}

// ── Config ──────────────────────────────────────────────────────────────

/// Spawn point definition in the RON config.
#[derive(Deserialize, Clone, Debug)]
pub struct SpawnPoint {
    pub x: f32,
    pub y: f32,
}

/// Spawning balance values loaded from `assets/config/spawning.ron`.
#[derive(Resource, Deserialize, Clone, Debug)]
pub struct SpawningConfig {
    pub asteroid_positions: Vec<SpawnPoint>,
    pub drone_positions: Vec<SpawnPoint>,
    pub asteroid_health: f32,
    pub asteroid_radius: f32,
    pub drone_health: f32,
    pub drone_radius: f32,
    pub respawn_delay: f32,
    pub asteroid_velocity_min: f32,
    pub asteroid_velocity_max: f32,
    pub drone_velocity_min: f32,
    pub drone_velocity_max: f32,
}

impl Default for SpawningConfig {
    fn default() -> Self {
        Self {
            asteroid_positions: vec![
                SpawnPoint { x: 150.0, y: 100.0 },
                SpawnPoint { x: -200.0, y: 50.0 },
                SpawnPoint { x: 100.0, y: -180.0 },
                SpawnPoint { x: -120.0, y: -150.0 },
            ],
            drone_positions: vec![
                SpawnPoint { x: 250.0, y: 200.0 },
                SpawnPoint { x: -180.0, y: 220.0 },
            ],
            asteroid_health: 50.0,
            asteroid_radius: 20.0,
            drone_health: 30.0,
            drone_radius: 10.0,
            respawn_delay: 5.0,
            asteroid_velocity_min: 5.0,
            asteroid_velocity_max: 15.0,
            drone_velocity_min: 10.0,
            drone_velocity_max: 25.0,
        }
    }
}

impl SpawningConfig {
    /// Load config from RON string.
    pub fn from_ron(ron_str: &str) -> Result<Self, ron::error::SpannedError> {
        ron::from_str(ron_str)
    }
}

// ── Systems ─────────────────────────────────────────────────────────────

/// Generates a random velocity vector with magnitude between min and max.
fn random_velocity(min: f32, max: f32) -> Vec2 {
    let angle = rand::random::<f32>() * std::f32::consts::TAU;
    let speed = min + rand::random::<f32>() * (max - min);
    Vec2::new(angle.cos() * speed, angle.sin() * speed)
}

/// Startup system: reads `SpawningConfig` and spawns initial asteroids and drones.
pub fn spawn_initial_entities(mut commands: Commands, config: Res<SpawningConfig>) {
    // Spawn asteroids
    for pos in &config.asteroid_positions {
        let velocity = random_velocity(config.asteroid_velocity_min, config.asteroid_velocity_max);
        commands.spawn((
            Asteroid,
            NeedsAsteroidVisual,
            Collider {
                radius: config.asteroid_radius,
            },
            Health {
                current: config.asteroid_health,
                max: config.asteroid_health,
            },
            Velocity(velocity),
            Transform::from_translation(Vec3::new(pos.x, pos.y, 0.0)),
        ));
    }

    // Spawn drones
    for pos in &config.drone_positions {
        let velocity = random_velocity(config.drone_velocity_min, config.drone_velocity_max);
        commands.spawn((
            ScoutDrone,
            NeedsDroneVisual,
            Collider {
                radius: config.drone_radius,
            },
            Health {
                current: config.drone_health,
                max: config.drone_health,
            },
            Velocity(velocity),
            Transform::from_translation(Vec3::new(pos.x, pos.y, 0.0)),
        ));
    }
}

/// Watches for destroyed Asteroid/ScoutDrone entities and spawns RespawnTimer entities.
/// Runs BEFORE despawn_destroyed in the Damage chain so we can detect entities with
/// health <= 0 and record their position and spawn type before they are removed.
#[allow(clippy::type_complexity)]
pub fn spawn_respawn_timers(
    mut commands: Commands,
    config: Res<SpawningConfig>,
    query: Query<
        (&Health, &Transform, Option<&Asteroid>, Option<&ScoutDrone>),
        (Without<Player>, Or<(With<Asteroid>, With<ScoutDrone>)>),
    >,
) {
    for (health, transform, asteroid, drone) in query.iter() {
        if health.current <= 0.0 {
            let position = Vec2::new(transform.translation.x, transform.translation.y);
            let spawn_type = if asteroid.is_some() {
                SpawnType::Asteroid
            } else if drone.is_some() {
                SpawnType::ScoutDrone
            } else {
                continue;
            };

            commands.spawn(RespawnTimer {
                timer: config.respawn_delay,
                spawn_type,
                position,
            });
        }
    }
}

/// Ticks respawn timers and spawns new entities when expired.
pub fn tick_respawn_timers(
    mut commands: Commands,
    time: Res<Time>,
    config: Res<SpawningConfig>,
    mut query: Query<(Entity, &mut RespawnTimer)>,
) {
    let dt = time.delta_secs();
    for (entity, mut timer) in query.iter_mut() {
        timer.timer -= dt;
        if timer.timer <= 0.0 {
            match timer.spawn_type {
                SpawnType::Asteroid => {
                    let velocity = random_velocity(
                        config.asteroid_velocity_min,
                        config.asteroid_velocity_max,
                    );
                    commands.spawn((
                        Asteroid,
                        NeedsAsteroidVisual,
                        Collider {
                            radius: config.asteroid_radius,
                        },
                        Health {
                            current: config.asteroid_health,
                            max: config.asteroid_health,
                        },
                        Velocity(velocity),
                        Transform::from_translation(timer.position.extend(0.0)),
                    ));
                }
                SpawnType::ScoutDrone => {
                    let velocity =
                        random_velocity(config.drone_velocity_min, config.drone_velocity_max);
                    commands.spawn((
                        ScoutDrone,
                        NeedsDroneVisual,
                        Collider {
                            radius: config.drone_radius,
                        },
                        Health {
                            current: config.drone_health,
                            max: config.drone_health,
                        },
                        Velocity(velocity),
                        Transform::from_translation(timer.position.extend(0.0)),
                    ));
                }
            }
            commands.entity(entity).despawn();
        }
    }
}

/// Applies velocity to all drifting entities (Asteroid and ScoutDrone).
/// Separate from Player's apply_velocity which includes drag/thrust.
#[allow(clippy::type_complexity)]
pub fn drift_entities(
    time: Res<Time>,
    mut query: Query<(&Velocity, &mut Transform), Or<(With<Asteroid>, With<ScoutDrone>)>>,
) {
    let dt = time.delta_secs();
    for (velocity, mut transform) in query.iter_mut() {
        transform.translation.x += velocity.0.x * dt;
        transform.translation.y += velocity.0.y * dt;
    }
}

// ── Unit tests ──────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::time::TimeUpdateStrategy;
    use std::time::Duration;

    #[test]
    fn spawning_config_default_has_valid_values() {
        let config = SpawningConfig::default();
        assert!(!config.asteroid_positions.is_empty(), "Should have asteroid positions");
        assert!(!config.drone_positions.is_empty(), "Should have drone positions");
        assert!(config.asteroid_health > 0.0);
        assert!(config.asteroid_radius > 0.0);
        assert!(config.drone_health > 0.0);
        assert!(config.drone_radius > 0.0);
        assert!(config.respawn_delay > 0.0);
        assert!(config.asteroid_velocity_min > 0.0);
        assert!(config.asteroid_velocity_max > config.asteroid_velocity_min);
        assert!(config.drone_velocity_min > 0.0);
        assert!(config.drone_velocity_max > config.drone_velocity_min);
    }

    #[test]
    fn spawning_config_from_ron() {
        let ron_str = r#"(
            asteroid_positions: [(x: 100.0, y: 200.0)],
            drone_positions: [(x: 50.0, y: 60.0)],
            asteroid_health: 40.0,
            asteroid_radius: 15.0,
            drone_health: 20.0,
            drone_radius: 8.0,
            respawn_delay: 3.0,
            asteroid_velocity_min: 4.0,
            asteroid_velocity_max: 12.0,
            drone_velocity_min: 8.0,
            drone_velocity_max: 20.0,
        )"#;
        let config = SpawningConfig::from_ron(ron_str).expect("Should parse RON");
        assert_eq!(config.asteroid_positions.len(), 1);
        assert!((config.asteroid_positions[0].x - 100.0).abs() < f32::EPSILON);
        assert!((config.asteroid_health - 40.0).abs() < f32::EPSILON);
        assert_eq!(config.drone_positions.len(), 1);
        assert!((config.drone_health - 20.0).abs() < f32::EPSILON);
        assert!((config.respawn_delay - 3.0).abs() < f32::EPSILON);
    }

    #[test]
    fn spawn_initial_entities_creates_correct_asteroid_count() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.insert_resource(SpawningConfig::default());
        app.add_systems(Startup, spawn_initial_entities);
        app.update();

        let count = app
            .world_mut()
            .query_filtered::<Entity, With<Asteroid>>()
            .iter(app.world())
            .count();
        let config = SpawningConfig::default();
        assert_eq!(
            count,
            config.asteroid_positions.len(),
            "Should spawn one asteroid per config position"
        );
    }

    #[test]
    fn spawn_initial_entities_creates_correct_drone_count() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.insert_resource(SpawningConfig::default());
        app.add_systems(Startup, spawn_initial_entities);
        app.update();

        let count = app
            .world_mut()
            .query_filtered::<Entity, With<ScoutDrone>>()
            .iter(app.world())
            .count();
        let config = SpawningConfig::default();
        assert_eq!(
            count,
            config.drone_positions.len(),
            "Should spawn one drone per config position"
        );
    }

    #[test]
    fn spawned_asteroids_have_correct_components() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.insert_resource(SpawningConfig::default());
        app.add_systems(Startup, spawn_initial_entities);
        app.update();

        let config = SpawningConfig::default();
        let mut query = app
            .world_mut()
            .query_filtered::<(
                &Health,
                &Collider,
                &Velocity,
                &Transform,
            ), With<Asteroid>>();

        for (health, collider, velocity, _transform) in query.iter(app.world()) {
            assert!(
                (health.max - config.asteroid_health).abs() < f32::EPSILON,
                "Asteroid health should match config"
            );
            assert!(
                (collider.radius - config.asteroid_radius).abs() < f32::EPSILON,
                "Asteroid radius should match config"
            );
            let speed = velocity.0.length();
            assert!(
                speed >= config.asteroid_velocity_min - 0.01
                    && speed <= config.asteroid_velocity_max + 0.01,
                "Asteroid velocity {speed} should be within configured range {}-{}",
                config.asteroid_velocity_min,
                config.asteroid_velocity_max
            );
        }
    }

    #[test]
    fn spawned_asteroids_have_needs_visual_marker() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.insert_resource(SpawningConfig::default());
        app.add_systems(Startup, spawn_initial_entities);
        app.update();

        let count = app
            .world_mut()
            .query_filtered::<Entity, (With<Asteroid>, With<NeedsAsteroidVisual>)>()
            .iter(app.world())
            .count();
        let config = SpawningConfig::default();
        assert_eq!(
            count,
            config.asteroid_positions.len(),
            "All asteroids should have NeedsAsteroidVisual"
        );
    }

    #[test]
    fn spawned_drones_have_needs_visual_marker() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.insert_resource(SpawningConfig::default());
        app.add_systems(Startup, spawn_initial_entities);
        app.update();

        let count = app
            .world_mut()
            .query_filtered::<Entity, (With<ScoutDrone>, With<NeedsDroneVisual>)>()
            .iter(app.world())
            .count();
        let config = SpawningConfig::default();
        assert_eq!(
            count,
            config.drone_positions.len(),
            "All drones should have NeedsDroneVisual"
        );
    }

    #[test]
    fn respawn_timer_ticks_down_and_spawns_entity() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.insert_resource(SpawningConfig::default());
        app.insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_secs_f64(
            1.0 / 60.0,
        )));
        app.insert_resource(Time::<Fixed>::from_seconds(1.0 / 60.0));
        app.add_systems(Update, tick_respawn_timers);
        app.update(); // Prime (dt=0)

        // Spawn a respawn timer with 0.5s delay
        app.world_mut().spawn(RespawnTimer {
            timer: 0.5,
            spawn_type: SpawnType::Asteroid,
            position: Vec2::new(100.0, 200.0),
        });

        // Tick 30 frames (0.5s at 60fps) — timer should NOT have expired yet at ~29 frames
        for _ in 0..29 {
            app.update();
        }
        let asteroid_count = app
            .world_mut()
            .query_filtered::<Entity, With<Asteroid>>()
            .iter(app.world())
            .count();
        assert_eq!(asteroid_count, 0, "Asteroid should not spawn before timer expires");

        // Tick a few more frames to guarantee expiry
        for _ in 0..5 {
            app.update();
        }

        let asteroid_count = app
            .world_mut()
            .query_filtered::<Entity, With<Asteroid>>()
            .iter(app.world())
            .count();
        assert_eq!(asteroid_count, 1, "One asteroid should spawn after timer expires");

        // Timer entity should be despawned
        let timer_count = app
            .world_mut()
            .query_filtered::<Entity, With<RespawnTimer>>()
            .iter(app.world())
            .count();
        assert_eq!(timer_count, 0, "RespawnTimer entity should be despawned after spawning");
    }

    #[test]
    fn respawn_timer_spawns_drone_type() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.insert_resource(SpawningConfig::default());
        app.insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_secs_f64(
            1.0 / 60.0,
        )));
        app.insert_resource(Time::<Fixed>::from_seconds(1.0 / 60.0));
        app.add_systems(Update, tick_respawn_timers);
        app.update(); // Prime

        app.world_mut().spawn(RespawnTimer {
            timer: 0.1,
            spawn_type: SpawnType::ScoutDrone,
            position: Vec2::new(50.0, 60.0),
        });

        // Run enough frames to expire (0.1s = ~6 frames at 60fps + margin)
        for _ in 0..12 {
            app.update();
        }

        let drone_count = app
            .world_mut()
            .query_filtered::<Entity, With<ScoutDrone>>()
            .iter(app.world())
            .count();
        assert_eq!(drone_count, 1, "One drone should spawn from respawn timer");
    }

    #[test]
    fn spawn_respawn_timers_creates_timer_for_destroyed_asteroid() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.insert_resource(SpawningConfig::default());
        app.add_systems(Update, spawn_respawn_timers);

        // Spawn an asteroid with zero health (destroyed)
        app.world_mut().spawn((
            Asteroid,
            Health {
                current: 0.0,
                max: 50.0,
            },
            Collider { radius: 20.0 },
            Transform::from_translation(Vec3::new(100.0, 200.0, 0.0)),
        ));

        app.update();

        let timer_count = app
            .world_mut()
            .query_filtered::<Entity, With<RespawnTimer>>()
            .iter(app.world())
            .count();
        assert_eq!(timer_count, 1, "Should create respawn timer for destroyed asteroid");

        let mut query = app.world_mut().query::<&RespawnTimer>();
        let timer = query.iter(app.world()).next().expect("Should have timer");
        assert_eq!(timer.spawn_type, SpawnType::Asteroid);
        assert!((timer.position.x - 100.0).abs() < f32::EPSILON);
        assert!((timer.position.y - 200.0).abs() < f32::EPSILON);
    }

    #[test]
    fn drift_entities_moves_asteroids() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_secs_f64(
            1.0 / 60.0,
        )));
        app.add_systems(Update, drift_entities);
        app.update(); // Prime

        let entity = app
            .world_mut()
            .spawn((
                Asteroid,
                Velocity(Vec2::new(60.0, 0.0)),
                Transform::default(),
            ))
            .id();

        app.update();

        let transform = app
            .world()
            .entity(entity)
            .get::<Transform>()
            .expect("Should have Transform");
        assert!(
            transform.translation.x > 0.0,
            "Asteroid should have moved in X direction"
        );
    }

    #[test]
    fn drift_entities_moves_drones() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_secs_f64(
            1.0 / 60.0,
        )));
        app.add_systems(Update, drift_entities);
        app.update(); // Prime

        let entity = app
            .world_mut()
            .spawn((
                ScoutDrone,
                Velocity(Vec2::new(0.0, 90.0)),
                Transform::default(),
            ))
            .id();

        app.update();

        let transform = app
            .world()
            .entity(entity)
            .get::<Transform>()
            .expect("Should have Transform");
        assert!(
            transform.translation.y > 0.0,
            "Drone should have moved in Y direction"
        );
    }

    #[test]
    fn spawn_respawn_timers_creates_timer_for_destroyed_drone() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.insert_resource(SpawningConfig::default());
        app.add_systems(Update, spawn_respawn_timers);

        // Spawn a drone with zero health (destroyed)
        app.world_mut().spawn((
            ScoutDrone,
            Health {
                current: 0.0,
                max: 30.0,
            },
            Collider { radius: 10.0 },
            Transform::from_translation(Vec3::new(50.0, 60.0, 0.0)),
        ));

        app.update();

        let timer_count = app
            .world_mut()
            .query_filtered::<Entity, With<RespawnTimer>>()
            .iter(app.world())
            .count();
        assert_eq!(timer_count, 1, "Should create respawn timer for destroyed drone");

        let mut query = app.world_mut().query::<&RespawnTimer>();
        let timer = query.iter(app.world()).next().expect("Should have timer");
        assert_eq!(timer.spawn_type, SpawnType::ScoutDrone);
        assert!((timer.position.x - 50.0).abs() < f32::EPSILON);
        assert!((timer.position.y - 60.0).abs() < f32::EPSILON);
    }
}
