use std::collections::HashSet;

use bevy::prelude::*;

/// Integer coordinate identifying a chunk in the world grid.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Default)]
pub struct ChunkCoord {
    pub x: i32,
    pub y: i32,
}

/// Maps a world-space position to the chunk coordinate that contains it.
pub fn world_to_chunk(position: Vec2, chunk_size: f32) -> ChunkCoord {
    ChunkCoord {
        x: position.x.div_euclid(chunk_size) as i32,
        y: position.y.div_euclid(chunk_size) as i32,
    }
}

/// Returns the world-space center of the given chunk.
pub fn chunk_to_world_center(coord: ChunkCoord, chunk_size: f32) -> Vec2 {
    Vec2::new(
        (coord.x as f32 + 0.5) * chunk_size,
        (coord.y as f32 + 0.5) * chunk_size,
    )
}

/// Returns the set of all chunk coordinates within `radius` of `center` (square neighborhood).
pub fn chunks_in_radius(center: ChunkCoord, radius: u32) -> HashSet<ChunkCoord> {
    let r = radius as i32;
    let mut chunks = HashSet::new();
    for dx in -r..=r {
        for dy in -r..=r {
            chunks.insert(ChunkCoord {
                x: center.x + dx,
                y: center.y + dy,
            });
        }
    }
    chunks
}

// ── Unit tests ──────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    // ── world_to_chunk ──

    #[test]
    fn world_to_chunk_origin() {
        let coord = world_to_chunk(Vec2::new(0.0, 0.0), 1000.0);
        assert_eq!(coord, ChunkCoord { x: 0, y: 0 });
    }

    #[test]
    fn world_to_chunk_positive() {
        let coord = world_to_chunk(Vec2::new(1500.0, 2500.0), 1000.0);
        assert_eq!(coord, ChunkCoord { x: 1, y: 2 });
    }

    #[test]
    fn world_to_chunk_negative() {
        let coord = world_to_chunk(Vec2::new(-1.0, -1.0), 1000.0);
        assert_eq!(coord, ChunkCoord { x: -1, y: -1 });
    }

    #[test]
    fn world_to_chunk_exact_boundary() {
        // Exactly on chunk boundary should be in the next chunk
        let coord = world_to_chunk(Vec2::new(1000.0, 0.0), 1000.0);
        assert_eq!(coord, ChunkCoord { x: 1, y: 0 });
    }

    #[test]
    fn world_to_chunk_just_before_boundary() {
        let coord = world_to_chunk(Vec2::new(999.9, 0.0), 1000.0);
        assert_eq!(coord, ChunkCoord { x: 0, y: 0 });
    }

    #[test]
    fn world_to_chunk_negative_boundary() {
        // -1000.0 should be in chunk -1 (div_euclid: -1000/1000 = -1)
        let coord = world_to_chunk(Vec2::new(-1000.0, 0.0), 1000.0);
        assert_eq!(coord, ChunkCoord { x: -1, y: 0 });
    }

    #[test]
    fn world_to_chunk_negative_just_past_boundary() {
        let coord = world_to_chunk(Vec2::new(-1001.0, 0.0), 1000.0);
        assert_eq!(coord, ChunkCoord { x: -2, y: 0 });
    }

    // ── chunk_to_world_center ──

    #[test]
    fn chunk_center_origin_chunk() {
        let center = chunk_to_world_center(ChunkCoord { x: 0, y: 0 }, 1000.0);
        assert!((center.x - 500.0).abs() < f32::EPSILON);
        assert!((center.y - 500.0).abs() < f32::EPSILON);
    }

    #[test]
    fn chunk_center_positive_chunk() {
        let center = chunk_to_world_center(ChunkCoord { x: 2, y: 3 }, 1000.0);
        assert!((center.x - 2500.0).abs() < f32::EPSILON);
        assert!((center.y - 3500.0).abs() < f32::EPSILON);
    }

    #[test]
    fn chunk_center_negative_chunk() {
        let center = chunk_to_world_center(ChunkCoord { x: -1, y: -1 }, 1000.0);
        assert!((center.x - (-500.0)).abs() < f32::EPSILON);
        assert!((center.y - (-500.0)).abs() < f32::EPSILON);
    }

    // ── chunks_in_radius ──

    #[test]
    fn chunks_in_radius_zero() {
        let chunks = chunks_in_radius(ChunkCoord { x: 0, y: 0 }, 0);
        assert_eq!(chunks.len(), 1);
        assert!(chunks.contains(&ChunkCoord { x: 0, y: 0 }));
    }

    #[test]
    fn chunks_in_radius_one() {
        let chunks = chunks_in_radius(ChunkCoord { x: 0, y: 0 }, 1);
        // 3x3 grid = 9 chunks
        assert_eq!(chunks.len(), 9);
        assert!(chunks.contains(&ChunkCoord { x: -1, y: -1 }));
        assert!(chunks.contains(&ChunkCoord { x: 0, y: 0 }));
        assert!(chunks.contains(&ChunkCoord { x: 1, y: 1 }));
    }

    #[test]
    fn chunks_in_radius_two() {
        let chunks = chunks_in_radius(ChunkCoord { x: 5, y: 5 }, 2);
        // 5x5 grid = 25 chunks
        assert_eq!(chunks.len(), 25);
        assert!(chunks.contains(&ChunkCoord { x: 3, y: 3 }));
        assert!(chunks.contains(&ChunkCoord { x: 7, y: 7 }));
        assert!(chunks.contains(&ChunkCoord { x: 5, y: 5 }));
    }

    #[test]
    fn chunks_in_radius_with_negative_center() {
        let chunks = chunks_in_radius(ChunkCoord { x: -3, y: -3 }, 1);
        assert_eq!(chunks.len(), 9);
        assert!(chunks.contains(&ChunkCoord { x: -4, y: -4 }));
        assert!(chunks.contains(&ChunkCoord { x: -2, y: -2 }));
    }
}
