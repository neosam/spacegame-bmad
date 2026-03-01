/// Integration tests for Story 3-1: Station Docking
///
/// Tests cover: docking on approach+interact, no dock without interact,
/// no dock out of range, undocking on distance, undocking on second interact,
/// and GameEvent emission.
mod helpers;

use std::time::Duration;

use bevy::prelude::*;
use bevy::time::TimeUpdateStrategy;
use void_drifter::core::flight::Player;
use void_drifter::core::input::ActionState;
use void_drifter::core::station::{Docked, NeedsStationVisual, Station, StationType};
use void_drifter::core::station::{update_docking, update_undocking};
use void_drifter::infrastructure::events::{record_game_events, EventSeverityConfig};
use void_drifter::infrastructure::logbook::Logbook;
use void_drifter::shared::events::{GameEvent, GameEventKind};

// ── Test helpers ─────────────────────────────────────────────────────────

fn station_test_app() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_secs_f64(
        1.0 / 60.0,
    )));
    app.insert_resource(Time::<Fixed>::from_seconds(1.0 / 60.0));
    app.init_resource::<ActionState>();
    app.insert_resource(EventSeverityConfig::default());
    app.init_resource::<Logbook>();
    app.add_message::<GameEvent>();
    app.add_systems(
        FixedUpdate,
        (update_docking, update_undocking, record_game_events).chain(),
    );
    // Prime first frame (dt = 0)
    app.update();
    app
}

fn spawn_player_at(app: &mut App, position: Vec2) -> Entity {
    app.world_mut()
        .spawn((
            Player,
            Transform::from_translation(position.extend(0.0)),
        ))
        .id()
}

fn spawn_station_at(app: &mut App, position: Vec2, dock_radius: f32) -> Entity {
    app.world_mut()
        .spawn((
            Station {
                name: "Test Station",
                dock_radius,
                station_type: StationType::Trading,
            },
            Transform::from_translation(position.extend(0.0)),
        ))
        .id()
}

fn press_interact(app: &mut App) {
    app.world_mut().resource_mut::<ActionState>().interact = true;
}

// ── Tests ─────────────────────────────────────────────────────────────────

#[test]
fn player_docks_when_in_range_and_interact_pressed() {
    let mut app = station_test_app();

    let player = spawn_player_at(&mut app, Vec2::ZERO);
    let station = spawn_station_at(&mut app, Vec2::new(50.0, 0.0), 120.0);
    press_interact(&mut app);

    app.update();

    let docked = app
        .world()
        .entity(player)
        .get::<Docked>()
        .expect("Player should have Docked component after interacting in range");
    assert_eq!(docked.station, station, "Docked should reference the station entity");
}

#[test]
fn player_does_not_dock_when_in_range_but_no_interact() {
    let mut app = station_test_app();

    let player = spawn_player_at(&mut app, Vec2::ZERO);
    spawn_station_at(&mut app, Vec2::new(50.0, 0.0), 120.0);
    // Do NOT press interact

    app.update();

    assert!(
        app.world().entity(player).get::<Docked>().is_none(),
        "Player should NOT be Docked without pressing interact"
    );
}

#[test]
fn player_does_not_dock_when_interact_but_out_of_range() {
    let mut app = station_test_app();

    let player = spawn_player_at(&mut app, Vec2::ZERO);
    spawn_station_at(&mut app, Vec2::new(300.0, 0.0), 120.0);
    press_interact(&mut app);

    app.update();

    assert!(
        app.world().entity(player).get::<Docked>().is_none(),
        "Player should NOT be Docked when out of dock_radius (300 > 120)"
    );
}

#[test]
fn player_undocks_when_moving_out_of_range() {
    let mut app = station_test_app();

    let player = spawn_player_at(&mut app, Vec2::ZERO);
    let station = spawn_station_at(&mut app, Vec2::new(50.0, 0.0), 120.0);
    press_interact(&mut app);
    app.update(); // dock

    // Verify docked
    assert!(
        app.world().entity(player).get::<Docked>().is_some(),
        "Should be docked after first interact"
    );

    // Move player far away (beyond dock_radius * 1.1 = 132)
    app.world_mut()
        .entity_mut(player)
        .get_mut::<Transform>()
        .expect("Player should have Transform")
        .translation = Vec3::new(500.0, 0.0, 0.0);

    // Make sure interact is not pressed
    app.world_mut().resource_mut::<ActionState>().interact = false;

    app.update(); // undocking check runs

    assert!(
        app.world().entity(player).get::<Docked>().is_none(),
        "Player should be undocked when {} units away from station (dock_radius=120, threshold=132)",
        (Vec2::new(500.0, 0.0) - Vec2::new(50.0, 0.0)).length()
    );
    let _ = station; // station entity used for station_at
}

#[test]
fn player_undocks_when_pressing_interact_again() {
    let mut app = station_test_app();

    let player = spawn_player_at(&mut app, Vec2::ZERO);
    spawn_station_at(&mut app, Vec2::new(50.0, 0.0), 120.0);

    // First interact: dock
    press_interact(&mut app);
    app.update();
    assert!(
        app.world().entity(player).get::<Docked>().is_some(),
        "Player should be docked after first interact"
    );

    // Second interact: undock
    press_interact(&mut app);
    app.update();

    assert!(
        app.world().entity(player).get::<Docked>().is_none(),
        "Player should be undocked after pressing interact again"
    );
}

#[test]
fn station_docked_event_emitted_on_dock() {
    let mut app = station_test_app();

    spawn_player_at(&mut app, Vec2::ZERO);
    spawn_station_at(&mut app, Vec2::new(50.0, 0.0), 120.0);
    press_interact(&mut app);

    app.update();

    let logbook = app.world().resource::<Logbook>();
    let docked_event = logbook
        .entries()
        .iter()
        .any(|e| e.kind == GameEventKind::StationDocked);
    assert!(
        docked_event,
        "StationDocked GameEvent should be in logbook after docking"
    );
}

#[test]
fn player_docks_at_nearest_station_first_in_range() {
    // Multiple stations: player docks at first found that's in range
    let mut app = station_test_app();

    let player = spawn_player_at(&mut app, Vec2::ZERO);
    spawn_station_at(&mut app, Vec2::new(50.0, 0.0), 120.0);
    // Second station also in range
    spawn_station_at(&mut app, Vec2::new(-40.0, 0.0), 120.0);
    press_interact(&mut app);

    app.update();

    // Player should be docked at some station (exactly one Docked component)
    assert!(
        app.world().entity(player).get::<Docked>().is_some(),
        "Player should dock at one of the in-range stations"
    );
}

#[test]
fn station_type_trading_can_be_constructed() {
    let s = Station {
        name: "Trading Hub",
        dock_radius: 100.0,
        station_type: StationType::Trading,
    };
    assert_eq!(s.station_type, StationType::Trading);
}

#[test]
fn needs_station_visual_can_be_inserted_and_queried() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);

    let entity = app.world_mut().spawn(NeedsStationVisual).id();

    let has_marker = app
        .world()
        .entity(entity)
        .get::<NeedsStationVisual>()
        .is_some();
    assert!(has_marker, "NeedsStationVisual should be queryable after insertion");
}
