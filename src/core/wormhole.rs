use bevy::ecs::message::MessageWriter;
use bevy::prelude::*;

use crate::core::collision::{Collider, Health};
use crate::core::flight::Player;
use crate::core::input::ActionState;
use crate::core::spawning::{
    Fighter, HeavyCruiser, NeedsFighterVisual, NeedsHeavyCruiserVisual, NeedsSniperVisual,
    ScoutDrone, NeedsDroneVisual, Sniper, SpawningConfig,
};
use crate::game_states::PlayingSubState;
use crate::infrastructure::events::EventSeverityConfig;
use crate::shared::components::Velocity;
use crate::shared::events::{EventSeverity, GameEvent, GameEventKind};
use crate::world::ChunkCoord;

/// Proximity radius: player must be within this many world units to enter a wormhole.
pub const WORMHOLE_ENTER_RADIUS: f32 = 60.0;

/// Arena boundary radius: player cannot leave this circle.
pub const ARENA_RADIUS: f32 = 800.0;

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

/// Marker für Arena-Feinde — werden beim State-Exit despawnt.
#[derive(Component)]
pub struct ArenaEnemy;

/// Arena-Boundary — unsichtbare Kreisgrenze.
#[derive(Component)]
pub struct ArenaBoundary;

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

/// OnEnter(PlayingSubState::InWormhole) — setzt up die Arena.
/// Setzt arena_state.wave = 0, total_waves = 3, enemies_remaining = 0.
pub fn setup_arena(mut arena_state: ResMut<ArenaState>) {
    arena_state.wave = 0;
    arena_state.total_waves = 3;
    arena_state.enemies_remaining = 0;
    arena_state.cleared = false;
}

/// FixedUpdate, nur in InWormhole — spawnt nächste Welle wenn alle Feinde besiegt.
pub fn spawn_arena_wave(
    mut arena_state: ResMut<ArenaState>,
    arena_enemies: Query<Entity, With<ArenaEnemy>>,
    mut commands: Commands,
    config: Res<SpawningConfig>,
) {
    // Count live arena enemies
    let live_count = arena_enemies.iter().count() as u32;

    // Sync enemies_remaining with actual count (in case of desyncs)
    arena_state.enemies_remaining = live_count;

    // Don't spawn if arena is cleared or all waves done
    if arena_state.cleared {
        return;
    }

    // Only spawn next wave when all current enemies are defeated
    if live_count > 0 {
        return;
    }

    // Check if there are more waves to spawn
    if arena_state.wave >= arena_state.total_waves {
        arena_state.cleared = true;
        return;
    }

    // Advance to the next wave
    arena_state.wave += 1;
    let wave = arena_state.wave;

    // Spawn positions: distribute enemies in a circle around the origin
    let spawn_enemies: Vec<(Vec2, u8)> = match wave {
        1 => {
            // Wave 1: 3 Scout Drones (light)
            (0..3u8).map(|i| {
                let angle = (i as f32 / 3.0) * std::f32::consts::TAU;
                let pos = Vec2::new(angle.cos() * 350.0, angle.sin() * 350.0);
                (pos, 0) // 0 = ScoutDrone
            }).collect()
        }
        2 => {
            // Wave 2: 5 Fighters (medium)
            (0..5u8).map(|i| {
                let angle = (i as f32 / 5.0) * std::f32::consts::TAU;
                let pos = Vec2::new(angle.cos() * 400.0, angle.sin() * 400.0);
                (pos, 1) // 1 = Fighter
            }).collect()
        }
        3 => {
            // Wave 3: 3 Heavy Cruisers (heavy)
            (0..3u8).map(|i| {
                let angle = (i as f32 / 3.0) * std::f32::consts::TAU;
                let pos = Vec2::new(angle.cos() * 450.0, angle.sin() * 450.0);
                (pos, 2) // 2 = HeavyCruiser
            }).collect()
        }
        _ => vec![],
    };

    let enemy_count = spawn_enemies.len() as u32;

    for (pos, enemy_type) in spawn_enemies {
        match enemy_type {
            0 => {
                // ScoutDrone
                commands.spawn((
                    ScoutDrone,
                    NeedsDroneVisual,
                    ArenaEnemy,
                    Collider { radius: config.drone_radius },
                    Health { current: config.drone_health, max: config.drone_health },
                    Velocity(Vec2::ZERO),
                    Transform::from_translation(pos.extend(0.0)),
                ));
            }
            1 => {
                // Fighter
                commands.spawn((
                    Fighter,
                    NeedsFighterVisual,
                    ArenaEnemy,
                    Collider { radius: config.fighter_radius },
                    Health { current: config.fighter_health, max: config.fighter_health },
                    Velocity(Vec2::ZERO),
                    Transform::from_translation(pos.extend(0.0)),
                ));
            }
            2 => {
                // HeavyCruiser
                commands.spawn((
                    HeavyCruiser,
                    NeedsHeavyCruiserVisual,
                    ArenaEnemy,
                    Collider { radius: config.heavy_cruiser_radius },
                    Health { current: config.heavy_cruiser_health, max: config.heavy_cruiser_health },
                    Velocity(Vec2::ZERO),
                    Transform::from_translation(pos.extend(0.0)),
                ));
            }
            _ => {}
        }
    }

    arena_state.enemies_remaining = enemy_count;
}

/// FixedUpdate, nur in InWormhole — verhindert dass Spieler die Arena verlässt.
/// Wenn player_position.length() > ARENA_RADIUS:
/// - Position auf Rand clampen
/// - Velocity auf null setzen
pub fn enforce_arena_boundary(
    mut player_query: Query<(&mut Transform, &mut Velocity), With<Player>>,
) {
    let Ok((mut transform, mut velocity)) = player_query.single_mut() else {
        return;
    };

    let pos2 = transform.translation.truncate();
    let dist = pos2.length();

    if dist > ARENA_RADIUS {
        let clamped = pos2.normalize() * ARENA_RADIUS;
        transform.translation.x = clamped.x;
        transform.translation.y = clamped.y;
        velocity.0 = Vec2::ZERO;
    }
}

/// OnExit(PlayingSubState::InWormhole) — räumt Arena-Entities auf.
pub fn cleanup_arena(
    arena_enemies: Query<Entity, With<ArenaEnemy>>,
    mut commands: Commands,
) {
    for entity in arena_enemies.iter() {
        if let Ok(mut e) = commands.get_entity(entity) {
            e.despawn();
        }
    }
}
