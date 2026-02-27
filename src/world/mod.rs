pub mod chunk;
pub mod generation;

use bevy::prelude::*;
use serde::Deserialize;

pub use self::chunk::ChunkCoord;
use self::chunk::{chunks_in_radius, world_to_chunk};
use self::generation::generate_chunk_content;

use crate::core::collision::{Collider, Health};
use crate::core::flight::Player;
use crate::core::spawning::{Asteroid, NeedsAsteroidVisual, NeedsDroneVisual, ScoutDrone};
use crate::shared::components::Velocity;

// ── Config ──────────────────────────────────────────────────────────────

/// World generation configuration loaded from `assets/config/world.ron`.
#[derive(Resource, Deserialize, Clone, Debug)]
pub struct WorldConfig {
    pub seed: u64,
    pub chunk_size: f32,
    pub load_radius: u32,
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
    pub entity_budget: usize,
}

impl Default for WorldConfig {
    fn default() -> Self {
        Self {
            seed: 42,
            chunk_size: 1000.0,
            load_radius: 2,
            asteroid_count_min: 3,
            asteroid_count_max: 8,
            drone_count_min: 0,
            drone_count_max: 2,
            asteroid_health: 50.0,
            asteroid_radius: 20.0,
            drone_health: 30.0,
            drone_radius: 10.0,
            asteroid_velocity_min: 5.0,
            asteroid_velocity_max: 15.0,
            drone_velocity_min: 10.0,
            drone_velocity_max: 25.0,
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

// ── Components ──────────────────────────────────────────────────────────

/// Links an entity to the chunk that spawned it.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct ChunkEntity {
    pub coord: ChunkCoord,
}

// ── Resources ───────────────────────────────────────────────────────────

/// Tracks which chunks are currently loaded.
#[derive(Resource, Default, Debug)]
pub struct ActiveChunks {
    pub chunks: std::collections::HashSet<ChunkCoord>,
}

// ── Systems ─────────────────────────────────────────────────────────────

/// Computes the desired set of chunks from the player's position,
/// spawns entities for new chunks, and despawns entities for removed chunks.
pub fn update_chunks(
    mut commands: Commands,
    player_query: Query<&Transform, With<Player>>,
    config: Res<WorldConfig>,
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
        .difference(&desired)
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
        .difference(&active_chunks.chunks)
        .copied()
        .collect();
    to_load.sort();

    // Count ALL game entities (not just chunk entities) for accurate budget enforcement.
    // Subtract despawned entities (still in query due to deferred commands).
    let mut total_entity_count = all_collidable.iter().count() - despawned_count;

    for coord in to_load {
        let blueprints = generate_chunk_content(config.seed, coord, &config);
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
                    ));
                }
            }
        }

        total_entity_count += spawn_count;
        active_chunks.chunks.insert(coord);
    }
}

// ── Plugin ──────────────────────────────────────────────────────────────

pub struct WorldPlugin;

impl Plugin for WorldPlugin {
    fn build(&self, app: &mut App) {
        let config_path = "assets/config/world.ron";
        let config = match std::fs::read_to_string(config_path) {
            Ok(contents) => match WorldConfig::from_ron(&contents) {
                Ok(config) => config,
                Err(e) => {
                    warn!("Failed to parse {config_path}: {e}. Using defaults.");
                    WorldConfig::default()
                }
            },
            Err(e) => {
                warn!("Failed to read {config_path}: {e}. Using defaults.");
                WorldConfig::default()
            }
        };

        app.insert_resource(config);
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
        assert!(config.asteroid_count_max >= config.asteroid_count_min);
        assert!(config.drone_count_max >= config.drone_count_min);
        assert!(config.asteroid_health > 0.0);
        assert!(config.asteroid_radius > 0.0);
        assert!(config.drone_health > 0.0);
        assert!(config.drone_radius > 0.0);
        assert!(config.asteroid_velocity_max > config.asteroid_velocity_min);
        assert!(config.drone_velocity_max > config.drone_velocity_min);
        assert!(config.entity_budget > 0);
    }

    #[test]
    fn world_config_from_ron() {
        let ron_str = r#"(
            seed: 12345,
            chunk_size: 500.0,
            load_radius: 3,
            asteroid_count_min: 2,
            asteroid_count_max: 5,
            drone_count_min: 1,
            drone_count_max: 3,
            asteroid_health: 40.0,
            asteroid_radius: 15.0,
            drone_health: 20.0,
            drone_radius: 8.0,
            asteroid_velocity_min: 4.0,
            asteroid_velocity_max: 12.0,
            drone_velocity_min: 8.0,
            drone_velocity_max: 20.0,
            entity_budget: 150,
        )"#;
        let config = WorldConfig::from_ron(ron_str).expect("Should parse RON");
        assert_eq!(config.seed, 12345);
        assert!((config.chunk_size - 500.0).abs() < f32::EPSILON);
        assert_eq!(config.load_radius, 3);
        assert_eq!(config.asteroid_count_min, 2);
        assert_eq!(config.asteroid_count_max, 5);
        assert!((config.asteroid_health - 40.0).abs() < f32::EPSILON);
        assert_eq!(config.entity_budget, 150);
    }

    #[test]
    fn world_config_from_ron_invalid_falls_back() {
        let result = WorldConfig::from_ron("not valid ron");
        assert!(result.is_err(), "Invalid RON should return error");
    }
}
