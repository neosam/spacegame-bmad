use bevy::prelude::*;
use serde::{Serialize, Deserialize};

use crate::world::{BiomeType, ChunkCoord, ExploredChunks, WorldConfig};

use super::delta::{ChunkDelta, WorldDeltas};
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
    #[serde(default)]
    pub chunk_deltas: Vec<ChunkDelta>,
}

impl WorldSave {
    /// Builds a WorldSave from seed, explored chunks, and world deltas.
    /// Single source of truth for world-to-save conversion.
    pub fn from_resources(seed: u64, explored_chunks: &ExploredChunks, world_deltas: &WorldDeltas) -> Self {
        let mut chunks: Vec<_> = explored_chunks.chunks.iter()
            .map(|(coord, biome)| ((coord.x, coord.y), biome_to_str(biome).to_string()))
            .collect();
        chunks.sort_by_key(|((x, y), _)| (*x, *y));

        let mut chunk_deltas: Vec<ChunkDelta> = world_deltas.deltas.values()
            .filter(|d| !d.destroyed.is_empty())
            .cloned()
            .collect();
        chunk_deltas.sort_by_key(|d| (d.coord.x, d.coord.y));

        WorldSave {
            schema_version: SAVE_VERSION,
            seed,
            explored_chunks: chunks,
            chunk_deltas,
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

    /// Restores both explored chunks and world deltas from save data.
    pub fn apply_to_world_resources(&self, explored_chunks: &mut ExploredChunks, world_deltas: &mut WorldDeltas) {
        self.apply_to_explored(explored_chunks);
        world_deltas.deltas.clear();
        for delta in &self.chunk_deltas {
            world_deltas.deltas.insert(delta.coord, delta.clone());
        }
    }

    /// Extracts world state from the ECS world.
    pub fn from_world(world: &World) -> Self {
        let seed = world.get_resource::<WorldConfig>()
            .map(|c| c.seed)
            .unwrap_or(0);

        let default_deltas = WorldDeltas::default();
        let world_deltas = world.get_resource::<WorldDeltas>()
            .unwrap_or(&default_deltas);

        match world.get_resource::<ExploredChunks>() {
            Some(ec) => Self::from_resources(seed, ec, world_deltas),
            None => WorldSave {
                schema_version: SAVE_VERSION,
                seed,
                explored_chunks: Vec::new(),
                chunk_deltas: Vec::new(),
            },
        }
    }

    /// Restores explored chunks and world deltas into the world.
    pub fn apply_to_world(&self, world: &mut World) {
        let Some(mut explored) = world.get_resource_mut::<ExploredChunks>() else {
            warn!("No ExploredChunks resource found to apply save data");
            return;
        };

        self.apply_to_explored(&mut explored);

        if let Some(mut deltas) = world.get_resource_mut::<WorldDeltas>() {
            deltas.deltas.clear();
            for delta in &self.chunk_deltas {
                deltas.deltas.insert(delta.coord, delta.clone());
            }
        } else {
            warn!("No WorldDeltas resource found to apply delta data");
        }
    }

    /// Serializes to pretty-printed RON.
    pub fn to_ron(&self) -> Result<String, SaveError> {
        let pretty = ron::ser::PrettyConfig::new()
            .depth_limit(4)
            .separate_tuple_members(true);
        ron::ser::to_string_pretty(self, pretty)
            .map_err(|e| SaveError::ParseError(format!("{e}")))
    }

    /// Deserializes from RON with version check and auto-migration.
    pub fn from_ron(ron_str: &str) -> Result<Self, SaveError> {
        let version = check_version(ron_str)?;
        if version == 1 {
            return super::migration::migrate_world_v1_to_v2(ron_str);
        }
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
            chunk_deltas: Vec::new(),
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

    #[test]
    fn world_save_with_deltas_roundtrip() {
        let save = WorldSave {
            schema_version: SAVE_VERSION,
            seed: 42,
            explored_chunks: vec![((0, 0), "DeepSpace".to_string())],
            chunk_deltas: vec![
                ChunkDelta {
                    coord: ChunkCoord { x: 0, y: 0 },
                    destroyed: vec![1, 3, 7],
                },
                ChunkDelta {
                    coord: ChunkCoord { x: 1, y: -1 },
                    destroyed: vec![0, 5],
                },
            ],
        };

        let ron_str = save.to_ron().expect("Should serialize");
        let restored = WorldSave::from_ron(&ron_str).expect("Should deserialize");

        assert_eq!(restored.chunk_deltas.len(), 2);
        assert_eq!(restored.chunk_deltas[0].coord, ChunkCoord { x: 0, y: 0 });
        assert_eq!(restored.chunk_deltas[0].destroyed, vec![1, 3, 7]);
        assert_eq!(restored.chunk_deltas[1].coord, ChunkCoord { x: 1, y: -1 });
        assert_eq!(restored.chunk_deltas[1].destroyed, vec![0, 5]);
    }

    #[test]
    fn world_save_from_ron_v1_auto_migrates() {
        let v1_ron = r#"(
            schema_version: 1,
            seed: 42,
            explored_chunks: [
                ((0, 0), "DeepSpace"),
            ],
        )"#;

        let save = WorldSave::from_ron(v1_ron).expect("Should auto-migrate v1");
        assert_eq!(save.schema_version, SAVE_VERSION);
        assert_eq!(save.seed, 42);
        assert_eq!(save.explored_chunks.len(), 1);
        assert!(save.chunk_deltas.is_empty());
    }

    #[test]
    fn world_save_apply_to_world_resources_restores_deltas() {
        let save = WorldSave {
            schema_version: SAVE_VERSION,
            seed: 42,
            explored_chunks: vec![((0, 0), "DeepSpace".to_string())],
            chunk_deltas: vec![ChunkDelta {
                coord: ChunkCoord { x: 0, y: 0 },
                destroyed: vec![2, 4],
            }],
        };

        let mut explored = ExploredChunks::default();
        let mut deltas = WorldDeltas::default();
        save.apply_to_world_resources(&mut explored, &mut deltas);

        assert_eq!(explored.chunks.len(), 1);
        assert_eq!(deltas.deltas.len(), 1);
        let chunk_delta = deltas
            .deltas
            .get(&ChunkCoord { x: 0, y: 0 })
            .expect("Should have delta");
        assert_eq!(chunk_delta.destroyed, vec![2, 4]);
    }

    #[test]
    fn world_save_empty_deltas_roundtrip() {
        let save = WorldSave {
            schema_version: SAVE_VERSION,
            seed: 42,
            explored_chunks: vec![],
            chunk_deltas: vec![],
        };

        let ron_str = save.to_ron().expect("Should serialize");
        let restored = WorldSave::from_ron(&ron_str).expect("Should deserialize");

        assert!(restored.chunk_deltas.is_empty());
    }
}
