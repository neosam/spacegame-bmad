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

/// Marker component for Fighter enemy entities (Story 4-2).
#[derive(Component)]
pub struct Fighter;

/// Marker component for Heavy Cruiser enemy entities (Story 4-3).
#[derive(Component)]
pub struct HeavyCruiser;

/// Marker component for Sniper enemy entities (Story 4-4).
#[derive(Component)]
pub struct Sniper;

/// Preferred engagement range for Sniper enemies (Story 4-4).
/// Sniper will move away if closer than `min`, approach if farther than `max`.
#[derive(Component, Debug, Clone)]
pub struct PreferredRange {
    pub min: f32,
    pub max: f32,
}

/// Marker for asteroid entities that need their visual mesh attached by RenderingPlugin.
#[derive(Component)]
pub struct NeedsAsteroidVisual;

/// Marker for drone entities that need their visual mesh attached by RenderingPlugin.
#[derive(Component)]
pub struct NeedsDroneVisual;

/// Marker for fighter entities that need their visual mesh attached by RenderingPlugin.
#[derive(Component)]
pub struct NeedsFighterVisual;

/// Marker for heavy cruiser entities that need their visual mesh attached by RenderingPlugin.
#[derive(Component)]
pub struct NeedsHeavyCruiserVisual;

/// Marker for sniper entities that need their visual mesh attached by RenderingPlugin.
#[derive(Component)]
pub struct NeedsSniperVisual;

// ── Story 4-10: Trader Ships ─────────────────────────────────────────────

/// Marker for Trader Ship entities.
/// Traders are Neutral faction — they don't attack but can be attacked.
#[derive(Component, Debug, Clone)]
pub struct TraderShip;

/// A linear route for a trader ship between two world positions.
/// The trader moves from `from` to `to`, then reverses.
#[derive(Component, Debug, Clone)]
pub struct TraderRoute {
    pub from: Vec2,
    pub to: Vec2,
    /// Progress along the route: 0.0 = at `from`, 1.0 = at `to`.
    pub progress: f32,
    /// Direction: true = from→to, false = to→from.
    pub forward: bool,
}

impl TraderRoute {
    /// Current world position along the route.
    pub fn current_position(&self) -> Vec2 {
        self.from.lerp(self.to, self.progress)
    }
}

/// Visual marker for trader ships.
#[derive(Component, Debug)]
pub struct NeedsTraderVisual;

// ── Story 4-5: Swarms ───────────────────────────────────────────────────

/// Groups a swarm entity with a unique swarm ID.
/// All entities sharing the same `swarm_id` belong to the same swarm.
#[derive(Component, Debug, Clone)]
pub struct Swarm {
    pub swarm_id: u32,
}

/// Marker for the leader of a swarm.
/// One member per swarm is the leader; followers orient toward leader position.
#[derive(Component, Debug, Clone)]
pub struct SwarmLeader;

/// Follower: stores the entity ID of its swarm leader.
#[derive(Component, Debug, Clone)]
pub struct SwarmFollower {
    pub leader: Entity,
}

/// Spawns a swarm of 3–5 Fighter entities at the given center position.
/// One entity is designated leader, the rest are followers.
pub fn spawn_swarm(
    commands: &mut Commands,
    center: Vec2,
    swarm_id: u32,
    count: usize,
    config: &SpawningConfig,
) -> Vec<Entity> {
    use crate::core::collision::{Collider, Health};
    use crate::shared::components::Velocity;

    let count = count.clamp(3, 5);
    let mut entities = Vec::with_capacity(count);

    // Spawn all swarm members without follower link first
    for i in 0..count {
        let angle = (i as f32 / count as f32) * std::f32::consts::TAU;
        let offset = Vec2::new(angle.cos(), angle.sin()) * 30.0;
        let pos = center + offset;

        let entity = commands.spawn((
            Fighter,
            NeedsFighterVisual,
            Swarm { swarm_id },
            Collider { radius: config.fighter_radius },
            Health { current: config.fighter_health, max: config.fighter_health },
            Velocity(Vec2::ZERO),
            Transform::from_translation(pos.extend(0.0)),
        )).id();
        entities.push(entity);
    }

    // First entity is the leader
    if let Some(&leader) = entities.first() {
        commands.entity(leader).insert(SwarmLeader);

        // Rest are followers
        for &follower_entity in entities.iter().skip(1) {
            commands.entity(follower_entity).insert(SwarmFollower { leader });
        }
    }

    entities
}

// ── Respawn ─────────────────────────────────────────────────────────────

/// Which type of entity to respawn.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpawnType {
    Asteroid,
    ScoutDrone,
    Fighter,
    HeavyCruiser,
    Sniper,
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
    // Story 4-2: Fighter config
    #[serde(default)]
    pub fighter_positions: Vec<SpawnPoint>,
    #[serde(default = "default_fighter_health")]
    pub fighter_health: f32,
    #[serde(default = "default_fighter_radius")]
    pub fighter_radius: f32,
    #[serde(default = "default_fighter_respawn_delay")]
    pub fighter_respawn_delay: f32,
    // Story 4-3: Heavy Cruiser config
    #[serde(default)]
    pub heavy_cruiser_positions: Vec<SpawnPoint>,
    #[serde(default = "default_heavy_health")]
    pub heavy_cruiser_health: f32,
    #[serde(default = "default_heavy_radius")]
    pub heavy_cruiser_radius: f32,
    #[serde(default = "default_heavy_respawn_delay")]
    pub heavy_cruiser_respawn_delay: f32,
    // Story 4-4: Sniper config
    #[serde(default)]
    pub sniper_positions: Vec<SpawnPoint>,
    #[serde(default = "default_sniper_health")]
    pub sniper_health: f32,
    #[serde(default = "default_sniper_radius")]
    pub sniper_radius: f32,
    #[serde(default = "default_sniper_respawn_delay")]
    pub sniper_respawn_delay: f32,
}

fn default_fighter_health() -> f32 { 50.0 }
fn default_fighter_radius() -> f32 { 12.0 }
fn default_fighter_respawn_delay() -> f32 { 8.0 }
fn default_heavy_health() -> f32 { 200.0 }
fn default_heavy_radius() -> f32 { 25.0 }
fn default_heavy_respawn_delay() -> f32 { 15.0 }
fn default_sniper_health() -> f32 { 40.0 }
fn default_sniper_radius() -> f32 { 10.0 }
fn default_sniper_respawn_delay() -> f32 { 10.0 }

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
            fighter_positions: vec![],
            fighter_health: default_fighter_health(),
            fighter_radius: default_fighter_radius(),
            fighter_respawn_delay: default_fighter_respawn_delay(),
            heavy_cruiser_positions: vec![],
            heavy_cruiser_health: default_heavy_health(),
            heavy_cruiser_radius: default_heavy_radius(),
            heavy_cruiser_respawn_delay: default_heavy_respawn_delay(),
            sniper_positions: vec![],
            sniper_health: default_sniper_health(),
            sniper_radius: default_sniper_radius(),
            sniper_respawn_delay: default_sniper_respawn_delay(),
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

/// Watches for destroyed enemy entities and spawns RespawnTimer entities.
/// Runs BEFORE despawn_destroyed in the Damage chain so we can detect entities with
/// health <= 0 and record their position and spawn type before they are removed.
/// TutorialEnemy entities are excluded — they must NOT respawn when destroyed.
#[allow(clippy::type_complexity)]
pub fn spawn_respawn_timers(
    mut commands: Commands,
    config: Res<SpawningConfig>,
    query: Query<
        (
            &Health,
            &Transform,
            Option<&Asteroid>,
            Option<&ScoutDrone>,
            Option<&Fighter>,
            Option<&HeavyCruiser>,
            Option<&Sniper>,
        ),
        (
            Without<Player>,
            Without<crate::core::tutorial::TutorialEnemy>,
            Or<(
                With<Asteroid>,
                With<ScoutDrone>,
                With<Fighter>,
                With<HeavyCruiser>,
                With<Sniper>,
            )>,
        ),
    >,
) {
    for (health, transform, asteroid, drone, fighter, heavy, sniper) in query.iter() {
        if health.current <= 0.0 {
            let position = Vec2::new(transform.translation.x, transform.translation.y);
            let (spawn_type, delay) = if asteroid.is_some() {
                (SpawnType::Asteroid, config.respawn_delay)
            } else if drone.is_some() {
                (SpawnType::ScoutDrone, config.respawn_delay)
            } else if fighter.is_some() {
                (SpawnType::Fighter, config.fighter_respawn_delay)
            } else if heavy.is_some() {
                (SpawnType::HeavyCruiser, config.heavy_cruiser_respawn_delay)
            } else if sniper.is_some() {
                (SpawnType::Sniper, config.sniper_respawn_delay)
            } else {
                continue;
            };

            commands.spawn(RespawnTimer {
                timer: delay,
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
                SpawnType::Fighter => {
                    commands.spawn((
                        Fighter,
                        NeedsFighterVisual,
                        Collider { radius: config.fighter_radius },
                        Health {
                            current: config.fighter_health,
                            max: config.fighter_health,
                        },
                        Velocity(Vec2::ZERO),
                        Transform::from_translation(timer.position.extend(0.0)),
                    ));
                }
                SpawnType::HeavyCruiser => {
                    commands.spawn((
                        HeavyCruiser,
                        NeedsHeavyCruiserVisual,
                        Collider { radius: config.heavy_cruiser_radius },
                        Health {
                            current: config.heavy_cruiser_health,
                            max: config.heavy_cruiser_health,
                        },
                        Velocity(Vec2::ZERO),
                        Transform::from_translation(timer.position.extend(0.0)),
                    ));
                }
                SpawnType::Sniper => {
                    commands.spawn((
                        Sniper,
                        NeedsSniperVisual,
                        Collider { radius: config.sniper_radius },
                        Health {
                            current: config.sniper_health,
                            max: config.sniper_health,
                        },
                        Velocity(Vec2::ZERO),
                        Transform::from_translation(timer.position.extend(0.0)),
                    ));
                }
            }
            commands.entity(entity).despawn();
        }
    }
}

/// Applies velocity to all drifting entities (Asteroid, ScoutDrone, and new enemy types).
/// Separate from Player's apply_velocity which includes drag/thrust.
#[allow(clippy::type_complexity)]
pub fn drift_entities(
    time: Res<Time>,
    mut query: Query<
        (&Velocity, &mut Transform),
        Or<(With<Asteroid>, With<ScoutDrone>, With<Fighter>, With<HeavyCruiser>, With<Sniper>)>,
    >,
) {
    let dt = time.delta_secs();
    for (velocity, mut transform) in query.iter_mut() {
        transform.translation.x += velocity.0.x * dt;
        transform.translation.y += velocity.0.y * dt;
    }
}

/// Moves trader ships along their route.
/// When progress reaches 1.0, the direction reverses.
pub fn update_trader_ships(
    time: Res<Time>,
    mut query: Query<(&mut TraderRoute, &mut Transform), With<TraderShip>>,
) {
    let dt = time.delta_secs();
    const TRADER_SPEED: f32 = 50.0; // world units per second

    for (mut route, mut transform) in query.iter_mut() {
        let total_dist = route.from.distance(route.to);
        if total_dist < f32::EPSILON {
            continue;
        }

        // Advance progress based on speed / total distance
        let progress_delta = (TRADER_SPEED / total_dist) * dt;

        if route.forward {
            route.progress = (route.progress + progress_delta).min(1.0);
            if route.progress >= 1.0 {
                route.forward = false;
            }
        } else {
            route.progress = (route.progress - progress_delta).max(0.0);
            if route.progress <= 0.0 {
                route.forward = true;
            }
        }

        let pos = route.current_position();
        transform.translation.x = pos.x;
        transform.translation.y = pos.y;
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

    // ── Story 4-2: Fighter tests ──

    #[test]
    fn spawning_config_default_has_fighter_stats() {
        let config = SpawningConfig::default();
        assert!(config.fighter_health > 0.0, "Fighter health should be positive");
        assert!(config.fighter_radius > 0.0, "Fighter radius should be positive");
        assert!(config.fighter_respawn_delay > 0.0, "Fighter respawn delay should be positive");
    }

    #[test]
    fn spawn_respawn_timers_creates_timer_for_destroyed_fighter() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.insert_resource(SpawningConfig::default());
        app.add_systems(Update, spawn_respawn_timers);

        app.world_mut().spawn((
            Fighter,
            Health { current: 0.0, max: 50.0 },
            Collider { radius: 12.0 },
            Transform::from_translation(Vec3::new(100.0, 200.0, 0.0)),
        ));

        app.update();

        let timer_count = app
            .world_mut()
            .query_filtered::<Entity, With<RespawnTimer>>()
            .iter(app.world())
            .count();
        assert_eq!(timer_count, 1, "Should create respawn timer for destroyed fighter");

        let mut query = app.world_mut().query::<&RespawnTimer>();
        let timer = query.iter(app.world()).next().expect("Should have timer");
        assert_eq!(timer.spawn_type, SpawnType::Fighter);
    }

    // ── Story 4-3: Heavy Cruiser tests ──

    #[test]
    fn spawning_config_default_has_heavy_cruiser_stats() {
        let config = SpawningConfig::default();
        assert!(config.heavy_cruiser_health >= 200.0, "Heavy Cruiser health should be >=200");
        assert!(config.heavy_cruiser_radius > 0.0, "Heavy Cruiser radius should be positive");
        assert!(config.heavy_cruiser_respawn_delay > 0.0, "Heavy Cruiser respawn delay should be positive");
    }

    #[test]
    fn spawn_respawn_timers_creates_timer_for_destroyed_heavy_cruiser() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.insert_resource(SpawningConfig::default());
        app.add_systems(Update, spawn_respawn_timers);

        app.world_mut().spawn((
            HeavyCruiser,
            Health { current: 0.0, max: 200.0 },
            Collider { radius: 25.0 },
            Transform::from_translation(Vec3::new(50.0, 75.0, 0.0)),
        ));

        app.update();

        let timer_count = app
            .world_mut()
            .query_filtered::<Entity, With<RespawnTimer>>()
            .iter(app.world())
            .count();
        assert_eq!(timer_count, 1, "Should create respawn timer for destroyed heavy cruiser");

        let mut query = app.world_mut().query::<&RespawnTimer>();
        let timer = query.iter(app.world()).next().expect("Should have timer");
        assert_eq!(timer.spawn_type, SpawnType::HeavyCruiser);
    }

    // ── Story 4-4: Sniper tests ──

    #[test]
    fn spawning_config_default_has_sniper_stats() {
        let config = SpawningConfig::default();
        assert!(config.sniper_health > 0.0, "Sniper health should be positive");
        assert!(config.sniper_radius > 0.0, "Sniper radius should be positive");
        assert!(config.sniper_respawn_delay > 0.0, "Sniper respawn delay should be positive");
    }

    #[test]
    fn spawn_respawn_timers_creates_timer_for_destroyed_sniper() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.insert_resource(SpawningConfig::default());
        app.add_systems(Update, spawn_respawn_timers);

        app.world_mut().spawn((
            Sniper,
            Health { current: 0.0, max: 40.0 },
            Collider { radius: 10.0 },
            Transform::from_translation(Vec3::new(300.0, 400.0, 0.0)),
        ));

        app.update();

        let timer_count = app
            .world_mut()
            .query_filtered::<Entity, With<RespawnTimer>>()
            .iter(app.world())
            .count();
        assert_eq!(timer_count, 1, "Should create respawn timer for destroyed sniper");

        let mut query = app.world_mut().query::<&RespawnTimer>();
        let timer = query.iter(app.world()).next().expect("Should have timer");
        assert_eq!(timer.spawn_type, SpawnType::Sniper);
    }

    #[test]
    fn preferred_range_component_valid() {
        let pref = PreferredRange { min: 150.0, max: 280.0 };
        assert!(pref.min > 0.0);
        assert!(pref.max > pref.min, "max ({}) should exceed min ({})", pref.max, pref.min);
    }

    // ── Story 4-5: Swarm tests ──

    #[test]
    fn spawn_swarm_creates_correct_entity_count() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        let config = SpawningConfig::default();

        let count = 4;
        app.world_mut().commands().queue(move |world: &mut bevy::ecs::world::World| {
            let mut commands = world.commands();
            spawn_swarm(&mut commands, Vec2::new(100.0, 0.0), 1, count, &config);
        });
        app.update();

        let fighter_count = app
            .world_mut()
            .query_filtered::<Entity, With<Fighter>>()
            .iter(app.world())
            .count();
        assert_eq!(fighter_count, count, "Swarm should spawn exactly {count} fighters");
    }

    #[test]
    fn spawn_swarm_has_exactly_one_leader() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        let config = SpawningConfig::default();

        app.world_mut().commands().queue(move |world: &mut bevy::ecs::world::World| {
            let mut commands = world.commands();
            spawn_swarm(&mut commands, Vec2::new(0.0, 0.0), 42, 3, &config);
        });
        app.update();

        let leader_count = app
            .world_mut()
            .query_filtered::<Entity, With<SwarmLeader>>()
            .iter(app.world())
            .count();
        assert_eq!(leader_count, 1, "Swarm should have exactly 1 leader");
    }

    #[test]
    fn spawn_swarm_followers_reference_leader() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        let config = SpawningConfig::default();

        app.world_mut().commands().queue(move |world: &mut bevy::ecs::world::World| {
            let mut commands = world.commands();
            spawn_swarm(&mut commands, Vec2::new(0.0, 0.0), 7, 4, &config);
        });
        app.update();

        let follower_count = app
            .world_mut()
            .query_filtered::<Entity, With<SwarmFollower>>()
            .iter(app.world())
            .count();
        // 4 total - 1 leader = 3 followers
        assert_eq!(follower_count, 3, "Swarm of 4 should have 3 followers");
    }

    #[test]
    fn spawn_swarm_all_share_same_swarm_id() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        let config = SpawningConfig::default();
        let swarm_id = 99u32;

        app.world_mut().commands().queue(move |world: &mut bevy::ecs::world::World| {
            let mut commands = world.commands();
            spawn_swarm(&mut commands, Vec2::new(0.0, 0.0), swarm_id, 3, &config);
        });
        app.update();

        let mut query = app.world_mut().query::<&Swarm>();
        for swarm in query.iter(app.world()) {
            assert_eq!(swarm.swarm_id, swarm_id, "All swarm members should share swarm_id {}", swarm_id);
        }
    }

    // ── Story 4-10: Trader Ship tests ──

    #[test]
    fn trader_route_current_position_at_zero_progress() {
        let route = TraderRoute {
            from: Vec2::new(0.0, 0.0),
            to: Vec2::new(100.0, 0.0),
            progress: 0.0,
            forward: true,
        };
        let pos = route.current_position();
        assert!((pos.x - 0.0).abs() < f32::EPSILON, "At progress=0 should be at from");
    }

    #[test]
    fn trader_route_current_position_at_full_progress() {
        let route = TraderRoute {
            from: Vec2::new(0.0, 0.0),
            to: Vec2::new(100.0, 0.0),
            progress: 1.0,
            forward: false,
        };
        let pos = route.current_position();
        assert!((pos.x - 100.0).abs() < f32::EPSILON, "At progress=1 should be at to");
    }

    #[test]
    fn trader_route_interpolates_correctly() {
        let route = TraderRoute {
            from: Vec2::new(0.0, 0.0),
            to: Vec2::new(200.0, 0.0),
            progress: 0.5,
            forward: true,
        };
        let pos = route.current_position();
        assert!((pos.x - 100.0).abs() < f32::EPSILON, "At progress=0.5 should be at midpoint (100)");
    }

    #[test]
    fn trader_ship_moves_along_route() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_secs_f64(1.0)));
        app.add_systems(Update, update_trader_ships);
        app.update(); // prime

        let trader = app.world_mut().spawn((
            TraderShip,
            Transform::from_translation(Vec3::ZERO),
            TraderRoute {
                from: Vec2::ZERO,
                to: Vec2::new(1000.0, 0.0),
                progress: 0.0,
                forward: true,
            },
        )).id();

        app.update(); // 1 second at TRADER_SPEED=50

        let transform = app.world().entity(trader).get::<Transform>()
            .expect("Trader should have Transform");
        assert!(
            transform.translation.x > 0.0,
            "Trader should have moved along route (positive X), got {}",
            transform.translation.x
        );
    }

    #[test]
    fn trader_ship_reverses_at_destination() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_secs_f64(1.0 / 60.0)));
        app.add_systems(Update, update_trader_ships);
        app.update(); // prime

        // Trader almost at destination
        let trader = app.world_mut().spawn((
            TraderShip,
            Transform::from_translation(Vec3::new(99.0, 0.0, 0.0)),
            TraderRoute {
                from: Vec2::ZERO,
                to: Vec2::new(100.0, 0.0),
                progress: 0.99,
                forward: true,
            },
        )).id();

        // Run enough frames to arrive and start returning
        for _ in 0..10 {
            app.update();
        }

        let route = app.world().entity(trader).get::<TraderRoute>()
            .expect("Trader should have TraderRoute");
        assert!(!route.forward, "Trader should have reversed direction after reaching destination");
    }

    #[test]
    fn spawn_swarm_clamped_to_3_to_5() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        let config = SpawningConfig::default();

        // Request 10 — should clamp to 5
        app.world_mut().commands().queue(move |world: &mut bevy::ecs::world::World| {
            let mut commands = world.commands();
            spawn_swarm(&mut commands, Vec2::ZERO, 1, 10, &config);
        });
        app.update();

        let fighter_count = app
            .world_mut()
            .query_filtered::<Entity, With<Fighter>>()
            .iter(app.world())
            .count();
        assert!(fighter_count <= 5, "Swarm count should be clamped to max 5, got {fighter_count}");
    }
}
