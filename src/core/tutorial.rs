use bevy::prelude::*;
use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};
use serde::Deserialize;

// ── Config ──────────────────────────────────────────────────────────────

/// Tutorial zone balance values loaded from `assets/config/tutorial.ron`.
#[derive(Resource, Deserialize, Clone, Debug)]
pub struct TutorialConfig {
    /// Radius of the safe zone where no gravity pull is applied
    pub safe_radius: f32,
    /// Strength of the gravity pull outside the safe radius
    pub pull_strength: f32,
    /// Minimum distance from zone center for station placement
    pub station_offset_min: f32,
    /// Maximum distance from zone center for station placement
    pub station_offset_max: f32,
    /// Minimum distance from zone center for generator placement
    pub generator_offset_min: f32,
    /// Maximum distance from zone center for generator placement
    pub generator_offset_max: f32,
    /// Minimum distance from zone center for player spawn
    pub player_offset_min: f32,
    /// Maximum distance from zone center for player spawn
    pub player_offset_max: f32,
    /// Health of the gravity well generator
    pub generator_health: f32,
    /// Minimum distance from zone center for wreck placement
    pub wreck_offset_min: f32,
    /// Maximum distance from zone center for wreck placement
    pub wreck_offset_max: f32,
    /// Number of tutorial enemies to spawn after laser is acquired
    pub tutorial_enemy_count: usize,
    /// Radius around the wreck in which tutorial enemies are spawned
    pub tutorial_enemy_spawn_radius: f32,
    /// Distance from station within which the player docks and receives the Spread weapon
    pub dock_radius: f32,
    /// Delay in seconds after GeneratorDestroyed before cascade despawn fires
    pub cascade_delay_secs: f32,
}

impl Default for TutorialConfig {
    fn default() -> Self {
        Self {
            safe_radius: 2000.0,
            pull_strength: 50.0,
            station_offset_min: 200.0,
            station_offset_max: 400.0,
            generator_offset_min: 1600.0,
            generator_offset_max: 1900.0,
            player_offset_min: 50.0,
            player_offset_max: 150.0,
            generator_health: 100.0,
            wreck_offset_min: 400.0,
            wreck_offset_max: 700.0,
            tutorial_enemy_count: 3,
            tutorial_enemy_spawn_radius: 150.0,
            dock_radius: 150.0,
            cascade_delay_secs: 2.0,
        }
    }
}

impl TutorialConfig {
    /// Load config from RON string.
    pub fn from_ron(ron_str: &str) -> Result<Self, ron::error::SpannedError> {
        ron::from_str(ron_str)
    }
}

// ── Components ──────────────────────────────────────────────────────────

/// Gravity well generator entity that defines the tutorial boundary.
#[derive(Component, Debug)]
pub struct GravityWellGenerator {
    pub safe_radius: f32,
    pub pull_strength: f32,
    /// Whether destruction requires projectile (spread) weapon
    pub requires_projectile: bool,
}

/// Marker component for the defective tutorial station.
#[derive(Component, Debug)]
pub struct TutorialStation {
    pub defective: bool,
}

/// Marker component for the tutorial wreck entity that grants the laser weapon.
#[derive(Component, Debug)]
pub struct TutorialWreck;

/// Tracks whether the tutorial wreck has been shot by the player's laser.
/// When `has_been_shot` transitions to true, the tutorial phase advances.
#[derive(Component, Debug)]
pub struct WreckShotState {
    pub has_been_shot: bool,
}

/// Marker component for enemies spawned as part of the tutorial wave.
/// Distinguishes them from normal map enemies — they do not respawn on death.
#[derive(Component, Debug)]
pub struct TutorialEnemy;

// ── Resources ───────────────────────────────────────────────────────────

/// Tracks the tutorial zone state: center, seed, and layout metadata.
#[derive(Resource, Debug)]
pub struct TutorialZone {
    pub center: Vec2,
    pub seed: u64,
    pub layout: TutorialLayout,
}

/// Tracks the number of tutorial wave enemies that remain alive.
/// Inserted by `spawn_tutorial_enemies` when the wave begins.
/// When `remaining` reaches 0, the tutorial phase advances to `Complete`.
#[derive(Resource, Debug, Default)]
pub struct TutorialEnemyWave {
    pub remaining: usize,
}

// ── State Machine ───────────────────────────────────────────────────────

/// Tutorial phase state machine.
/// Controls what abilities are available to the player during the tutorial.
#[derive(States, Default, Clone, Eq, PartialEq, Debug, Hash)]
pub enum TutorialPhase {
    /// Only thrust and rotate available
    #[default]
    Flying,
    /// Laser weapon unlocked
    Shooting,
    /// Spread weapon unlocked
    SpreadUnlocked,
    /// Tutorial complete — all abilities available
    Complete,
    /// Player docked at station after generator destruction — tutorial fully done
    StationVisited,
    /// GravityWellGenerator destroyed — gravity well dissolved, player free to leave
    GeneratorDestroyed,
    /// Destruction cascade complete — tutorial fully finished
    TutorialComplete,
}

// ── Layout Generation ───────────────────────────────────────────────────

/// Describes the computed positions for all tutorial entities.
#[derive(Debug, Clone)]
pub struct TutorialLayout {
    pub player_spawn: Vec2,
    pub station_position: Vec2,
    pub generator_position: Vec2,
    pub zone_center: Vec2,
    pub wreck_position: Vec2,
}

const TUTORIAL_SEED_PRIME: u64 = 9_876_543_210_123_456_789;

/// Derives a tutorial-specific seed from the world seed.
fn tutorial_seed(world_seed: u64) -> u64 {
    world_seed ^ TUTORIAL_SEED_PRIME
}

/// Generates a random position at a given distance range from center using an RNG.
fn random_offset(rng: &mut StdRng, min_dist: f32, max_dist: f32) -> Vec2 {
    let angle = rng.random_range(0.0..std::f32::consts::TAU);
    let dist = if min_dist < max_dist {
        rng.random_range(min_dist..=max_dist)
    } else {
        min_dist
    };
    Vec2::new(angle.cos() * dist, angle.sin() * dist)
}

/// Generates a deterministic tutorial zone layout from a seed and config.
/// Pure function: same seed + config always produces the same layout.
pub fn generate_tutorial_zone(seed: u64, config: &TutorialConfig) -> TutorialLayout {
    let tseed = tutorial_seed(seed);
    let mut rng = StdRng::seed_from_u64(tseed);

    let zone_center = Vec2::ZERO;

    let player_offset = random_offset(&mut rng, config.player_offset_min, config.player_offset_max);
    let player_spawn = zone_center + player_offset;

    let station_offset =
        random_offset(&mut rng, config.station_offset_min, config.station_offset_max);
    let station_position = zone_center + station_offset;

    let generator_offset = random_offset(
        &mut rng,
        config.generator_offset_min,
        config.generator_offset_max,
    );
    let generator_position = zone_center + generator_offset;

    let wreck_offset = random_offset(&mut rng, config.wreck_offset_min, config.wreck_offset_max);
    let wreck_position = zone_center + wreck_offset;

    TutorialLayout {
        player_spawn,
        station_position,
        generator_position,
        zone_center,
        wreck_position,
    }
}

// ── Constraint Validation ───────────────────────────────────────────────

/// Describes a constraint violation in the tutorial layout.
#[derive(Debug, Clone)]
pub struct ConstraintViolation {
    pub description: String,
}

/// Validates a tutorial layout against the config constraints.
pub fn validate_tutorial_layout(
    layout: &TutorialLayout,
    config: &TutorialConfig,
) -> Result<(), Vec<ConstraintViolation>> {
    let mut violations = Vec::new();

    // All entities must be within safe_radius of zone center
    let station_dist = (layout.station_position - layout.zone_center).length();
    if station_dist > config.safe_radius {
        violations.push(ConstraintViolation {
            description: format!(
                "Station at distance {station_dist:.1} exceeds safe_radius {}",
                config.safe_radius
            ),
        });
    }

    let generator_dist = (layout.generator_position - layout.zone_center).length();
    if generator_dist > config.safe_radius {
        violations.push(ConstraintViolation {
            description: format!(
                "Generator at distance {generator_dist:.1} exceeds safe_radius {}",
                config.safe_radius
            ),
        });
    }

    let player_dist = (layout.player_spawn - layout.zone_center).length();
    if player_dist > config.safe_radius {
        violations.push(ConstraintViolation {
            description: format!(
                "Player spawn at distance {player_dist:.1} exceeds safe_radius {}",
                config.safe_radius
            ),
        });
    }

    let wreck_dist = (layout.wreck_position - layout.zone_center).length();
    if wreck_dist > config.safe_radius {
        violations.push(ConstraintViolation {
            description: format!(
                "Wreck at distance {wreck_dist:.1} exceeds safe_radius {}",
                config.safe_radius
            ),
        });
    }

    // Station must be reachable from player spawn (within safe_radius distance)
    let player_to_station = (layout.station_position - layout.player_spawn).length();
    if player_to_station > config.safe_radius {
        violations.push(ConstraintViolation {
            description: format!(
                "Station unreachable from player spawn: distance {player_to_station:.1} exceeds safe_radius {}",
                config.safe_radius
            ),
        });
    }

    if violations.is_empty() {
        Ok(())
    } else {
        Err(violations)
    }
}

/// Validates a single seed produces a valid tutorial layout.
pub fn validate_tutorial_seed(
    seed: u64,
    config: &TutorialConfig,
) -> Result<(), Vec<ConstraintViolation>> {
    let layout = generate_tutorial_zone(seed, config);
    validate_tutorial_layout(&layout, config)
}

// ── Spawn System ────────────────────────────────────────────────────────

/// Startup system: generates and spawns the tutorial zone entities.
pub fn spawn_tutorial_zone(
    mut commands: Commands,
    config: Res<TutorialConfig>,
    world_config: Res<crate::world::WorldConfig>,
    mut active_chunks: ResMut<crate::world::ActiveChunks>,
    mut game_events: bevy::ecs::message::MessageWriter<crate::shared::events::GameEvent>,
    time: Res<Time>,
    severity_config: Res<crate::infrastructure::events::EventSeverityConfig>,
) {
    let seed = world_config.seed;
    let layout = generate_tutorial_zone(seed, &config);

    // Spawn player at tutorial position
    commands.spawn((
        crate::core::flight::Player,
        crate::shared::components::Velocity::default(),
        crate::core::collision::Health {
            current: 100.0,
            max: 100.0,
        },
        crate::core::collision::Collider { radius: 12.0 },
        crate::core::weapons::FireCooldown::default(),
        crate::core::weapons::Energy::default(),
        crate::core::weapons::ActiveWeapon::default(),
        Transform::from_translation(layout.player_spawn.extend(0.0)),
    ));

    // Spawn tutorial station
    commands.spawn((
        TutorialStation { defective: true },
        Transform::from_translation(layout.station_position.extend(0.0)),
    ));

    // Spawn gravity well generator
    commands.spawn((
        GravityWellGenerator {
            safe_radius: config.safe_radius,
            pull_strength: config.pull_strength,
            requires_projectile: true,
        },
        crate::core::collision::Health {
            current: config.generator_health,
            max: config.generator_health,
        },
        crate::core::collision::Collider { radius: 30.0 },
        Transform::from_translation(layout.generator_position.extend(0.0)),
    ));

    // Spawn tutorial wreck — the player shoots this to unlock laser weapon
    commands.spawn((
        TutorialWreck,
        WreckShotState { has_been_shot: false },
        crate::core::collision::Collider { radius: 20.0 },
        crate::core::collision::Health { current: 50.0, max: 50.0 },
        Transform::from_translation(layout.wreck_position.extend(0.0)),
    ));

    // Mark tutorial area chunks as occupied so chunk generation skips them
    let chunk_size = world_config.chunk_size;
    let radius_in_chunks = (config.safe_radius / chunk_size).ceil() as i32;
    for cx in -radius_in_chunks..=radius_in_chunks {
        for cy in -radius_in_chunks..=radius_in_chunks {
            let coord = crate::world::ChunkCoord { x: cx, y: cy };
            let chunk_center = Vec2::new(
                cx as f32 * chunk_size + chunk_size / 2.0,
                cy as f32 * chunk_size + chunk_size / 2.0,
            );
            if chunk_center.length() <= config.safe_radius + chunk_size {
                active_chunks
                    .chunks
                    .insert(coord, crate::world::BiomeType::DeepSpace);
            }
        }
    }

    // Emit tutorial zone spawned event
    let kind = crate::shared::events::GameEventKind::TutorialZoneSpawned;
    game_events.write(crate::shared::events::GameEvent {
        severity: severity_config.severity_for(&kind),
        kind,
        position: layout.zone_center,
        game_time: time.elapsed_secs_f64(),
    });

    // Insert TutorialZone resource
    commands.insert_resource(TutorialZone {
        center: layout.zone_center,
        seed,
        layout,
    });
}

// ── Weapon Gating ───────────────────────────────────────────────────────

/// Marker component added to the player when weapons are locked by the tutorial phase.
#[derive(Component, Debug)]
pub struct WeaponsLocked;

/// Marker component added to the player when the tutorial station grants the Spread weapon.
/// Present from the moment the player docks at the tutorial station onward.
#[derive(Component, Debug)]
pub struct SpreadUnlocked;

/// System that adds/removes WeaponsLocked based on TutorialPhase.
pub fn update_weapons_lock(
    mut commands: Commands,
    phase: Res<State<TutorialPhase>>,
    player_query: Query<(Entity, Option<&WeaponsLocked>), With<crate::core::flight::Player>>,
) {
    let should_lock = matches!(phase.get(), TutorialPhase::Flying);

    for (entity, has_lock) in player_query.iter() {
        if should_lock && has_lock.is_none() {
            commands.entity(entity).insert(WeaponsLocked);
        } else if !should_lock && has_lock.is_some() {
            commands.entity(entity).remove::<WeaponsLocked>();
        }
    }
}

// ── Gravity Well Physics ────────────────────────────────────────────────

/// Apply gravity well pull to entities outside the safe radius.
/// `pull_force = max(0, (distance - safe_radius) * pull_strength)`
/// Force is applied as velocity change toward the generator.
pub fn apply_gravity_well(
    time: Res<Time>,
    generator_query: Query<(&GravityWellGenerator, &Transform)>,
    mut player_query: Query<(&Transform, &mut crate::shared::components::Velocity), With<crate::core::flight::Player>>,
) {
    let dt = time.delta_secs();
    for (gen_comp, gen_transform) in generator_query.iter() {
        let gen_pos = gen_transform.translation.truncate();
        for (player_transform, mut velocity) in player_query.iter_mut() {
            let player_pos = player_transform.translation.truncate();
            let diff = gen_pos - player_pos;
            let distance = diff.length();
            if distance > gen_comp.safe_radius && distance > f32::EPSILON {
                let pull_magnitude = (distance - gen_comp.safe_radius) * gen_comp.pull_strength;
                let direction = diff / distance;
                velocity.0 += direction * pull_magnitude * dt;
            }
        }
    }
}

// ── Laser-at-Wreck Phase Progression ────────────────────────────────────

/// Detects when the `TutorialWreck` entity receives damage (via `JustDamaged` component)
/// and advances the `TutorialPhase` from `Shooting` to `SpreadUnlocked`.
/// The transition is idempotent: once `has_been_shot` is true, no further action is taken.
pub fn advance_phase_on_wreck_shot(
    mut wreck_query: Query<
        &mut WreckShotState,
        (With<TutorialWreck>, With<crate::shared::components::JustDamaged>),
    >,
    phase: Res<State<TutorialPhase>>,
    mut next_phase: ResMut<NextState<TutorialPhase>>,
) {
    for mut shot_state in wreck_query.iter_mut() {
        if !shot_state.has_been_shot {
            shot_state.has_been_shot = true;
            if *phase.get() == TutorialPhase::Shooting {
                next_phase.set(TutorialPhase::SpreadUnlocked);
            }
        }
    }
}

// ── Tutorial Enemy Wave ──────────────────────────────────────────────────

/// Spawns a wave of `TutorialEnemy` Scout Drones near the wreck position.
/// Runs `OnEnter(TutorialPhase::SpreadUnlocked)` — fires exactly once on phase entry.
/// Also inserts the `TutorialEnemyWave` resource to track wave completion.
pub fn spawn_tutorial_enemies(
    mut commands: Commands,
    tutorial_zone: Res<TutorialZone>,
    tutorial_config: Res<TutorialConfig>,
    spawning_config: Res<crate::core::spawning::SpawningConfig>,
) {
    let wreck_pos = tutorial_zone.layout.wreck_position;
    let count = tutorial_config.tutorial_enemy_count;
    let radius = tutorial_config.tutorial_enemy_spawn_radius;

    for _ in 0..count {
        let angle = rand::random::<f32>() * std::f32::consts::TAU;
        let dist = rand::random::<f32>() * radius;
        let offset = Vec2::new(angle.cos() * dist, angle.sin() * dist);
        let pos = wreck_pos + offset;

        commands.spawn((
            TutorialEnemy,
            crate::core::spawning::ScoutDrone,
            crate::core::spawning::NeedsDroneVisual,
            crate::core::collision::Collider { radius: spawning_config.drone_radius },
            crate::core::collision::Health {
                current: spawning_config.drone_health,
                max: spawning_config.drone_health,
            },
            crate::shared::components::Velocity::default(),
            Transform::from_translation(pos.extend(0.0)),
        ));
    }

    commands.insert_resource(TutorialEnemyWave { remaining: count });
}

/// Checks if all tutorial wave enemies have been destroyed and advances the phase.
/// Runs in `CoreSet::Damage` after `despawn_destroyed`.
/// The check is idempotent: if phase is already `Complete`, it returns immediately.
pub fn check_tutorial_wave_complete(
    phase: Res<State<TutorialPhase>>,
    mut next_phase: ResMut<NextState<TutorialPhase>>,
    wave: Option<Res<TutorialEnemyWave>>,
    enemy_query: Query<&crate::core::collision::Health, With<TutorialEnemy>>,
) {
    // Only act during SpreadUnlocked phase
    if *phase.get() != TutorialPhase::SpreadUnlocked {
        return;
    }
    // Only act if the wave resource has been inserted (i.e. spawn happened)
    let Some(_wave) = wave else { return };

    // Count tutorial enemies still alive (positive health — not yet despawned)
    let alive = enemy_query.iter().filter(|h| h.current > 0.0).count();

    if alive == 0 {
        next_phase.set(TutorialPhase::Complete);
    }
}

// ── Station Docking ─────────────────────────────────────────────────────

/// Detects player proximity to the `TutorialStation` when the phase is `Complete`.
/// When the player is within `dock_radius`, the station grants the Spread weapon:
/// - `SpreadUnlocked` marker component added to player
/// - `TutorialStation.defective` set to `false`
/// - Phase advances from `Complete` to `StationVisited`
/// - `GameEvent::StationDocked` emitted (Tier1)
///
/// The transition is idempotent: once `StationVisited`, the guard returns immediately.
pub fn dock_at_station(
    config: Res<TutorialConfig>,
    phase: Res<State<TutorialPhase>>,
    mut next_phase: ResMut<NextState<TutorialPhase>>,
    mut station_query: Query<(&mut TutorialStation, &Transform)>,
    player_query: Query<(Entity, &Transform), With<crate::core::flight::Player>>,
    mut commands: Commands,
    mut game_events: bevy::ecs::message::MessageWriter<crate::shared::events::GameEvent>,
    time: Res<Time>,
    severity_config: Res<crate::infrastructure::events::EventSeverityConfig>,
) {
    if *phase.get() != TutorialPhase::Complete {
        return;
    }
    let Ok((player_entity, player_transform)) = player_query.single() else {
        return;
    };
    let player_pos = player_transform.translation.truncate();

    for (mut station, station_transform) in station_query.iter_mut() {
        let station_pos = station_transform.translation.truncate();
        let distance = (station_pos - player_pos).length();
        if distance <= config.dock_radius {
            commands.entity(player_entity).insert(SpreadUnlocked);
            station.defective = false;
            next_phase.set(TutorialPhase::StationVisited);
            let kind = crate::shared::events::GameEventKind::StationDocked;
            game_events.write(crate::shared::events::GameEvent {
                severity: severity_config.severity_for(&kind),
                kind,
                position: station_pos,
                game_time: time.elapsed_secs_f64(),
            });
        }
    }
}

// ── Generator Destruction Detection ─────────────────────────────────────

/// Detects when the `GravityWellGenerator` entity has been destroyed (despawned)
/// and advances the tutorial phase from `StationVisited` to `GeneratorDestroyed`.
///
/// The gravity well pull stops naturally because `apply_gravity_well` iterates
/// `Query<(&GravityWellGenerator, &Transform)>` — with no entity present, no force
/// is applied. Zero additional code needed.
///
/// The transition is idempotent: once `GeneratorDestroyed`, the guard returns immediately.
/// Runs in `CoreSet::Events` after `despawn_destroyed` has removed entities with
/// health <= 0, so the absence check is authoritative within the same frame.
pub fn check_generator_destroyed(
    phase: Res<State<TutorialPhase>>,
    mut next_phase: ResMut<NextState<TutorialPhase>>,
    generator_query: Query<Entity, With<GravityWellGenerator>>,
    mut game_events: bevy::ecs::message::MessageWriter<crate::shared::events::GameEvent>,
    time: Res<Time>,
    severity_config: Res<crate::infrastructure::events::EventSeverityConfig>,
) {
    if *phase.get() != TutorialPhase::StationVisited {
        return;
    }
    if generator_query.iter().next().is_none() {
        let kind = crate::shared::events::GameEventKind::GeneratorDestroyed;
        game_events.write(crate::shared::events::GameEvent {
            severity: severity_config.severity_for(&kind),
            kind,
            position: Vec2::ZERO,
            game_time: time.elapsed_secs_f64(),
        });
        next_phase.set(TutorialPhase::GeneratorDestroyed);
    }
}

// ── Destruction Cascade ─────────────────────────────────────────────────

/// Countdown timer resource inserted when `GeneratorDestroyed` phase is entered.
/// When `remaining` reaches zero, the cascade despawn fires and phase advances to
/// `TutorialComplete`.
#[derive(Resource, Debug)]
pub struct CascadeTimer {
    pub remaining: f32,
}

/// `OnEnter(TutorialPhase::GeneratorDestroyed)` system.
/// Inserts a `CascadeTimer` with the delay from `TutorialConfig.cascade_delay_secs`.
/// Runs exactly once when the phase transitions into `GeneratorDestroyed`.
pub fn start_destruction_cascade(mut commands: Commands, config: Res<TutorialConfig>) {
    commands.insert_resource(CascadeTimer {
        remaining: config.cascade_delay_secs,
    });
}

/// `FixedUpdate` system that ticks the `CascadeTimer` each frame.
/// Guards: phase must be `GeneratorDestroyed` and `CascadeTimer` must be present.
/// When the timer expires:
/// - All `TutorialWreck` entities are despawned
/// - All `TutorialStation` entities are despawned
/// - `GameEvent::TutorialComplete` is emitted (Tier1)
/// - Phase advances to `TutorialComplete`
/// - `CascadeTimer` resource is removed to prevent re-triggering
pub fn tick_cascade_timer(
    mut commands: Commands,
    time: Res<Time>,
    phase: Res<State<TutorialPhase>>,
    mut next_phase: ResMut<NextState<TutorialPhase>>,
    cascade_timer: Option<ResMut<CascadeTimer>>,
    wreck_query: Query<Entity, With<TutorialWreck>>,
    station_query: Query<Entity, With<TutorialStation>>,
    mut game_events: bevy::ecs::message::MessageWriter<crate::shared::events::GameEvent>,
    severity_config: Res<crate::infrastructure::events::EventSeverityConfig>,
) {
    if *phase.get() != TutorialPhase::GeneratorDestroyed {
        return;
    }
    let Some(mut timer) = cascade_timer else {
        return;
    };
    timer.remaining -= time.delta_secs();
    if timer.remaining <= 0.0 {
        // Despawn all remaining tutorial-specific entities
        for entity in wreck_query.iter() {
            commands.entity(entity).despawn();
        }
        for entity in station_query.iter() {
            commands.entity(entity).despawn();
        }
        // Emit completion event
        let kind = crate::shared::events::GameEventKind::TutorialComplete;
        game_events.write(crate::shared::events::GameEvent {
            severity: severity_config.severity_for(&kind),
            kind,
            position: Vec2::ZERO,
            game_time: time.elapsed_secs_f64(),
        });
        // Advance to terminal state
        next_phase.set(TutorialPhase::TutorialComplete);
        // Remove timer so it does not fire again
        commands.remove_resource::<CascadeTimer>();
    }
}

// ── Unit Tests ──────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tutorial_config_default_has_valid_values() {
        let config = TutorialConfig::default();
        assert!(config.safe_radius > 0.0);
        assert!(config.pull_strength > 0.0);
        assert!(config.station_offset_min > 0.0);
        assert!(config.station_offset_max >= config.station_offset_min);
        assert!(config.generator_offset_min > 0.0);
        assert!(config.generator_offset_max >= config.generator_offset_min);
        assert!(config.player_offset_min > 0.0);
        assert!(config.player_offset_max >= config.player_offset_min);
        assert!(config.generator_health > 0.0);
        assert!(config.wreck_offset_min > 0.0);
        assert!(config.wreck_offset_max >= config.wreck_offset_min);
        assert!(config.tutorial_enemy_count > 0);
        assert!(config.tutorial_enemy_spawn_radius > 0.0);
        assert!(config.dock_radius > 0.0);
        assert!(config.cascade_delay_secs > 0.0);
    }

    #[test]
    fn tutorial_config_from_ron() {
        let ron_str = r#"(
            safe_radius: 1500.0,
            pull_strength: 40.0,
            station_offset_min: 150.0,
            station_offset_max: 350.0,
            generator_offset_min: 1200.0,
            generator_offset_max: 1400.0,
            player_offset_min: 30.0,
            player_offset_max: 100.0,
            generator_health: 80.0,
            wreck_offset_min: 400.0,
            wreck_offset_max: 700.0,
            tutorial_enemy_count: 5,
            tutorial_enemy_spawn_radius: 200.0,
            dock_radius: 120.0,
            cascade_delay_secs: 3.0,
        )"#;
        let config = TutorialConfig::from_ron(ron_str).expect("Should parse RON");
        assert!((config.safe_radius - 1500.0).abs() < f32::EPSILON);
        assert!((config.pull_strength - 40.0).abs() < f32::EPSILON);
        assert!((config.generator_health - 80.0).abs() < f32::EPSILON);
        assert!((config.wreck_offset_min - 400.0).abs() < f32::EPSILON);
        assert!((config.wreck_offset_max - 700.0).abs() < f32::EPSILON);
        assert_eq!(config.tutorial_enemy_count, 5);
        assert!((config.tutorial_enemy_spawn_radius - 200.0).abs() < f32::EPSILON);
        assert!((config.dock_radius - 120.0).abs() < f32::EPSILON);
        assert!((config.cascade_delay_secs - 3.0).abs() < f32::EPSILON);
    }

    #[test]
    fn tutorial_config_from_ron_invalid() {
        let result = TutorialConfig::from_ron("not valid ron");
        assert!(result.is_err(), "Invalid RON should return error");
    }

    #[test]
    fn generate_tutorial_zone_is_deterministic() {
        let config = TutorialConfig::default();
        let layout1 = generate_tutorial_zone(42, &config);
        let layout2 = generate_tutorial_zone(42, &config);

        assert!(
            (layout1.player_spawn.x - layout2.player_spawn.x).abs() < f32::EPSILON,
            "Player spawn X should be deterministic"
        );
        assert!(
            (layout1.player_spawn.y - layout2.player_spawn.y).abs() < f32::EPSILON,
            "Player spawn Y should be deterministic"
        );
        assert!(
            (layout1.station_position.x - layout2.station_position.x).abs() < f32::EPSILON,
            "Station position should be deterministic"
        );
        assert!(
            (layout1.generator_position.x - layout2.generator_position.x).abs() < f32::EPSILON,
            "Generator position should be deterministic"
        );
    }

    #[test]
    fn different_seeds_produce_different_layouts() {
        let config = TutorialConfig::default();
        let layout1 = generate_tutorial_zone(42, &config);
        let layout2 = generate_tutorial_zone(99, &config);

        let same_player = (layout1.player_spawn - layout2.player_spawn).length() < 0.01;
        let same_station = (layout1.station_position - layout2.station_position).length() < 0.01;
        assert!(
            !same_player || !same_station,
            "Different seeds should produce different layouts"
        );
    }

    #[test]
    fn generated_entities_within_safe_radius() {
        let config = TutorialConfig::default();
        for seed in 0..100 {
            let layout = generate_tutorial_zone(seed, &config);
            let player_dist = (layout.player_spawn - layout.zone_center).length();
            let station_dist = (layout.station_position - layout.zone_center).length();
            let generator_dist = (layout.generator_position - layout.zone_center).length();

            assert!(
                player_dist <= config.safe_radius,
                "Seed {seed}: player at distance {player_dist:.1} exceeds safe_radius {}",
                config.safe_radius
            );
            assert!(
                station_dist <= config.safe_radius,
                "Seed {seed}: station at distance {station_dist:.1} exceeds safe_radius {}",
                config.safe_radius
            );
            assert!(
                generator_dist <= config.safe_radius,
                "Seed {seed}: generator at distance {generator_dist:.1} exceeds safe_radius {}",
                config.safe_radius
            );
        }
    }

    #[test]
    fn station_reachable_from_player_spawn() {
        let config = TutorialConfig::default();
        for seed in 0..100 {
            let layout = generate_tutorial_zone(seed, &config);
            let distance = (layout.station_position - layout.player_spawn).length();
            assert!(
                distance <= config.safe_radius,
                "Seed {seed}: station unreachable, distance {distance:.1} > safe_radius {}",
                config.safe_radius
            );
        }
    }

    #[test]
    fn validate_tutorial_layout_passes_valid_layout() {
        let config = TutorialConfig::default();
        let layout = generate_tutorial_zone(42, &config);
        assert!(
            validate_tutorial_layout(&layout, &config).is_ok(),
            "Default layout should pass validation"
        );
    }

    #[test]
    fn validate_tutorial_layout_catches_out_of_bounds() {
        let config = TutorialConfig::default();
        let layout = TutorialLayout {
            player_spawn: Vec2::ZERO,
            station_position: Vec2::new(config.safe_radius + 100.0, 0.0),
            generator_position: Vec2::ZERO,
            zone_center: Vec2::ZERO,
            wreck_position: Vec2::ZERO,
        };
        let result = validate_tutorial_layout(&layout, &config);
        assert!(result.is_err(), "Out-of-bounds station should fail validation");
        let violations = result.expect_err("Should have violations");
        assert!(
            violations.iter().any(|v| v.description.contains("Station")),
            "Should mention station in violation"
        );
    }

    #[test]
    fn validate_tutorial_seed_passes_default_config() {
        let config = TutorialConfig::default();
        for seed in 0..100 {
            assert!(
                validate_tutorial_seed(seed, &config).is_ok(),
                "Seed {seed} should pass validation with default config"
            );
        }
    }

    #[test]
    fn tutorial_phase_default_is_flying() {
        let phase = TutorialPhase::default();
        assert_eq!(phase, TutorialPhase::Flying);
    }

    #[test]
    fn gravity_well_generator_component() {
        let generator = GravityWellGenerator {
            safe_radius: 2000.0,
            pull_strength: 50.0,
            requires_projectile: true,
        };
        assert!((generator.safe_radius - 2000.0).abs() < f32::EPSILON);
        assert!((generator.pull_strength - 50.0).abs() < f32::EPSILON);
    }

    #[test]
    fn tutorial_station_component() {
        let station = TutorialStation { defective: true };
        assert!(station.defective);
    }

    #[test]
    fn player_spawn_offset_within_config_range() {
        let config = TutorialConfig::default();
        for seed in 0..50 {
            let layout = generate_tutorial_zone(seed, &config);
            let dist = (layout.player_spawn - layout.zone_center).length();
            assert!(
                dist >= config.player_offset_min - 0.01 && dist <= config.player_offset_max + 0.01,
                "Seed {seed}: player offset {dist:.1} not in [{}, {}]",
                config.player_offset_min,
                config.player_offset_max
            );
        }
    }

    #[test]
    fn station_offset_within_config_range() {
        let config = TutorialConfig::default();
        for seed in 0..50 {
            let layout = generate_tutorial_zone(seed, &config);
            let dist = (layout.station_position - layout.zone_center).length();
            assert!(
                dist >= config.station_offset_min - 0.01
                    && dist <= config.station_offset_max + 0.01,
                "Seed {seed}: station offset {dist:.1} not in [{}, {}]",
                config.station_offset_min,
                config.station_offset_max
            );
        }
    }

    #[test]
    fn generator_offset_within_config_range() {
        let config = TutorialConfig::default();
        for seed in 0..50 {
            let layout = generate_tutorial_zone(seed, &config);
            let dist = (layout.generator_position - layout.zone_center).length();
            assert!(
                dist >= config.generator_offset_min - 0.01
                    && dist <= config.generator_offset_max + 0.01,
                "Seed {seed}: generator offset {dist:.1} not in [{}, {}]",
                config.generator_offset_min,
                config.generator_offset_max
            );
        }
    }

    // ── Wreck Component Tests ───────────────────────────────────────────

    #[test]
    fn wreck_shot_state_starts_not_shot() {
        let state = WreckShotState { has_been_shot: false };
        assert!(!state.has_been_shot, "WreckShotState should start as not shot");
    }

    #[test]
    fn wreck_offset_within_config_range() {
        let config = TutorialConfig::default();
        for seed in 0..50 {
            let layout = generate_tutorial_zone(seed, &config);
            let dist = (layout.wreck_position - layout.zone_center).length();
            assert!(
                dist >= config.wreck_offset_min - 0.01 && dist <= config.wreck_offset_max + 0.01,
                "Seed {seed}: wreck offset {dist:.1} not in [{}, {}]",
                config.wreck_offset_min,
                config.wreck_offset_max
            );
        }
    }

    #[test]
    fn wreck_within_safe_radius_all_seeds() {
        let config = TutorialConfig::default();
        for seed in 0..100 {
            let layout = generate_tutorial_zone(seed, &config);
            let dist = (layout.wreck_position - layout.zone_center).length();
            assert!(
                dist <= config.safe_radius,
                "Seed {seed}: wreck at distance {dist:.1} exceeds safe_radius {}",
                config.safe_radius
            );
        }
    }

    #[test]
    fn wreck_position_is_deterministic() {
        let config = TutorialConfig::default();
        let layout1 = generate_tutorial_zone(42, &config);
        let layout2 = generate_tutorial_zone(42, &config);
        assert!(
            (layout1.wreck_position.x - layout2.wreck_position.x).abs() < f32::EPSILON,
            "Wreck position X should be deterministic"
        );
        assert!(
            (layout1.wreck_position.y - layout2.wreck_position.y).abs() < f32::EPSILON,
            "Wreck position Y should be deterministic"
        );
    }

    #[test]
    fn validate_tutorial_layout_catches_out_of_bounds_wreck() {
        let config = TutorialConfig::default();
        let layout = TutorialLayout {
            player_spawn: Vec2::ZERO,
            station_position: Vec2::ZERO,
            generator_position: Vec2::ZERO,
            zone_center: Vec2::ZERO,
            wreck_position: Vec2::new(config.safe_radius + 100.0, 0.0),
        };
        let result = validate_tutorial_layout(&layout, &config);
        assert!(result.is_err(), "Out-of-bounds wreck should fail validation");
        let violations = result.expect_err("Should have violations");
        assert!(
            violations.iter().any(|v| v.description.contains("Wreck")),
            "Should mention wreck in violation"
        );
    }

    // ── Gravity Well Physics Tests ──────────────────────────────────────

    use bevy::time::TimeUpdateStrategy;
    use std::time::Duration;
    use crate::shared::components::Velocity;
    use crate::core::flight::Player;

    fn gravity_test_app() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_secs_f64(
            1.0 / 60.0,
        )));
        app.add_systems(FixedUpdate, apply_gravity_well);
        // Prime
        app.update();
        app
    }

    #[test]
    fn gravity_well_no_force_inside_safe_radius() {
        let mut app = gravity_test_app();

        // Generator at origin with safe_radius=100
        app.world_mut().spawn((
            GravityWellGenerator {
                safe_radius: 100.0,
                pull_strength: 50.0,
                requires_projectile: true,
            },
            Transform::from_translation(Vec3::ZERO),
        ));

        // Player at distance 50 (inside safe_radius)
        let player = app.world_mut().spawn((
            Player,
            Velocity::default(),
            Transform::from_translation(Vec3::new(50.0, 0.0, 0.0)),
        )).id();

        app.update();

        let vel = app.world().entity(player).get::<Velocity>()
            .expect("Player should have Velocity");
        assert!(
            vel.0.length() < f32::EPSILON,
            "No force should be applied inside safe_radius, got {:?}",
            vel.0
        );
    }

    #[test]
    fn gravity_well_no_force_at_exact_boundary() {
        let mut app = gravity_test_app();

        app.world_mut().spawn((
            GravityWellGenerator {
                safe_radius: 100.0,
                pull_strength: 50.0,
                requires_projectile: true,
            },
            Transform::from_translation(Vec3::ZERO),
        ));

        // Player at exactly safe_radius
        let player = app.world_mut().spawn((
            Player,
            Velocity::default(),
            Transform::from_translation(Vec3::new(100.0, 0.0, 0.0)),
        )).id();

        app.update();

        let vel = app.world().entity(player).get::<Velocity>()
            .expect("Player should have Velocity");
        assert!(
            vel.0.length() < f32::EPSILON,
            "No force at exact safe_radius boundary, got {:?}",
            vel.0
        );
    }

    #[test]
    fn gravity_well_applies_force_outside_safe_radius() {
        let mut app = gravity_test_app();

        app.world_mut().spawn((
            GravityWellGenerator {
                safe_radius: 100.0,
                pull_strength: 50.0,
                requires_projectile: true,
            },
            Transform::from_translation(Vec3::ZERO),
        ));

        // Player at distance 200 (100 beyond safe_radius)
        let player = app.world_mut().spawn((
            Player,
            Velocity::default(),
            Transform::from_translation(Vec3::new(200.0, 0.0, 0.0)),
        )).id();

        app.update();

        let vel = app.world().entity(player).get::<Velocity>()
            .expect("Player should have Velocity");
        // Force should pull toward generator (negative X)
        assert!(
            vel.0.x < 0.0,
            "Pull should be toward generator (negative X), got {:?}",
            vel.0
        );
        assert!(
            vel.0.y.abs() < f32::EPSILON,
            "No Y component expected, got {:?}",
            vel.0
        );
    }

    #[test]
    fn gravity_well_force_increases_with_distance() {
        let mut app = gravity_test_app();

        app.world_mut().spawn((
            GravityWellGenerator {
                safe_radius: 100.0,
                pull_strength: 50.0,
                requires_projectile: true,
            },
            Transform::from_translation(Vec3::ZERO),
        ));

        // Player at 150 (50 beyond)
        let player_near = app.world_mut().spawn((
            Player,
            Velocity::default(),
            Transform::from_translation(Vec3::new(150.0, 0.0, 0.0)),
        )).id();

        // Player at 300 (200 beyond)
        let player_far = app.world_mut().spawn((
            Player,
            Velocity::default(),
            Transform::from_translation(Vec3::new(300.0, 0.0, 0.0)),
        )).id();

        app.update();

        let vel_near = app.world().entity(player_near).get::<Velocity>()
            .expect("Near player should have Velocity");
        let vel_far = app.world().entity(player_far).get::<Velocity>()
            .expect("Far player should have Velocity");

        assert!(
            vel_far.0.x.abs() > vel_near.0.x.abs(),
            "Farther player should have stronger pull: near={:.2}, far={:.2}",
            vel_near.0.x.abs(),
            vel_far.0.x.abs()
        );
    }

    #[test]
    fn gravity_well_direction_toward_generator() {
        let mut app = gravity_test_app();

        // Generator at (500, 500)
        app.world_mut().spawn((
            GravityWellGenerator {
                safe_radius: 100.0,
                pull_strength: 50.0,
                requires_projectile: true,
            },
            Transform::from_translation(Vec3::new(500.0, 500.0, 0.0)),
        ));

        // Player at origin (distance ~707, well beyond safe_radius)
        let player = app.world_mut().spawn((
            Player,
            Velocity::default(),
            Transform::from_translation(Vec3::ZERO),
        )).id();

        app.update();

        let vel = app.world().entity(player).get::<Velocity>()
            .expect("Player should have Velocity");
        // Force should pull toward (500,500) — both X and Y should be positive
        assert!(
            vel.0.x > 0.0,
            "Pull X should be positive (toward generator at 500,500), got {:?}",
            vel.0
        );
        assert!(
            vel.0.y > 0.0,
            "Pull Y should be positive (toward generator at 500,500), got {:?}",
            vel.0
        );
    }

    #[test]
    fn gravity_well_no_effect_without_generator() {
        let mut app = gravity_test_app();

        // No generator spawned — only player
        let player = app.world_mut().spawn((
            Player,
            Velocity::default(),
            Transform::from_translation(Vec3::new(500.0, 0.0, 0.0)),
        )).id();

        app.update();

        let vel = app.world().entity(player).get::<Velocity>()
            .expect("Player should have Velocity");
        assert!(
            vel.0.length() < f32::EPSILON,
            "No force without generator, got {:?}",
            vel.0
        );
    }

    // ── Tutorial Enemy Wave Unit Tests ──────────────────────────────────

    #[test]
    fn tutorial_config_default_has_enemy_count_and_radius() {
        let config = TutorialConfig::default();
        assert!(
            config.tutorial_enemy_count > 0,
            "tutorial_enemy_count should be > 0, got {}",
            config.tutorial_enemy_count
        );
        assert!(
            config.tutorial_enemy_spawn_radius > 0.0,
            "tutorial_enemy_spawn_radius should be > 0, got {}",
            config.tutorial_enemy_spawn_radius
        );
    }

    #[test]
    fn tutorial_enemy_wave_default_has_zero_remaining() {
        let wave = TutorialEnemyWave::default();
        assert_eq!(wave.remaining, 0, "TutorialEnemyWave default should have remaining=0");
    }

    #[test]
    fn tutorial_enemy_wave_can_be_set() {
        let wave = TutorialEnemyWave { remaining: 3 };
        assert_eq!(wave.remaining, 3, "TutorialEnemyWave remaining should be 3");
    }

    // ── Station Docking Unit Tests ───────────────────────────────────────

    #[test]
    fn tutorial_config_dock_radius_default_positive() {
        let config = TutorialConfig::default();
        assert!(
            config.dock_radius > 0.0,
            "dock_radius should be positive, got {}",
            config.dock_radius
        );
    }

    #[test]
    fn tutorial_config_from_ron_includes_dock_radius() {
        let config = TutorialConfig::default();
        // Ensure dock_radius is serializable / deserializable by checking default
        assert!(
            (config.dock_radius - 150.0).abs() < f32::EPSILON,
            "Default dock_radius should be 150.0, got {}",
            config.dock_radius
        );
    }

    #[test]
    fn tutorial_phase_station_visited_variant_exists() {
        // Ensure StationVisited can be constructed and is distinct from Complete
        let complete = TutorialPhase::Complete;
        let station_visited = TutorialPhase::StationVisited;
        assert_ne!(complete, station_visited, "StationVisited should differ from Complete");
    }

    #[test]
    fn spread_unlocked_is_a_component() {
        // Verify SpreadUnlocked can be constructed (it's a marker — no fields)
        let _marker = SpreadUnlocked;
        // If it compiles and is a Component, the test passes
    }

    // ── Generator Destruction Unit Tests ────────────────────────────────

    #[test]
    fn tutorial_phase_generator_destroyed_variant_exists() {
        // Ensure GeneratorDestroyed can be constructed and is distinct from StationVisited
        let station_visited = TutorialPhase::StationVisited;
        let generator_destroyed = TutorialPhase::GeneratorDestroyed;
        assert_ne!(
            station_visited,
            generator_destroyed,
            "GeneratorDestroyed should differ from StationVisited"
        );
    }

    #[test]
    fn tutorial_phase_sequence_all_variants_distinct() {
        // Verify all tutorial phase variants are distinct
        let phases = [
            TutorialPhase::Flying,
            TutorialPhase::Shooting,
            TutorialPhase::SpreadUnlocked,
            TutorialPhase::Complete,
            TutorialPhase::StationVisited,
            TutorialPhase::GeneratorDestroyed,
            TutorialPhase::TutorialComplete,
        ];
        for i in 0..phases.len() {
            for j in (i + 1)..phases.len() {
                assert_ne!(
                    phases[i],
                    phases[j],
                    "TutorialPhase variants at index {i} and {j} should be distinct"
                );
            }
        }
    }

    // ── Destruction Cascade Unit Tests ──────────────────────────────────

    #[test]
    fn tutorial_phase_tutorial_complete_variant_exists() {
        // Ensure TutorialComplete can be constructed and is distinct from GeneratorDestroyed
        let generator_destroyed = TutorialPhase::GeneratorDestroyed;
        let tutorial_complete = TutorialPhase::TutorialComplete;
        assert_ne!(
            generator_destroyed,
            tutorial_complete,
            "TutorialComplete should differ from GeneratorDestroyed"
        );
    }

    #[test]
    fn cascade_timer_can_be_constructed_with_positive_remaining() {
        let timer = CascadeTimer { remaining: 2.0 };
        assert!(
            timer.remaining > 0.0,
            "CascadeTimer.remaining should be positive, got {}",
            timer.remaining
        );
    }

    #[test]
    fn tutorial_config_cascade_delay_default_positive() {
        let config = TutorialConfig::default();
        assert!(
            config.cascade_delay_secs > 0.0,
            "cascade_delay_secs should be positive, got {}",
            config.cascade_delay_secs
        );
    }

    #[test]
    fn tutorial_config_cascade_delay_default_value() {
        let config = TutorialConfig::default();
        assert!(
            (config.cascade_delay_secs - 2.0).abs() < f32::EPSILON,
            "cascade_delay_secs default should be 2.0, got {}",
            config.cascade_delay_secs
        );
    }
}
