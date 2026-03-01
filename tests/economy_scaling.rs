#![deny(clippy::unwrap_used)]
/// Integration tests for Story 3-7: Economy Scaling
///
/// Tests cover: distance_tier pure function, scale_credits pure function,
/// tiered drone drop tables, scaled kill awards.

use bevy::ecs::message::MessageWriter;
use bevy::ecs::system::RunSystemOnce;
use bevy::prelude::*;
use void_drifter::core::economy::{
    award_credits_on_kill, decide_material_drop, distance_tier, emit_credit_events, scale_credits,
    Credits, PendingCreditEvents,
};
use void_drifter::infrastructure::events::EventSeverityConfig;
use void_drifter::shared::components::MaterialType;
use void_drifter::shared::events::{GameEvent, GameEventKind};
use void_drifter::world::{ChunkCoord, WorldConfig};

// ── distance_tier tests ────────────────────────────────────────────────────

#[test]
fn distance_tier_origin() {
    assert_eq!(distance_tier(ChunkCoord { x: 0, y: 0 }), 0);
}

#[test]
fn distance_tier_at_5() {
    assert_eq!(distance_tier(ChunkCoord { x: 5, y: 0 }), 1, "(5,0) → tier 1");
    assert_eq!(distance_tier(ChunkCoord { x: 25, y: 0 }), 5, "(25,0) → tier 5");
}

#[test]
fn distance_tier_capped_at_5() {
    assert_eq!(distance_tier(ChunkCoord { x: 100, y: 100 }), 5, "(100,100) → capped at 5");
    assert_eq!(distance_tier(ChunkCoord { x: -50, y: -50 }), 5, "(-50,-50) → capped at 5");
}

#[test]
fn distance_tier_chebyshev() {
    // max(3,7) = 7, 7/5 = 1 → tier 1
    assert_eq!(distance_tier(ChunkCoord { x: 3, y: 7 }), 1, "(3,7) → tier 1");
    // max(12,4) = 12, 12/5 = 2 → tier 2
    assert_eq!(distance_tier(ChunkCoord { x: -12, y: 4 }), 2, "(-12,4) → tier 2");
}

// ── scale_credits tests ────────────────────────────────────────────────────

#[test]
fn scale_credits_tier_0() {
    assert_eq!(scale_credits(2, 0), 2, "Asteroid base at tier 0");
    assert_eq!(scale_credits(10, 0), 10, "Drone base at tier 0");
}

#[test]
fn scale_credits_tier_5() {
    assert_eq!(scale_credits(2, 5), 3, "2 * 15 / 10 = 3");
    assert_eq!(scale_credits(10, 5), 15, "10 * 15 / 10 = 15");
}

#[test]
fn scale_credits_tier_3() {
    assert_eq!(scale_credits(10, 3), 13, "10 * 13 / 10 = 13");
}

// ── tiered drone drop tests ────────────────────────────────────────────────

#[test]
fn decide_material_drop_drone_tier3_improved() {
    // Tier 0 base table: roll < 0.6 → Scrap, so 0.5 → Scrap.
    // Tier 3 mid table: roll < 0.40 → Scrap, roll < 0.85 → Alloy, so 0.5 → Alloy.
    // Demonstrates that the same roll yields a better drop at higher tier.
    let at_tier0 = decide_material_drop("drone", 0.5, 0);
    assert_eq!(at_tier0, Some(MaterialType::CommonScrap), "roll 0.5 tier 0 → Scrap");
    let at_tier3 = decide_material_drop("drone", 0.5, 3);
    assert_eq!(at_tier3, Some(MaterialType::RareAlloy), "roll 0.5 tier 3 → Alloy");
}

#[test]
fn decide_material_drop_drone_tier5_improved() {
    // tier 5 table: roll < 0.20 → Scrap, roll < 0.70 → Alloy, else Core
    let scrap = decide_material_drop("drone", 0.1, 5);
    assert_eq!(scrap, Some(MaterialType::CommonScrap), "roll 0.1 tier 5 → Scrap");
    let alloy = decide_material_drop("drone", 0.4, 5);
    assert_eq!(alloy, Some(MaterialType::RareAlloy), "roll 0.4 tier 5 → Alloy");
}

#[test]
fn distance_tier_negative_coordinates() {
    // Negative coords: unsigned_abs gives correct Chebyshev distance
    assert_eq!(distance_tier(ChunkCoord { x: -5, y: 0 }), 1, "(-5,0) → tier 1");
    assert_eq!(distance_tier(ChunkCoord { x: -25, y: -10 }), 5, "(-25,-10) → tier 5");
}

// ── scaled kill award integration tests ───────────────────────────────────

fn scaling_test_app() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_message::<GameEvent>();
    app.init_resource::<Credits>();
    app.init_resource::<PendingCreditEvents>();
    app.init_resource::<EventSeverityConfig>();
    app.insert_resource(WorldConfig::default()); // chunk_size=1000
    app.add_systems(Update, (award_credits_on_kill, emit_credit_events).chain());
    app.update(); // prime
    app
}

fn write_enemy_destroyed_at(app: &mut App, entity_type: &'static str, position: Vec2) {
    app.world_mut()
        .run_system_once(
            move |mut w: MessageWriter<GameEvent>, config: Res<EventSeverityConfig>| {
                let kind = GameEventKind::EnemyDestroyed { entity_type };
                w.write(GameEvent {
                    severity: config.severity_for(&kind),
                    kind,
                    position,
                    game_time: 0.0,
                });
            },
        )
        .expect("Should run enemy destroyed writer");
}

#[test]
fn scaled_kill_awards_tier0_asteroid() {
    // position Vec2::ZERO → chunk (0,0) → tier 0 → scale_credits(2, 0) = 2
    let mut app = scaling_test_app();
    write_enemy_destroyed_at(&mut app, "asteroid", Vec2::ZERO);
    app.update();
    let credits = app.world().resource::<Credits>();
    assert_eq!(credits.balance, 2, "Tier 0 asteroid should award 2 credits");
}

#[test]
fn scaled_kill_awards_tier0_drone() {
    // position Vec2::ZERO → tier 0 → scale_credits(10, 0) = 10
    let mut app = scaling_test_app();
    write_enemy_destroyed_at(&mut app, "drone", Vec2::ZERO);
    app.update();
    let credits = app.world().resource::<Credits>();
    assert_eq!(credits.balance, 10, "Tier 0 drone should award 10 credits");
}

#[test]
fn scaled_kill_awards_tier5_drone() {
    // chunk_size=1000, position at x=25000 → chunk x=25 → tier 5 → scale_credits(10, 5) = 15
    let mut app = scaling_test_app();
    write_enemy_destroyed_at(&mut app, "drone", Vec2::new(25000.0, 0.0));
    app.update();
    let credits = app.world().resource::<Credits>();
    assert_eq!(credits.balance, 15, "Tier 5 drone should award 15 credits");
}
