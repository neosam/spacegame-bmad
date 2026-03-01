#![deny(clippy::unwrap_used)]
/// Integration tests for Story 3-6: Station Types
///
/// Tests cover: StationType display names, station entity with type component.

use bevy::prelude::*;
use void_drifter::core::station::{Station, StationType};

// ── Tests ──────────────────────────────────────────────────────────────────

#[test]
fn display_name_trading_post() {
    assert_eq!(StationType::TradingPost.display_name(), "Trading Post");
}

#[test]
fn display_name_repair_station() {
    assert_eq!(StationType::RepairStation.display_name(), "Repair Station");
}

#[test]
fn display_name_black_market() {
    assert_eq!(StationType::BlackMarket.display_name(), "Black Market");
}

#[test]
fn station_spawn_has_station_type_component() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);

    let entity = app
        .world_mut()
        .spawn(Station {
            name: StationType::TradingPost.display_name(),
            dock_radius: 120.0,
            station_type: StationType::TradingPost,
        })
        .id();

    let station = app
        .world()
        .entity(entity)
        .get::<Station>()
        .expect("Station component should exist");
    assert_eq!(station.station_type, StationType::TradingPost);
    assert_eq!(station.name, "Trading Post");
}

#[test]
fn all_station_types_have_unique_display_names() {
    let names = [
        StationType::TradingPost.display_name(),
        StationType::RepairStation.display_name(),
        StationType::BlackMarket.display_name(),
    ];
    // All names are distinct
    assert_ne!(names[0], names[1]);
    assert_ne!(names[1], names[2]);
    assert_ne!(names[0], names[2]);
}
