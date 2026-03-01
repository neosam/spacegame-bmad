use std::collections::{HashMap, HashSet};

use bevy::ecs::message::{MessageReader, MessageWriter};
use bevy::prelude::*;

use crate::core::collision::Collider;
use crate::core::flight::Player;
use crate::infrastructure::events::EventSeverityConfig;
use crate::shared::components::{MaterialDrop, MaterialType, NeedsMaterialDropVisual};
use crate::shared::events::{GameEvent, GameEventKind};
use crate::world::ChunkCoord;

/// Player's current credit balance.
#[derive(Resource, Default, Debug, Clone)]
pub struct Credits {
    pub balance: u32,
}

/// Tracks which chunk coords have already been credited for first-discovery.
/// Prevents double-awarding when a previously-explored chunk re-enters ActiveChunks.
#[derive(Resource, Default, Debug)]
pub struct DiscoveredChunks {
    pub chunks: HashSet<ChunkCoord>,
}

/// Buffers credit award events to emit in a separate system (avoids MessageReader + MessageWriter
/// conflict on the same `Messages<GameEvent>` resource).
#[derive(Resource, Default)]
pub struct PendingCreditEvents {
    pub events: Vec<(u32, Vec2, f64)>, // (amount, position, game_time)
}

/// Player's current material inventory.
#[derive(Resource, Default, Debug, Clone)]
pub struct PlayerInventory {
    pub items: HashMap<MaterialType, u32>,
}

/// Buffer for pending material drop spawns — avoids spawning inside a message-reader system.
#[derive(Resource, Default)]
pub struct PendingDropSpawns {
    pub drops: Vec<(MaterialType, Vec2)>,
}

/// Buffer for pending pickup events — avoids MessageReader+MessageWriter conflict (B0002).
#[derive(Resource, Default)]
pub struct PendingPickupEvents {
    pub events: Vec<(MaterialType, Vec2, f64)>, // (material, position, game_time)
}

/// Pure function for testability — decides what material (if any) drops from a destroyed enemy.
/// Takes a pre-rolled f32 in [0.0, 1.0) to allow deterministic testing.
/// Asteroid: 80% CommonScrap. Drone: 60% CommonScrap, 30% RareAlloy, 10% EnergyCore.
pub fn decide_material_drop(entity_type: &str, roll: f32) -> Option<MaterialType> {
    match entity_type {
        "asteroid" => {
            if roll < 0.8 {
                Some(MaterialType::CommonScrap)
            } else {
                None
            }
        }
        "drone" => {
            if roll < 0.6 {
                Some(MaterialType::CommonScrap)
            } else if roll < 0.9 {
                Some(MaterialType::RareAlloy)
            } else {
                Some(MaterialType::EnergyCore)
            }
        }
        _ => None,
    }
}

/// Awards credits when an enemy is destroyed.
/// +2 for asteroids, +10 for drones.
/// Pushes to `PendingCreditEvents`; actual GameEvent emission happens in `emit_credit_events`.
pub fn award_credits_on_kill(
    mut reader: MessageReader<GameEvent>,
    mut credits: ResMut<Credits>,
    mut pending: ResMut<PendingCreditEvents>,
) {
    for event in reader.read() {
        if let GameEventKind::EnemyDestroyed { entity_type } = &event.kind {
            let amount = match *entity_type {
                "drone" => 10u32,
                _ => 2u32,
            };
            credits.balance += amount;
            pending.events.push((amount, event.position, event.game_time));
        }
    }
}

/// Awards credits when a chunk is discovered for the first time.
/// +5 per new chunk. Uses `DiscoveredChunks` to prevent double-awarding on re-entry.
/// Pushes to `PendingCreditEvents`; actual GameEvent emission happens in `emit_credit_events`.
pub fn award_credits_on_discovery(
    mut reader: MessageReader<GameEvent>,
    mut discovered: ResMut<DiscoveredChunks>,
    mut credits: ResMut<Credits>,
    mut pending: ResMut<PendingCreditEvents>,
) {
    for event in reader.read() {
        if let GameEventKind::ChunkLoaded { coord, .. } = &event.kind {
            if discovered.chunks.insert(*coord) {
                credits.balance += 5;
                pending.events.push((5, event.position, event.game_time));
            }
        }
    }
}

/// Drains `PendingCreditEvents` and emits `GameEventKind::CreditsEarned` messages.
/// Split from the award systems to avoid `MessageReader` + `MessageWriter` conflict (B0002).
pub fn emit_credit_events(
    mut pending: ResMut<PendingCreditEvents>,
    mut game_events: MessageWriter<GameEvent>,
    severity_config: Res<EventSeverityConfig>,
) {
    for (amount, position, game_time) in pending.events.drain(..) {
        let kind = GameEventKind::CreditsEarned { amount };
        game_events.write(GameEvent {
            severity: severity_config.severity_for(&kind),
            kind,
            position,
            game_time,
        });
    }
}

/// Deducts 10% of Credits on PlayerDeath (floor integer division).
/// Materials (PlayerInventory) are unaffected.
/// No pending buffer needed — only writes Credits, no GameEvent emitted.
pub fn on_player_death_deduct_credits(
    mut reader: MessageReader<GameEvent>,
    mut credits: ResMut<Credits>,
) {
    for event in reader.read() {
        if matches!(event.kind, GameEventKind::PlayerDeath) {
            credits.balance -= credits.balance / 10;
        }
    }
}

/// Reads EnemyDestroyed events and queues material drops in PendingDropSpawns.
/// Uses rand::random for the probability roll. Does NOT spawn entities (no Commands).
pub fn queue_material_drops(
    mut reader: MessageReader<GameEvent>,
    mut pending: ResMut<PendingDropSpawns>,
) {
    for event in reader.read() {
        if let GameEventKind::EnemyDestroyed { entity_type } = &event.kind {
            let roll = rand::random::<f32>();
            if let Some(material) = decide_material_drop(entity_type, roll) {
                pending.drops.push((material, event.position));
            }
        }
    }
}

/// Drains PendingDropSpawns and spawns MaterialDrop entities into the world.
pub fn spawn_material_drops(
    mut commands: Commands,
    mut pending: ResMut<PendingDropSpawns>,
) {
    for (material, position) in pending.drops.drain(..) {
        commands.spawn((
            MaterialDrop,
            material,
            Transform::from_translation(position.extend(0.0)),
            Collider { radius: 15.0 },
            NeedsMaterialDropVisual,
        ));
    }
}

/// Detects player collisions with MaterialDrop entities, updates inventory, queues pickup events.
/// Does NOT write GameEvents directly — avoids B0002. Uses PendingPickupEvents buffer.
pub fn collect_material_drops(
    mut commands: Commands,
    player_query: Query<(&Transform, &Collider), With<Player>>,
    drop_query: Query<(Entity, &Transform, &MaterialType, &Collider), With<MaterialDrop>>,
    mut inventory: ResMut<PlayerInventory>,
    mut pending: ResMut<PendingPickupEvents>,
    time: Res<Time>,
) {
    let Ok((player_transform, player_collider)) = player_query.single() else {
        return;
    };
    let player_pos = player_transform.translation.truncate();

    for (drop_entity, drop_transform, material_type, drop_collider) in drop_query.iter() {
        let drop_pos = drop_transform.translation.truncate();
        let pickup_radius = player_collider.radius + drop_collider.radius;
        if player_pos.distance(drop_pos) <= pickup_radius {
            *inventory.items.entry(*material_type).or_insert(0) += 1;
            commands.entity(drop_entity).despawn();
            pending.events.push((*material_type, drop_pos, time.elapsed_secs_f64()));
        }
    }
}

/// Drains PendingPickupEvents and emits MaterialCollected GameEvents.
pub fn emit_pickup_events(
    mut pending: ResMut<PendingPickupEvents>,
    mut game_events: MessageWriter<GameEvent>,
    severity_config: Res<EventSeverityConfig>,
) {
    for (material, position, game_time) in pending.events.drain(..) {
        let kind = GameEventKind::MaterialCollected { material };
        game_events.write(GameEvent {
            severity: severity_config.severity_for(&kind),
            kind,
            position,
            game_time,
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn credits_default_balance_is_zero() {
        let credits = Credits::default();
        assert_eq!(credits.balance, 0);
    }

    #[test]
    fn discovered_chunks_default_is_empty() {
        let discovered = DiscoveredChunks::default();
        assert!(discovered.chunks.is_empty());
    }

    #[test]
    fn credits_can_be_incremented() {
        let mut credits = Credits::default();
        credits.balance += 10;
        assert_eq!(credits.balance, 10);
    }

    #[test]
    fn pending_credit_events_default_is_empty() {
        let pending = PendingCreditEvents::default();
        assert!(pending.events.is_empty());
    }

    #[test]
    fn decide_material_drop_asteroid_below_threshold_gives_scrap() {
        let result = decide_material_drop("asteroid", 0.0);
        assert_eq!(result, Some(MaterialType::CommonScrap));
        let result = decide_material_drop("asteroid", 0.79);
        assert_eq!(result, Some(MaterialType::CommonScrap));
    }

    #[test]
    fn decide_material_drop_asteroid_above_threshold_gives_none() {
        let result = decide_material_drop("asteroid", 0.8);
        assert_eq!(result, None);
        let result = decide_material_drop("asteroid", 0.99);
        assert_eq!(result, None);
    }

    #[test]
    fn decide_material_drop_drone_scrap_range() {
        let result = decide_material_drop("drone", 0.0);
        assert_eq!(result, Some(MaterialType::CommonScrap));
        let result = decide_material_drop("drone", 0.59);
        assert_eq!(result, Some(MaterialType::CommonScrap));
    }

    #[test]
    fn decide_material_drop_drone_alloy_range() {
        let result = decide_material_drop("drone", 0.6);
        assert_eq!(result, Some(MaterialType::RareAlloy));
        let result = decide_material_drop("drone", 0.89);
        assert_eq!(result, Some(MaterialType::RareAlloy));
    }

    #[test]
    fn decide_material_drop_drone_energy_core_range() {
        let result = decide_material_drop("drone", 0.9);
        assert_eq!(result, Some(MaterialType::EnergyCore));
        let result = decide_material_drop("drone", 0.99);
        assert_eq!(result, Some(MaterialType::EnergyCore));
    }

    #[test]
    fn decide_material_drop_unknown_entity_gives_none() {
        let result = decide_material_drop("boss", 0.0);
        assert_eq!(result, None);
    }

    #[test]
    fn player_inventory_default_is_empty() {
        let inv = PlayerInventory::default();
        assert!(inv.items.is_empty());
    }
}
