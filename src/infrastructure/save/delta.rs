use std::collections::HashMap;

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::world::ChunkCoord;

/// Deterministic identity for a seed-generated entity.
/// Same seed + chunk = same generation order = same index.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct SeededEntityId {
    pub chunk: ChunkCoord,
    pub index: u32,
}

/// Per-chunk delta: tracks which seed-generated entities have been destroyed.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct ChunkDelta {
    pub coord: ChunkCoord,
    pub destroyed: Vec<u32>,
}

/// Resource tracking per-chunk entity destructions across sessions.
/// Seed reproduces base world; only deviations are persisted.
#[derive(Resource, Default, Debug, Clone)]
pub struct WorldDeltas {
    pub deltas: HashMap<ChunkCoord, ChunkDelta>,
}

/// Index of a seed-generated entity within its chunk's blueprint list.
/// Only assigned to entities spawned from `generate_chunk_content` blueprints.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct SeedIndex(pub u32);

/// Tracks destroyed seed-generated entities in WorldDeltas.
/// Runs after apply_damage but before despawn_destroyed so entities still exist.
/// Only records health-based destruction of entities with ChunkEntity + SeedIndex.
/// Chunk-unloading despawns (from update_chunks) are NOT recorded.
pub fn track_destroyed_entities(
    query: Query<(&crate::world::ChunkEntity, &SeedIndex, &crate::core::collision::Health)>,
    mut world_deltas: ResMut<WorldDeltas>,
) {
    for (chunk_entity, seed_index, health) in query.iter() {
        if health.current <= 0.0 {
            let delta = world_deltas
                .deltas
                .entry(chunk_entity.coord)
                .or_insert_with(|| ChunkDelta {
                    coord: chunk_entity.coord,
                    destroyed: Vec::new(),
                });
            if !delta.destroyed.contains(&seed_index.0) {
                delta.destroyed.push(seed_index.0);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn seeded_entity_id_roundtrip() {
        let id = SeededEntityId {
            chunk: ChunkCoord { x: 3, y: -7 },
            index: 42,
        };
        let ron_str = ron::ser::to_string_pretty(&id, ron::ser::PrettyConfig::default())
            .expect("Should serialize SeededEntityId");
        let restored: SeededEntityId =
            ron::from_str(&ron_str).expect("Should deserialize SeededEntityId");
        assert_eq!(id, restored);
    }

    #[test]
    fn chunk_delta_roundtrip() {
        let delta = ChunkDelta {
            coord: ChunkCoord { x: 1, y: -2 },
            destroyed: vec![0, 3, 7, 12],
        };
        let ron_str = ron::ser::to_string_pretty(&delta, ron::ser::PrettyConfig::default())
            .expect("Should serialize ChunkDelta");
        let restored: ChunkDelta =
            ron::from_str(&ron_str).expect("Should deserialize ChunkDelta");
        assert_eq!(delta, restored);
    }

    #[test]
    fn world_deltas_tracks_destroyed_entity() {
        let mut deltas = WorldDeltas::default();
        let coord = ChunkCoord { x: 0, y: 0 };

        // Add a destroyed entity
        let delta = deltas.deltas.entry(coord).or_insert_with(|| ChunkDelta {
            coord,
            destroyed: Vec::new(),
        });
        delta.destroyed.push(5);

        assert_eq!(deltas.deltas.len(), 1);
        let chunk_delta = deltas.deltas.get(&coord).expect("Should have chunk delta");
        assert_eq!(chunk_delta.destroyed, vec![5]);
    }

    #[test]
    fn world_deltas_multiple_chunks() {
        let mut deltas = WorldDeltas::default();
        let coord1 = ChunkCoord { x: 0, y: 0 };
        let coord2 = ChunkCoord { x: 1, y: 1 };

        deltas.deltas.insert(
            coord1,
            ChunkDelta {
                coord: coord1,
                destroyed: vec![1, 3],
            },
        );
        deltas.deltas.insert(
            coord2,
            ChunkDelta {
                coord: coord2,
                destroyed: vec![0],
            },
        );

        assert_eq!(deltas.deltas.len(), 2);
        assert_eq!(
            deltas
                .deltas
                .get(&coord1)
                .expect("chunk1")
                .destroyed
                .len(),
            2
        );
        assert_eq!(
            deltas
                .deltas
                .get(&coord2)
                .expect("chunk2")
                .destroyed
                .len(),
            1
        );
    }

    #[test]
    fn seed_index_component_value() {
        let idx = SeedIndex(42);
        assert_eq!(idx.0, 42);
    }

    #[test]
    fn chunk_delta_empty_destroyed_roundtrip() {
        let delta = ChunkDelta {
            coord: ChunkCoord { x: 0, y: 0 },
            destroyed: Vec::new(),
        };
        let ron_str = ron::ser::to_string_pretty(&delta, ron::ser::PrettyConfig::default())
            .expect("Should serialize empty ChunkDelta");
        let restored: ChunkDelta =
            ron::from_str(&ron_str).expect("Should deserialize empty ChunkDelta");
        assert_eq!(delta, restored);
    }
}
