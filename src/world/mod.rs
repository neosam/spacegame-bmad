pub mod chunk;
pub mod generation;
pub mod noise_layers;

use bevy::ecs::message::MessageWriter;
use bevy::prelude::*;
use serde::Deserialize;

pub use self::chunk::ChunkCoord;
pub use self::generation::BiomeType;
use self::chunk::{chunks_in_radius, manhattan_distance, world_to_chunk};
use self::generation::{determine_biome, generate_chunk_content};
use std::collections::{HashSet, VecDeque};

use crate::core::collision::{Collider, Health};
use crate::core::flight::Player;
use crate::core::spawning::{Asteroid, BossEnemy, NeedsAsteroidVisual, NeedsBossVisual, NeedsDroneVisual, ScoutDrone};
use crate::core::station::{NeedsStationVisual, Station, StationType};
use crate::core::wormhole::{ClearedWormholes, NeedsWormholeVisual, Wormhole, should_spawn_wormhole};
use crate::infrastructure::events::EventSeverityConfig;
use crate::infrastructure::save::delta::{SeedIndex, WorldDeltas};
use crate::shared::components::Velocity;
use crate::shared::events::{GameEvent, GameEventKind};
use crate::social::enemy_ai::{AiState, BossVariant, ErraticOffset, EnemyFireCooldown};
use crate::social::faction::{
    faction_at_position, AggroRange, AttackRange, FacingDirection, FactionId, FleeThreshold,
    PatrolRadius, TurnRate,
};
use crate::game_states::PlayingSubState;

// ── World Config ─────────────────────────────────────────────────────────

/// World generation configuration loaded from `assets/config/world.ron`.
#[derive(Resource, Deserialize, Clone, Debug)]
pub struct WorldConfig {
    pub seed: u64,
    pub chunk_size: f32,
    pub load_radius: u32,
    pub entity_budget: usize,
    #[serde(default = "default_max_chunks_per_frame")]
    pub max_chunks_per_frame: usize,
    /// Health multiplier per 100 world-units of distance from origin.
    /// e.g. 0.1 means +10% health every 100 units.
    #[serde(default = "default_difficulty_health_scale_per_100u")]
    pub difficulty_health_scale_per_100u: f32,
    /// Extra enemy count multiplier per 100 world-units of distance from origin.
    /// Currently informational — used by spawn-count calculations in future stories.
    #[serde(default = "default_difficulty_count_scale_per_100u")]
    pub difficulty_count_scale_per_100u: f32,
}

fn default_max_chunks_per_frame() -> usize {
    4
}

fn default_difficulty_health_scale_per_100u() -> f32 {
    0.1
}

fn default_difficulty_count_scale_per_100u() -> f32 {
    0.05
}

impl Default for WorldConfig {
    fn default() -> Self {
        Self {
            seed: 42,
            chunk_size: 1000.0,
            load_radius: 2,
            entity_budget: 200,
            max_chunks_per_frame: 4,
            difficulty_health_scale_per_100u: 0.1,
            difficulty_count_scale_per_100u: 0.05,
        }
    }
}

impl WorldConfig {
    /// Load config from RON string.
    pub fn from_ron(ron_str: &str) -> Result<Self, ron::error::SpannedError> {
        ron::from_str(ron_str)
    }
}

// ── Story 4-11: Distance Difficulty Scaling ───────────────────────────────

/// Returns scaled (health, damage) values for an enemy at `distance` world-units from origin.
///
/// Formula: `scaled = base * (1.0 + scale_per_100u * (distance / 100.0))`
///
/// Both values are floored at their `base` values (scaling never reduces below base).
///
/// # Arguments
/// * `distance`   — Euclidean distance from world origin to enemy spawn position
/// * `base_health` — Base health value before scaling
/// * `base_damage` — Base damage value before scaling
/// * `health_scale_per_100u` — Fractional health increase per 100 world-units
pub fn enemy_stats_for_distance(
    distance: f32,
    base_health: f32,
    base_damage: f32,
    health_scale_per_100u: f32,
) -> (f32, f32) {
    let tiers = (distance / 100.0).max(0.0);
    let multiplier = 1.0 + health_scale_per_100u * tiers;
    let scaled_health = base_health * multiplier;
    let scaled_damage = base_damage * multiplier;
    (scaled_health, scaled_damage)
}

// ── Biome Config ─────────────────────────────────────────────────────────

/// Per-biome spawn parameters for entity generation.
#[derive(Deserialize, Clone, Debug)]
pub struct BiomeSpawnParams {
    pub asteroid_count_min: u32,
    pub asteroid_count_max: u32,
    pub drone_count_min: u32,
    pub drone_count_max: u32,
    pub asteroid_health: f32,
    pub asteroid_radius: f32,
    pub drone_health: f32,
    pub drone_radius: f32,
    pub asteroid_velocity_min: f32,
    pub asteroid_velocity_max: f32,
    pub drone_velocity_min: f32,
    pub drone_velocity_max: f32,
}

/// Biome configuration loaded from `assets/config/biome.ron`.
#[derive(Resource, Deserialize, Clone, Debug)]
pub struct BiomeConfig {
    pub deep_space_threshold: f32,
    pub asteroid_field_threshold: f32,
    pub deep_space: BiomeSpawnParams,
    pub asteroid_field: BiomeSpawnParams,
    pub wreck_field: BiomeSpawnParams,
    #[serde(default)]
    pub noise: noise_layers::BiomeNoiseConfig,
}

impl BiomeConfig {
    /// Returns the spawn parameters for the given biome type.
    pub fn params_for(&self, biome: BiomeType) -> &BiomeSpawnParams {
        match biome {
            BiomeType::DeepSpace => &self.deep_space,
            BiomeType::AsteroidField => &self.asteroid_field,
            BiomeType::WreckField => &self.wreck_field,
        }
    }

    /// Load config from RON string.
    pub fn from_ron(ron_str: &str) -> Result<Self, ron::error::SpannedError> {
        ron::from_str(ron_str)
    }

    /// Warns if thresholds are in invalid order.
    pub fn validate_thresholds(&self) {
        if self.deep_space_threshold >= self.asteroid_field_threshold {
            warn!(
                "BiomeConfig: deep_space_threshold ({}) >= asteroid_field_threshold ({}). \
                 AsteroidField biome will never be selected.",
                self.deep_space_threshold, self.asteroid_field_threshold
            );
        }
        if self.deep_space_threshold < 0.0 || self.deep_space_threshold > 1.0 {
            warn!(
                "BiomeConfig: deep_space_threshold ({}) is outside [0.0, 1.0].",
                self.deep_space_threshold
            );
        }
        if self.asteroid_field_threshold < 0.0 || self.asteroid_field_threshold > 1.0 {
            warn!(
                "BiomeConfig: asteroid_field_threshold ({}) is outside [0.0, 1.0].",
                self.asteroid_field_threshold
            );
        }
    }
}

impl Default for BiomeConfig {
    fn default() -> Self {
        Self {
            deep_space_threshold: 0.3,
            asteroid_field_threshold: 0.7,
            deep_space: BiomeSpawnParams {
                asteroid_count_min: 0,
                asteroid_count_max: 2,
                drone_count_min: 0,
                drone_count_max: 1,
                asteroid_health: 50.0,
                asteroid_radius: 20.0,
                drone_health: 30.0,
                drone_radius: 10.0,
                asteroid_velocity_min: 5.0,
                asteroid_velocity_max: 15.0,
                drone_velocity_min: 10.0,
                drone_velocity_max: 25.0,
            },
            asteroid_field: BiomeSpawnParams {
                asteroid_count_min: 6,
                asteroid_count_max: 12,
                drone_count_min: 1,
                drone_count_max: 3,
                asteroid_health: 50.0,
                asteroid_radius: 20.0,
                drone_health: 30.0,
                drone_radius: 10.0,
                asteroid_velocity_min: 5.0,
                asteroid_velocity_max: 15.0,
                drone_velocity_min: 10.0,
                drone_velocity_max: 25.0,
            },
            wreck_field: BiomeSpawnParams {
                asteroid_count_min: 2,
                asteroid_count_max: 5,
                drone_count_min: 2,
                drone_count_max: 4,
                asteroid_health: 80.0,
                asteroid_radius: 25.0,
                drone_health: 30.0,
                drone_radius: 10.0,
                asteroid_velocity_min: 2.0,
                asteroid_velocity_max: 8.0,
                drone_velocity_min: 5.0,
                drone_velocity_max: 15.0,
            },
            noise: noise_layers::BiomeNoiseConfig::default(),
        }
    }
}

// ── Components ──────────────────────────────────────────────────────────

/// Links an entity to the chunk that spawned it.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct ChunkEntity {
    pub coord: ChunkCoord,
}

// ── Resources ───────────────────────────────────────────────────────────

/// Tracks which chunks are currently loaded and their biome types.
#[derive(Resource, Default, Debug)]
pub struct ActiveChunks {
    pub chunks: std::collections::HashMap<ChunkCoord, BiomeType>,
}

/// Tracks all chunks the player has ever visited (permanent discovery).
/// Used by the world map to show explored areas.
#[derive(Resource, Default, Debug)]
pub struct ExploredChunks {
    pub chunks: std::collections::HashMap<ChunkCoord, BiomeType>,
}

/// Tracks entities per chunk for O(1) despawn lookup.
#[derive(Resource, Default, Debug)]
pub struct ChunkEntityIndex {
    pub chunks: std::collections::HashMap<ChunkCoord, Vec<Entity>>,
}

impl ChunkEntityIndex {
    /// Total entity count across all chunks.
    pub fn entity_count(&self) -> usize {
        self.chunks.values().map(|v| v.len()).sum()
    }
}

/// Tracks the chunks that belong to the tutorial zone and must never be unloaded
/// by the procedural chunk system, even when the player is far away.
///
/// Populated at startup by `init_tutorial_zone_chunks`. Chunk (0,0) and its
/// immediate neighbours are registered so that the tutorial zone remains intact
/// after the player leaves and respawns.
#[derive(Resource, Default, Debug)]
pub struct TutorialZoneChunks {
    pub coords: HashSet<ChunkCoord>,
}

/// Startup system that registers tutorial-zone chunks in `TutorialZoneChunks`.
/// The tutorial zone is always centered on chunk (0,0).
pub fn init_tutorial_zone_chunks(mut tutorial_chunks: ResMut<TutorialZoneChunks>) {
    // Register (0,0) and its direct 8-neighbours (radius 1).
    for dx in -1..=1i32 {
        for dy in -1..=1i32 {
            tutorial_chunks.coords.insert(ChunkCoord { x: dx, y: dy });
        }
    }
}

/// Queue of chunks waiting to be loaded, sorted by distance (nearest first).
#[derive(Resource, Default, Debug)]
pub struct PendingChunks {
    pub chunks: std::collections::VecDeque<ChunkCoord>,
}

/// Tracks player's last known chunk for change detection.
#[derive(Resource, Default, Debug)]
pub struct ChunkLoadState {
    pub last_player_chunk: Option<ChunkCoord>,
}

// ── Systems ─────────────────────────────────────────────────────────────

/// Computes the desired set of chunks from the player's position,
/// spawns entities for new chunks, and despawns entities for removed chunks.
/// Uses `ChunkEntityIndex` for O(1) despawn, `PendingChunks` for staggered loading,
/// and `ChunkLoadState` for chunk-change detection.
#[allow(clippy::too_many_arguments)]
pub fn update_chunks(
    mut commands: Commands,
    player_query: Query<&Transform, With<Player>>,
    config: Res<WorldConfig>,
    biome_config: Res<BiomeConfig>,
    mut active_chunks: ResMut<ActiveChunks>,
    mut explored_chunks: ResMut<ExploredChunks>,
    mut chunk_entity_index: ResMut<ChunkEntityIndex>,
    mut pending_chunks: ResMut<PendingChunks>,
    mut chunk_load_state: ResMut<ChunkLoadState>,
    tutorial_chunks: Res<TutorialZoneChunks>,
    cleared_wormholes: Option<Res<ClearedWormholes>>,
    all_collidable: Query<Entity, With<Collider>>,
    mut game_events: MessageWriter<GameEvent>,
    time: Res<Time>,
    severity_config: Res<EventSeverityConfig>,
    world_deltas: Res<WorldDeltas>,
) {
    let Ok(player_transform) = player_query.single() else {
        return;
    };

    let player_pos = Vec2::new(
        player_transform.translation.x,
        player_transform.translation.y,
    );
    let player_chunk = world_to_chunk(player_pos, config.chunk_size);
    let desired = chunks_in_radius(player_chunk, config.load_radius);

    // Phase 1: UNLOAD (immediate — all at once, not deferred)
    // Tutorial-zone chunks are never unloaded so the zone survives player absence.
    let mut to_unload: Vec<ChunkCoord> = active_chunks
        .chunks
        .keys()
        .filter(|k| !desired.contains(k) && !tutorial_chunks.coords.contains(k))
        .copied()
        .collect();
    to_unload.sort();
    let mut despawned_count = 0usize;
    for coord in &to_unload {
        if let Some(entities) = chunk_entity_index.chunks.remove(coord) {
            for entity in &entities {
                if let Ok(mut entity_cmds) = commands.get_entity(*entity) {
                    entity_cmds.despawn();
                }
            }
            despawned_count += entities.len();
        }
        active_chunks.chunks.remove(coord);

        let kind = GameEventKind::ChunkUnloaded { coord: *coord };
        game_events.write(GameEvent {
            severity: severity_config.severity_for(&kind),
            kind,
            position: chunk::chunk_to_world_center(*coord, config.chunk_size),
            game_time: time.elapsed_secs_f64(),
        });
    }

    // Phase 2: QUEUE (only on chunk change or first frame)
    let chunk_changed = chunk_load_state.last_player_chunk != Some(player_chunk);
    if chunk_changed {
        let active_set = &active_chunks.chunks;
        let mut new_pending: Vec<ChunkCoord> = desired
            .iter()
            .filter(|c| !active_set.contains_key(c))
            .copied()
            .collect();
        new_pending.sort_by_key(|c| (manhattan_distance(*c, player_chunk), *c));
        pending_chunks.chunks = VecDeque::from(new_pending);
        chunk_load_state.last_player_chunk = Some(player_chunk);
    }

    // Phase 3: LOAD (staggered — up to max_chunks_per_frame)
    let mut total_entity_count = all_collidable.iter().count().saturating_sub(despawned_count);
    let mut loaded = 0usize;

    while loaded < config.max_chunks_per_frame {
        let Some(coord) = pending_chunks.chunks.pop_front() else {
            break;
        };
        if active_chunks.chunks.contains_key(&coord) {
            continue; // already loaded
        }
        if !desired.contains(&coord) {
            continue; // no longer desired
        }

        let biome = determine_biome(config.seed, coord, &biome_config);
        let blueprints =
            generate_chunk_content(config.seed, coord, config.chunk_size, biome, &biome_config);
        let remaining_budget = config.entity_budget.saturating_sub(total_entity_count);

        // Get destroyed indices for this chunk from WorldDeltas
        let destroyed = world_deltas
            .deltas
            .get(&coord)
            .map(|d| &d.destroyed[..])
            .unwrap_or(&[]);

        let mut chunk_entities = Vec::with_capacity(remaining_budget);
        for (i, blueprint) in blueprints.into_iter().enumerate() {
            if chunk_entities.len() >= remaining_budget {
                break;
            }
            // Skip entities that were destroyed in a previous session
            if destroyed.contains(&(i as u32)) {
                continue;
            }
            let chunk_marker = ChunkEntity { coord };
            let seed_index = SeedIndex(i as u32);
            let entity = match blueprint.entity_type {
                generation::BlueprintType::Asteroid => commands
                    .spawn((
                        Asteroid,
                        NeedsAsteroidVisual,
                        Collider {
                            radius: blueprint.radius,
                        },
                        Health {
                            current: blueprint.health,
                            max: blueprint.health,
                        },
                        Velocity(blueprint.velocity),
                        Transform::from_translation(blueprint.position.extend(0.0)),
                        chunk_marker,
                        biome,
                        seed_index,
                    ))
                    .id(),
                generation::BlueprintType::ScoutDrone => {
                    // Story 4-11: Scale health with distance from origin
                    let distance = blueprint.position.length();
                    let (scaled_health, _scaled_damage) = enemy_stats_for_distance(
                        distance,
                        blueprint.health,
                        5.0, // base drone shot damage
                        config.difficulty_health_scale_per_100u,
                    );
                    commands
                    .spawn((
                        ScoutDrone,
                        NeedsDroneVisual,
                        Collider { radius: blueprint.radius },
                        Health { current: scaled_health, max: scaled_health },
                        Velocity(blueprint.velocity),
                        Transform::from_translation(blueprint.position.extend(0.0)),
                        chunk_marker,
                        biome,
                        seed_index,
                    ))
                    .insert((
                        // Story 4-1+4-7: AI components with faction-territory-based faction
                        faction_at_position(
                            blueprint.position.x,
                            blueprint.position.y,
                            config.seed as u32,
                        ),
                        AiState::Idle,
                        AggroRange(200.0),
                        AttackRange(80.0),
                        FleeThreshold(0.2),
                        PatrolRadius(150.0),
                        ErraticOffset::default(),
                        EnemyFireCooldown::default(),
                        // Story 4-8: Attack Telegraphing
                        FacingDirection::default(),
                        TurnRate(3.0),
                    ))
                    .id()
                },
                generation::BlueprintType::Station => {
                    let station_type = match rand::random::<u8>() % 3 {
                        0 => StationType::TradingPost,
                        1 => StationType::RepairStation,
                        _ => StationType::BlackMarket,
                    };
                    commands
                        .spawn((
                            Station {
                                name: station_type.display_name(),
                                dock_radius: 120.0,
                                station_type,
                            },
                            NeedsStationVisual,
                            Transform::from_translation(blueprint.position.extend(0.0)),
                            chunk_marker,
                            biome,
                            seed_index,
                        ))
                        .id()
                },
                generation::BlueprintType::Boss => {
                    // Story 7-1: Boss enemy with full AI components
                    let boss_faction = faction_at_position(
                        blueprint.position.x,
                        blueprint.position.y,
                        config.seed as u32,
                    );
                    // Story 7-3: Derive BossVariant from faction
                    let boss_variant = match &boss_faction {
                        FactionId::Pirates     => BossVariant::PirateWarlord,
                        FactionId::Military    => BossVariant::Admiral,
                        FactionId::Aliens      => BossVariant::HiveMind,
                        FactionId::RogueDrones => BossVariant::AlphaDrone,
                        FactionId::Neutral     => BossVariant::Admiral, // fallback
                    };
                    // Scale boss health with distance
                    let distance = blueprint.position.length();
                    let (scaled_health, _) = enemy_stats_for_distance(
                        distance,
                        blueprint.health,
                        40.0, // boss contact damage base
                        config.difficulty_health_scale_per_100u,
                    );
                    let entity = commands
                        .spawn((
                            BossEnemy,
                            NeedsBossVisual,
                            Collider { radius: blueprint.radius },
                            Health { current: scaled_health, max: scaled_health },
                            Velocity(Vec2::ZERO),
                            Transform::from_translation(blueprint.position.extend(0.0)),
                            chunk_marker,
                            biome,
                            seed_index,
                        ))
                        .insert((
                            boss_faction.clone(),
                            boss_variant,
                            AiState::Idle,
                            AggroRange(350.0),
                            AttackRange(150.0),
                            FleeThreshold(0.2), // Story 7-5: bosses flee at <20% HP
                            EnemyFireCooldown::default(),
                            FacingDirection::default(),
                            TurnRate(1.5), // Slow, deliberate turning
                        ))
                        .id();
                    // Emit BossSpawned event
                    let kind = GameEventKind::BossSpawned { faction: boss_faction };
                    game_events.write(GameEvent {
                        severity: severity_config.severity_for(&kind),
                        kind,
                        position: blueprint.position,
                        game_time: time.elapsed_secs_f64(),
                    });
                    entity
                },
            };
            chunk_entities.push(entity);
        }

        total_entity_count += chunk_entities.len();
        chunk_entity_index.chunks.insert(coord, chunk_entities);
        active_chunks.chunks.insert(coord, biome);
        explored_chunks.chunks.entry(coord).or_insert(biome);

        // Story 9-1: Spawn a Wormhole entity for eligible chunks
        if should_spawn_wormhole(coord, config.seed) {
            let chunk_center = chunk::chunk_to_world_center(coord, config.chunk_size);
            // Story 9-4: Restore cleared flag from persistent ClearedWormholes resource
            let already_cleared = cleared_wormholes
                .as_ref()
                .map(|cw| cw.coords.contains(&coord))
                .unwrap_or(false);
            let wormhole_entity = commands.spawn((
                Wormhole { coord, cleared: already_cleared },
                NeedsWormholeVisual,
                Transform::from_translation(chunk_center.extend(0.0)),
                ChunkEntity { coord },
            )).id();
            // Register with chunk index so it is despawned on chunk unload
            chunk_entity_index
                .chunks
                .entry(coord)
                .or_default()
                .push(wormhole_entity);
        }

        let kind = GameEventKind::ChunkLoaded { coord, biome };
        game_events.write(GameEvent {
            severity: severity_config.severity_for(&kind),
            kind,
            position: chunk::chunk_to_world_center(coord, config.chunk_size),
            game_time: time.elapsed_secs_f64(),
        });

        loaded += 1;
    }
}

// ── Plugin ──────────────────────────────────────────────────────────────

pub struct WorldPlugin;

impl Plugin for WorldPlugin {
    fn build(&self, app: &mut App) {
        // Load WorldConfig
        let world_config_path = "assets/config/world.ron";
        let world_config = match std::fs::read_to_string(world_config_path) {
            Ok(contents) => match WorldConfig::from_ron(&contents) {
                Ok(config) => config,
                Err(e) => {
                    warn!("Failed to parse {world_config_path}: {e}. Using defaults.");
                    WorldConfig::default()
                }
            },
            Err(e) => {
                warn!("Failed to read {world_config_path}: {e}. Using defaults.");
                WorldConfig::default()
            }
        };

        // Load BiomeConfig
        let biome_config_path = "assets/config/biome.ron";
        let biome_config = match std::fs::read_to_string(biome_config_path) {
            Ok(contents) => match BiomeConfig::from_ron(&contents) {
                Ok(config) => config,
                Err(e) => {
                    warn!("Failed to parse {biome_config_path}: {e}. Using defaults.");
                    BiomeConfig::default()
                }
            },
            Err(e) => {
                warn!("Failed to read {biome_config_path}: {e}. Using defaults.");
                BiomeConfig::default()
            }
        };

        biome_config.validate_thresholds();
        biome_config.noise.validate();
        if world_config.max_chunks_per_frame == 0 {
            warn!("WorldConfig: max_chunks_per_frame is 0. No chunks will ever load.");
        }
        app.insert_resource(world_config);
        app.insert_resource(biome_config);
        app.init_resource::<ActiveChunks>();
        app.init_resource::<ExploredChunks>();
        app.init_resource::<ChunkEntityIndex>();
        app.init_resource::<PendingChunks>();
        app.init_resource::<ChunkLoadState>();
        app.init_resource::<TutorialZoneChunks>();
        app.add_systems(Startup, init_tutorial_zone_chunks);
        app.add_systems(
            FixedUpdate,
            update_chunks
                .before(crate::core::CoreSet::Collision)
                .run_if(in_state(PlayingSubState::Flying)),
        );
    }
}

// ── Unit tests ──────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn world_config_default_has_valid_values() {
        let config = WorldConfig::default();
        assert_eq!(config.seed, 42);
        assert!(config.chunk_size > 0.0);
        assert!(config.load_radius > 0);
        assert!(config.entity_budget > 0);
    }

    #[test]
    fn world_config_from_ron() {
        let ron_str = r#"(
            seed: 12345,
            chunk_size: 500.0,
            load_radius: 3,
            entity_budget: 150,
        )"#;
        let config = WorldConfig::from_ron(ron_str).expect("Should parse RON");
        assert_eq!(config.seed, 12345);
        assert!((config.chunk_size - 500.0).abs() < f32::EPSILON);
        assert_eq!(config.load_radius, 3);
        assert_eq!(config.entity_budget, 150);
        assert_eq!(
            config.max_chunks_per_frame, 4,
            "Omitted max_chunks_per_frame should default to 4 via serde"
        );
    }

    #[test]
    fn world_config_from_ron_invalid_falls_back() {
        let result = WorldConfig::from_ron("not valid ron");
        assert!(result.is_err(), "Invalid RON should return error");
    }

    #[test]
    fn biome_config_default_has_valid_values() {
        let config = BiomeConfig::default();
        assert!(config.deep_space_threshold > 0.0 && config.deep_space_threshold < 1.0);
        assert!(config.asteroid_field_threshold > config.deep_space_threshold);
        assert!(config.asteroid_field_threshold < 1.0);
        // Deep Space: sparse
        assert!(config.deep_space.asteroid_count_max <= 2);
        assert!(config.deep_space.drone_count_max <= 1);
        // Asteroid Field: dense
        assert!(config.asteroid_field.asteroid_count_min >= 6);
        // Wreck Field: higher drones
        assert!(config.wreck_field.drone_count_min >= 2);
    }

    #[test]
    fn biome_config_from_ron() {
        let ron_str = r#"(
            deep_space_threshold: 0.25,
            asteroid_field_threshold: 0.65,
            deep_space: (
                asteroid_count_min: 0,
                asteroid_count_max: 1,
                drone_count_min: 0,
                drone_count_max: 0,
                asteroid_health: 40.0,
                asteroid_radius: 18.0,
                drone_health: 25.0,
                drone_radius: 8.0,
                asteroid_velocity_min: 3.0,
                asteroid_velocity_max: 10.0,
                drone_velocity_min: 8.0,
                drone_velocity_max: 20.0,
            ),
            asteroid_field: (
                asteroid_count_min: 5,
                asteroid_count_max: 10,
                drone_count_min: 1,
                drone_count_max: 2,
                asteroid_health: 50.0,
                asteroid_radius: 20.0,
                drone_health: 30.0,
                drone_radius: 10.0,
                asteroid_velocity_min: 5.0,
                asteroid_velocity_max: 15.0,
                drone_velocity_min: 10.0,
                drone_velocity_max: 25.0,
            ),
            wreck_field: (
                asteroid_count_min: 2,
                asteroid_count_max: 4,
                drone_count_min: 1,
                drone_count_max: 3,
                asteroid_health: 70.0,
                asteroid_radius: 22.0,
                drone_health: 35.0,
                drone_radius: 12.0,
                asteroid_velocity_min: 2.0,
                asteroid_velocity_max: 6.0,
                drone_velocity_min: 5.0,
                drone_velocity_max: 12.0,
            ),
        )"#;
        let config = BiomeConfig::from_ron(ron_str).expect("Should parse BiomeConfig RON");
        assert!((config.deep_space_threshold - 0.25).abs() < f32::EPSILON);
        assert!((config.asteroid_field_threshold - 0.65).abs() < f32::EPSILON);
        assert_eq!(config.deep_space.asteroid_count_max, 1);
        assert_eq!(config.asteroid_field.asteroid_count_min, 5);
        assert!((config.wreck_field.asteroid_health - 70.0).abs() < f32::EPSILON);
    }

    #[test]
    fn biome_config_from_ron_with_noise_params() {
        let ron_str = r#"(
            deep_space_threshold: 0.3,
            asteroid_field_threshold: 0.7,
            deep_space: (
                asteroid_count_min: 0, asteroid_count_max: 2,
                drone_count_min: 0, drone_count_max: 1,
                asteroid_health: 50.0, asteroid_radius: 20.0,
                drone_health: 30.0, drone_radius: 10.0,
                asteroid_velocity_min: 5.0, asteroid_velocity_max: 15.0,
                drone_velocity_min: 10.0, drone_velocity_max: 25.0,
            ),
            asteroid_field: (
                asteroid_count_min: 6, asteroid_count_max: 12,
                drone_count_min: 1, drone_count_max: 3,
                asteroid_health: 50.0, asteroid_radius: 20.0,
                drone_health: 30.0, drone_radius: 10.0,
                asteroid_velocity_min: 5.0, asteroid_velocity_max: 15.0,
                drone_velocity_min: 10.0, drone_velocity_max: 25.0,
            ),
            wreck_field: (
                asteroid_count_min: 2, asteroid_count_max: 5,
                drone_count_min: 2, drone_count_max: 4,
                asteroid_health: 80.0, asteroid_radius: 25.0,
                drone_health: 30.0, drone_radius: 10.0,
                asteroid_velocity_min: 2.0, asteroid_velocity_max: 8.0,
                drone_velocity_min: 5.0, drone_velocity_max: 15.0,
            ),
            noise: (
                noise_scale: 0.5,
                noise_octaves: 6,
                noise_persistence: 0.4,
                noise_lacunarity: 2.5,
            ),
        )"#;
        let config = BiomeConfig::from_ron(ron_str).expect("Should parse BiomeConfig with noise params");
        assert!((config.noise.noise_scale - 0.5).abs() < f64::EPSILON);
        assert_eq!(config.noise.noise_octaves, 6);
        assert!((config.noise.noise_persistence - 0.4).abs() < f64::EPSILON);
        assert!((config.noise.noise_lacunarity - 2.5).abs() < f64::EPSILON);
    }

    #[test]
    fn biome_config_from_ron_invalid() {
        let result = BiomeConfig::from_ron("not valid ron");
        assert!(result.is_err(), "Invalid RON should return error");
    }

    #[test]
    fn biome_config_params_for_returns_correct_params() {
        let config = BiomeConfig::default();
        let deep = config.params_for(BiomeType::DeepSpace);
        let asteroid = config.params_for(BiomeType::AsteroidField);
        let wreck = config.params_for(BiomeType::WreckField);

        assert_eq!(deep.asteroid_count_max, config.deep_space.asteroid_count_max);
        assert_eq!(
            asteroid.asteroid_count_min,
            config.asteroid_field.asteroid_count_min
        );
        assert_eq!(wreck.drone_count_min, config.wreck_field.drone_count_min);
    }

    #[test]
    fn biome_config_default_thresholds_are_valid() {
        let config = BiomeConfig::default();
        assert!(
            config.deep_space_threshold < config.asteroid_field_threshold,
            "deep_space_threshold should be < asteroid_field_threshold"
        );
        assert!(config.deep_space_threshold >= 0.0 && config.deep_space_threshold <= 1.0);
        assert!(config.asteroid_field_threshold >= 0.0 && config.asteroid_field_threshold <= 1.0);
    }

    // ── ChunkLoadState ──

    #[test]
    fn chunk_load_state_default_is_none() {
        let state = ChunkLoadState::default();
        assert!(state.last_player_chunk.is_none());
    }

    // ── ChunkEntityIndex ──

    #[test]
    fn chunk_entity_index_empty_count_is_zero() {
        let index = ChunkEntityIndex::default();
        assert_eq!(index.entity_count(), 0);
    }

    // ── WorldConfig max_chunks_per_frame ──

    #[test]
    fn world_config_ron_with_max_chunks_per_frame() {
        let ron_str = r#"(
            seed: 42,
            chunk_size: 1000.0,
            load_radius: 2,
            entity_budget: 200,
            max_chunks_per_frame: 8,
        )"#;
        let config = WorldConfig::from_ron(ron_str).expect("Should parse RON with max_chunks_per_frame");
        assert_eq!(config.max_chunks_per_frame, 8);
    }

    #[test]
    fn world_config_default_includes_max_chunks_per_frame() {
        let config = WorldConfig::default();
        assert_eq!(config.max_chunks_per_frame, 4);
    }

    // ── PendingChunks ──

    #[test]
    fn pending_chunks_default_is_empty() {
        let pending = PendingChunks::default();
        assert!(pending.chunks.is_empty());
    }

    // ── ChunkEntityIndex ──

    #[test]
    fn chunk_entity_index_multiple_chunks_summed() {
        let mut index = ChunkEntityIndex::default();
        index
            .chunks
            .insert(ChunkCoord { x: 0, y: 0 }, vec![Entity::from_bits(1), Entity::from_bits(2)]);
        index
            .chunks
            .insert(ChunkCoord { x: 1, y: 0 }, vec![Entity::from_bits(3)]);
        assert_eq!(index.entity_count(), 3);
    }

    // ── Story 4-11: Distance Difficulty Scaling ──

    #[test]
    fn enemy_stats_at_origin_equals_base() {
        let (health, damage) = enemy_stats_for_distance(0.0, 30.0, 5.0, 0.1);
        assert!(
            (health - 30.0).abs() < f32::EPSILON,
            "Health at origin should equal base health"
        );
        assert!(
            (damage - 5.0).abs() < f32::EPSILON,
            "Damage at origin should equal base damage"
        );
    }

    #[test]
    fn enemy_stats_at_100_units_scaled_by_one_tier() {
        // 100 units = 1 tier, scale 0.1 → multiplier 1.1
        let (health, damage) = enemy_stats_for_distance(100.0, 30.0, 5.0, 0.1);
        let expected_health = 30.0 * 1.1;
        let expected_damage = 5.0 * 1.1;
        assert!(
            (health - expected_health).abs() < 0.001,
            "Health should be scaled by 1.1 at 100 units distance, got {health}"
        );
        assert!(
            (damage - expected_damage).abs() < 0.001,
            "Damage should be scaled by 1.1 at 100 units distance, got {damage}"
        );
    }

    #[test]
    fn enemy_stats_at_1000_units_scaled_by_ten_tiers() {
        // 1000 units = 10 tiers, scale 0.1 → multiplier 2.0
        let (health, _damage) = enemy_stats_for_distance(1000.0, 30.0, 5.0, 0.1);
        let expected_health = 30.0 * 2.0;
        assert!(
            (health - expected_health).abs() < 0.001,
            "Health at 1000 units should be doubled with scale 0.1, got {health}"
        );
    }

    #[test]
    fn enemy_stats_negative_distance_treated_as_zero() {
        // Negative distance (origin side) should not reduce below base
        let (health, damage) = enemy_stats_for_distance(-500.0, 30.0, 5.0, 0.1);
        assert!(
            (health - 30.0).abs() < f32::EPSILON,
            "Health with negative distance should equal base (no penalty), got {health}"
        );
        assert!(
            (damage - 5.0).abs() < f32::EPSILON,
            "Damage with negative distance should equal base, got {damage}"
        );
    }

    #[test]
    fn enemy_stats_zero_scale_always_returns_base() {
        let (health, damage) = enemy_stats_for_distance(5000.0, 50.0, 10.0, 0.0);
        assert!(
            (health - 50.0).abs() < f32::EPSILON,
            "Zero scale should keep health at base"
        );
        assert!(
            (damage - 10.0).abs() < f32::EPSILON,
            "Zero scale should keep damage at base"
        );
    }

    #[test]
    fn world_config_default_has_difficulty_scaling_fields() {
        let config = WorldConfig::default();
        assert!(
            config.difficulty_health_scale_per_100u > 0.0,
            "Default difficulty_health_scale_per_100u should be positive"
        );
        assert!(
            config.difficulty_count_scale_per_100u > 0.0,
            "Default difficulty_count_scale_per_100u should be positive"
        );
    }

    #[test]
    fn world_config_ron_without_difficulty_fields_uses_defaults() {
        let ron_str = r#"(
            seed: 42,
            chunk_size: 1000.0,
            load_radius: 2,
            entity_budget: 200,
        )"#;
        let config = WorldConfig::from_ron(ron_str)
            .expect("Should parse RON without difficulty fields");
        assert!(
            (config.difficulty_health_scale_per_100u - 0.1).abs() < f32::EPSILON,
            "Should default to 0.1"
        );
        assert!(
            (config.difficulty_count_scale_per_100u - 0.05).abs() < f32::EPSILON,
            "Should default to 0.05"
        );
    }

    // ── BF-2: Tutorial Zone Chunk Protection ──────────────────────────────

    /// Tutorial-zone chunks (0,0) and neighbours must be registered by
    /// `init_tutorial_zone_chunks`.
    #[test]
    fn init_tutorial_zone_chunks_registers_origin_and_neighbours() {
        let mut tutorial_chunks = TutorialZoneChunks::default();
        // Simulate what the startup system does
        for dx in -1..=1i32 {
            for dy in -1..=1i32 {
                tutorial_chunks.coords.insert(ChunkCoord { x: dx, y: dy });
            }
        }
        assert_eq!(tutorial_chunks.coords.len(), 9, "3x3 neighbourhood = 9 chunks");
        assert!(
            tutorial_chunks.coords.contains(&ChunkCoord { x: 0, y: 0 }),
            "Origin chunk must be registered"
        );
        assert!(
            tutorial_chunks.coords.contains(&ChunkCoord { x: -1, y: -1 }),
            "Top-left neighbour must be registered"
        );
        assert!(
            tutorial_chunks.coords.contains(&ChunkCoord { x: 1, y: 1 }),
            "Bottom-right neighbour must be registered"
        );
    }

    /// When a tutorial chunk is active and the player moves far away (so it is
    /// outside `desired`), the filter must keep it out of `to_unload`.
    #[test]
    fn tutorial_zone_chunks_not_unloaded() {
        // Player is far away — chunk (0,0) is NOT in `desired`.
        let player_chunk = ChunkCoord { x: 100, y: 100 };
        let load_radius = 2u32;
        let desired = chunk::chunks_in_radius(player_chunk, load_radius);

        // Verify the tutorial chunk is indeed outside the desired set.
        let tutorial_coord = ChunkCoord { x: 0, y: 0 };
        assert!(
            !desired.contains(&tutorial_coord),
            "Tutorial chunk must not be desired when player is far away"
        );

        // Build the tutorial-zone resource.
        let mut tutorial_chunks = TutorialZoneChunks::default();
        for dx in -1..=1i32 {
            for dy in -1..=1i32 {
                tutorial_chunks.coords.insert(ChunkCoord { x: dx, y: dy });
            }
        }

        // Simulate the to_unload filter from update_chunks.
        let mut active_keys = std::collections::HashSet::new();
        active_keys.insert(tutorial_coord);
        active_keys.insert(ChunkCoord { x: 100, y: 100 }); // player's current chunk

        let to_unload: Vec<ChunkCoord> = active_keys
            .iter()
            .filter(|k| !desired.contains(k) && !tutorial_chunks.coords.contains(k))
            .copied()
            .collect();

        assert!(
            !to_unload.contains(&tutorial_coord),
            "Tutorial chunk (0,0) must NOT be in to_unload"
        );
    }

    /// Non-tutorial chunks that are outside the load radius must still be unloaded.
    #[test]
    fn non_tutorial_chunks_still_unloaded() {
        let player_chunk = ChunkCoord { x: 0, y: 0 };
        let load_radius = 2u32;
        let desired = chunk::chunks_in_radius(player_chunk, load_radius);

        let mut tutorial_chunks = TutorialZoneChunks::default();
        for dx in -1..=1i32 {
            for dy in -1..=1i32 {
                tutorial_chunks.coords.insert(ChunkCoord { x: dx, y: dy });
            }
        }

        // A chunk far from both player and tutorial zone.
        let far_chunk = ChunkCoord { x: 50, y: 50 };
        assert!(
            !desired.contains(&far_chunk),
            "Far chunk must not be desired"
        );
        assert!(
            !tutorial_chunks.coords.contains(&far_chunk),
            "Far chunk must not be a tutorial chunk"
        );

        let mut active_keys = std::collections::HashSet::new();
        active_keys.insert(far_chunk);
        active_keys.insert(ChunkCoord { x: 0, y: 0 });

        let to_unload: Vec<ChunkCoord> = active_keys
            .iter()
            .filter(|k| !desired.contains(k) && !tutorial_chunks.coords.contains(k))
            .copied()
            .collect();

        assert!(
            to_unload.contains(&far_chunk),
            "Far chunk (50,50) must be in to_unload"
        );
        assert!(
            !to_unload.contains(&ChunkCoord { x: 0, y: 0 }),
            "Tutorial origin chunk must NOT be in to_unload (protected)"
        );
    }
}
