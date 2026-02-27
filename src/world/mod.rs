pub mod chunk;
pub mod generation;

use bevy::prelude::*;
use serde::Deserialize;

pub use self::chunk::ChunkCoord;
pub use self::generation::BiomeType;
use self::chunk::{chunks_in_radius, world_to_chunk};
use self::generation::{determine_biome, generate_chunk_content};

use crate::core::collision::{Collider, Health};
use crate::core::flight::Player;
use crate::core::spawning::{Asteroid, NeedsAsteroidVisual, NeedsDroneVisual, ScoutDrone};
use crate::shared::components::Velocity;

// ── World Config ─────────────────────────────────────────────────────────

/// World generation configuration loaded from `assets/config/world.ron`.
#[derive(Resource, Deserialize, Clone, Debug)]
pub struct WorldConfig {
    pub seed: u64,
    pub chunk_size: f32,
    pub load_radius: u32,
    pub entity_budget: usize,
}

impl Default for WorldConfig {
    fn default() -> Self {
        Self {
            seed: 42,
            chunk_size: 1000.0,
            load_radius: 2,
            entity_budget: 200,
        }
    }
}

impl WorldConfig {
    /// Load config from RON string.
    pub fn from_ron(ron_str: &str) -> Result<Self, ron::error::SpannedError> {
        ron::from_str(ron_str)
    }
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

// ── Systems ─────────────────────────────────────────────────────────────

/// Computes the desired set of chunks from the player's position,
/// spawns entities for new chunks, and despawns entities for removed chunks.
pub fn update_chunks(
    mut commands: Commands,
    player_query: Query<&Transform, With<Player>>,
    config: Res<WorldConfig>,
    biome_config: Res<BiomeConfig>,
    mut active_chunks: ResMut<ActiveChunks>,
    chunk_entities: Query<(Entity, &ChunkEntity)>,
    all_collidable: Query<Entity, With<Collider>>,
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

    // Unload chunks no longer in range (sorted for deterministic order)
    let mut to_unload: Vec<ChunkCoord> = active_chunks
        .chunks
        .keys()
        .filter(|k| !desired.contains(k))
        .copied()
        .collect();
    to_unload.sort();
    let mut despawned_count = 0usize;
    for coord in &to_unload {
        for (entity, chunk_ent) in chunk_entities.iter() {
            if chunk_ent.coord == *coord {
                commands.entity(entity).despawn();
                despawned_count += 1;
            }
        }
        active_chunks.chunks.remove(coord);
    }

    // Load new chunks (sorted for deterministic budget distribution)
    let mut to_load: Vec<ChunkCoord> = desired
        .iter()
        .filter(|k| !active_chunks.chunks.contains_key(k))
        .copied()
        .collect();
    to_load.sort();

    // Count ALL game entities (not just chunk entities) for accurate budget enforcement.
    // Subtract despawned entities (still in query due to deferred commands).
    let mut total_entity_count = all_collidable.iter().count() - despawned_count;

    for coord in to_load {
        let biome = determine_biome(config.seed, coord, &biome_config);
        let blueprints =
            generate_chunk_content(config.seed, coord, config.chunk_size, biome, &biome_config);
        let remaining_budget = config.entity_budget.saturating_sub(total_entity_count);
        let spawn_count = blueprints.len().min(remaining_budget);

        for blueprint in blueprints.into_iter().take(spawn_count) {
            let chunk_marker = ChunkEntity { coord };
            match blueprint.entity_type {
                generation::BlueprintType::Asteroid => {
                    commands.spawn((
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
                    ));
                }
                generation::BlueprintType::ScoutDrone => {
                    commands.spawn((
                        ScoutDrone,
                        NeedsDroneVisual,
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
                    ));
                }
            }
        }

        total_entity_count += spawn_count;
        active_chunks.chunks.insert(coord, biome);
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
        app.insert_resource(world_config);
        app.insert_resource(biome_config);
        app.init_resource::<ActiveChunks>();
        app.add_systems(
            FixedUpdate,
            update_chunks.before(crate::core::CoreSet::Collision),
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
}
