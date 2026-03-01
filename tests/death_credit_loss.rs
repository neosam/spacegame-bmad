#![deny(clippy::unwrap_used)]
/// Integration tests for Story 3-5: Death Credit Loss
///
/// Tests cover: 10% deduction on PlayerDeath, zero balance stays zero,
/// floor division for small balances, materials unaffected (AC4).

use bevy::ecs::message::MessageWriter;
use bevy::ecs::system::RunSystemOnce;
use bevy::prelude::*;
use void_drifter::core::economy::{on_player_death_deduct_credits, Credits, PlayerInventory};
use void_drifter::infrastructure::events::EventSeverityConfig;
use void_drifter::shared::events::{GameEvent, GameEventKind};
use void_drifter::shared::components::MaterialType;

// ── Test helpers ───────────────────────────────────────────────────────────

fn death_test_app() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_message::<GameEvent>();
    app.init_resource::<Credits>();
    app.init_resource::<PlayerInventory>();
    app.init_resource::<EventSeverityConfig>();
    app.add_systems(Update, on_player_death_deduct_credits);
    app.update(); // prime first frame
    app
}

fn write_player_death(app: &mut App) {
    app.world_mut()
        .run_system_once(
            |mut w: MessageWriter<GameEvent>, config: Res<EventSeverityConfig>| {
                let kind = GameEventKind::PlayerDeath;
                w.write(GameEvent {
                    severity: config.severity_for(&kind),
                    kind,
                    position: Vec2::ZERO,
                    game_time: 0.0,
                });
            },
        )
        .expect("Should run player death writer");
}

// ── Tests ──────────────────────────────────────────────────────────────────

#[test]
fn player_death_deducts_ten_percent() {
    let mut app = death_test_app();
    app.world_mut().resource_mut::<Credits>().balance = 100;

    write_player_death(&mut app);
    app.update();

    let credits = app.world().resource::<Credits>();
    assert_eq!(credits.balance, 90, "100 credits - 10% = 90");
}

#[test]
fn player_death_with_zero_credits_stays_zero() {
    let mut app = death_test_app();
    // Credits start at 0 (default)

    write_player_death(&mut app);
    app.update();

    let credits = app.world().resource::<Credits>();
    assert_eq!(credits.balance, 0, "0 credits - 10% = 0 (no underflow)");
}

#[test]
fn player_death_nine_credits_loses_zero() {
    let mut app = death_test_app();
    app.world_mut().resource_mut::<Credits>().balance = 9;

    write_player_death(&mut app);
    app.update();

    let credits = app.world().resource::<Credits>();
    assert_eq!(credits.balance, 9, "9 credits - floor(9/10)=0, stays 9");
}

#[test]
fn player_death_ten_credits_loses_one() {
    let mut app = death_test_app();
    app.world_mut().resource_mut::<Credits>().balance = 10;

    write_player_death(&mut app);
    app.update();

    let credits = app.world().resource::<Credits>();
    assert_eq!(credits.balance, 9, "10 credits - floor(10/10)=1 = 9");
}

#[test]
fn player_death_does_not_affect_inventory() {
    // AC4: PlayerInventory (materials) is unaffected by PlayerDeath
    let mut app = death_test_app();
    app.world_mut().resource_mut::<Credits>().balance = 50;

    // Pre-load inventory with some materials
    {
        let mut inv = app.world_mut().resource_mut::<PlayerInventory>();
        inv.items.insert(MaterialType::CommonScrap, 3);
        inv.items.insert(MaterialType::RareAlloy, 1);
    }

    write_player_death(&mut app);
    app.update();

    let inv = app.world().resource::<PlayerInventory>();
    assert_eq!(
        inv.items.get(&MaterialType::CommonScrap).copied().unwrap_or(0),
        3,
        "CommonScrap should be unchanged after death"
    );
    assert_eq!(
        inv.items.get(&MaterialType::RareAlloy).copied().unwrap_or(0),
        1,
        "RareAlloy should be unchanged after death"
    );
    // Credits should still be deducted
    let credits = app.world().resource::<Credits>();
    assert_eq!(credits.balance, 45, "50 credits - 10% = 45");
}
