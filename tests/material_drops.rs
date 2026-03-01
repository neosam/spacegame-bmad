#![deny(clippy::unwrap_used)]
/// Integration tests for Story 3-4: Material Drops
///
/// Tests cover: decide_material_drop logic, pickup adds to inventory,
/// no double-pickup, save/load roundtrip for inventory.
mod helpers;

use bevy::prelude::*;
use void_drifter::core::collision::Collider;
use void_drifter::core::economy::{
    collect_material_drops, decide_material_drop, emit_pickup_events, PlayerInventory,
    PendingPickupEvents,
};
use void_drifter::core::flight::Player;
use void_drifter::infrastructure::events::EventSeverityConfig;
use void_drifter::shared::components::{MaterialDrop, MaterialType};
use void_drifter::shared::events::GameEvent;

// ── Test helpers ───────────────────────────────────────────────────────────

fn pickup_test_app() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_message::<GameEvent>();
    app.init_resource::<PlayerInventory>();
    app.init_resource::<PendingPickupEvents>();
    app.init_resource::<EventSeverityConfig>();
    app.add_systems(Update, (collect_material_drops, emit_pickup_events).chain());
    app.update(); // prime
    app
}

fn spawn_player_at(app: &mut App, pos: Vec2) -> Entity {
    app.world_mut()
        .spawn((
            Player,
            Transform::from_translation(pos.extend(0.0)),
            Collider { radius: 20.0 },
        ))
        .id()
}

fn spawn_drop_at(app: &mut App, material: MaterialType, pos: Vec2) -> Entity {
    app.world_mut()
        .spawn((
            MaterialDrop,
            material,
            Transform::from_translation(pos.extend(0.0)),
            Collider { radius: 15.0 },
        ))
        .id()
}

// ── decide_material_drop tests ─────────────────────────────────────────────

#[test]
fn decide_material_drop_asteroid_low_roll_gives_scrap() {
    let result = decide_material_drop("asteroid", 0.0);
    assert_eq!(result, Some(MaterialType::CommonScrap), "Roll 0.0 → CommonScrap");
    let result = decide_material_drop("asteroid", 0.79);
    assert_eq!(result, Some(MaterialType::CommonScrap), "Roll 0.79 → CommonScrap");
}

#[test]
fn decide_material_drop_asteroid_high_roll_gives_none() {
    let result = decide_material_drop("asteroid", 0.8);
    assert_eq!(result, None, "Roll 0.8 → no drop");
    let result = decide_material_drop("asteroid", 0.99);
    assert_eq!(result, None, "Roll 0.99 → no drop");
}

#[test]
fn decide_material_drop_drone_full_range() {
    // Scrap range: [0.0, 0.6)
    assert_eq!(decide_material_drop("drone", 0.0), Some(MaterialType::CommonScrap));
    assert_eq!(decide_material_drop("drone", 0.59), Some(MaterialType::CommonScrap));
    // RareAlloy range: [0.6, 0.9)
    assert_eq!(decide_material_drop("drone", 0.6), Some(MaterialType::RareAlloy));
    assert_eq!(decide_material_drop("drone", 0.89), Some(MaterialType::RareAlloy));
    // EnergyCore range: [0.9, 1.0)
    assert_eq!(decide_material_drop("drone", 0.9), Some(MaterialType::EnergyCore));
    assert_eq!(decide_material_drop("drone", 0.99), Some(MaterialType::EnergyCore));
}

#[test]
fn decide_material_drop_unknown_entity_gives_none() {
    assert_eq!(decide_material_drop("turret", 0.0), None);
}

// ── Pickup integration tests ───────────────────────────────────────────────

#[test]
fn pickup_adds_to_inventory() {
    let mut app = pickup_test_app();

    spawn_player_at(&mut app, Vec2::ZERO);
    spawn_drop_at(&mut app, MaterialType::RareAlloy, Vec2::ZERO);

    app.update();

    let inv = app.world().resource::<PlayerInventory>();
    assert_eq!(
        inv.items.get(&MaterialType::RareAlloy).copied().unwrap_or(0),
        1,
        "RareAlloy count should be 1 after pickup"
    );
}

#[test]
fn no_double_pickup() {
    let mut app = pickup_test_app();

    spawn_player_at(&mut app, Vec2::ZERO);
    spawn_drop_at(&mut app, MaterialType::CommonScrap, Vec2::ZERO);

    // First update: drop picked up, despawned
    app.update();

    let inv_after_first = app
        .world()
        .resource::<PlayerInventory>()
        .items
        .get(&MaterialType::CommonScrap)
        .copied()
        .unwrap_or(0);
    assert_eq!(inv_after_first, 1, "First pickup should give 1 scrap");

    // Second update: drop is gone, count should not increase
    app.update();

    let inv_after_second = app
        .world()
        .resource::<PlayerInventory>()
        .items
        .get(&MaterialType::CommonScrap)
        .copied()
        .unwrap_or(0);
    assert_eq!(inv_after_second, 1, "No double-pickup: count should still be 1");
}

#[test]
fn drop_out_of_range_not_collected() {
    let mut app = pickup_test_app();

    // Player at origin, drop far away (radius 20+15=35, drop at distance 100)
    spawn_player_at(&mut app, Vec2::ZERO);
    spawn_drop_at(&mut app, MaterialType::EnergyCore, Vec2::new(100.0, 0.0));

    app.update();

    let inv = app.world().resource::<PlayerInventory>();
    assert_eq!(
        inv.items.get(&MaterialType::EnergyCore).copied().unwrap_or(0),
        0,
        "Drop out of range should not be collected"
    );
}

// ── Save/load roundtrip ────────────────────────────────────────────────────

#[test]
fn inventory_save_load_roundtrip() {
    use void_drifter::infrastructure::save::player_save::PlayerSave;
    use void_drifter::infrastructure::save::schema::SAVE_VERSION;

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
        credits: 0,
        inventory_common_scrap: 5,
        inventory_rare_alloy: 3,
        inventory_energy_core: 1,
    };

    let ron_str = save.to_ron().expect("Should serialize inventory save");
    let restored = PlayerSave::from_ron(&ron_str).expect("Should deserialize inventory save");

    assert_eq!(restored.inventory_common_scrap, 5, "CommonScrap should survive roundtrip");
    assert_eq!(restored.inventory_rare_alloy, 3, "RareAlloy should survive roundtrip");
    assert_eq!(restored.inventory_energy_core, 1, "EnergyCore should survive roundtrip");
}

#[test]
fn v3_save_missing_inventory_defaults_to_zero() {
    use void_drifter::infrastructure::save::player_save::PlayerSave;

    // Simulate a v3 save without inventory fields
    let ron_str = r#"(
        schema_version: 3,
        position: (10.0, 20.0),
        rotation: 0.5,
        velocity: (0.0, 0.0),
        health_current: 90.0,
        health_max: 100.0,
        active_weapon: "Laser",
        energy_current: 80.0,
        energy_max: 100.0,
        credits: 42,
    )"#;

    let restored = PlayerSave::from_ron(ron_str).expect("Should deserialize v3 save");
    assert_eq!(restored.inventory_common_scrap, 0, "Missing field defaults to 0");
    assert_eq!(restored.inventory_rare_alloy, 0, "Missing field defaults to 0");
    assert_eq!(restored.inventory_energy_core, 0, "Missing field defaults to 0");
    assert_eq!(restored.credits, 42, "Credits should be preserved");
}
