use bevy::ecs::message::Message;
use bevy::prelude::*;
use serde::Deserialize;

use crate::shared::components::MaterialType;
use crate::world::{BiomeType, ChunkCoord};

/// Which weapon the player fired or switched to/from.
/// Mirrors `ActiveWeapon` variants but decoupled for the event system.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Deserialize)]
pub enum WeaponKind {
    Laser,
    Spread,
}

/// Severity tier for game events. Controls logbook filtering and display priority.
#[derive(Deserialize, Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum EventSeverity {
    /// Critical — always shown (e.g. player death)
    Tier1,
    /// Notable (e.g. player respawned)
    Tier2,
    /// Minor (e.g. enemy destroyed, chunk loaded)
    Tier3,
}

/// Categorizes what happened, with context-carrying variant data.
#[derive(Clone, Debug, PartialEq)]
pub enum GameEventKind {
    /// A non-player entity was destroyed at health <= 0.
    EnemyDestroyed { entity_type: &'static str },
    /// Player health reached 0.
    PlayerDeath,
    /// Player reset to origin after death.
    PlayerRespawned,
    /// Chunk entered ActiveChunks.
    ChunkLoaded { coord: ChunkCoord, biome: BiomeType },
    /// Chunk removed from ActiveChunks.
    ChunkUnloaded { coord: ChunkCoord },
    /// Laser or spread weapon fired.
    WeaponFired { weapon: WeaponKind },
    /// Player toggled weapon.
    WeaponSwitched { from: WeaponKind, to: WeaponKind },
    /// Game was saved to disk.
    GameSaved,
    /// Tutorial zone was spawned.
    TutorialZoneSpawned,
    /// Player docked at the tutorial station, receiving the Spread weapon.
    StationDocked,
    /// GravityWellGenerator was destroyed — tutorial escape complete.
    GeneratorDestroyed,
    /// Destruction cascade complete — tutorial fully finished.
    TutorialComplete,
    /// Player earned credits (from kill or discovery).
    CreditsEarned { amount: u32 },
    /// Player picked up a material drop.
    MaterialCollected { material: MaterialType },
    /// Player crafted or bought an upgrade at a station.
    UpgradeCrafted { system_name: String, tier: u8 },
}

/// A game event emitted as a Bevy message by gameplay systems.
/// Consumed by the event-observer to populate the Logbook.
#[derive(Message, Clone, Debug)]
pub struct GameEvent {
    pub kind: GameEventKind,
    pub severity: EventSeverity,
    pub position: Vec2,
    pub game_time: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn game_event_kind_enemy_destroyed_carries_entity_type() {
        let kind = GameEventKind::EnemyDestroyed {
            entity_type: "asteroid",
        };
        if let GameEventKind::EnemyDestroyed { entity_type } = kind {
            assert_eq!(entity_type, "asteroid");
        } else {
            panic!("Expected EnemyDestroyed variant");
        }
    }

    #[test]
    fn game_event_kind_chunk_loaded_carries_coord_and_biome() {
        let kind = GameEventKind::ChunkLoaded {
            coord: ChunkCoord { x: 1, y: -2 },
            biome: BiomeType::AsteroidField,
        };
        if let GameEventKind::ChunkLoaded { coord, biome } = &kind {
            assert_eq!(coord.x, 1);
            assert_eq!(coord.y, -2);
            assert_eq!(*biome, BiomeType::AsteroidField);
        } else {
            panic!("Expected ChunkLoaded variant");
        }
    }

    #[test]
    fn weapon_kind_has_laser_and_spread() {
        let laser = WeaponKind::Laser;
        let spread = WeaponKind::Spread;
        assert_ne!(laser, spread);
    }

    #[test]
    fn event_severity_has_three_tiers() {
        let t1 = EventSeverity::Tier1;
        let t2 = EventSeverity::Tier2;
        let t3 = EventSeverity::Tier3;
        assert_ne!(t1, t2);
        assert_ne!(t2, t3);
        assert_ne!(t1, t3);
    }

    #[test]
    fn game_event_clone_and_debug() {
        let event = GameEvent {
            kind: GameEventKind::PlayerRespawned,
            severity: EventSeverity::Tier2,
            position: Vec2::ZERO,
            game_time: 1.0,
        };
        let cloned = event.clone();
        assert_eq!(cloned.kind, GameEventKind::PlayerRespawned);
        assert_eq!(cloned.severity, EventSeverity::Tier2);
        // Debug should not panic
        let _ = format!("{:?}", cloned);
    }
}
