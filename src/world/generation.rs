use bevy::prelude::*;
use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};

use super::chunk::ChunkCoord;
use super::noise_layers::{biome_noise_value, noise_to_unit};
use super::BiomeConfig;

/// Generates a random speed within [min, max]. Falls back to `min` if range is invalid.
fn safe_speed(rng: &mut StdRng, min: f32, max: f32) -> f32 {
    if min < max {
        rng.random_range(min..=max)
    } else {
        min
    }
}

// ── Seed derivation ─────────────────────────────────────────────────────

const PRIME1: u64 = 6_364_136_223_846_793_005;
const PRIME2: u64 = 1_442_695_040_888_963_407;

/// Derives a per-chunk seed from the world seed and chunk coordinate.
fn chunk_seed(world_seed: u64, coord: ChunkCoord) -> u64 {
    world_seed
        ^ (coord.x as u64).wrapping_mul(PRIME1)
        ^ (coord.y as u64).wrapping_mul(PRIME2)
}

// ── Biome ────────────────────────────────────────────────────────────────

/// The three biome types in the world.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Component)]
pub enum BiomeType {
    DeepSpace,
    AsteroidField,
    WreckField,
}

/// Determines the biome type for a chunk using continuous noise.
/// Pure function: same seed + coord always produces the same biome.
pub fn determine_biome(seed: u64, coord: ChunkCoord, config: &BiomeConfig) -> BiomeType {
    let raw = biome_noise_value(seed, coord, &config.noise);
    let value = noise_to_unit(raw);
    if value < config.deep_space_threshold {
        BiomeType::DeepSpace
    } else if value < config.asteroid_field_threshold {
        BiomeType::AsteroidField
    } else {
        BiomeType::WreckField
    }
}

// ── Blueprint ───────────────────────────────────────────────────────────

/// Type of entity to spawn from a blueprint.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BlueprintType {
    Asteroid,
    ScoutDrone,
}

/// Describes a single entity to spawn within a chunk.
#[derive(Debug, Clone)]
pub struct EntityBlueprint {
    pub entity_type: BlueprintType,
    pub position: Vec2,
    pub velocity: Vec2,
    pub health: f32,
    pub radius: f32,
    pub biome: BiomeType,
}

// ── Generation ──────────────────────────────────────────────────────────

/// Generates entities for a chunk with biome-specific parameters.
/// Pure function: same inputs always produce same outputs.
pub fn generate_chunk_content(
    seed: u64,
    coord: ChunkCoord,
    chunk_size: f32,
    biome: BiomeType,
    config: &BiomeConfig,
) -> Vec<EntityBlueprint> {
    let mut rng = StdRng::seed_from_u64(chunk_seed(seed, coord));
    let mut blueprints = Vec::new();
    let params = config.params_for(biome);

    let chunk_origin = Vec2::new(
        coord.x as f32 * chunk_size,
        coord.y as f32 * chunk_size,
    );

    // NOTE: Asteroids are generated before drones. Under entity budget pressure,
    // drones may be disproportionately cut (budget truncates from the end).
    // Consider shuffling blueprints if balanced entity-type distribution matters.
    let asteroid_count = rng.random_range(params.asteroid_count_min..=params.asteroid_count_max);
    for _ in 0..asteroid_count {
        let x = chunk_origin.x + rng.random_range(0.0..chunk_size);
        let y = chunk_origin.y + rng.random_range(0.0..chunk_size);
        let angle = rng.random_range(0.0..std::f32::consts::TAU);
        let speed = safe_speed(&mut rng, params.asteroid_velocity_min, params.asteroid_velocity_max);
        blueprints.push(EntityBlueprint {
            entity_type: BlueprintType::Asteroid,
            position: Vec2::new(x, y),
            velocity: Vec2::new(angle.cos() * speed, angle.sin() * speed),
            health: params.asteroid_health,
            radius: params.asteroid_radius,
            biome,
        });
    }

    // Generate drones
    let drone_count = rng.random_range(params.drone_count_min..=params.drone_count_max);
    for _ in 0..drone_count {
        let x = chunk_origin.x + rng.random_range(0.0..chunk_size);
        let y = chunk_origin.y + rng.random_range(0.0..chunk_size);
        let angle = rng.random_range(0.0..std::f32::consts::TAU);
        let speed = safe_speed(&mut rng, params.drone_velocity_min, params.drone_velocity_max);
        blueprints.push(EntityBlueprint {
            entity_type: BlueprintType::ScoutDrone,
            position: Vec2::new(x, y),
            velocity: Vec2::new(angle.cos() * speed, angle.sin() * speed),
            health: params.drone_health,
            radius: params.drone_radius,
            biome,
        });
    }

    blueprints
}

// ── Unit tests ──────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    const TEST_SEED: u64 = 42;
    const TEST_CHUNK_SIZE: f32 = 1000.0;

    fn default_config() -> BiomeConfig {
        BiomeConfig::default()
    }

    #[test]
    fn generate_chunk_is_deterministic() {
        let config = default_config();
        let coord = ChunkCoord { x: 5, y: -3 };
        let biome = determine_biome(TEST_SEED, coord, &config);

        let result1 = generate_chunk_content(TEST_SEED, coord, TEST_CHUNK_SIZE, biome, &config);
        let result2 = generate_chunk_content(TEST_SEED, coord, TEST_CHUNK_SIZE, biome, &config);

        assert_eq!(result1.len(), result2.len(), "Same seed+coord should produce same count");
        for (a, b) in result1.iter().zip(result2.iter()) {
            assert_eq!(a.entity_type, b.entity_type);
            assert!((a.position.x - b.position.x).abs() < f32::EPSILON);
            assert!((a.position.y - b.position.y).abs() < f32::EPSILON);
            assert!((a.velocity.x - b.velocity.x).abs() < f32::EPSILON);
            assert!((a.velocity.y - b.velocity.y).abs() < f32::EPSILON);
            assert!((a.health - b.health).abs() < f32::EPSILON);
            assert!((a.radius - b.radius).abs() < f32::EPSILON);
            assert_eq!(a.biome, b.biome);
        }
    }

    #[test]
    fn different_coords_produce_different_content() {
        let config = default_config();
        let biome = BiomeType::AsteroidField; // Use same biome for fair comparison
        let a = generate_chunk_content(TEST_SEED, ChunkCoord { x: 0, y: 0 }, TEST_CHUNK_SIZE, biome, &config);
        let b = generate_chunk_content(TEST_SEED, ChunkCoord { x: 1, y: 0 }, TEST_CHUNK_SIZE, biome, &config);
        let c = generate_chunk_content(TEST_SEED, ChunkCoord { x: 0, y: 1 }, TEST_CHUNK_SIZE, biome, &config);

        // At least one pair should differ (extremely unlikely all three are identical)
        let all_same = a.len() == b.len()
            && a.len() == c.len()
            && a.iter().zip(b.iter()).all(|(x, y)| {
                (x.position.x - y.position.x).abs() < f32::EPSILON
                    && (x.position.y - y.position.y).abs() < f32::EPSILON
            })
            && a.iter().zip(c.iter()).all(|(x, y)| {
                (x.position.x - y.position.x).abs() < f32::EPSILON
                    && (x.position.y - y.position.y).abs() < f32::EPSILON
            });
        assert!(!all_same, "Different coords should produce different content");
    }

    #[test]
    fn generated_entities_within_chunk_bounds() {
        let config = default_config();
        let coord = ChunkCoord { x: 3, y: -2 };
        let biome = determine_biome(TEST_SEED, coord, &config);
        let blueprints = generate_chunk_content(TEST_SEED, coord, TEST_CHUNK_SIZE, biome, &config);

        let chunk_min_x = coord.x as f32 * TEST_CHUNK_SIZE;
        let chunk_min_y = coord.y as f32 * TEST_CHUNK_SIZE;
        let chunk_max_x = chunk_min_x + TEST_CHUNK_SIZE;
        let chunk_max_y = chunk_min_y + TEST_CHUNK_SIZE;

        for bp in &blueprints {
            assert!(
                bp.position.x >= chunk_min_x && bp.position.x < chunk_max_x,
                "Entity x={} should be within chunk [{}, {})",
                bp.position.x, chunk_min_x, chunk_max_x
            );
            assert!(
                bp.position.y >= chunk_min_y && bp.position.y < chunk_max_y,
                "Entity y={} should be within chunk [{}, {})",
                bp.position.y, chunk_min_y, chunk_max_y
            );
        }
    }

    #[test]
    fn generated_entities_have_valid_properties() {
        let config = default_config();
        let coord = ChunkCoord { x: 0, y: 0 };
        let biome = determine_biome(TEST_SEED, coord, &config);
        let params = config.params_for(biome);
        let blueprints = generate_chunk_content(TEST_SEED, coord, TEST_CHUNK_SIZE, biome, &config);

        for bp in &blueprints {
            assert!(bp.health > 0.0);
            assert!(bp.radius > 0.0);
            assert_eq!(bp.biome, biome);
            let speed = bp.velocity.length();
            match bp.entity_type {
                BlueprintType::Asteroid => {
                    assert!((bp.health - params.asteroid_health).abs() < f32::EPSILON);
                    assert!((bp.radius - params.asteroid_radius).abs() < f32::EPSILON);
                    assert!(
                        speed >= params.asteroid_velocity_min - 0.01
                            && speed <= params.asteroid_velocity_max + 0.01,
                        "Asteroid speed {speed} should be in [{}, {}]",
                        params.asteroid_velocity_min,
                        params.asteroid_velocity_max
                    );
                }
                BlueprintType::ScoutDrone => {
                    assert!((bp.health - params.drone_health).abs() < f32::EPSILON);
                    assert!((bp.radius - params.drone_radius).abs() < f32::EPSILON);
                    assert!(
                        speed >= params.drone_velocity_min - 0.01
                            && speed <= params.drone_velocity_max + 0.01,
                        "Drone speed {speed} should be in [{}, {}]",
                        params.drone_velocity_min,
                        params.drone_velocity_max
                    );
                }
            }
        }
    }

    #[test]
    fn entity_count_within_config_bounds() {
        let config = default_config();
        for x in -5..5 {
            for y in -5..5 {
                let coord = ChunkCoord { x, y };
                let biome = determine_biome(TEST_SEED, coord, &config);
                let params = config.params_for(biome);
                let blueprints =
                    generate_chunk_content(TEST_SEED, coord, TEST_CHUNK_SIZE, biome, &config);
                let asteroids = blueprints
                    .iter()
                    .filter(|b| b.entity_type == BlueprintType::Asteroid)
                    .count() as u32;
                let drones = blueprints
                    .iter()
                    .filter(|b| b.entity_type == BlueprintType::ScoutDrone)
                    .count() as u32;

                assert!(
                    asteroids >= params.asteroid_count_min
                        && asteroids <= params.asteroid_count_max,
                    "Chunk ({x},{y}) biome {biome:?}: asteroid count {asteroids} out of range [{}, {}]",
                    params.asteroid_count_min,
                    params.asteroid_count_max
                );
                assert!(
                    drones >= params.drone_count_min && drones <= params.drone_count_max,
                    "Chunk ({x},{y}) biome {biome:?}: drone count {drones} out of range [{}, {}]",
                    params.drone_count_min,
                    params.drone_count_max
                );
            }
        }
    }

    #[test]
    fn equal_velocity_ranges_do_not_panic() {
        let mut config = default_config();
        // Set all biomes to equal velocity ranges
        for params in [&mut config.deep_space, &mut config.asteroid_field, &mut config.wreck_field] {
            params.asteroid_velocity_min = 10.0;
            params.asteroid_velocity_max = 10.0;
            params.drone_velocity_min = 15.0;
            params.drone_velocity_max = 15.0;
        }

        let biome = BiomeType::AsteroidField;
        let blueprints = generate_chunk_content(TEST_SEED, ChunkCoord { x: 0, y: 0 }, TEST_CHUNK_SIZE, biome, &config);
        for bp in &blueprints {
            let speed = bp.velocity.length();
            match bp.entity_type {
                BlueprintType::Asteroid => {
                    assert!(
                        (speed - 10.0).abs() < 0.01,
                        "Asteroid speed should be 10.0 when min==max, got {speed}"
                    );
                }
                BlueprintType::ScoutDrone => {
                    assert!(
                        (speed - 15.0).abs() < 0.01,
                        "Drone speed should be 15.0 when min==max, got {speed}"
                    );
                }
            }
        }
    }

    // ── Biome determination tests (Task 5) ──

    #[test]
    fn determine_biome_is_deterministic() {
        let config = default_config();
        let coord = ChunkCoord { x: 7, y: -11 };
        let biome1 = determine_biome(TEST_SEED, coord, &config);
        let biome2 = determine_biome(TEST_SEED, coord, &config);
        assert_eq!(biome1, biome2, "Same seed+coord should produce same biome");
    }

    #[test]
    fn determine_biome_deterministic_different_seed() {
        let config = default_config();
        let coord = ChunkCoord { x: 3, y: 5 };
        let biome_a1 = determine_biome(100, coord, &config);
        let biome_a2 = determine_biome(100, coord, &config);
        assert_eq!(biome_a1, biome_a2, "Same seed should produce same biome");
    }

    #[test]
    fn determine_biome_variety_across_coords() {
        let config = default_config();
        let mut found = std::collections::HashSet::new();
        let mut counts = std::collections::HashMap::<BiomeType, u32>::new();
        let total_chunks = 400u32; // 20x20
        for x in -10..10 {
            for y in -10..10 {
                let biome = determine_biome(TEST_SEED, ChunkCoord { x, y }, &config);
                found.insert(biome);
                *counts.entry(biome).or_insert(0) += 1;
            }
        }
        assert!(
            found.len() >= 3,
            "Should have all 3 biome types across {total_chunks} chunks, got {found:?}"
        );
        // AC6: No single biome dominates >60%
        for (biome, count) in &counts {
            let pct = (*count as f32 / total_chunks as f32) * 100.0;
            assert!(
                pct <= 60.0,
                "Biome {biome:?} dominates {pct:.1}% (count {count}/{total_chunks}), exceeds 60% limit"
            );
        }
    }

    #[test]
    fn asteroid_field_produces_more_asteroids_than_deep_space() {
        let config = default_config();
        let mut deep_space_total = 0u32;
        let mut asteroid_field_total = 0u32;
        let sample_size = 50;

        for i in 0..sample_size {
            let coord = ChunkCoord { x: i, y: 0 };
            let ds_blueprints = generate_chunk_content(TEST_SEED, coord, TEST_CHUNK_SIZE, BiomeType::DeepSpace, &config);
            let af_blueprints = generate_chunk_content(TEST_SEED, coord, TEST_CHUNK_SIZE, BiomeType::AsteroidField, &config);
            deep_space_total += ds_blueprints.iter().filter(|b| b.entity_type == BlueprintType::Asteroid).count() as u32;
            asteroid_field_total += af_blueprints.iter().filter(|b| b.entity_type == BlueprintType::Asteroid).count() as u32;
        }

        assert!(
            asteroid_field_total > deep_space_total,
            "AsteroidField should produce more asteroids ({asteroid_field_total}) than DeepSpace ({deep_space_total})"
        );
    }

    #[test]
    fn wreck_field_produces_more_drones_than_deep_space() {
        let config = default_config();
        let mut deep_space_total = 0u32;
        let mut wreck_field_total = 0u32;
        let sample_size = 50;

        for i in 0..sample_size {
            let coord = ChunkCoord { x: i, y: 0 };
            let ds_blueprints = generate_chunk_content(TEST_SEED, coord, TEST_CHUNK_SIZE, BiomeType::DeepSpace, &config);
            let wf_blueprints = generate_chunk_content(TEST_SEED, coord, TEST_CHUNK_SIZE, BiomeType::WreckField, &config);
            deep_space_total += ds_blueprints.iter().filter(|b| b.entity_type == BlueprintType::ScoutDrone).count() as u32;
            wreck_field_total += wf_blueprints.iter().filter(|b| b.entity_type == BlueprintType::ScoutDrone).count() as u32;
        }

        assert!(
            wreck_field_total > deep_space_total,
            "WreckField should produce more drones ({wreck_field_total}) than DeepSpace ({deep_space_total})"
        );
    }

    #[test]
    fn biome_threshold_boundary_all_deep_space() {
        // deep_space_threshold = 1.0 → every value < 1.0 → all DeepSpace
        let mut config = default_config();
        config.deep_space_threshold = 1.0;
        config.asteroid_field_threshold = 1.0;

        for x in -5..5 {
            for y in -5..5 {
                let biome = determine_biome(TEST_SEED, ChunkCoord { x, y }, &config);
                assert_eq!(
                    biome,
                    BiomeType::DeepSpace,
                    "With threshold 1.0, all chunks should be DeepSpace"
                );
            }
        }
    }

    #[test]
    fn biome_threshold_boundary_no_deep_space() {
        // deep_space_threshold = 0.0 → value < 0.0 is always false → DeepSpace never selected
        let mut config = default_config();
        config.deep_space_threshold = 0.0;
        config.asteroid_field_threshold = 0.7;

        for x in -5..5 {
            for y in -5..5 {
                let biome = determine_biome(TEST_SEED, ChunkCoord { x, y }, &config);
                assert_ne!(
                    biome,
                    BiomeType::DeepSpace,
                    "With deep_space_threshold 0.0, DeepSpace should never be selected"
                );
            }
        }
    }

    #[test]
    fn biome_threshold_boundary_all_wreck_field() {
        // Both thresholds at 0.0 → neither DeepSpace nor AsteroidField → all WreckField
        let mut config = default_config();
        config.deep_space_threshold = 0.0;
        config.asteroid_field_threshold = 0.0;

        for x in -5..5 {
            for y in -5..5 {
                let biome = determine_biome(TEST_SEED, ChunkCoord { x, y }, &config);
                assert_eq!(
                    biome,
                    BiomeType::WreckField,
                    "With both thresholds 0.0, all chunks should be WreckField"
                );
            }
        }
    }
}
