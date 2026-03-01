use bevy::prelude::*;
use serde::{Serialize, Deserialize};

use crate::world::{BiomeType, ChunkCoord, ExploredChunks, WorldConfig};

use super::schema::{check_version, SaveError, SAVE_VERSION};

/// Converts a BiomeType to its string representation for save files.
pub fn biome_to_str(biome: &BiomeType) -> &'static str {
    match biome {
        BiomeType::DeepSpace => "DeepSpace",
        BiomeType::AsteroidField => "AsteroidField",
        BiomeType::WreckField => "WreckField",
    }
}

/// Converts a save-file string back to a BiomeType.
/// Defaults to DeepSpace for unknown values.
pub fn str_to_biome(s: &str) -> BiomeType {
    match s {
        "AsteroidField" => BiomeType::AsteroidField,
        "WreckField" => BiomeType::WreckField,
        _ => BiomeType::DeepSpace,
    }
}

/// Serializable snapshot of world state.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct WorldSave {
    pub schema_version: u32,
    pub seed: u64,
    pub explored_chunks: Vec<((i32, i32), String)>,
}

impl WorldSave {
    /// Builds a WorldSave from seed and explored chunks.
    /// Single source of truth for world-to-save conversion.
    pub fn from_resources(seed: u64, explored_chunks: &ExploredChunks) -> Self {
        let mut chunks: Vec<_> = explored_chunks.chunks.iter()
            .map(|(coord, biome)| ((coord.x, coord.y), biome_to_str(biome).to_string()))
            .collect();
        chunks.sort_by_key(|((x, y), _)| (*x, *y));
        WorldSave {
            schema_version: SAVE_VERSION,
            seed,
            explored_chunks: chunks,
        }
    }

    /// Restores explored chunks from save data into the resource.
    /// Single source of truth for save-to-world conversion.
    pub fn apply_to_explored(&self, explored_chunks: &mut ExploredChunks) {
        explored_chunks.chunks.clear();
        for ((x, y), biome_str) in &self.explored_chunks {
            explored_chunks.chunks.insert(
                ChunkCoord { x: *x, y: *y },
                str_to_biome(biome_str),
            );
        }
    }

    /// Extracts world state from the ECS world.
    pub fn from_world(world: &World) -> Self {
        let seed = world.get_resource::<WorldConfig>()
            .map(|c| c.seed)
            .unwrap_or(0);

        match world.get_resource::<ExploredChunks>() {
            Some(ec) => Self::from_resources(seed, ec),
            None => WorldSave {
                schema_version: SAVE_VERSION,
                seed,
                explored_chunks: Vec::new(),
            },
        }
    }

    /// Restores explored chunks into the world.
    pub fn apply_to_world(&self, world: &mut World) {
        let Some(mut explored) = world.get_resource_mut::<ExploredChunks>() else {
            warn!("No ExploredChunks resource found to apply save data");
            return;
        };

        self.apply_to_explored(&mut explored);
    }

    /// Serializes to pretty-printed RON.
    pub fn to_ron(&self) -> Result<String, SaveError> {
        let pretty = ron::ser::PrettyConfig::new()
            .depth_limit(4)
            .separate_tuple_members(true);
        ron::ser::to_string_pretty(self, pretty)
            .map_err(|e| SaveError::ParseError(format!("{e}")))
    }

    /// Deserializes from RON with version check.
    pub fn from_ron(ron_str: &str) -> Result<Self, SaveError> {
        check_version(ron_str)?;
        ron::from_str(ron_str).map_err(|e| SaveError::ParseError(format!("{e}")))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_world_save() -> WorldSave {
        WorldSave {
            schema_version: SAVE_VERSION,
            seed: 42,
            explored_chunks: vec![
                ((0, 0), "DeepSpace".to_string()),
                ((1, 0), "AsteroidField".to_string()),
                ((-1, 2), "WreckField".to_string()),
            ],
        }
    }

    #[test]
    fn world_save_roundtrip() {
        let original = sample_world_save();
        let ron_str = original.to_ron().expect("Should serialize");
        let restored = WorldSave::from_ron(&ron_str).expect("Should deserialize");

        assert_eq!(restored.schema_version, original.schema_version);
        assert_eq!(restored.seed, original.seed);
        assert_eq!(restored.explored_chunks.len(), original.explored_chunks.len());
        for (a, b) in restored.explored_chunks.iter().zip(original.explored_chunks.iter()) {
            assert_eq!(a.0, b.0);
            assert_eq!(a.1, b.1);
        }
    }

    #[test]
    fn world_save_from_ron_corrupt_returns_error() {
        let result = WorldSave::from_ron("not valid ron data {{{");
        assert!(result.is_err());
    }
}
