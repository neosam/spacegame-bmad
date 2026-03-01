/// Faction identity and behavior parameter components for Epic 4: Combat Depth.
use bevy::prelude::*;
use noise::{Fbm, MultiFractal, NoiseFn, Perlin};

/// Which faction an entity belongs to.
/// Faction determines FSM parameters (aggression, flee threshold, reinforcement calls).
#[derive(Component, Debug, Clone, PartialEq, Eq, Hash)]
pub enum FactionId {
    Pirates,
    Military,
    Aliens,
    RogueDrones,
    Neutral,
}

/// Radius within which an enemy patrols when no player is detected.
#[derive(Component, Debug, Clone)]
pub struct PatrolRadius(pub f32);

/// Distance at which an enemy detects the player and transitions to Chase.
#[derive(Component, Debug, Clone)]
pub struct AggroRange(pub f32);

/// Health ratio (0.0–1.0) below which an enemy transitions to Flee.
#[derive(Component, Debug, Clone)]
pub struct FleeThreshold(pub f32);

/// Distance at which an enemy transitions from Chase to Attack.
#[derive(Component, Debug, Clone)]
pub struct AttackRange(pub f32);

// ── Faction Noise ─────────────────────────────────────────────────────────

/// Noise seed offset for faction territory generation.
/// Independent of biome noise offset.
const FACTION_NOISE_OFFSET: u32 = 0xFAC7_1337;

/// Pure function: determines which faction controls a world position.
///
/// Uses fractal Perlin noise to create smooth faction territories.
/// Same `x`, `y`, and `seed` always produce the same `FactionId`.
/// `seed` should be the world seed from `WorldConfig`.
pub fn faction_at_position(x: f32, y: f32, seed: u32) -> FactionId {
    let noise_seed = seed.wrapping_add(FACTION_NOISE_OFFSET);
    let fbm = Fbm::<Perlin>::new(noise_seed)
        .set_octaves(3)
        .set_persistence(0.5)
        .set_lacunarity(2.0);

    // Scale world coordinates down so territories are large (~2000 unit bands)
    const SCALE: f64 = 0.0003;
    let raw = fbm.get([x as f64 * SCALE, y as f64 * SCALE]);

    // Remap [-1, 1] → [0, 1]
    let value = ((raw + 1.0) / 2.0).clamp(0.0, 1.0) as f32;

    // Map value to faction using thresholds
    // 0.0–0.25: Pirates, 0.25–0.50: Military, 0.50–0.75: Aliens, 0.75–1.0: RogueDrones
    if value < 0.25 {
        FactionId::Pirates
    } else if value < 0.50 {
        FactionId::Military
    } else if value < 0.75 {
        FactionId::Aliens
    } else {
        FactionId::RogueDrones
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn faction_at_position_is_deterministic() {
        let f1 = faction_at_position(100.0, 200.0, 42);
        let f2 = faction_at_position(100.0, 200.0, 42);
        assert_eq!(f1, f2, "Same position+seed must always produce the same faction");
    }

    #[test]
    fn faction_at_position_different_seeds_may_differ() {
        // With different seeds, at least some positions should produce different results
        let mut found_difference = false;
        for i in 0..50 {
            let x = i as f32 * 1000.0;
            let f1 = faction_at_position(x, 0.0, 42);
            let f2 = faction_at_position(x, 0.0, 99);
            if f1 != f2 {
                found_difference = true;
                break;
            }
        }
        assert!(found_difference, "Different seeds should produce different faction maps");
    }

    #[test]
    fn faction_at_position_returns_all_four_factions_spatially() {
        // Sample a large area and check we get all 4 factions
        let mut found_pirates = false;
        let mut found_military = false;
        let mut found_aliens = false;
        let mut found_rogue_drones = false;
        for i in -20..20i32 {
            for j in -20..20i32 {
                let x = i as f32 * 2000.0;
                let y = j as f32 * 2000.0;
                match faction_at_position(x, y, 42) {
                    FactionId::Pirates => found_pirates = true,
                    FactionId::Military => found_military = true,
                    FactionId::Aliens => found_aliens = true,
                    FactionId::RogueDrones => found_rogue_drones = true,
                    FactionId::Neutral => {}
                }
            }
        }
        let count = [found_pirates, found_military, found_aliens, found_rogue_drones]
            .iter()
            .filter(|&&x| x)
            .count();
        assert!(
            count >= 3,
            "Expected at least 3 factions across large area, got {count}"
        );
    }

    #[test]
    fn faction_at_position_origin_consistent() {
        // Origin position always returns the same value for the same seed
        let f1 = faction_at_position(0.0, 0.0, 42);
        let f2 = faction_at_position(0.0, 0.0, 42);
        assert_eq!(f1, f2);
    }

    #[test]
    fn faction_id_never_neutral_from_noise() {
        // faction_at_position should never return Neutral (that's for trader ships)
        for i in 0..100 {
            let x = i as f32 * 500.0 - 25000.0;
            let y = i as f32 * 300.0 - 15000.0;
            let faction = faction_at_position(x, y, 42);
            assert_ne!(
                faction,
                FactionId::Neutral,
                "faction_at_position should never return Neutral at ({x}, {y})"
            );
        }
    }
}
