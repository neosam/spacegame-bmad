#![deny(clippy::unwrap_used)]
/// Integration tests for Story 3-2: Station Shop UI
///
/// Tests cover: UI panel spawns when player docks, UI panel despawns when player
/// undocks, and no duplicate panels on re-docking.
mod helpers;

use bevy::prelude::*;
use void_drifter::core::flight::Player;
use void_drifter::core::station::{Docked, Station, StationType};
use void_drifter::rendering::{despawn_station_ui, spawn_station_ui, StationUiRoot};

// ── Test helpers ─────────────────────────────────────────────────────────

fn station_ui_test_app() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_systems(Update, (spawn_station_ui, despawn_station_ui).chain());
    // Prime first frame
    app.update();
    app
}

/// Spawns a station and a player (without Docked) and returns their entities.
fn spawn_player_and_station(app: &mut App) -> (Entity, Entity) {
    let station = app
        .world_mut()
        .spawn(Station {
            name: "Test Station",
            dock_radius: 120.0,
            station_type: StationType::Trading,
        })
        .id();
    let player = app
        .world_mut()
        .spawn((Player, Transform::default()))
        .id();
    (player, station)
}

/// Inserts `Docked` onto the player entity (immediate, not via commands).
fn dock_player(app: &mut App, player: Entity, station: Entity) {
    app.world_mut()
        .entity_mut(player)
        .insert(Docked { station });
}

/// Removes `Docked` from the player entity (immediate, not via commands).
fn undock_player(app: &mut App, player: Entity) {
    app.world_mut().entity_mut(player).remove::<Docked>();
}

/// Count entities with `StationUiRoot` marker.
fn count_station_ui_roots(app: &mut App) -> usize {
    app.world_mut()
        .query_filtered::<Entity, With<StationUiRoot>>()
        .iter(app.world())
        .count()
}

// ── Tests ─────────────────────────────────────────────────────────────────

#[test]
fn station_ui_spawns_on_dock() {
    let mut app = station_ui_test_app();
    let (player, station) = spawn_player_and_station(&mut app);

    // Run a frame without Docked — UI should not exist
    app.update();
    assert_eq!(
        count_station_ui_roots(&mut app),
        0,
        "StationUiRoot should NOT exist before docking"
    );

    // Insert Docked component and run frame
    dock_player(&mut app, player, station);
    app.update();

    assert_eq!(
        count_station_ui_roots(&mut app),
        1,
        "StationUiRoot should be spawned after player gains Docked component"
    );
}

#[test]
fn station_ui_despawns_on_undock() {
    let mut app = station_ui_test_app();
    let (player, station) = spawn_player_and_station(&mut app);

    // Dock → UI spawns
    dock_player(&mut app, player, station);
    app.update();
    assert_eq!(
        count_station_ui_roots(&mut app),
        1,
        "StationUiRoot should exist after docking"
    );

    // Undock → UI despawns
    undock_player(&mut app, player);
    app.update();
    assert_eq!(
        count_station_ui_roots(&mut app),
        0,
        "StationUiRoot should be removed after player loses Docked component"
    );
}

#[test]
fn only_one_ui_panel_per_dock() {
    let mut app = station_ui_test_app();
    let (player, station) = spawn_player_and_station(&mut app);

    // Dock → UI spawns
    dock_player(&mut app, player, station);
    app.update();
    assert_eq!(
        count_station_ui_roots(&mut app),
        1,
        "Should have exactly one UI panel after first dock"
    );

    // Undock → UI despawns
    undock_player(&mut app, player);
    app.update();

    // Dock again → UI spawns (exactly one, not two)
    dock_player(&mut app, player, station);
    app.update();
    assert_eq!(
        count_station_ui_roots(&mut app),
        1,
        "Should have exactly one UI panel after re-docking (not two)"
    );
}
