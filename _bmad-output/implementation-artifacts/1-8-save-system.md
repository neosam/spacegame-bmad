# Story 1.8: Save System

Status: done

## Story

As a player,
I want my position and world state saved when I quit,
so that I can continue where I left off.

## Acceptance Criteria

1. **SavePlugin** — New `src/infrastructure/save/mod.rs` with `SavePlugin` struct. Registers save/load systems, `SaveConfig` resource, and `SaveState` resource. Wired into `InfrastructurePlugin`.
2. **PlayerSave struct** — `src/infrastructure/save/player_save.rs` with `#[derive(Serialize, Deserialize)]`. Fields: `schema_version: u32`, `position: (f32, f32)`, `rotation: f32`, `velocity: (f32, f32)`, `health_current: f32`, `health_max: f32`, `active_weapon: String`, `energy_current: f32`, `energy_max: f32`.
3. **WorldSave struct** — `src/infrastructure/save/world_save.rs` with `#[derive(Serialize, Deserialize)]`. Fields: `schema_version: u32`, `seed: u64`, `explored_chunks: Vec<((i32, i32), String)>` (coord + biome as string for RON readability).
4. **Schema versioning** — `src/infrastructure/save/schema.rs` with `const SAVE_VERSION: u32 = 1`. Functions `check_version(data: &str) -> Result<u32>` and version validation on load.
5. **Save trigger** — `save_game` system runs when `ActionState.save` is true. Writes `player.ron` and `world.ron` to save directory. Emits `GameEvent::GameSaved` (new variant, Tier2).
6. **Load on startup** — `load_game` system runs at `Startup`. If save files exist, restores player state and explored chunks. If missing → start fresh. If corrupt → `warn!`, start fresh.
7. **Save directory** — `SaveConfig.save_dir: String` defaults to `"saves/"` (relative to working dir). Configurable for tests.
8. **RON format** — Human-readable RON output with `ron::ser::PrettyConfig`.
9. **Graceful degradation** — Corrupt or missing files logged with `warn!`, game starts fresh. Never panic on bad save data.
10. **Input binding** — `ActionState` extended with `save: bool` field, bound to F5 key in `read_input`.
11. **Backward compatible** — All 262 existing tests pass unchanged. Serialize derives added to `ChunkCoord`, `BiomeType`, `ActiveWeapon` are additive.
12. **Test coverage** — Unit tests: PlayerSave/WorldSave roundtrip serialization, schema version check, corrupt data handling. Integration tests: save-then-load cycle restores player position, explored chunks persist across save/load.

## Tasks / Subtasks

- [x] Task 1: Add Serialize/Deserialize to shared types (AC: #11)
  - [x] 1.1 Add `Serialize, Deserialize` derives to `ChunkCoord` in `src/world/chunk.rs`
  - [x] 1.2 Add `Serialize, Deserialize` derives to `BiomeType` in `src/world/generation.rs`
  - [x] 1.3 Add `Serialize, Deserialize` derives to `ActiveWeapon` in `src/core/weapons.rs`
  - [x] 1.4 Verify all 262 existing tests still pass after derive additions

- [x] Task 2: Create schema versioning (AC: #4)
  - [x] 2.1 Create `src/infrastructure/save/schema.rs`
  - [x] 2.2 Define `const SAVE_VERSION: u32 = 1`
  - [x] 2.3 Implement `VersionHeader` struct with `schema_version: u32` + Deserialize
  - [x] 2.4 Implement `check_version(ron_str: &str) -> Result<u32, SaveError>` that extracts version from RON data
  - [x] 2.5 Define `SaveError` enum: `VersionMismatch { expected, found }`, `ParseError(String)`, `IoError(String)`

- [x] Task 3: Create PlayerSave (AC: #2, #8)
  - [x] 3.1 Create `src/infrastructure/save/player_save.rs`
  - [x] 3.2 `PlayerSave` struct with all player state fields + `schema_version`
  - [x] 3.3 Implement `PlayerSave::from_world(world: &mut World) -> Option<Self>` — queries Player entity for Transform, Velocity, Health, ActiveWeapon, Energy
  - [x] 3.4 Implement `PlayerSave::apply_to_world(self, world: &mut World)` — sets Player entity components from save data
  - [x] 3.5 Implement `PlayerSave::to_ron(&self) -> Result<String, SaveError>` — pretty-printed RON
  - [x] 3.6 Implement `PlayerSave::from_ron(ron_str: &str) -> Result<Self, SaveError>` — with version check

- [x] Task 4: Create WorldSave (AC: #3, #8)
  - [x] 4.1 Create `src/infrastructure/save/world_save.rs`
  - [x] 4.2 `WorldSave` struct with seed + explored chunks + `schema_version`
  - [x] 4.3 Implement `WorldSave::from_world(world: &World) -> Self` — reads WorldConfig.seed + ExploredChunks
  - [x] 4.4 Implement `WorldSave::apply_to_world(self, world: &mut World)` — restores ExploredChunks resource
  - [x] 4.5 Implement `WorldSave::to_ron(&self) -> Result<String, SaveError>` — pretty-printed RON
  - [x] 4.6 Implement `WorldSave::from_ron(ron_str: &str) -> Result<Self, SaveError>` — with version check

- [x] Task 5: Create SavePlugin (AC: #1, #5, #6, #7)
  - [x] 5.1 Create `src/infrastructure/save/mod.rs` with `pub mod schema; pub mod player_save; pub mod world_save;`
  - [x] 5.2 `SaveConfig` resource: `save_dir: String` (default `"saves/"`)
  - [x] 5.3 `SaveState` resource: `last_save_time: Option<f64>`, `loaded_from_save: bool`
  - [x] 5.4 `save_game` system: when `ActionState.save` → serialize PlayerSave + WorldSave → write to `{save_dir}/player.ron` + `{save_dir}/world.ron`. Emit `GameEvent` (add `GameSaved` variant). Create save dir if not exists.
  - [x] 5.5 `load_game` system (runs at `Startup`): read files → deserialize → apply to world. Graceful fallback on any error.
  - [x] 5.6 `SavePlugin` struct implementing `Plugin`: registers SaveConfig, SaveState, save_game system (in `FixedUpdate` in `CoreSet::Events`), load_game (at `Startup`).
  - [x] 5.7 Wire `SavePlugin` into `InfrastructurePlugin` (add `save::SavePlugin` to build)

- [x] Task 6: Extend input and event types (AC: #10, #5)
  - [x] 6.1 Add `save: bool` to `ActionState` in `src/core/input.rs`
  - [x] 6.2 Map F5 key to `save` in `read_input` system
  - [x] 6.3 Add `GameSaved` variant to `GameEventKind` in `src/shared/events.rs`
  - [x] 6.4 Update `EventSeverityConfig::default()` — map `GameSaved` to `Tier2`
  - [x] 6.5 Update `EventSeverityConfig::severity_for()` match arm for `GameSaved`
  - [x] 6.6 Update `EventSeverityConfig::validate()` known_keys array to include `"GameSaved"`
  - [x] 6.7 Update `assets/config/event_severity.ron` with `"GameSaved": Tier2`

- [x] Task 7: Unit tests (AC: #12)
  - [x] 7.1 `player_save_roundtrip` — serialize → deserialize → fields match
  - [x] 7.2 `world_save_roundtrip` — serialize → deserialize → fields match
  - [x] 7.3 `player_save_from_ron_corrupt_returns_error` — bad RON input → SaveError
  - [x] 7.4 `world_save_from_ron_corrupt_returns_error` — bad RON input → SaveError
  - [x] 7.5 `schema_version_mismatch_returns_error` — version 99 → SaveError::VersionMismatch
  - [x] 7.6 `save_config_default_has_save_dir` — default SaveConfig has save_dir set
  - [x] 7.7 `player_save_from_world_extracts_components` — build App with Player, extract PlayerSave, verify fields

- [x] Task 8: Integration tests (AC: #12)
  - [x] 8.1 Create `tests/save_system.rs`
  - [x] 8.2 `save_then_load_restores_player_position` — write save file, load, verify position restored
  - [x] 8.3 `save_then_load_restores_explored_chunks` — write world save, load, verify explored chunks restored
  - [x] 8.4 `load_missing_files_starts_fresh` — no save files → player at origin, no error
  - [x] 8.5 `load_corrupt_file_starts_fresh` — write garbage to player.ron → load succeeds with defaults
  - [x] 8.6 `save_game_creates_files` — trigger save via ActionState, verify player.ron + world.ron created with valid RON

## Dev Notes

### Architecture Patterns & Constraints

- **Save Boundary** — Only `InfrastructurePlugin` writes save files. Other plugins emit events; the save system observes them. [Source: game-architecture.md — Architectural Boundaries rule #5]
- **Split File Architecture** — `player.ron` (fixed size, fast load for save-slot preview) + `world.ron` (grows with exploration). [Source: game-architecture.md — Data Persistence]
- **Schema Versioning** — Version header in every save file from day one. Migration functions for schema changes. [Source: game-architecture.md — Data Persistence]
- **RON over Binary** — ADR-002: Readability beats performance at <1MB. Debugging save corruption requires human-readable files. [Source: game-architecture.md — ADR-002]
- **Graceful Degradation** — Corrupt save → discard, start fresh, log warning. Never crash on bad save data. [Source: game-architecture.md — Error Handling Strategy]
- **No unwrap()** — `#[deny(clippy::unwrap_used)]` enforced crate-wide. Use `.expect()` in tests only.
- **Config pattern** — `SaveConfig` follows same RON fallback pattern as `FlightConfig`, `WeaponConfig`, `EventSeverityConfig`.
- **Message system** — `GameEvent` uses `MessageWriter`/`MessageReader` (Bevy 0.18 Message API, NOT old Event API). Use `.write()` not `.send()`.

### What Changes vs What Stays

**NEW FILES:**
- `src/infrastructure/save/mod.rs` — SavePlugin, SaveConfig, SaveState
- `src/infrastructure/save/schema.rs` — SAVE_VERSION, SaveError, version check
- `src/infrastructure/save/player_save.rs` — PlayerSave serialization
- `src/infrastructure/save/world_save.rs` — WorldSave serialization
- `tests/save_system.rs` — Integration tests

**MODIFIED FILES:**
- `src/infrastructure/mod.rs` — Add `pub mod save;`, wire SavePlugin
- `src/world/chunk.rs` — Add `Serialize, Deserialize` derives to `ChunkCoord`
- `src/world/generation.rs` — Add `Serialize, Deserialize` derives to `BiomeType`
- `src/core/weapons.rs` — Add `Serialize, Deserialize` derives to `ActiveWeapon`
- `src/core/input.rs` — Add `save: bool` to `ActionState`, F5 binding
- `src/shared/events.rs` — Add `GameSaved` variant to `GameEventKind`
- `src/infrastructure/events.rs` — Update severity_for, default mappings, validate known_keys
- `assets/config/event_severity.ron` — Add `"GameSaved": Tier2`

**STAYS THE SAME:**
- All core gameplay systems — no changes
- All rendering systems — no changes
- Flight physics, camera, collision — no changes
- Chunk loading/unloading logic — unchanged (ExploredChunks just gets restored on load)
- Logbook — unchanged

### Implementation Guidance

**PlayerSave struct pattern:**
```rust
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct PlayerSave {
    pub schema_version: u32,
    pub position: (f32, f32),
    pub rotation: f32,
    pub velocity: (f32, f32),
    pub health_current: f32,
    pub health_max: f32,
    pub active_weapon: String,  // "Laser" or "Spread"
    pub energy_current: f32,
    pub energy_max: f32,
}
```

**WorldSave struct pattern:**
```rust
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct WorldSave {
    pub schema_version: u32,
    pub seed: u64,
    pub explored_chunks: Vec<((i32, i32), String)>,  // (coord, biome_name)
}
```

**Save system pattern:**
```rust
pub fn save_game(
    action_state: Res<ActionState>,
    config: Res<SaveConfig>,
    player_query: Query<(&Transform, &Velocity, &Health, &ActiveWeapon, &Energy), With<Player>>,
    world_config: Res<WorldConfig>,
    explored_chunks: Res<ExploredChunks>,
    mut game_events: MessageWriter<GameEvent>,
    time: Res<Time>,
    severity_config: Res<EventSeverityConfig>,
    mut save_state: ResMut<SaveState>,
) {
    if !action_state.save {
        return;
    }
    // Create save dir, serialize, write files, emit GameSaved event
}
```

**Load system pattern:**
```rust
pub fn load_game(
    config: Res<SaveConfig>,
    mut player_query: Query<(&mut Transform, &mut Velocity, &mut Health, &mut ActiveWeapon, &mut Energy), With<Player>>,
    mut explored_chunks: ResMut<ExploredChunks>,
    mut save_state: ResMut<SaveState>,
) {
    // Try read files, deserialize, apply to world. Warn on errors.
}
```

**Pretty RON output:**
```rust
use ron::ser::PrettyConfig;

let pretty = PrettyConfig::new()
    .depth_limit(4)
    .separate_tuple_members(true);
let ron_str = ron::ser::to_string_pretty(&save, pretty)?;
```

**Serialize derive additions (must keep existing derives):**
```rust
// chunk.rs — add Serialize, Deserialize to existing derives
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Default, Serialize, Deserialize)]
pub struct ChunkCoord { ... }

// generation.rs
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Component, Serialize, Deserialize)]
pub enum BiomeType { ... }

// weapons.rs
#[derive(Component, Default, Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ActiveWeapon { ... }
```

**IMPORTANT: serde import pattern** — files using Serialize/Deserialize need `use serde::{Serialize, Deserialize};`. Check if already imported before adding.

### Previous Story Intelligence (1-7: Event System)

- **MessageWriter uses `.write()` not `.send()`** — corrected during 1-7 implementation. Same pattern for save events.
- **EventSeverityConfig has `validate()` method** — Added in review. New `GameSaved` key must be added to known_keys array.
- **Logbook uses VecDeque** — Changed from Vec in review. No impact on save system.
- **`EnemyDestroyed.entity_type` is `&'static str`** — Changed from String in review. GameSaved can be a simple unit variant.
- **`PlayerDeath` has no position field** — Removed in review. GameSaved also should be a simple variant (no inner data needed — position is in GameEvent.position).
- **Test harness** — `tests/helpers/mod.rs` registers event infrastructure. Save system tests may need `SaveConfig` + `SaveState` registered too.
- **262 tests pass** — Must remain green after save system additions.

### Key Files to Touch

| File | Action |
|------|--------|
| `src/infrastructure/save/mod.rs` | CREATE — SavePlugin, SaveConfig, SaveState, save_game, load_game |
| `src/infrastructure/save/schema.rs` | CREATE — SAVE_VERSION, SaveError, check_version |
| `src/infrastructure/save/player_save.rs` | CREATE — PlayerSave struct + serialization |
| `src/infrastructure/save/world_save.rs` | CREATE — WorldSave struct + serialization |
| `src/infrastructure/mod.rs` | MODIFY — add `pub mod save;`, wire SavePlugin |
| `src/world/chunk.rs` | MODIFY — add Serialize, Deserialize to ChunkCoord |
| `src/world/generation.rs` | MODIFY — add Serialize, Deserialize to BiomeType |
| `src/core/weapons.rs` | MODIFY — add Serialize, Deserialize to ActiveWeapon |
| `src/core/input.rs` | MODIFY — add `save: bool` to ActionState, F5 binding |
| `src/shared/events.rs` | MODIFY — add GameSaved variant to GameEventKind |
| `src/infrastructure/events.rs` | MODIFY — update severity_for, defaults, validate |
| `assets/config/event_severity.ron` | MODIFY — add GameSaved mapping |
| `tests/save_system.rs` | CREATE — integration tests |

### References

- [Source: game-architecture.md — Data Persistence: split file architecture, schema versioning]
- [Source: game-architecture.md — ADR-002: RON over Binary for Saves]
- [Source: game-architecture.md — Error Handling Strategy: save load/write graceful degradation]
- [Source: game-architecture.md — Infrastructure Layer: save/ directory structure]
- [Source: game-architecture.md — Delta-Save Pattern: SeededEntityId, ChunkDelta (for Story 1.9, NOT this story)]
- [Source: game-architecture.md — Architectural Boundaries rule #5: Only InfrastructurePlugin writes save files]
- [Source: game-architecture.md — System Ordering: CoreSet::Events runs after all gameplay]
- [Source: epics.md — Epic 1 Story 8: "my position and world state are saved when I quit"]
- [Source: epics.md — Epic 1 Story 9: "delta-based save" — FUTURE story, do NOT implement deltas here]
- [Source: 1-7-event-system.md — MessageWriter .write() pattern, EventSeverityConfig.validate()]
- [Source: src/infrastructure/mod.rs — InfrastructurePlugin plugin pattern]
- [Source: src/core/input.rs — ActionState pattern, read_input key bindings]
- [Source: src/world/chunk.rs — ChunkCoord derives (needs Serialize, Deserialize added)]

## Dev Agent Record

### Agent Model Used

Claude Opus 4.6

### Debug Log References

- Fixed `ron::Value::Map` iteration (ron 0.10 API) — replaced with serde-based `VersionHeader` approach for `check_version`
- Fixed `&World` → `&mut World` for `PlayerSave::from_world` (Bevy 0.18 query requires mutable world)
- Updated EventSeverityConfig default mapping count assertions from 7 to 8

### Completion Notes List

- All 8 tasks completed with 280 tests passing (174 unit + 106 integration), 0 regressions
- 18 new tests added: 13 unit tests (schema, player_save, world_save, save config/state) + 5 integration tests (save/load cycles, missing files, corrupt files, file creation)
- Split file architecture: player.ron (fixed-size player state) + world.ron (explored chunks)
- Schema versioning from day one with SAVE_VERSION=1 and check_version validation
- Graceful degradation: corrupt/missing files logged with warn!, game starts fresh
- RON format with PrettyConfig for human-readable saves
- F5 key binding for save trigger, GameSaved event (Tier2) emitted on save
- All Serialize/Deserialize derives are additive — no existing behavior changed

### Change Log

- 2026-02-27: Implemented save system — Tasks 1-8 complete. 280 tests, 0 regressions.
- 2026-02-27: Code review fixes — Centralized conversion logic (from_components/apply_to_components, from_resources/apply_to_explored, biome_to_str/str_to_biome). Fixed path construction with PathBuf::join. Added GameSaved event emission test. Fixed stale comment. 282 tests, 0 regressions.

### File List

**NEW:**
- `src/infrastructure/save/mod.rs` — SavePlugin, SaveConfig, SaveState, save_game, load_game
- `src/infrastructure/save/schema.rs` — SAVE_VERSION, SaveError, VersionHeader, check_version
- `src/infrastructure/save/player_save.rs` — PlayerSave struct + serialization + from_world/apply_to_world
- `src/infrastructure/save/world_save.rs` — WorldSave struct + serialization + from_world/apply_to_world
- `tests/save_system.rs` — 5 integration tests

**MODIFIED:**
- `src/infrastructure/mod.rs` — Added `pub mod save;`, wired SavePlugin into InfrastructurePlugin
- `src/world/chunk.rs` — Added `Serialize, Deserialize` derives + serde import to ChunkCoord
- `src/world/generation.rs` — Added `Serialize, Deserialize` derives + serde import to BiomeType
- `src/core/weapons.rs` — Added `Serialize, Deserialize` to ActiveWeapon, updated serde import
- `src/core/input.rs` — Added `save: bool` to ActionState, F5 key binding in read_input
- `src/shared/events.rs` — Added `GameSaved` variant to GameEventKind
- `src/infrastructure/events.rs` — Added GameSaved to default mappings (Tier2), severity_for match, validate known_keys; updated mapping count assertions
- `assets/config/event_severity.ron` — Added `"GameSaved": Tier2`
