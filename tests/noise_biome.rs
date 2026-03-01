#![deny(clippy::unwrap_used)]

mod helpers;

use bevy::prelude::*;
use helpers::{run_until_loaded, spawn_player, test_app};
use void_drifter::world::{ActiveChunks, ChunkCoord};

/// Adjacent chunks share biomes more than random (>50% vs ~33% for uniform random) (AC: #1, #7)
#[test]
fn noise_biomes_are_spatially_coherent() {
    let mut app = test_app();
    spawn_player(&mut app);
    run_until_loaded(&mut app);

    let active = app.world().resource::<ActiveChunks>();
    let mut same_count = 0u32;
    let mut total_pairs = 0u32;

    for (&coord, &biome) in &active.chunks {
        let right = ChunkCoord { x: coord.x + 1, y: coord.y };
        let up = ChunkCoord { x: coord.x, y: coord.y + 1 };

        if let Some(&neighbor_biome) = active.chunks.get(&right) {
            total_pairs += 1;
            if biome == neighbor_biome {
                same_count += 1;
            }
        }
        if let Some(&neighbor_biome) = active.chunks.get(&up) {
            total_pairs += 1;
            if biome == neighbor_biome {
                same_count += 1;
            }
        }
    }

    assert!(
        total_pairs > 0,
        "Should have adjacent chunk pairs to compare"
    );

    let pct = (same_count as f32 / total_pairs as f32) * 100.0;
    assert!(
        pct > 50.0,
        "Adjacent chunks should share biomes >50% of the time (spatial coherence). Got {pct:.1}% ({same_count}/{total_pairs})"
    );
}

/// Unload and reload chunks, verify same biomes assigned (AC: #2)
#[test]
fn noise_biomes_seed_deterministic_across_reload() {
    let mut app = test_app();
    let player = spawn_player(&mut app);
    run_until_loaded(&mut app);

    // Record initial biome map
    let initial_biomes: std::collections::HashMap<ChunkCoord, _> =
        app.world().resource::<ActiveChunks>().chunks.clone();

    assert!(
        !initial_biomes.is_empty(),
        "Should have loaded some chunks initially"
    );

    // Move player far away to unload all original chunks
    app.world_mut()
        .entity_mut(player)
        .get_mut::<Transform>()
        .expect("Player should have Transform")
        .translation = Vec3::new(50000.0, 50000.0, 0.0);
    run_until_loaded(&mut app);

    // Verify original chunks are unloaded
    let active = app.world().resource::<ActiveChunks>();
    for coord in initial_biomes.keys() {
        assert!(
            !active.chunks.contains_key(coord),
            "Original chunk ({},{}) should be unloaded",
            coord.x, coord.y
        );
    }

    // Move player back to origin to reload
    app.world_mut()
        .entity_mut(player)
        .get_mut::<Transform>()
        .expect("Player should have Transform")
        .translation = Vec3::ZERO;
    run_until_loaded(&mut app);

    // Verify same biomes after reload
    let reloaded = app.world().resource::<ActiveChunks>();
    for (coord, expected_biome) in &initial_biomes {
        let reloaded_biome = reloaded
            .chunks
            .get(coord)
            .unwrap_or_else(|| panic!("Chunk ({},{}) should be reloaded", coord.x, coord.y));
        assert_eq!(
            expected_biome, reloaded_biome,
            "Chunk ({},{}) biome should be identical after reload: expected {:?}, got {:?}",
            coord.x, coord.y, expected_biome, reloaded_biome
        );
    }
}
