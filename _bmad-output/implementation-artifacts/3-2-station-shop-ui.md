# Story 3.2: Station Shop UI

Status: done

## Story

As a player,
I want to see a station UI panel when I dock at a station,
so that I know I am docked and can see available services.

## Acceptance Criteria

1. When the player has a `Docked` component (added by Story 3-1), a UI panel spawns at the bottom of the screen showing:
   - The station name (from `Station.name` via `Docked.station` entity lookup)
   - A "Shop" placeholder row: `"Shop  [coming in Story 3-3]"`
   - A "Repair" placeholder row: `"Repair  [not yet available]"`
   - A close hint: `"Press E to undock"`
2. When the player's `Docked` component is removed (undock), the UI panel despawns entirely
3. The UI panel does NOT block game world input (player can still interact with game)
4. A `StationUiRoot` marker component is placed on the root UI entity for test queries and despawning
5. The UI uses `GlobalZIndex(50)` — visible above the game world but below the world map overlay (which uses 100)
6. No new `src/core/` code — this is purely a rendering concern reacting to the existing `Docked` component
7. All existing 472 tests remain green
8. New tests in `tests/station_ui.rs` cover: UI spawns on dock, UI despawns on undock, single UI per dock

## Tasks / Subtasks

- [x] Task 1: Create `StationUiRoot` marker and UI systems in `src/rendering/` (AC: #1, #2, #4, #5)
  - [x] Define `StationUiRoot` marker component in `src/rendering/mod.rs`
  - [x] System `spawn_station_ui`: triggered by `Added<Docked>` on the player — spawns UI panel hierarchy
  - [x] System `despawn_station_ui`: triggered by `RemovedComponents<Docked>` — despawns all `StationUiRoot` entities
  - [x] Register both systems in `RenderingPlugin` (Update schedule, chained)

- [x] Task 2: UI panel layout (AC: #1, #3, #5)
  - [x] Root: full-width bottom strip, height 180px, `PositionType::Absolute`, `bottom: Val::Px(0.0)`, semi-transparent dark background
  - [x] Title text: station name in white, `font_size: 22.0`
  - [x] Two placeholder rows: "Shop — coming in Story 3-3", "Repair — not yet available"
  - [x] Hint text: "Press E to undock", `font_size: 13.0`, dim color
  - [x] `GlobalZIndex(50)` (below world map overlay at 100)
  - [x] No font file loading — uses embedded Bevy default font

- [x] Task 3: Integration tests `tests/station_ui.rs` (AC: #7, #8)
  - [x] Test: `station_ui_spawns_on_dock` — player gets Docked → StationUiRoot entity exists
  - [x] Test: `station_ui_despawns_on_undock` — player loses Docked → StationUiRoot entity removed
  - [x] Test: `only_one_ui_panel_per_dock` — docking twice does not create duplicate panels

## Dev Notes

### Critical Architecture Rules (from CLAUDE.md)

- **Core/Rendering separation:** `StationUiRoot` and UI systems live in `src/rendering/`. The `Docked` component (from `src/core/station.rs`) is read-only from Rendering's perspective. NO new Core components needed.
- **No `unwrap()` in tests** — always `.expect("description")`
- **Kein doppelter Player-Spawn** — Tests spawnen eigenen Player; kein Aufruf von `spawn_tutorial_zone`

### Bevy 0.18 UI API (used in this project)

From `src/rendering/world_map.rs` — the established pattern:

```rust
// Root node (fullscreen overlay):
commands.spawn((
    MarkerComponent,
    Node {
        width: Val::Percent(100.0),
        height: Val::Px(180.0),
        position_type: PositionType::Absolute,
        bottom: Val::Px(0.0),
        flex_direction: FlexDirection::Column,
        align_items: AlignItems::Center,
        justify_content: JustifyContent::Center,
        padding: UiRect::all(Val::Px(12.0)),
        ..default()
    },
    BackgroundColor(Color::srgba(0.05, 0.05, 0.15, 0.9)),
    GlobalZIndex(50),
));

// Text child node (Bevy 0.18):
commands.spawn((
    Text("Station Name".to_string()),
    TextFont { font_size: 22.0, ..default() },
    TextColor(Color::WHITE),
));
// Then: commands.entity(root).add_child(text_entity);
```

**IMPORTANT**: In Bevy 0.18, `Text` is a standalone component (not `TextBundle`). Use:
- `Text("string".to_string())` — the text content
- `TextFont { font_size: f32, ..default() }` — font size (no font handle needed for default embedded font)
- `TextColor(Color::...)` — text color

### Detecting Undocking with RemovedComponents

```rust
pub fn despawn_station_ui(
    mut removed: RemovedComponents<Docked>,
    query: Query<Entity, With<StationUiRoot>>,
    mut commands: Commands,
) {
    if removed.read().next().is_some() {
        for entity in query.iter() {
            commands.entity(entity).despawn_recursive();
        }
    }
}
```

### System Registration in RenderingPlugin

Add to `RenderingPlugin::build()`:
```rust
app.add_systems(Update, (spawn_station_ui, despawn_station_ui).chain());
```

The `chain()` ensures `spawn_station_ui` runs before `despawn_station_ui` — prevents same-frame race if dock + undock happened (though unlikely).

### Test App Setup for station_ui tests

```rust
fn station_ui_test_app() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_systems(Update, (spawn_station_ui, despawn_station_ui).chain());
    app.update(); // prime
    app
}
```

**Note**: `Added<Docked>` detection works in `Update` because the component change detection is frame-relative. When `Docked` is inserted via `commands`, it is "added" for the next frame's change detection. So:
1. Insert `Docked` → call `app.update()` → `spawn_station_ui` sees `Added<Docked>` → spawns UI
2. Remove `Docked` → call `app.update()` → `despawn_station_ui` sees `RemovedComponents<Docked>` → despawns UI

### Test Helper: Spawn minimal player+station

```rust
fn spawn_docked_player(app: &mut App, station_name: &'static str) -> (Entity, Entity) {
    let station = app.world_mut().spawn(Station {
        name: station_name,
        dock_radius: 120.0,
        station_type: StationType::Trading,
    }).id();
    let player = app.world_mut().spawn((
        Player,
        Docked { station },
        Transform::default(),
    )).id();
    (player, station)
}
```

### Avoiding Asset Loading

The UI panel does NOT load any font asset. Bevy 0.18 embeds "FiraMono-Medium" in the binary when `bevy_ui` feature is enabled (which it is in `Cargo.toml`). Using `TextFont::default()` uses the embedded font — no `AssetServer` needed.

**CRITICAL**: Do NOT use `AssetServer` in this story. No font file loading. This keeps tests fast and asset-free.

### Files to Modify

| File | Action | Why |
|------|--------|-----|
| `src/rendering/mod.rs` | MODIFY | Add `StationUiRoot` component + `spawn_station_ui` + `despawn_station_ui` systems, register in `RenderingPlugin` |
| `tests/station_ui.rs` | CREATE | 3+ integration tests |

Note: No changes to `src/core/` are needed. The `Docked` component (Story 3-1) is the only trigger.

### References

- `Docked` component: [src/core/station.rs]
- `Station` component (for name lookup): [src/core/station.rs]
- Bevy UI Node pattern: [src/rendering/world_map.rs:195-230]
- `WorldMapRoot` despawning pattern (toggle off): [src/rendering/world_map.rs:155-162]
- `GlobalZIndex` usage: [src/rendering/world_map.rs:207]
- Previous story (3-1): [_bmad-output/implementation-artifacts/3-1-station-docking.md]

### Previous Story Intelligence (3-1)

- **Bevy 0.18 `.chain()` flushes commands** between systems (apply_deferred) — be aware if UI spawn + despawn run in the same frame
- **`Docked` component on Player** — not on Station entity
- Station name is in `Station.name: &'static str` — accessible via `Docked.station` entity reference + Station query
- The `update_docking` system sets `interact = false` after successful dock, so `interact` is never true at dock time in production — no risk of UI spawning and immediately some other system acting on interact

## Dev Agent Record

### Agent Model Used

claude-sonnet-4-6

### Debug Log References

- `despawn_recursive()` removed in Bevy 0.18 — use `despawn()` (now recursive by default)
- `query_filtered` on `app.world()` requires `&mut App` in tests — use `app.world_mut()`

### Completion Notes List

- Story 3-2 implemented: StationUiRoot, spawn_station_ui, despawn_station_ui in src/rendering/mod.rs
- 3 new integration tests in tests/station_ui.rs, all passing
- 475 total tests, all green

### File List

- `src/rendering/mod.rs` — Added StationUiRoot, spawn_station_ui, despawn_station_ui; registered in RenderingPlugin
- `tests/station_ui.rs` — NEW: 3 integration tests
