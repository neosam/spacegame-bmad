#![deny(clippy::unwrap_used)]
/// Integration tests for Story 3-3: Earn Credits
///
/// Tests cover: credits awarded on asteroid/drone kill, credits awarded on
/// first chunk discovery, no double-award on same chunk, save/load roundtrip.
mod helpers;

use bevy::ecs::message::MessageWriter;
use bevy::ecs::system::RunSystemOnce;
use bevy::prelude::*;
use void_drifter::core::economy::{award_credits_on_discovery, award_credits_on_kill, emit_credit_events, Credits, DiscoveredChunks, PendingCreditEvents};
use void_drifter::infrastructure::events::EventSeverityConfig;
use void_drifter::shared::events::{GameEvent, GameEventKind};
use void_drifter::world::{BiomeType, ChunkCoord};

// ── Test helpers ──────────────────────────────────────────────────────────

fn credits_test_app() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_message::<GameEvent>();
    app.init_resource::<Credits>();
    app.init_resource::<DiscoveredChunks>();
    app.init_resource::<PendingCreditEvents>();
    app.init_resource::<EventSeverityConfig>();
    app.add_systems(
        Update,
        (award_credits_on_kill, award_credits_on_discovery, emit_credit_events).chain(),
    );
    // Prime first frame
    app.update();
    app
}

/// Writes a GameEvent with EnemyDestroyed kind via a one-shot system.
fn write_enemy_destroyed(app: &mut App, entity_type: &'static str) {
    app.world_mut()
        .run_system_once(
            move |mut w: MessageWriter<GameEvent>, config: Res<EventSeverityConfig>| {
                let kind = GameEventKind::EnemyDestroyed { entity_type };
                w.write(GameEvent {
                    severity: config.severity_for(&kind),
                    kind,
                    position: Vec2::ZERO,
                    game_time: 0.0,
                });
            },
        )
        .expect("Should run enemy destroyed writer");
}

/// Writes a GameEvent with ChunkLoaded kind via a one-shot system.
fn write_chunk_loaded(app: &mut App, coord: ChunkCoord) {
    app.world_mut()
        .run_system_once(
            move |mut w: MessageWriter<GameEvent>, config: Res<EventSeverityConfig>| {
                let kind = GameEventKind::ChunkLoaded { coord, biome: BiomeType::DeepSpace };
                w.write(GameEvent {
                    severity: config.severity_for(&kind),
                    kind,
                    position: Vec2::ZERO,
                    game_time: 0.0,
                });
            },
        )
        .expect("Should run chunk loaded writer");
}

// ── Tests ─────────────────────────────────────────────────────────────────

#[test]
fn asteroid_kill_awards_two_credits() {
    let mut app = credits_test_app();

    write_enemy_destroyed(&mut app, "asteroid");
    app.update();

    let credits = app.world().resource::<Credits>();
    assert_eq!(credits.balance, 2, "Asteroid kill should award 2 credits");
}

#[test]
fn drone_kill_awards_ten_credits() {
    let mut app = credits_test_app();

    write_enemy_destroyed(&mut app, "drone");
    app.update();

    let credits = app.world().resource::<Credits>();
    assert_eq!(credits.balance, 10, "Drone kill should award 10 credits");
}

#[test]
fn chunk_discovery_awards_five_credits() {
    let mut app = credits_test_app();

    write_chunk_loaded(&mut app, ChunkCoord { x: 1, y: 2 });
    app.update();

    let credits = app.world().resource::<Credits>();
    assert_eq!(credits.balance, 5, "First chunk discovery should award 5 credits");
}

#[test]
fn same_chunk_not_awarded_twice() {
    let mut app = credits_test_app();
    let coord = ChunkCoord { x: 3, y: 4 };

    // First discovery
    write_chunk_loaded(&mut app, coord);
    app.update();

    // Same chunk re-enters (e.g., player leaves and returns)
    write_chunk_loaded(&mut app, coord);
    app.update();

    let credits = app.world().resource::<Credits>();
    assert_eq!(
        credits.balance, 5,
        "Same chunk should only award credits once (expected 5, not 10)"
    );
}

#[test]
fn credits_save_load_roundtrip() {
    use void_drifter::infrastructure::save::player_save::PlayerSave;
    use void_drifter::infrastructure::save::schema::SAVE_VERSION;

    // Build a PlayerSave with 42 credits
    let save = PlayerSave {
        schema_version: SAVE_VERSION,
        position: (0.0, 0.0),
        rotation: 0.0,
        velocity: (0.0, 0.0),
        health_current: 100.0,
        health_max: 100.0,
        active_weapon: "Laser".to_string(),
        energy_current: 100.0,
        energy_max: 100.0,
        credits: 42,
        inventory_common_scrap: 0,
        inventory_rare_alloy: 0,
        inventory_energy_core: 0,
    };

    // Serialize and deserialize
    let ron_str = save.to_ron().expect("Should serialize");
    let restored = PlayerSave::from_ron(&ron_str).expect("Should deserialize");

    assert_eq!(restored.credits, 42, "Credits should survive save/load roundtrip");
}
