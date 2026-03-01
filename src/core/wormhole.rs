use bevy::prelude::*;

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
