# Story 1.7: Event System

Status: done

## Story

As a developer,
I want all game systems to emit events via Bevy's message system,
so that future systems (logbook, telemetry) can subscribe without code changes.

## Acceptance Criteria

1. **GameEvent message type** — A `GameEvent` message struct exists in `src/shared/events.rs` with fields: `kind: GameEventKind`, `severity: EventSeverity`, `position: Vec2`, `game_time: f64`. Derives `Message`.
2. **GameEventKind enum** — Covers all current gameplay-relevant state changes: `EnemyDestroyed`, `PlayerDeath`, `PlayerRespawned`, `ChunkLoaded`, `ChunkUnloaded`, `WeaponFired`, `WeaponSwitched`. Each variant carries relevant context data.
3. **EventSeverity enum** — Three tiers: `Tier1` (critical — always shown), `Tier2` (notable), `Tier3` (minor). Derives `Deserialize` for RON config.
4. **Severity config** — `assets/config/event_severity.ron` maps `GameEventKind` variants to `EventSeverity`. Config loaded at startup with fallback defaults.
5. **Event-Observer system** — A system in `src/infrastructure/events.rs` reads all `GameEvent` messages each frame and appends matching entries to the `Logbook` resource. Runs in `CoreSet::Events`.
6. **Logbook resource** — `src/infrastructure/logbook.rs` provides `Logbook` with `Vec<LogbookEntry>`, capacity cap (`max_entries`), and query methods (`entries_by_severity`, `recent_entries`).
7. **Existing systems retrofitted** — `despawn_destroyed()`, `handle_player_death()`, `update_chunks()`, and `fire_weapon()`/`switch_weapon()` emit `GameEvent` messages at appropriate points. Existing behavior unchanged.
8. **InfrastructurePlugin** — New `src/infrastructure/mod.rs` registers the event-observer system, logbook resource, and severity config. Added to `game_plugins()` in `lib.rs`.
9. **Backward compatible** — All 239 existing tests pass. No changes to existing system signatures beyond adding `MessageWriter<GameEvent>` parameters.
10. **Test coverage** — Unit tests for severity config loading, logbook capacity, event-observer recording. Integration tests verifying events are emitted during gameplay (enemy destroyed, player death, chunk loading).

## Tasks / Subtasks

- [x] Task 1: Create `src/shared/events.rs` with core types (AC: #1, #2, #3)
  - [x] 1.1 Create `src/shared/events.rs` with `GameEvent`, `GameEventKind`, `EventSeverity`
  - [x] 1.2 `GameEvent` struct: `kind: GameEventKind`, `severity: EventSeverity`, `position: Vec2`, `game_time: f64`. Derive `Message, Clone, Debug`.
  - [x] 1.3 `GameEventKind` enum with context-carrying variants:
    - `EnemyDestroyed { entity_type: String }` — when non-player entity despawned at health <= 0
    - `PlayerDeath { position: Vec2 }` — when player health reaches 0
    - `PlayerRespawned` — when player reset to origin
    - `ChunkLoaded { coord: ChunkCoord, biome: BiomeType }` — when chunk enters ActiveChunks
    - `ChunkUnloaded { coord: ChunkCoord }` — when chunk removed from ActiveChunks
    - `WeaponFired { weapon: WeaponKind }` — when laser/spread fires (use existing `ActiveWeapon` or new `WeaponKind` enum)
    - `WeaponSwitched { from: WeaponKind, to: WeaponKind }` — when weapon toggles
  - [x] 1.4 `EventSeverity` enum: `Tier1`, `Tier2`, `Tier3`. Derive `Deserialize, Clone, Copy, Debug, PartialEq, Eq, Hash`.
  - [x] 1.5 Add `pub mod events;` to `src/shared/mod.rs`, re-export key types
  - [x] 1.6 Add `WeaponKind` enum (`Laser`, `Spread`) if not already present. Reuse existing `ActiveWeapon` if compatible.

- [x] Task 2: Create severity config (AC: #4)
  - [x] 2.1 Create `EventSeverityConfig` struct with `HashMap<String, EventSeverity>` (or enum-keyed map) + `#[derive(Deserialize)]`
  - [x] 2.2 Implement `Default` with sensible mappings: `PlayerDeath → Tier1`, `EnemyDestroyed → Tier3`, `ChunkLoaded → Tier3`, `WeaponFired → Tier3`, `WeaponSwitched → Tier3`, `PlayerRespawned → Tier2`, `ChunkUnloaded → Tier3`
  - [x] 2.3 Create `assets/config/event_severity.ron` with default mappings
  - [x] 2.4 Add `severity_for(&self, kind: &GameEventKind) -> EventSeverity` method with fallback to `Tier3`
  - [x] 2.5 Load in InfrastructurePlugin with RON fallback pattern (same as BiomeConfig, WorldConfig)

- [x] Task 3: Create Logbook resource (AC: #6)
  - [x] 3.1 Create `src/infrastructure/logbook.rs`
  - [x] 3.2 `LogbookEntry` struct: `kind: GameEventKind`, `severity: EventSeverity`, `game_time: f64`, `position: Vec2`. Derive `Clone, Debug`.
  - [x] 3.3 `Logbook` resource: `entries: Vec<LogbookEntry>`, `max_entries: usize` (default: 500)
  - [x] 3.4 Methods: `push(&mut self, entry: LogbookEntry)` — appends, drops oldest if at capacity
  - [x] 3.5 Methods: `entries_by_severity(&self, severity: EventSeverity) -> impl Iterator`
  - [x] 3.6 Methods: `recent_entries(&self, count: usize) -> &[LogbookEntry]`

- [x] Task 4: Create Event-Observer system (AC: #5)
  - [x] 4.1 Create `src/infrastructure/events.rs`
  - [x] 4.2 `record_game_events` system: reads `MessageReader<GameEvent>`, writes to `ResMut<Logbook>`
  - [x] 4.3 System constructs `LogbookEntry` from each `GameEvent` and pushes to Logbook
  - [x] 4.4 System runs in `CoreSet::Events` (after all other systems that emit events)

- [x] Task 5: Create InfrastructurePlugin (AC: #8)
  - [x] 5.1 Create `src/infrastructure/mod.rs` with `pub mod events; pub mod logbook;`
  - [x] 5.2 `InfrastructurePlugin` registers: `GameEvent` message, `Logbook` resource, `EventSeverityConfig` resource, `record_game_events` system
  - [x] 5.3 Add `pub mod infrastructure;` to `src/lib.rs`
  - [x] 5.4 Add `InfrastructurePlugin` to `game_plugins()` return tuple
  - [x] 5.5 Load `EventSeverityConfig` from `assets/config/event_severity.ron` with fallback

- [x] Task 6: Retrofit existing systems to emit GameEvent (AC: #7, #9)
  - [x] 6.1 `despawn_destroyed()` in `collision.rs` — emit `GameEvent { kind: EnemyDestroyed, .. }` for each non-player entity despawned. Add `MessageWriter<GameEvent>` + `Res<Time>` + `Res<EventSeverityConfig>` params.
  - [x] 6.2 `handle_player_death()` in `collision.rs` — emit `PlayerDeath` event when player dies, `PlayerRespawned` when reset. Add `MessageWriter<GameEvent>` param.
  - [x] 6.3 `update_chunks()` in `world/mod.rs` — emit `ChunkLoaded` for each newly loaded chunk, `ChunkUnloaded` for each unloaded chunk. Add `MessageWriter<GameEvent>` + `Res<EventSeverityConfig>` params.
  - [x] 6.4 `fire_weapon()` in `weapons.rs` — emit `WeaponFired` when laser/spread fires. Add `MessageWriter<GameEvent>` param.
  - [x] 6.5 `switch_weapon()` in `weapons.rs` — emit `WeaponSwitched` on toggle. Add `MessageWriter<GameEvent>` param.
  - [x] 6.6 Verify all 239 existing tests still pass after retrofit

- [x] Task 7: Update test harness (AC: #9, #10)
  - [x] 7.1 Update `tests/helpers/mod.rs` — register `GameEvent` message, `Logbook` resource, `EventSeverityConfig` resource, `record_game_events` system
  - [x] 7.2 Verify all existing tests pass with updated harness

- [x] Task 8: Unit tests (AC: #10)
  - [x] 8.1 `severity_config_default_maps_player_death_to_tier1` — default config returns Tier1 for PlayerDeath
  - [x] 8.2 `severity_config_from_ron_parses_correctly` — RON deserialization works
  - [x] 8.3 `severity_config_unknown_kind_falls_back_to_tier3` — unmapped kinds default
  - [x] 8.4 `logbook_push_appends_entry` — basic push works
  - [x] 8.5 `logbook_capacity_drops_oldest` — when max_entries reached, oldest entry removed
  - [x] 8.6 `logbook_entries_by_severity_filters` — filter returns only matching tier
  - [x] 8.7 `logbook_recent_entries_returns_last_n` — returns correct count from end
  - [x] 8.8 `record_game_events_writes_to_logbook` — event-observer system populates logbook

- [x] Task 9: Integration tests (AC: #10)
  - [x] 9.1 `enemy_destroyed_emits_game_event` — destroy an asteroid, verify Logbook has EnemyDestroyed entry
  - [x] 9.2 `player_death_emits_game_event` — kill player, verify PlayerDeath + PlayerRespawned entries
  - [x] 9.3 `chunk_loading_emits_game_events` — load world, verify ChunkLoaded entries in Logbook
  - [x] 9.4 `weapon_fire_emits_game_event` — fire laser, verify WeaponFired entry
  - [x] 9.5 `all_existing_tests_pass` — run full suite, 0 regressions

## Dev Notes

### Architecture Patterns & Constraints

- **Bevy 0.18 Message system** — Use `#[derive(Message)]`, `app.add_message::<T>()`, `MessageWriter<T>`, `MessageReader<T>`. NOT the old `Event`/`EventWriter`/`EventReader` API. This is the current Bevy 0.18 pattern already used for `LaserFired` and `SpreadFired`.
- **Core/Rendering separation** — Event types in `src/shared/events.rs`. Observer system in `src/infrastructure/events.rs`. No rendering changes.
- **No unwrap()** — `#[deny(clippy::unwrap_used)]` enforced crate-wide. Use `.expect()` in tests only.
- **Config backward compat** — `EventSeverityConfig` uses same RON fallback pattern as `BiomeConfig`, `WorldConfig`, `FlightConfig`.
- **System ordering** — `CoreSet::Events` already declared in `src/core/mod.rs:37`. The `record_game_events` system goes in this set. Emitting systems (collision, weapons, chunks) run in their existing sets BEFORE `CoreSet::Events`.
- **Graceful degradation** — If `event_severity.ron` is missing/corrupt, fall back to `Default`. Log warning.

### What Changes vs What Stays

**NEW FILES:**
- `src/shared/events.rs` — `GameEvent`, `GameEventKind`, `EventSeverity`, `WeaponKind`
- `src/infrastructure/mod.rs` — `InfrastructurePlugin`
- `src/infrastructure/events.rs` — `record_game_events` system, `EventSeverityConfig`
- `src/infrastructure/logbook.rs` — `Logbook`, `LogbookEntry`
- `assets/config/event_severity.ron` — Severity mapping config
- `tests/event_system.rs` — Integration tests

**MODIFIED FILES:**
- `src/shared/mod.rs` — Add `pub mod events;`
- `src/lib.rs` — Add `pub mod infrastructure;`, add `InfrastructurePlugin` to `game_plugins()`
- `src/core/collision.rs` — Add `MessageWriter<GameEvent>` to `despawn_destroyed()` and `handle_player_death()`, emit events
- `src/core/weapons.rs` — Add `MessageWriter<GameEvent>` to `fire_weapon()` and `switch_weapon()`, emit events
- `src/world/mod.rs` — Add `MessageWriter<GameEvent>` to `update_chunks()`, emit ChunkLoaded/ChunkUnloaded events
- `tests/helpers/mod.rs` — Register GameEvent message, Logbook, EventSeverityConfig, record_game_events system

**STAYS THE SAME:**
- All rendering systems — no changes
- Flight physics, camera, input — no event emission needed yet
- `src/world/generation.rs`, `noise_layers.rs`, `chunk.rs` — no changes
- `src/core/spawning.rs` — no changes (spawning emits via chunk loading)
- All existing message types (`LaserFired`, `SpreadFired`) — unchanged

### Implementation Guidance

**Message struct pattern (Bevy 0.18):**
```rust
use bevy::ecs::message::Message;

#[derive(Message, Clone, Debug)]
pub struct GameEvent {
    pub kind: GameEventKind,
    pub severity: EventSeverity,
    pub position: Vec2,
    pub game_time: f64,
}
```

**Emitting from existing systems (example: despawn_destroyed):**
```rust
pub fn despawn_destroyed(
    mut commands: Commands,
    query: Query<(Entity, &Health, &Transform), Without<Player>>,
    mut destroyed_positions: ResMut<DestroyedPositions>,
    mut game_events: MessageWriter<GameEvent>,
    time: Res<Time>,
    severity_config: Res<EventSeverityConfig>,
) {
    for (entity, health, transform) in &query {
        if health.current <= 0.0 {
            let pos = Vec2::new(transform.translation.x, transform.translation.y);
            destroyed_positions.0.push(pos);
            commands.entity(entity).despawn();

            let kind = GameEventKind::EnemyDestroyed { entity_type: "asteroid".to_string() };
            game_events.send(GameEvent {
                severity: severity_config.severity_for(&kind),
                kind,
                position: pos,
                game_time: time.elapsed_secs_f64(),
            });
        }
    }
}
```

**Severity config RON pattern:**
```rust
// assets/config/event_severity.ron
(
    mappings: {
        "EnemyDestroyed": Tier3,
        "PlayerDeath": Tier1,
        "PlayerRespawned": Tier2,
        "ChunkLoaded": Tier3,
        "ChunkUnloaded": Tier3,
        "WeaponFired": Tier3,
        "WeaponSwitched": Tier3,
    },
)
```

**Observer system pattern:**
```rust
pub fn record_game_events(
    mut events: MessageReader<GameEvent>,
    mut logbook: ResMut<Logbook>,
) {
    for event in events.read() {
        logbook.push(LogbookEntry {
            kind: event.kind.clone(),
            severity: event.severity,
            game_time: event.game_time,
            position: event.position,
        });
    }
}
```

**Entity type detection:** When emitting `EnemyDestroyed`, check for `Asteroid` or `ScoutDrone` marker components to set `entity_type` string. Use `With<Asteroid>`/`With<ScoutDrone>` in the query or check component presence.

### Previous Story Intelligence (1-6: Noise Biome Distribution)

- **Config RON pattern** — `BiomeConfig::from_ron()` with fallback to `Default`. Same pattern for `EventSeverityConfig`.
- **`#[serde(default)]`** — Used for backward-compatible config fields. Apply to `EventSeverityConfig` fields.
- **Test harness** — `tests/helpers/mod.rs` needs updates: register new message type, resources, and system.
- **`run_until_loaded()` helper** — Integration tests that need chunk events should use this.
- **239 tests** — All must pass. Run `cargo test` before and after.
- **Code review feedback (1-6):** Validate config parameters (added `BiomeNoiseConfig::validate()`). Consider similar for `EventSeverityConfig`.
- **Message system:** `app.add_message::<T>()` pattern used for `LaserFired`/`SpreadFired` in `CorePlugin`. Follow identical registration pattern.

### Key Files to Touch

| File | Action |
|------|--------|
| `src/shared/events.rs` | CREATE — GameEvent, GameEventKind, EventSeverity, WeaponKind |
| `src/shared/mod.rs` | MODIFY — add `pub mod events;` |
| `src/infrastructure/mod.rs` | CREATE — InfrastructurePlugin |
| `src/infrastructure/events.rs` | CREATE — record_game_events, EventSeverityConfig |
| `src/infrastructure/logbook.rs` | CREATE — Logbook, LogbookEntry |
| `src/lib.rs` | MODIFY — add infrastructure module, update game_plugins() |
| `src/core/collision.rs` | MODIFY — emit GameEvent from despawn_destroyed, handle_player_death |
| `src/core/weapons.rs` | MODIFY — emit GameEvent from fire_weapon, switch_weapon |
| `src/world/mod.rs` | MODIFY — emit GameEvent from update_chunks |
| `assets/config/event_severity.ron` | CREATE — severity mapping |
| `tests/helpers/mod.rs` | MODIFY — register event infrastructure |
| `tests/event_system.rs` | CREATE — integration tests |

### References

- [Source: _bmad-output/game-architecture.md — Event System section: GameEvent, EventSeverity, GameEventKind types]
- [Source: _bmad-output/game-architecture.md — Novel Pattern #4: Event-Severity Logbook]
- [Source: _bmad-output/game-architecture.md — Cross-cutting: System Ordering, CoreSet::Events]
- [Source: _bmad-output/game-architecture.md — Project Structure: src/shared/events.rs, src/infrastructure/events.rs, src/infrastructure/logbook.rs]
- [Source: _bmad-output/game-architecture.md — Consistency Rules: "All gameplay-relevant state changes MUST emit a GameEvent"]
- [Source: _bmad-output/epics.md — Epic 1 Story 7: "all game systems emit events via Bevy's event system"]
- [Source: src/core/weapons.rs:133-147 — Existing Message pattern: LaserFired, SpreadFired]
- [Source: src/core/mod.rs:27-41 — CoreSet enum with Events placeholder]
- [Source: src/core/collision.rs — DamageQueue, DestroyedPositions, despawn_destroyed, handle_player_death]
- [Source: 1-6-noise-biome-distribution.md — Config RON fallback pattern, serde(default), test harness]

## Dev Agent Record

### Agent Model Used
Claude Opus 4.6

### Debug Log References
- MessageWriter uses `.write()` not `.send()` in Bevy 0.18 (corrected during implementation)
- Existing unit tests for `switch_weapon`, `handle_player_death` needed `GameEvent` message + `EventSeverityConfig` resource registration
- `switch_weapon` query changed to include `Transform` for position in emitted events
- Integration tests in `chunk_loading.rs` and `world_generation.rs` with manual App setups needed event infrastructure registration

### Completion Notes List
- All 9 tasks completed with 20 new tests (15 unit + 5 integration)
- Total test count: 259 (was 239), 0 failures, 0 regressions
- Clippy clean with `-D warnings`
- New `WeaponKind` enum created (decoupled from `ActiveWeapon`) for event system independence
- `despawn_destroyed` detects entity type via `Asteroid`/`ScoutDrone` marker components
- `handle_player_death` emits both `PlayerDeath` and `PlayerRespawned` in sequence
- `update_chunks` emits `ChunkLoaded` and `ChunkUnloaded` with world-space position via `chunk_to_world_center`

### File List
- `src/shared/events.rs` — NEW: GameEvent, GameEventKind, EventSeverity, WeaponKind + 5 unit tests
- `src/shared/mod.rs` — MODIFIED: added `pub mod events;`
- `src/infrastructure/mod.rs` — NEW: InfrastructurePlugin
- `src/infrastructure/events.rs` — NEW: EventSeverityConfig, record_game_events + 5 unit tests
- `src/infrastructure/logbook.rs` — NEW: Logbook, LogbookEntry + 5 unit tests
- `src/lib.rs` — MODIFIED: added `pub mod infrastructure;`, InfrastructurePlugin to game_plugins()
- `src/core/collision.rs` — MODIFIED: despawn_destroyed emits EnemyDestroyed, handle_player_death emits PlayerDeath+PlayerRespawned, updated unit tests
- `src/core/weapons.rs` — MODIFIED: fire_weapon emits WeaponFired, switch_weapon emits WeaponSwitched, updated unit tests
- `src/world/mod.rs` — MODIFIED: update_chunks emits ChunkLoaded/ChunkUnloaded
- `assets/config/event_severity.ron` — NEW: severity mappings
- `tests/helpers/mod.rs` — MODIFIED: registered GameEvent, Logbook, EventSeverityConfig, record_game_events
- `tests/event_system.rs` — NEW: 5 integration tests
- `tests/chunk_loading.rs` — MODIFIED: added event infrastructure to manual App setups
- `tests/world_generation.rs` — MODIFIED: added event infrastructure to manual App setups

## Senior Developer Review (AI)

**Reviewer:** Simon on 2026-02-27
**Outcome:** Approved with fixes applied

### Review Findings (4 MEDIUM, 3 LOW)

**Fixed (MEDIUM):**
1. **Logbook `push()` O(n)** — Changed `Vec` to `VecDeque` for O(1) front-removal at capacity
2. **String allocation in EnemyDestroyed** — Changed `entity_type: String` to `&'static str` to avoid heap allocation per event
3. **Redundant `position` in `PlayerDeath`** — Removed `position: Vec2` from variant (already in `GameEvent.position`)
4. **EventSeverityConfig lacks validate()** — Added `validate()` method warning about unknown/missing mapping keys

**Not fixed (LOW):**
5. No test for `ChunkUnloaded` event emission
6. `update_chunks` has 13 system parameters
7. `"unknown"` fallback in entity_type could mask future bugs

### Post-Review Stats
- Issues fixed: 4 MEDIUM
- New tests added: 3 (validate config tests)
- Total test count: 262 (was 259), 0 failures
- Clippy clean with `-D warnings`

## Change Log
- 2026-02-27: Implemented event system (Story 1.7) — GameEvent message type, EventSeverityConfig, Logbook resource, InfrastructurePlugin. Retrofitted despawn_destroyed, handle_player_death, update_chunks, fire_weapon, switch_weapon to emit events. 20 new tests (15 unit + 5 integration), 259 total tests passing.
- 2026-02-27: Code review fixes — Logbook VecDeque, EnemyDestroyed &'static str, PlayerDeath position removal, EventSeverityConfig::validate(). 3 new tests, 262 total.
