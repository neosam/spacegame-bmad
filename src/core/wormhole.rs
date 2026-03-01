use bevy::prelude::*;

use crate::world::ChunkCoord;

/// Speichert die Weltposition wo der Spieler das Wormhole betreten hat.
/// Für Crash-Recovery: Spieler kehrt hierhin bei Tod oder Exit zurück.
#[derive(Resource, Debug)]
pub struct WormholeEntrance {
    pub world_position: Vec2,
    pub wormhole_entity: Entity,
}

/// Verfolgt den Zustand der aktuellen Wormhole-Arena.
#[derive(Resource, Default, Debug)]
pub struct ArenaState {
    pub wave: u32,
    pub total_waves: u32,
    pub enemies_remaining: u32,
    pub cleared: bool,
}

/// Marker-Komponente für eine Wormhole-Entity in der Hauptwelt.
#[derive(Component, Debug, Clone)]
pub struct Wormhole {
    pub coord: ChunkCoord,
    pub cleared: bool,
}

/// Marker: Rendering-Plugin soll Wormhole-Mesh anhängen.
#[derive(Component)]
pub struct NeedsWormholeVisual;

/// Pure Funktion: Soll in diesem Chunk ein Wormhole spawnen?
/// Mindestdistanz 2 Chunks vom Ursprung, ca. 1 von 8 Chunks.
pub fn should_spawn_wormhole(coord: ChunkCoord, seed: u64) -> bool {
    let hash = (coord.x as u64).wrapping_mul(2654435761)
        ^ (coord.y as u64).wrapping_mul(2246822519)
        ^ seed;
    let dist = ((coord.x * coord.x + coord.y * coord.y) as f32).sqrt();
    dist >= 2.0 && hash % 8 == 0
}
