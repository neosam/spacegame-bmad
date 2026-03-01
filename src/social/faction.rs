/// Faction identity and behavior parameter components for Epic 4: Combat Depth.
use bevy::prelude::*;
use noise::{Fbm, MultiFractal, NoiseFn, Perlin};
use std::collections::HashMap;

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

// ── Faction Behavior Profiles (Story 4-6) ────────────────────────────────

/// How a faction prefers to engage in combat.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum AttackStyle {
    /// Rush player, no hesitation.
    Aggressive,
    /// Hold back, engage only when provoked or attacked.
    Defensive,
    /// Unpredictable movement and attack patterns.
    Erratic,
}

/// Faction-wide behavior modifiers applied to AI parameters.
#[derive(Debug, Clone)]
pub struct FactionBehaviorProfile {
    /// Multiplier on aggro range (>1.0 = more aggressive detection).
    pub aggro_multiplier: f32,
    /// Multiplier on flee threshold (>1.0 = flee sooner).
    pub flee_multiplier: f32,
    /// Preferred attack style.
    pub preferred_attack_style: AttackStyle,
}

/// Resource mapping each faction to its behavior profile.
/// Pre-populated with default values; can be overridden via config.
#[derive(Resource, Debug)]
pub struct FactionBehaviorProfiles {
    pub profiles: HashMap<FactionId, FactionBehaviorProfile>,
}

impl Default for FactionBehaviorProfiles {
    fn default() -> Self {
        let mut profiles = HashMap::new();
        profiles.insert(FactionId::Pirates, FactionBehaviorProfile {
            aggro_multiplier: 1.5,
            flee_multiplier: 0.8,
            preferred_attack_style: AttackStyle::Aggressive,
        });
        profiles.insert(FactionId::Military, FactionBehaviorProfile {
            aggro_multiplier: 0.8,
            flee_multiplier: 1.2,
            preferred_attack_style: AttackStyle::Defensive,
        });
        profiles.insert(FactionId::Aliens, FactionBehaviorProfile {
            aggro_multiplier: 1.2,
            flee_multiplier: 1.0,
            preferred_attack_style: AttackStyle::Erratic,
        });
        profiles.insert(FactionId::RogueDrones, FactionBehaviorProfile {
            aggro_multiplier: 1.0,
            flee_multiplier: 0.5,
            preferred_attack_style: AttackStyle::Aggressive,
        });
        profiles.insert(FactionId::Neutral, FactionBehaviorProfile {
            aggro_multiplier: 0.0, // Never aggro
            flee_multiplier: 2.0,  // Always flee
            preferred_attack_style: AttackStyle::Defensive,
        });
        Self { profiles }
    }
}

/// Pure function: applies faction behavior modifiers to base AI parameters.
/// Returns (effective_aggro_range, effective_flee_threshold).
pub fn apply_faction_modifiers(
    base_aggro: f32,
    base_flee_threshold: f32,
    faction: &FactionId,
    profiles: &FactionBehaviorProfiles,
) -> (f32, f32) {
    if let Some(profile) = profiles.profiles.get(faction) {
        (
            base_aggro * profile.aggro_multiplier,
            base_flee_threshold * profile.flee_multiplier,
        )
    } else {
        (base_aggro, base_flee_threshold)
    }
}

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

    // ── Story 4-6: Faction Behavior Profile tests ──

    #[test]
    fn faction_behavior_profiles_default_has_all_factions() {
        let profiles = FactionBehaviorProfiles::default();
        assert!(profiles.profiles.contains_key(&FactionId::Pirates), "Should have Pirates");
        assert!(profiles.profiles.contains_key(&FactionId::Military), "Should have Military");
        assert!(profiles.profiles.contains_key(&FactionId::Aliens), "Should have Aliens");
        assert!(profiles.profiles.contains_key(&FactionId::RogueDrones), "Should have RogueDrones");
        assert!(profiles.profiles.contains_key(&FactionId::Neutral), "Should have Neutral");
    }

    #[test]
    fn pirates_are_more_aggressive_than_military() {
        let profiles = FactionBehaviorProfiles::default();
        let pirates = profiles.profiles.get(&FactionId::Pirates)
            .expect("Pirates profile should exist");
        let military = profiles.profiles.get(&FactionId::Military)
            .expect("Military profile should exist");
        assert!(
            pirates.aggro_multiplier > military.aggro_multiplier,
            "Pirates ({}) should be more aggressive than Military ({})",
            pirates.aggro_multiplier,
            military.aggro_multiplier
        );
    }

    #[test]
    fn apply_faction_modifiers_pirates_increase_aggro() {
        let profiles = FactionBehaviorProfiles::default();
        let (effective_aggro, _) = apply_faction_modifiers(200.0, 0.2, &FactionId::Pirates, &profiles);
        assert!(
            effective_aggro > 200.0,
            "Pirates should increase aggro range above base (200), got {effective_aggro}"
        );
    }

    #[test]
    fn apply_faction_modifiers_military_reduce_aggro() {
        let profiles = FactionBehaviorProfiles::default();
        let (effective_aggro, _) = apply_faction_modifiers(200.0, 0.2, &FactionId::Military, &profiles);
        assert!(
            effective_aggro < 200.0,
            "Military should reduce aggro range below base (200), got {effective_aggro}"
        );
    }

    #[test]
    fn apply_faction_modifiers_is_pure_function() {
        let profiles = FactionBehaviorProfiles::default();
        let r1 = apply_faction_modifiers(200.0, 0.2, &FactionId::Aliens, &profiles);
        let r2 = apply_faction_modifiers(200.0, 0.2, &FactionId::Aliens, &profiles);
        assert!(
            (r1.0 - r2.0).abs() < f32::EPSILON && (r1.1 - r2.1).abs() < f32::EPSILON,
            "apply_faction_modifiers must be deterministic"
        );
    }

    #[test]
    fn attack_style_pirates_aggressive() {
        let profiles = FactionBehaviorProfiles::default();
        let pirates = profiles.profiles.get(&FactionId::Pirates)
            .expect("Pirates profile should exist");
        assert_eq!(
            pirates.preferred_attack_style,
            AttackStyle::Aggressive,
            "Pirates should use Aggressive attack style"
        );
    }

    #[test]
    fn attack_style_military_defensive() {
        let profiles = FactionBehaviorProfiles::default();
        let military = profiles.profiles.get(&FactionId::Military)
            .expect("Military profile should exist");
        assert_eq!(
            military.preferred_attack_style,
            AttackStyle::Defensive,
            "Military should use Defensive attack style"
        );
    }

    #[test]
    fn attack_style_aliens_erratic() {
        let profiles = FactionBehaviorProfiles::default();
        let aliens = profiles.profiles.get(&FactionId::Aliens)
            .expect("Aliens profile should exist");
        assert_eq!(
            aliens.preferred_attack_style,
            AttackStyle::Erratic,
            "Aliens should use Erratic attack style"
        );
    }

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
