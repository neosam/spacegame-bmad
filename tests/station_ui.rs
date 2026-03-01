#![deny(clippy::unwrap_used)]
/// Integration tests for Stories 3-2 and 5-7: Station Shop UI
///
/// Tests cover: UI panel spawns when player docks, UI panel despawns when player
/// undocks, no duplicate panels on re-docking, and recipe list navigation/craft input.
mod helpers;

use bevy::prelude::*;
use void_drifter::core::economy::{Credits, PlayerInventory};
use void_drifter::core::flight::Player;
use void_drifter::core::input::ActionState;
use void_drifter::core::station::{Docked, Station, StationType};
use void_drifter::core::upgrades::{
    handle_craft_input, navigate_station_ui, CraftingRequest, DiscoveredRecipes, InstalledUpgrades,
    StationUiState,
};
use void_drifter::rendering::{despawn_station_ui, spawn_station_ui, update_station_ui, StationUiRoot};

// ── Test helpers ─────────────────────────────────────────────────────────

fn station_ui_test_app() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    // Resources required by spawn_station_ui and update_station_ui
    app.init_resource::<Credits>();
    app.init_resource::<PlayerInventory>();
    app.init_resource::<DiscoveredRecipes>();
    app.init_resource::<InstalledUpgrades>();
    app.init_resource::<StationUiState>();
    app.add_systems(
        Update,
        (spawn_station_ui, update_station_ui, despawn_station_ui).chain(),
    );
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
            station_type: StationType::TradingPost,
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

#[test]
fn station_ui_shows_station_type_label() {
    // AC5: dock UI spawns a child Text with the station type name (e.g. "Trading Post")
    let mut app = station_ui_test_app();
    let (player, station) = spawn_player_and_station(&mut app);

    dock_player(&mut app, player, station);
    app.update();

    // The UI root should have been spawned
    assert_eq!(count_station_ui_roots(&mut app), 1, "UI root must exist");

    // Verify that at least one Text child contains the station type display name
    let station_type_label = StationType::TradingPost.display_name();
    let type_text_exists = app
        .world_mut()
        .query::<&Text>()
        .iter(app.world())
        .any(|t| t.0.contains(station_type_label));
    assert!(
        type_text_exists,
        "UI should contain a Text with the station type label 'Trading Post'"
    );
}

#[test]
fn station_ui_shows_recipe_names() {
    // AC1: Station UI shows available recipes from DiscoveredRecipes
    let mut app = station_ui_test_app();
    let (player, station) = spawn_player_and_station(&mut app);

    dock_player(&mut app, player, station);
    app.update();

    assert_eq!(count_station_ui_roots(&mut app), 1, "UI root must exist");

    // At least one recipe name should appear in the UI text
    let recipes = app.world().resource::<DiscoveredRecipes>().clone();
    let first_recipe_name = recipes.recipes[0].display_name;
    let recipe_text_exists = app
        .world_mut()
        .query::<&Text>()
        .iter(app.world())
        .any(|t| t.0.contains(first_recipe_name));
    assert!(
        recipe_text_exists,
        "UI should contain text showing recipe names from DiscoveredRecipes"
    );
}

#[test]
fn station_ui_shows_craft_hint() {
    // AC5/AC6: UI shows navigation and craft key hints
    let mut app = station_ui_test_app();
    let (player, station) = spawn_player_and_station(&mut app);

    dock_player(&mut app, player, station);
    app.update();

    let hint_exists = app
        .world_mut()
        .query::<&Text>()
        .iter(app.world())
        .any(|t| t.0.contains("craft") || t.0.contains("CRAFT") || t.0.contains("undock"));
    assert!(hint_exists, "UI should contain a craft/navigation hint");
}

// ── Navigation Tests (navigate_station_ui system) ─────────────────────────

fn nav_test_app() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.init_resource::<ActionState>();
    app.init_resource::<DiscoveredRecipes>();
    app.init_resource::<StationUiState>();
    app.add_systems(Update, navigate_station_ui);
    app.update(); // prime
    app
}

#[test]
fn navigate_down_increments_selected_index() {
    let mut app = nav_test_app();
    // Spawn a docked player
    let station = app.world_mut().spawn(()).id();
    app.world_mut().spawn((Player, Docked { station }));

    // Simulate nav_down press
    app.world_mut().resource_mut::<ActionState>().nav_down = true;
    app.update();

    let idx = app.world().resource::<StationUiState>().selected_recipe_index;
    assert_eq!(idx, 1, "nav_down should increment selected_recipe_index to 1");
}

#[test]
fn navigate_up_wraps_around_to_last() {
    let mut app = nav_test_app();
    let station = app.world_mut().spawn(()).id();
    app.world_mut().spawn((Player, Docked { station }));

    // selected_recipe_index starts at 0; nav_up should wrap to last
    app.world_mut().resource_mut::<ActionState>().nav_up = true;
    app.update();

    let recipes_count = app.world().resource::<DiscoveredRecipes>().recipes.len();
    let idx = app.world().resource::<StationUiState>().selected_recipe_index;
    assert_eq!(
        idx,
        recipes_count - 1,
        "nav_up from index 0 should wrap around to last recipe"
    );
}

#[test]
fn navigate_down_wraps_around_to_first() {
    let mut app = nav_test_app();
    let station = app.world_mut().spawn(()).id();
    app.world_mut().spawn((Player, Docked { station }));

    // Set index to last recipe, then nav_down should wrap to 0
    let last_idx = {
        let recipes = app.world().resource::<DiscoveredRecipes>();
        recipes.recipes.len() - 1
    };
    app.world_mut()
        .resource_mut::<StationUiState>()
        .selected_recipe_index = last_idx;

    app.world_mut().resource_mut::<ActionState>().nav_down = true;
    app.update();

    let idx = app.world().resource::<StationUiState>().selected_recipe_index;
    assert_eq!(idx, 0, "nav_down from last index should wrap around to 0");
}

#[test]
fn navigate_does_nothing_when_not_docked() {
    let mut app = nav_test_app();
    // Player without Docked component
    app.world_mut().spawn(Player);

    app.world_mut().resource_mut::<ActionState>().nav_down = true;
    app.update();

    let idx = app.world().resource::<StationUiState>().selected_recipe_index;
    assert_eq!(idx, 0, "Navigation should not change index when player is not docked");
}

// ── Craft Input Tests (handle_craft_input system) ─────────────────────────

fn craft_input_test_app() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.init_resource::<ActionState>();
    app.init_resource::<StationUiState>();
    app.init_resource::<CraftingRequest>();
    app.add_systems(Update, handle_craft_input);
    app.update(); // prime
    app
}

#[test]
fn craft_key_sets_crafting_request_when_docked() {
    let mut app = craft_input_test_app();
    let station = app.world_mut().spawn(()).id();
    app.world_mut().spawn((Player, Docked { station }));

    app.world_mut().resource_mut::<ActionState>().craft = true;
    app.update();

    let request = app.world().resource::<CraftingRequest>();
    assert_eq!(
        request.recipe_index,
        Some(0),
        "Craft key should set CraftingRequest to selected index 0"
    );
}

#[test]
fn craft_key_does_nothing_when_not_docked() {
    let mut app = craft_input_test_app();
    app.world_mut().spawn(Player);

    app.world_mut().resource_mut::<ActionState>().craft = true;
    app.update();

    let request = app.world().resource::<CraftingRequest>();
    assert!(
        request.recipe_index.is_none(),
        "Craft key should not set CraftingRequest when player is not docked"
    );
}
