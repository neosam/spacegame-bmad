#![deny(clippy::unwrap_used)]
/// Integration tests for Story 7-4: Boss Loot
///
/// Tests cover: PendingDropSpawns queued on BossDestroyed event (3–5 drops),
/// and +500 Credits awarded to player on BossDestroyed event.

mod helpers;

use bevy::ecs::message::MessageWriter;
use bevy::ecs::system::RunSystemOnce;
use bevy::prelude::*;
use void_drifter::core::economy::{spawn_boss_loot, Credits, PendingDropSpawns};
use void_drifter::infrastructure::events::EventSeverityConfig;
use void_drifter::shared::events::{EventSeverity, GameEvent, GameEventKind};
use void_drifter::social::faction::FactionId;

// ── Test helpers ────────────────────────────────────────────────────────────

fn boss_loot_test_app() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_message::<GameEvent>();
    app.init_resource::<Credits>();
    app.init_resource::<PendingDropSpawns>();
    app.insert_resource(EventSeverityConfig::default());
    app.add_systems(Update, spawn_boss_loot);
    app.update(); // prime
    app
}

fn write_boss_destroyed(app: &mut App, faction: FactionId, position: Vec2) {
    app.world_mut()
        .run_system_once(
            move |mut w: MessageWriter<GameEvent>| {
                let kind = GameEventKind::BossDestroyed {
                    faction: faction.clone(),
                    position,
                };
                w.write(GameEvent {
                    severity: EventSeverity::Tier1,
                    kind,
                    position,
                    game_time: 0.0,
                });
            },
        )
        .expect("Should run boss destroyed writer");
}

// ── Tests ───────────────────────────────────────────────────────────────────

#[test]
fn boss_loot_queues_drops_on_boss_destroyed() {
    let mut app = boss_loot_test_app();

    write_boss_destroyed(&mut app, FactionId::Pirates, Vec2::ZERO);
    app.update();

    let pending = app.world().resource::<PendingDropSpawns>();
    let drop_count = pending.drops.len();
    assert!(
        (3..=5).contains(&drop_count),
        "Expected 3–5 drops after BossDestroyed, got {drop_count}"
    );
}

#[test]
fn boss_loot_adds_credits_on_boss_destroyed() {
    let mut app = boss_loot_test_app();

    write_boss_destroyed(&mut app, FactionId::Military, Vec2::new(100.0, 200.0));
    app.update();

    let credits = app.world().resource::<Credits>();
    assert_eq!(
        credits.balance, 500,
        "BossDestroyed should award +500 credits, got {}",
        credits.balance
    );
}

#[test]
fn boss_loot_drops_are_near_boss_position() {
    let mut app = boss_loot_test_app();

    let boss_pos = Vec2::new(50.0, 75.0);
    write_boss_destroyed(&mut app, FactionId::Aliens, boss_pos);
    app.update();

    let pending = app.world().resource::<PendingDropSpawns>();
    for (_, drop_pos) in &pending.drops {
        let dist = (*drop_pos - boss_pos).length();
        assert!(
            dist <= 30.0 * std::f32::consts::SQRT_2,
            "Drop at {drop_pos:?} is too far from boss position {boss_pos:?} (distance {dist})"
        );
    }
}

#[test]
fn boss_loot_no_drops_without_event() {
    let mut app = boss_loot_test_app();

    // No event written — run two frames
    app.update();
    app.update();

    let pending = app.world().resource::<PendingDropSpawns>();
    assert!(
        pending.drops.is_empty(),
        "No drops should be queued without a BossDestroyed event"
    );
}
