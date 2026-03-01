use std::collections::HashSet;

use bevy::ecs::message::{MessageReader, MessageWriter};
use bevy::prelude::*;

use crate::infrastructure::events::EventSeverityConfig;
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
}
