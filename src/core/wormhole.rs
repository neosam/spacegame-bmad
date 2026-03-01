use bevy::ecs::message::MessageWriter;
use bevy::prelude::*;

use crate::core::flight::Player;
use crate::core::input::ActionState;
use crate::game_states::PlayingSubState;
use crate::infrastructure::events::EventSeverityConfig;
use crate::shared::events::{EventSeverity, GameEvent, GameEventKind};
use crate::world::ChunkCoord;

/// Proximity radius: player must be within this many world units to enter a wormhole.
pub const WORMHOLE_ENTER_RADIUS: f32 = 60.0;

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

/// Proximity-System: Läuft in Update, nur wenn PlayingSubState::Flying.
/// Wenn Spieler innerhalb WORMHOLE_ENTER_RADIUS Einheiten eines nicht-gecleared Wormholes
/// und E drückt → State-Transition Flying → InWormhole.
/// Teleportiert den Spieler zum Arena-Zentrum (0,0) und speichert den Entry-Point.
pub fn check_wormhole_proximity(
    mut player_query: Query<&mut Transform, With<Player>>,
    wormhole_query: Query<(Entity, &Transform, &Wormhole), Without<Player>>,
    action_state: Res<ActionState>,
    mut next_sub_state: ResMut<NextState<PlayingSubState>>,
    mut commands: Commands,
    mut game_events: MessageWriter<GameEvent>,
    severity_config: Res<EventSeverityConfig>,
    time: Res<Time>,
) {
    let Ok(mut player_transform) = player_query.single_mut() else {
        return;
    };
    let player_pos = player_transform.translation.truncate();

    for (wormhole_entity, wormhole_transform, wormhole) in wormhole_query.iter() {
        if wormhole.cleared {
            continue;
        }
        let wormhole_pos = wormhole_transform.translation.truncate();
        let distance = player_pos.distance(wormhole_pos);

        if distance < WORMHOLE_ENTER_RADIUS && action_state.interact {
            // Store entry point so player can respawn here on death
            commands.insert_resource(WormholeEntrance {
                world_position: player_pos,
                wormhole_entity,
            });

            // Initialize arena state
            commands.insert_resource(ArenaState {
                wave: 0,
                total_waves: 3,
                enemies_remaining: 0,
                cleared: false,
            });

            // Teleport player to arena center
            player_transform.translation = Vec3::ZERO;

            // Transition to InWormhole sub-state
            next_sub_state.set(PlayingSubState::InWormhole);

            // Emit WormholeEntered event
            let kind = GameEventKind::WormholeEntered { coord: wormhole.coord };
            let severity = severity_config
                .mappings
                .get("WormholeEntered")
                .copied()
                .unwrap_or(EventSeverity::Tier1);
            game_events.write(GameEvent {
                kind,
                severity,
                position: player_pos,
                game_time: time.elapsed_secs_f64(),
            });

            // Only enter one wormhole per frame
            break;
        }
    }
}
