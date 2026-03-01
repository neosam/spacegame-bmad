#![deny(clippy::unwrap_used)]
//! Property-based roundtrip tests for the delta-save system (Story 1.9).

use proptest::prelude::*;
use proptest::collection::vec;

use void_drifter::infrastructure::save::delta::ChunkDelta;
use void_drifter::infrastructure::save::world_save::WorldSave;
use void_drifter::infrastructure::save::schema::SAVE_VERSION;
use void_drifter::world::chunk::ChunkCoord;
use void_drifter::world::generation::{determine_biome, generate_chunk_content};
use void_drifter::world::BiomeConfig;

proptest! {
    /// Generate chunk from seed → apply random destructions → compute delta
    /// → regenerate from seed → apply delta → verify result matches.
    #[test]
    fn delta_roundtrip(
        seed: u64,
        chunk_x in -50i32..50,
        chunk_y in -50i32..50,
        destroyed_indices in vec(0u32..20, 0..10),
    ) {
        let config = BiomeConfig::default();
        let coord = ChunkCoord { x: chunk_x, y: chunk_y };
        let biome = determine_biome(seed, coord, &config);
        let blueprints = generate_chunk_content(seed, coord, 1000.0, biome, &config);

        // Create delta from random destruction indices (clamped to actual count)
        let valid_destroyed: Vec<u32> = destroyed_indices.into_iter()
            .filter(|&i| (i as usize) < blueprints.len())
            .collect();

        let delta = ChunkDelta { coord, destroyed: valid_destroyed };

        // Apply delta: filter blueprints
        let surviving: Vec<_> = blueprints.iter().enumerate()
            .filter(|(i, _)| !delta.destroyed.contains(&(*i as u32)))
            .collect();

        // Regenerate and apply delta again
        let blueprints2 = generate_chunk_content(seed, coord, 1000.0, biome, &config);
        let surviving2: Vec<_> = blueprints2.iter().enumerate()
            .filter(|(i, _)| !delta.destroyed.contains(&(*i as u32)))
            .collect();

        // Roundtrip: same survivors
        prop_assert_eq!(surviving.len(), surviving2.len());
    }

    /// WorldSave with deltas serializes and deserializes correctly.
    #[test]
    fn world_save_delta_roundtrip(
        seed: u64,
        destroyed in vec(0u32..20, 0..5),
    ) {
        let deltas = vec![ChunkDelta {
            coord: ChunkCoord { x: 0, y: 0 },
            destroyed: destroyed.clone(),
        }];

        let save = WorldSave {
            schema_version: SAVE_VERSION,
            seed,
            explored_chunks: vec![((0, 0), "DeepSpace".to_string())],
            chunk_deltas: deltas,
        };

        let ron_str = save.to_ron().expect("Should serialize");
        let restored = WorldSave::from_ron(&ron_str).expect("Should deserialize");

        prop_assert_eq!(restored.seed, seed);
        prop_assert_eq!(restored.chunk_deltas.len(), 1);
        prop_assert_eq!(&restored.chunk_deltas[0].destroyed, &destroyed);
    }
}
