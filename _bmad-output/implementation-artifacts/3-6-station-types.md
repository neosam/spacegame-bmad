# Story 3.6: Station Types

Status: done

## Story

As a player,
I want to see visually distinct station types (Trading Post, Repair Station, Black Market),
so that the open world feels varied and navigation has meaning.

## Acceptance Criteria

1. `StationType` enum in `src/core/station.rs` has exactly three variants: `TradingPost`, `RepairStation`, `BlackMarket` (renamed from current `Trading`, `Repair`, `Research`)
2. `StationType::display_name(&self) -> &'static str` method returns human-readable strings: `"Trading Post"`, `"Repair Station"`, `"Black Market"`
3. Station spawning in `src/world/mod.rs` assigns a random `StationType` (not hardcoded `TradingPost`); uses `rand::random::<u8>() % 3` or equivalent
4. Rendering uses per-type mesh/material — hexagon sizes and colors:
   - `TradingPost`: green `#00FF88` (0.0, 1.0, 0.53), radius 40px
   - `RepairStation`: blue `#4488FF` (0.27, 0.53, 1.0), radius 35px
   - `BlackMarket`: purple `#AA44FF` (0.67, 0.27, 1.0), radius 30px
5. Station docking UI shows a type label (e.g. `"Trading Post"`) as a subtitle below the station name
6. All existing tests updated for renamed variants (station.rs unit tests, any other references)
7. New tests in `tests/station_types.rs`: `display_name_trading_post`, `display_name_repair_station`, `display_name_black_market`, `station_spawn_has_station_type_component`

## Tasks / Subtasks

- [x] Task 1: Rename `StationType` variants + add `display_name()` in `src/core/station.rs` (AC: #1, #2)
  - [x] Rename `Trading` → `TradingPost`, `Repair` → `RepairStation`, `Research` → `BlackMarket`
  - [x] Add `impl StationType { pub fn display_name(&self) -> &'static str }` matching AC #2
  - [x] Update `station_type_variants_are_distinct` test for new names
  - [x] Update `station_component_fields` test: `StationType::Trading` → `StationType::TradingPost`

- [x] Task 2: Update station spawning in `src/world/mod.rs` (AC: #3)
  - [x] Replace hardcoded `station_type: StationType::Trading` with random type selection
  - [x] Use pattern: `let roll = rand::random::<u8>() % 3; ...`
  - [x] Update hardcoded `name: "Trading Post"` to use `station_type.display_name()`
  - [x] `rand` crate is already in Cargo.toml from Story 3-4

- [x] Task 3: Update rendering in `src/rendering/mod.rs` (AC: #4)
  - [x] Replace `StationAssets { mesh, material }` with `StationTypeAssets { trading_mesh, trading_mat, repair_mesh, repair_mat, black_market_mesh, black_market_mat }`
  - [x] Update `setup_station_assets`: create 3 mesh/material pairs using `generate_tutorial_station_mesh(radius)` at sizes 40/35/30
  - [x] Update `render_stations`: query `(Entity, &Station)` with `With<NeedsStationVisual>`, pick handles from `station.station_type`
  - [x] Import `StationType` in rendering/mod.rs

- [x] Task 4: Add type label to station UI in `src/rendering/mod.rs` (AC: #5)
  - [x] In `spawn_station_ui`: after looking up `station_name`, also read `station.station_type.display_name()` → `station_type_label`
  - [x] Spawn a subtitle text node with `station_type_label` (font_size 16, grey color srgba(0.7, 0.7, 0.7, 1.0))
  - [x] Insert subtitle between title and shop rows in `add_children`

- [x] Task 5: Integration tests `tests/station_types.rs` (AC: #6, #7)
  - [x] `display_name_trading_post`: asserts "Trading Post"
  - [x] `display_name_repair_station`: asserts "Repair Station"
  - [x] `display_name_black_market`: asserts "Black Market"
  - [x] `station_spawn_has_station_type_component`: spawns Station, verifies component
  - [x] `all_station_types_have_unique_display_names`: all 3 names distinct
  - [x] Fixed `tests/station_docking.rs` and `tests/station_ui.rs`: `StationType::Trading` → `StationType::TradingPost`

## Dev Notes

### CRITICAL: What Already Exists (Do NOT Reinvent)

- `StationType` is in `src/core/station.rs` (NOT `src/shared/components.rs`!) with current variants `Trading`, `Repair`, `Research`
- `Station` struct (`src/core/station.rs:17`) ALREADY has `station_type: StationType` field
- `NeedsStationVisual` marker ALREADY exists in `src/core/station.rs`
- `generate_tutorial_station_mesh(radius)` in `src/rendering/vector_art.rs` — reuse for all station types
- `setup_station_assets` and `render_stations` in `src/rendering/mod.rs:476-505`
- `spawn_station_ui` in `src/rendering/mod.rs:555` — add type label here
- Station spawning: `src/world/mod.rs:382-395` — only one spawn site (all stations are `TradingPost` currently)
- `rand` crate already in `Cargo.toml` (added in Story 3-4 for `rand::random::<f32>()`)

### Compilation Impact of Rename

Renaming `Trading` → `TradingPost` etc. will cause compile errors in:
1. `src/core/station.rs` — unit tests (3 tests referencing old names)
2. `src/world/mod.rs:387` — `station_type: StationType::Trading`
3. `src/rendering/mod.rs` — if `StationType` is imported (check imports)

Fix all of these in one pass. The compiler will catch them all.

### Rendering Pattern for Per-Type Assets

```rust
#[derive(Resource)]
struct StationTypeAssets {
    trading_mesh: Handle<Mesh>,
    trading_mat: Handle<ColorMaterial>,
    repair_mesh: Handle<Mesh>,
    repair_mat: Handle<ColorMaterial>,
    black_market_mesh: Handle<Mesh>,
    black_market_mat: Handle<ColorMaterial>,
}
```

In `render_stations`, query `(Entity, &Station)` instead of just `Entity`:
```rust
fn render_stations(
    mut commands: Commands,
    assets: Res<StationTypeAssets>,
    query: Query<(Entity, &Station), With<NeedsStationVisual>>,
) {
    for (entity, station) in query.iter() {
        let (mesh, mat) = match station.station_type {
            StationType::TradingPost => (assets.trading_mesh.clone(), assets.trading_mat.clone()),
            StationType::RepairStation => (assets.repair_mesh.clone(), assets.repair_mat.clone()),
            StationType::BlackMarket => (assets.black_market_mesh.clone(), assets.black_market_mat.clone()),
        };
        commands.entity(entity)
            .insert((Mesh2d(mesh), MeshMaterial2d(mat)))
            .remove::<NeedsStationVisual>();
    }
}
```

### UI Label Addition

In `spawn_station_ui`, after `station_name`:
```rust
let station_type_label = station_query
    .get(docked.station)
    .map(|s| s.station_type.display_name())
    .unwrap_or("Unknown Type");
```
Spawn an additional text node and include it in `add_children`.

### Color Values (sRGB)

- `#00FF88` = `Color::srgb(0.0, 1.0, 0.533)` (TradingPost green)
- `#4488FF` = `Color::srgb(0.267, 0.533, 1.0)` (RepairStation blue)
- `#AA44FF` = `Color::srgb(0.667, 0.267, 1.0)` (BlackMarket purple)

### Random Station Type

```rust
let station_type = match rand::random::<u8>() % 3 {
    0 => StationType::TradingPost,
    1 => StationType::RepairStation,
    _ => StationType::BlackMarket,
};
Station {
    name: station_type.display_name(),
    dock_radius: 120.0,
    station_type,
}
```

Wait — `Station.name` is `&'static str`. Since `display_name()` returns `&'static str`, this works directly.

### Architecture Rules

- `StationType` stays in `src/core/station.rs` — NOT moved to `src/shared/components.rs` (no circular import issue here since station.rs doesn't import economy.rs or events.rs in a cyclic way)
- Core/Rendering separation: Core assigns `StationType` component, Rendering reads it to pick assets
- No save changes: `SAVE_VERSION` stays at `4` — station types are regenerated deterministically from world seed on load

### References

- `StationType` current location: `src/core/station.rs:6-11`
- Station spawn: `src/world/mod.rs:382-395`
- Rendering setup: `src/rendering/mod.rs:469-505`
- Station UI: `src/rendering/mod.rs:555-642`
- `generate_tutorial_station_mesh`: `src/rendering/vector_art.rs`

## Dev Agent Record

### Agent Model Used

claude-sonnet-4-6

### Debug Log References

### Completion Notes List

- Renamed StationType variants: Trading→TradingPost, Repair→RepairStation, Research→BlackMarket
- Added display_name() method returning "Trading Post", "Repair Station", "Black Market"
- Random spawning via rand::random::<u8>() % 3 in world/mod.rs
- Per-type StationTypeAssets in rendering: green/40px, blue/35px, purple/30px hexagons
- Type label added to station dock UI as subtitle between name and shop rows
- Fixed old variant references in station_docking.rs and station_ui.rs
- 6 new tests: 3 display names, 1 spawn, 1 uniqueness, + 1 station_type_display_names in station.rs
- Total: 513 tests (was 507)

### File List

- src/core/station.rs
- src/world/mod.rs
- src/rendering/mod.rs
- tests/station_types.rs
- tests/station_docking.rs
- tests/station_ui.rs
