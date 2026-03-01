use bevy::log::warn;
use noise::{Fbm, MultiFractal, NoiseFn, Perlin};

use super::chunk::ChunkCoord;

/// Offset added to the world seed for biome noise evaluation,
/// keeping biome noise independent of entity generation and future noise layers.
pub const BIOME_NOISE_OFFSET: u64 = 0xB10E_B10E_B10E_B10E;

/// Configuration for biome noise generation.
#[derive(Clone, Debug, serde::Deserialize)]
pub struct BiomeNoiseConfig {
    /// Controls biome region size; smaller = larger regions.
    #[serde(default = "default_noise_scale")]
    pub noise_scale: f64,
    /// Number of octaves for fractal noise.
    #[serde(default = "default_noise_octaves")]
    pub noise_octaves: usize,
    /// Persistence for fractal noise.
    #[serde(default = "default_noise_persistence")]
    pub noise_persistence: f64,
    /// Lacunarity for fractal noise.
    #[serde(default = "default_noise_lacunarity")]
    pub noise_lacunarity: f64,
}

fn default_noise_scale() -> f64 {
    0.37
}
fn default_noise_octaves() -> usize {
    4
}
fn default_noise_persistence() -> f64 {
    0.5
}
fn default_noise_lacunarity() -> f64 {
    2.0
}

impl BiomeNoiseConfig {
    /// Warns if noise parameters have degenerate values that would break biome distribution.
    pub fn validate(&self) {
        if self.noise_octaves == 0 {
            warn!(
                "BiomeNoiseConfig: noise_octaves is 0. Noise will produce NaN; all chunks will map to the same biome."
            );
        }
        if self.noise_scale.abs() < f64::EPSILON {
            warn!(
                "BiomeNoiseConfig: noise_scale is ~0.0. All chunks will sample the same noise point."
            );
        }
        if self.noise_persistence <= 0.0 {
            warn!(
                "BiomeNoiseConfig: noise_persistence ({}) is <= 0.0. Noise distribution may be degenerate.",
                self.noise_persistence
            );
        }
        if self.noise_lacunarity <= 0.0 {
            warn!(
                "BiomeNoiseConfig: noise_lacunarity ({}) is <= 0.0. Noise distribution may be degenerate.",
                self.noise_lacunarity
            );
        }
    }
}

impl Default for BiomeNoiseConfig {
    fn default() -> Self {
        Self {
            noise_scale: default_noise_scale(),
            noise_octaves: default_noise_octaves(),
            noise_persistence: default_noise_persistence(),
            noise_lacunarity: default_noise_lacunarity(),
        }
    }
}

/// Evaluates fractal Perlin noise for the given chunk coordinate and seed.
///
/// Raw FBM output is divided by its theoretical max amplitude (geometric
/// sum of persistence) to normalize back to the base Perlin range. A
/// contrast factor (`CONTRAST = 4.0`) is then applied to spread the values
/// toward the extremes so that threshold-based biome selection works with
/// the standard `[0.0, 1.0]` thresholds.
///
/// **Note:** The returned value can exceed `[-1.0, 1.0]` due to the contrast
/// multiplier. Use [`noise_to_unit`] to remap and clamp into `[0.0, 1.0)`.
pub fn biome_noise_value(seed: u64, coord: ChunkCoord, config: &BiomeNoiseConfig) -> f64 {
    let noise_seed = seed.wrapping_add(BIOME_NOISE_OFFSET);
    let fbm = Fbm::<Perlin>::new(noise_seed as u32)
        .set_octaves(config.noise_octaves)
        .set_persistence(config.noise_persistence)
        .set_lacunarity(config.noise_lacunarity);
    let raw = fbm.get([
        coord.x as f64 * config.noise_scale,
        coord.y as f64 * config.noise_scale,
    ]);
    // Normalize by max amplitude, then apply contrast to spread distribution.
    let max_amplitude = if (config.noise_persistence - 1.0).abs() < f64::EPSILON {
        config.noise_octaves as f64
    } else {
        (1.0 - config.noise_persistence.powi(config.noise_octaves as i32))
            / (1.0 - config.noise_persistence)
    };
    const CONTRAST: f64 = 4.0;
    (raw / max_amplitude) * CONTRAST
}

/// Remaps a noise value from `[-1.0, 1.0]` to `[0.0, 1.0)` with clamping.
/// The upper bound is exclusive to match the threshold comparison behavior
/// (same as the `[0, 1)` range produced by the previous RNG approach).
pub fn noise_to_unit(value: f64) -> f32 {
    let v = ((value + 1.0) / 2.0).clamp(0.0, 1.0) as f32;
    v.min(1.0 - f32::EPSILON)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn default_config() -> BiomeNoiseConfig {
        BiomeNoiseConfig::default()
    }

    #[test]
    fn biome_noise_value_is_deterministic() {
        let config = default_config();
        let coord = ChunkCoord { x: 5, y: -3 };
        let v1 = biome_noise_value(42, coord, &config);
        let v2 = biome_noise_value(42, coord, &config);
        assert!(
            (v1 - v2).abs() < f64::EPSILON,
            "Same seed+coord must produce identical noise: {v1} vs {v2}"
        );
    }

    #[test]
    fn biome_noise_value_varies_spatially() {
        let config = default_config();
        let seed = 42u64;
        let mut values = std::collections::HashSet::new();
        for x in 0..10 {
            for y in 0..10 {
                let v = biome_noise_value(seed, ChunkCoord { x, y }, &config);
                // Convert to bits for exact comparison in HashSet
                values.insert(v.to_bits());
            }
        }
        assert!(
            values.len() > 1,
            "Different coords should produce different noise values, got only {} unique",
            values.len()
        );
    }

    #[test]
    fn noise_to_unit_clamps_to_0_1() {
        assert!((noise_to_unit(-2.0) - 0.0).abs() < f32::EPSILON, "Values below -1.0 should clamp to 0.0");
        // Upper bound is exclusive [0, 1), so extreme values clamp just below 1.0
        assert!(noise_to_unit(2.0) < 1.0, "Values above 1.0 should clamp below 1.0");
        assert!(noise_to_unit(2.0) > 0.99, "Values above 1.0 should be close to 1.0");
        assert!(noise_to_unit(-5.0) >= 0.0);
        assert!(noise_to_unit(5.0) < 1.0);
    }

    #[test]
    fn noise_to_unit_remaps_correctly() {
        assert!((noise_to_unit(-1.0) - 0.0).abs() < f32::EPSILON, "-1.0 should map to 0.0");
        assert!((noise_to_unit(0.0) - 0.5).abs() < f32::EPSILON, "0.0 should map to 0.5");
        // 1.0 maps to just below 1.0 (exclusive upper bound)
        assert!(noise_to_unit(1.0) > 0.99, "1.0 should map close to 1.0");
        assert!(noise_to_unit(1.0) < 1.0, "1.0 should map below 1.0 (exclusive)");
    }

    #[test]
    fn adjacent_chunks_tend_to_share_biome() {
        use crate::world::BiomeConfig;
        use crate::world::generation::determine_biome;

        let config = BiomeConfig::default();
        let seed = 42u64;
        let mut same_count = 0u32;
        let total_pairs = 100u32;

        // Sample 100 adjacent horizontal pairs
        for i in 0..100 {
            let x = i - 50;
            let y = 0;
            let biome_a = determine_biome(seed, ChunkCoord { x, y }, &config);
            let biome_b = determine_biome(seed, ChunkCoord { x: x + 1, y }, &config);
            if biome_a == biome_b {
                same_count += 1;
            }
        }

        assert!(
            same_count > 50,
            "Adjacent chunks should share biomes >50% of the time (spatial coherence). \
             Got {same_count}/{total_pairs} same-biome pairs."
        );
    }
}
