# Story 5-7: Station Crafting UI — Implementation

## Summary

Replaced the placeholder station UI with a full crafting interface that displays
discovered recipes, player resources, navigation cursor, and per-recipe affordability
coloring. Added navigation (R/T keys) and craft/buy (F key) inputs.

## Changes

### `src/core/input.rs`

Added three new fields to `ActionState`:

- `craft: bool` — F key, craft/buy the currently selected recipe when docked
- `nav_up: bool` — R key, navigate recipe list upward
- `nav_down: bool` — T key, navigate recipe list downward

**Key choice rationale:** R/T are adjacent, conflict-free with thrust (W), rotation
(A/D), fire (Space), interact (E), and weapon switch (Tab). F (craft) is mnemonic
for "fabricate" and does not overlap with any existing binding.

### `src/core/upgrades.rs`

Added:

- `StationUiState` resource — tracks `selected_recipe_index: usize` (cursor position
  in the recipe list).
- `navigate_station_ui` system — reads `nav_up`/`nav_down` from `ActionState`,
  updates `selected_recipe_index` with wrap-around. Only runs when player has `Docked`.
- `handle_craft_input` system — reads `craft` from `ActionState`, writes
  `CraftingRequest { recipe_index: Some(selected_recipe_index) }` when docked.

### `src/core/mod.rs`

- Initialized `StationUiState` resource.
- Registered `navigate_station_ui` and `handle_craft_input` systems in `CoreSet::Input`
  (chained, so navigation runs before craft dispatch).

### `src/rendering/mod.rs`

Replaced placeholder `spawn_station_ui` with a full implementation and added
`update_station_ui`:

**`build_station_ui` (private helper):**
Constructs the full UI panel with:
- Header: `{station_name}  [{station_type}]`
- Resource row: `Credits: X  |  Scrap: X  Alloy: X  Core: X`
- Up to 5 recipe rows (sliding window centered on selected index):
  - `►  {Name:<14} Xs Xa Xc  Xcr  [CRAFT|BUY|CRAFT/BUY]  ✓`
  - White when affordable, grey when not, green when already installed
- Navigation hint: `R/T = navigate  |  F = craft/buy  |  E = undock`

**`update_station_ui` (new public system):**
Runs every frame while docked, but only rebuilds the UI when `Credits`,
`PlayerInventory`, `DiscoveredRecipes`, `InstalledUpgrades`, or `StationUiState`
has changed (change-detection guard). Despawns old `StationUiRoot` and respawns.

**`spawn_station_ui`:**
Now accepts `Credits`, `PlayerInventory`, `DiscoveredRecipes`, `InstalledUpgrades`,
and `StationUiState` as parameters and calls `build_station_ui`.

**Plugin registration:**
Added `update_station_ui` to the `(spawn_station_ui, update_station_ui, despawn_station_ui).chain()` system group.

### `tests/station_ui.rs`

Updated `station_ui_test_app()` to initialize the new required resources.
Updated `station_ui_shows_station_type_label` test to use `contains()` instead of
exact equality (station type is now embedded in the combined header text).
Added 8 new tests covering:

- `station_ui_shows_recipe_names` — recipe names from DiscoveredRecipes appear in UI
- `station_ui_shows_craft_hint` — hint text with craft/undock instructions is present
- `navigate_down_increments_selected_index` — nav_down advances cursor
- `navigate_up_wraps_around_to_last` — nav_up from 0 wraps to last
- `navigate_down_wraps_around_to_first` — nav_down from last wraps to 0
- `navigate_does_nothing_when_not_docked` — navigation ignored outside station
- `craft_key_sets_crafting_request_when_docked` — F key emits CraftingRequest
- `craft_key_does_nothing_when_not_docked` — craft ignored outside station

## Test Results

630 tests passing (was 622; 8 new tests added).

## Acceptance Criteria Status

1. Station UI shows available recipes from DiscoveredRecipes — DONE
2. Each recipe row shows name, material costs, credit cost, acquisition method — DONE
3. Selected recipe highlighted with ► cursor — DONE
4. Recipes the player cannot afford are shown in grey — DONE
5. Navigation with R (up) / T (down) keys — DONE
6. Craft/buy with F key when docked — DONE
7. After successful craft, UI refreshes to show updated material counts — DONE (change-detection on Credits/PlayerInventory triggers rebuild)
8. cargo test green — DONE (630 tests)
