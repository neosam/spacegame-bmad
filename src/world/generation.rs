use bevy::prelude::*;
use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};

use super::chunk::ChunkCoord;
use super::WorldConfig;

/// Generates a random speed within [min, max]. Falls back to `min` if range is invalid.
fn safe_speed(rng: &mut StdRng, min: f32, max: f32) -> f32 {
    if min < max {
        rng.random_range(min..=max)
    } else {
        min
    }
}

// ── Seed derivation primes ──────────────────────────────────────────────

const PRIME1: u64 = 6_364_136_223_846_793_005;
const PRIME2: u64 = 1_442_695_040_888_963_407;

/// Derives a per-chunk seed from the world seed and chunk coordinate.
fn chunk_seed(world_seed: u64, coord: ChunkCoord) -> u64 {
    world_seed
        ^ (coord.x as u64).wrapping_mul(PRIME1)
        ^ (coord.y as u64).wrapping_mul(PRIME2)
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
}

// ── Generation ──────────────────────────────────────────────────────────

/// Generates entities for a chunk. Pure function: same inputs always produce same outputs.
pub fn generate_chunk_content(
    seed: u64,
    coord: ChunkCoord,
    config: &WorldConfig,
) -> Vec<EntityBlueprint> {
    let mut rng = StdRng::seed_from_u64(chunk_seed(seed, coord));
    let mut blueprints = Vec::new();

    let chunk_origin = Vec2::new(
        coord.x as f32 * config.chunk_size,
        coord.y as f32 * config.chunk_size,
    );

    // Generate asteroids
    let asteroid_count = rng.random_range(config.asteroid_count_min..=config.asteroid_count_max);
    for _ in 0..asteroid_count {
        let x = chunk_origin.x + rng.random_range(0.0..config.chunk_size);
        let y = chunk_origin.y + rng.random_range(0.0..config.chunk_size);
        let angle = rng.random_range(0.0..std::f32::consts::TAU);
        let speed = safe_speed(&mut rng, config.asteroid_velocity_min, config.asteroid_velocity_max);
        blueprints.push(EntityBlueprint {
            entity_type: BlueprintType::Asteroid,
            position: Vec2::new(x, y),
            velocity: Vec2::new(angle.cos() * speed, angle.sin() * speed),
            health: config.asteroid_health,
            radius: config.asteroid_radius,
        });
    }

    // Generate drones
    let drone_count = rng.random_range(config.drone_count_min..=config.drone_count_max);
    for _ in 0..drone_count {
        let x = chunk_origin.x + rng.random_range(0.0..config.chunk_size);
        let y = chunk_origin.y + rng.random_range(0.0..config.chunk_size);
        let angle = rng.random_range(0.0..std::f32::consts::TAU);
        let speed = safe_speed(&mut rng, config.drone_velocity_min, config.drone_velocity_max);
        blueprints.push(EntityBlueprint {
            entity_type: BlueprintType::ScoutDrone,
            position: Vec2::new(x, y),
            velocity: Vec2::new(angle.cos() * speed, angle.sin() * speed),
            health: config.drone_health,
            radius: config.drone_radius,
        });
    }

    blueprints
}

// ── Unit tests ──────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn default_config() -> WorldConfig {
        WorldConfig::default()
    }

    #[test]
    fn generate_chunk_is_deterministic() {
        let config = default_config();
        let coord = ChunkCoord { x: 5, y: -3 };

        let result1 = generate_chunk_content(config.seed, coord, &config);
        let result2 = generate_chunk_content(config.seed, coord, &config);

        assert_eq!(result1.len(), result2.len(), "Same seed+coord should produce same count");
        for (a, b) in result1.iter().zip(result2.iter()) {
            assert_eq!(a.entity_type, b.entity_type);
            assert!((a.position.x - b.position.x).abs() < f32::EPSILON);
            assert!((a.position.y - b.position.y).abs() < f32::EPSILON);
            assert!((a.velocity.x - b.velocity.x).abs() < f32::EPSILON);
            assert!((a.velocity.y - b.velocity.y).abs() < f32::EPSILON);
            assert!((a.health - b.health).abs() < f32::EPSILON);
            assert!((a.radius - b.radius).abs() < f32::EPSILON);
        }
    }

    #[test]
    fn different_coords_produce_different_content() {
        let config = default_config();
        let a = generate_chunk_content(config.seed, ChunkCoord { x: 0, y: 0 }, &config);
        let b = generate_chunk_content(config.seed, ChunkCoord { x: 1, y: 0 }, &config);
        let c = generate_chunk_content(config.seed, ChunkCoord { x: 0, y: 1 }, &config);

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
        let blueprints = generate_chunk_content(config.seed, coord, &config);

        let chunk_min_x = coord.x as f32 * config.chunk_size;
        let chunk_min_y = coord.y as f32 * config.chunk_size;
        let chunk_max_x = chunk_min_x + config.chunk_size;
        let chunk_max_y = chunk_min_y + config.chunk_size;

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
        let blueprints = generate_chunk_content(config.seed, ChunkCoord { x: 0, y: 0 }, &config);

        for bp in &blueprints {
            assert!(bp.health > 0.0);
            assert!(bp.radius > 0.0);
            let speed = bp.velocity.length();
            match bp.entity_type {
                BlueprintType::Asteroid => {
                    assert!((bp.health - config.asteroid_health).abs() < f32::EPSILON);
                    assert!((bp.radius - config.asteroid_radius).abs() < f32::EPSILON);
                    assert!(
                        speed >= config.asteroid_velocity_min - 0.01
                            && speed <= config.asteroid_velocity_max + 0.01,
                        "Asteroid speed {speed} should be in [{}, {}]",
                        config.asteroid_velocity_min,
                        config.asteroid_velocity_max
                    );
                }
                BlueprintType::ScoutDrone => {
                    assert!((bp.health - config.drone_health).abs() < f32::EPSILON);
                    assert!((bp.radius - config.drone_radius).abs() < f32::EPSILON);
                    assert!(
                        speed >= config.drone_velocity_min - 0.01
                            && speed <= config.drone_velocity_max + 0.01,
                        "Drone speed {speed} should be in [{}, {}]",
                        config.drone_velocity_min,
                        config.drone_velocity_max
                    );
                }
            }
        }
    }

    #[test]
    fn entity_count_within_config_bounds() {
        let config = default_config();
        // Test many chunks to check count bounds
        for x in -5..5 {
            for y in -5..5 {
                let blueprints =
                    generate_chunk_content(config.seed, ChunkCoord { x, y }, &config);
                let asteroids = blueprints
                    .iter()
                    .filter(|b| b.entity_type == BlueprintType::Asteroid)
                    .count() as u32;
                let drones = blueprints
                    .iter()
                    .filter(|b| b.entity_type == BlueprintType::ScoutDrone)
                    .count() as u32;

                assert!(
                    asteroids >= config.asteroid_count_min
                        && asteroids <= config.asteroid_count_max,
                    "Chunk ({x},{y}): asteroid count {asteroids} out of range [{}, {}]",
                    config.asteroid_count_min,
                    config.asteroid_count_max
                );
                assert!(
                    drones >= config.drone_count_min && drones <= config.drone_count_max,
                    "Chunk ({x},{y}): drone count {drones} out of range [{}, {}]",
                    config.drone_count_min,
                    config.drone_count_max
                );
            }
        }
    }

    #[test]
    fn equal_velocity_ranges_do_not_panic() {
        let mut config = default_config();
        config.asteroid_velocity_min = 10.0;
        config.asteroid_velocity_max = 10.0;
        config.drone_velocity_min = 15.0;
        config.drone_velocity_max = 15.0;

        // Should not panic — falls back to min value
        let blueprints = generate_chunk_content(config.seed, ChunkCoord { x: 0, y: 0 }, &config);
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
}
