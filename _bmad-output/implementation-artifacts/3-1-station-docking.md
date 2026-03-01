# Story 3.1: Station Docking

Status: done

## Story

As a player,
I want to dock at stations by approaching and pressing interact (E),
so that I have safe harbors in the open world.

## Acceptance Criteria

1. A `Station` component exists in `src/core/station.rs` with fields: `name`, `dock_radius`, `station_type` (enum with at least `Trading` variant)
2. The E key maps to `ActionState.interact` (already declared, just needs key binding in `read_input`)
3. When the player is within `dock_radius` of a Station AND presses `interact`, the player receives a `Docked { station: Entity }` component
4. When the player moves farther than `dock_radius` from the docked station OR presses interact again while docked, `Docked` is removed
5. On successful dock, `GameEventKind::StationDocked` is emitted (already exists in shared/events.rs)
6. Station entities have a `NeedsStationVisual` marker component — rendering adds a hexagon mesh (teal color, distinct from `TutorialStation`)
7. At least one Station is spawned in world chunk generation (e.g., in `WreckField` biome or as occasional deepspace presence)
8. All existing 458+ tests remain green
9. New tests in `tests/station_docking.rs` cover: dock on approach+interact, no dock on approach-only, undock on distance, undock on second interact

## Tasks / Subtasks

- [x] Task 1: Add E key to input (AC: #2)
  - [x] In `src/core/input.rs` `read_input()`: add `keyboard.just_pressed(KeyCode::KeyE)` → `action_state.interact = true`
  - [x] Add corresponding gamepad mapping (e.g., `gamepad.just_pressed(GamepadButton::East)`)

- [x] Task 2: Create `src/core/station.rs` (AC: #1, #3, #4, #5)
  - [x] Define `StationType` enum: `Trading`, `Repair`, `Research`
  - [x] Define `Station` component: `pub name: &'static str`, `pub dock_radius: f32`, `pub station_type: StationType`
  - [x] Define `NeedsStationVisual` marker component
  - [x] Define `Docked` component: `pub station: Entity`
  - [x] System `update_docking`: Query player WITHOUT `Docked`, iterate `Station` entities, if distance <= dock_radius AND `action_state.interact` → insert `Docked { station }`, emit `GameEventKind::StationDocked`; consumes `interact=false` on success
  - [x] System `update_undocking`: Query player WITH `Docked`, get station position, if distance > dock_radius * 1.1 OR `action_state.interact` → remove `Docked`
  - [x] Unit tests inside `station.rs` (component construction, StationType variants)

- [x] Task 3: Register in CorePlugin (AC: #3, #4)
  - [x] `pub mod station;` in `src/core/mod.rs`
  - [x] Import and add `update_docking` and `update_undocking` systems in `CoreSet::Events`
  - [x] `update_docking` runs before `update_undocking`

- [x] Task 4: World spawning — add Station to chunk generation (AC: #7)
  - [x] In `src/world/generation.rs`, extend `generate_chunk_content` to occasionally spawn a `Station` Blueprint
  - [x] Use noise/rng: ~5% chance per `WreckField` chunk, seed-deterministic
  - [x] Blueprint spawns entity with: `Station { name: "Trading Post", dock_radius: 120.0, station_type: StationType::Trading }`, `NeedsStationVisual`, `Transform`, `ChunkEntity { coord }`

- [x] Task 5: Rendering — hexagon visual (AC: #6)
  - [x] In `src/rendering/mod.rs`, added `StationAssets` resource + `setup_station_assets` + `render_stations` system
  - [x] Hexagon radius 35, medium-teal `Color::srgb(0.2, 0.7, 0.6)`, distinct from TutorialStation
  - [x] Removes `NeedsStationVisual` marker after adding mesh

- [x] Task 6: Integration tests `tests/station_docking.rs` (AC: #9)
  - [x] Test: `player_docks_when_in_range_and_interact_pressed`
  - [x] Test: `player_does_not_dock_when_in_range_but_no_interact`
  - [x] Test: `player_does_not_dock_when_interact_but_out_of_range`
  - [x] Test: `player_undocks_when_moving_out_of_range`
  - [x] Test: `player_undocks_when_pressing_interact_again`
  - [x] Test: `station_docked_event_emitted_on_dock`
  - [x] Test: `player_docks_at_nearest_station_first_in_range`
  - [x] Test: `station_type_trading_can_be_constructed`
  - [x] Test: `needs_station_visual_can_be_inserted_and_queried`

## Dev Notes

### Critical Architecture Rules (from CLAUDE.md)
- **Core/Rendering separation:** `src/core/station.rs` gets components + systems only. NO mesh/lyon code there.
- **Rendering adds visuals:** `NeedsStationVisual` marker → Rendering system adds `Mesh2d + MeshMaterial2d` then removes marker.
- **No `unwrap()` in tests** — always `.expect("description")`

### Existing Patterns to Reuse

**Proximity detection (from tutorial.rs `dock_at_station`):**
```rust
let distance = (station_pos - player_pos).length();
if distance <= config.dock_radius {
    // ...
}
```
Story 3-1 adds the `ActionState.interact` gate on top of this.

**Rendering marker pattern (from spawning.rs):**
```rust
pub struct NeedsAsteroidVisual;  // Core
// Rendering: Query<(Entity, &Transform), With<NeedsAsteroidVisual>>
//   → insert Mesh2d, remove NeedsAsteroidVisual
```
Follow exact same pattern for `NeedsStationVisual`.

**GameEvent emission (from tutorial.rs):**
```rust
let kind = GameEventKind::StationDocked;
game_events.write(GameEvent {
    severity: severity_config.severity_for(&kind),
    kind,
    position: station_pos,
    game_time: time.elapsed_secs_f64(),
});
```
`GameEventKind::StationDocked` already exists in `src/shared/events.rs:48`.

**World chunk Blueprint spawning (from world/generation.rs):**
```rust
// Pattern: use seed-derived rng to decide entity presence
// Return entities as Blueprints from generate_chunk_content
// Spawned entities must have ChunkEntity { coord } for unloading
```

### Files to Modify

| File | Action | Why |
|------|--------|-----|
| `src/core/input.rs` | MODIFY | Add E key → `interact`, gamepad East button |
| `src/core/station.rs` | CREATE | `Station`, `Docked`, `NeedsStationVisual`, `StationType`, systems |
| `src/core/mod.rs` | MODIFY | `pub mod station;`, import + register systems |
| `src/world/generation.rs` | MODIFY | Spawn Station blueprints in chunk generation |
| `src/rendering/mod.rs` | MODIFY | Add `spawn_station_visuals` system |
| `tests/station_docking.rs` | CREATE | Integration tests |
| `tests/helpers/mod.rs` | MODIFY | Add `spawn_station` helper + import `station::*` if needed |

### Key Constraints
- `Docked` component lives on the **player** entity (not the station)
- Undocking hysteresis: use `dock_radius * 1.1` for undock distance to prevent jitter
- `interact` is `just_pressed` (rising edge), not `pressed` — ensure `read_input` uses `just_pressed(KeyCode::KeyE)`
- `update_undocking` must NOT check `just_pressed(interact)` directly from ActionState for the "press again" case — it already runs after `read_input` so ActionState.interact being true is sufficient
- Station entity must NOT have `Health` or `Collider` (it's not destructible in this story)

### TutorialStation vs Station
- `TutorialStation` (`src/core/tutorial.rs`) — tutorial-specific, despawns after tutorial, triggers phase transitions
- `Station` (`src/core/station.rs`) — open-world persistent stations, entirely separate component/system

### Test App Setup
Tests use `test_app()` from `tests/helpers/mod.rs`. For station tests, add the new systems manually:
```rust
fn station_test_app() -> App {
    let mut app = test_app(); // includes all existing systems
    app.init_resource::<ActionState>();
    app.add_systems(FixedUpdate, (
        update_docking,
        update_undocking,
    ).chain().after(record_game_events));
    app
}
```

### Project Structure Notes
- New file: `src/core/station.rs` — follows same style as `tutorial.rs`, `weapons.rs`
- New test file: `tests/station_docking.rs` — follows same style as `tests/tutorial_zone.rs`
- No new config file needed for this story (hardcode defaults in `Station::default_trading()`)

### References
- `ActionState.interact` declared: [src/core/input.rs:16]
- `GameEventKind::StationDocked`: [src/shared/events.rs:48]
- `dock_at_station` proximity pattern: [src/core/tutorial.rs:602-637]
- `NeedsAsteroidVisual` rendering pattern: [src/core/spawning.rs] + [src/rendering/mod.rs]
- `generate_chunk_content` Blueprint spawning: [src/world/generation.rs]
- `TutorialStation` hexagon rendering: [src/rendering/mod.rs] — use as visual reference
- Test helpers: [tests/helpers/mod.rs]

## Dev Agent Record

### Agent Model Used

claude-sonnet-4-6

### Debug Log References

- **Bug fixed**: `.chain()` in Bevy 0.18 flushes commands between chained systems (apply_deferred). `update_docking` consumed `interact = false` on successful dock to prevent `update_undocking` from immediately undocking in the same frame.

### Completion Notes List

- Story 3-1 implemented with 471 total tests passing (9 new integration tests in station_docking.rs + 4 new unit tests in generation.rs + 1 new unit test in input.rs)
- `update_docking` takes `ResMut<ActionState>` and sets `interact = false` after successful dock — architectural decision to prevent same-frame dock+undock due to Bevy's `.chain()` apply_deferred behavior.

### File List

- `src/core/input.rs` — Added E key → `interact`, Gamepad East → `interact`, unit test
- `src/core/station.rs` — NEW: `Station`, `Docked`, `NeedsStationVisual`, `StationType`, `update_docking`, `update_undocking`, unit tests
- `src/core/mod.rs` — Added `pub mod station;`, imported + registered `update_docking`/`update_undocking` in CoreSet::Events
- `src/world/generation.rs` — Added `BlueprintType::Station`, `STATION_SPAWN_CHANCE`, station spawn in `generate_chunk_content`
- `src/world/mod.rs` — Added `BlueprintType::Station` arm in `update_chunks`, spawns Station+NeedsStationVisual
- `src/rendering/mod.rs` — Added `StationAssets`, `setup_station_assets`, `render_stations` system
- `tests/station_docking.rs` — NEW: 9 integration tests for docking/undocking mechanics
