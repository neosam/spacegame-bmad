# Story 1.9: Delta-Based Save

Status: done

## Story

As a player,
I want the save to be delta-based (seed + changes),
so that save files stay small even after hours of play.

## Acceptance Criteria

1. **SeedIndex component** — New `SeedIndex(pub u32)` component in `src/infrastructure/save/delta.rs`. Assigned to every entity spawned from `generate_chunk_content` blueprints during chunk loading, with index matching its position in the deterministic blueprint list. Entities spawned by other systems (RespawnTimer, projectiles) do NOT get a SeedIndex.
2. **Delta types** — `SeededEntityId { chunk: ChunkCoord, index: u32 }` and `ChunkDelta { coord: ChunkCoord, destroyed: Vec<u32> }` in `src/infrastructure/save/delta.rs`. Both `#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]`.
3. **WorldDeltas resource** — `WorldDeltas { deltas: HashMap<ChunkCoord, ChunkDelta> }` resource registered by SavePlugin. Tracks per-chunk entity destructions across sessions.
4. **Destroyed entity tracking** — New system `track_destroyed_entities` runs after `despawn_destroyed`. When an entity with both `ChunkEntity` + `SeedIndex` has `Health.current <= 0`, its seed index is added to that chunk's `ChunkDelta.destroyed` in `WorldDeltas`. Chunk-unloading despawns (from `update_chunks`) must NOT be recorded as deltas.
5. **Extend WorldSave** — Add `chunk_deltas: Vec<ChunkDelta>` to `WorldSave`. `WorldSave::from_resources` accepts `&WorldDeltas` and serializes non-empty deltas. `WorldSave::apply_to_world_resources` restores both `ExploredChunks` and `WorldDeltas`.
6. **Schema version 2** — Bump `SAVE_VERSION` from 1 to 2. `check_version` accepts version 1 (migration) or 2 (current). New world.ron files written with version 2.
7. **Migration v1→v2** — New `src/infrastructure/save/migration.rs` with `migrate_world_v1_to_v2(ron_str: &str) -> Result<String, SaveError>`. Injects empty `chunk_deltas: []` into v1 RON data. `WorldSave::from_ron` detects v1 and auto-migrates.
8. **Apply deltas on chunk load** — In `update_chunks` (or a new system running immediately after entity spawning), after generating blueprints from seed, filter out entities whose blueprint index is in `WorldDeltas.deltas[coord].destroyed` before spawning.
9. **Property-based roundtrip** — Add `proptest` as dev dependency. New `tests/delta_roundtrip.rs`: generate chunk from seed → apply random destructions → compute delta → regenerate from seed → apply delta → verify result matches modified state.
10. **Test coverage** — Unit tests: ChunkDelta/SeededEntityId serialization roundtrip, SeedIndex assignment during spawn, migration v1→v2, WorldDeltas tracking. Integration tests: destroy entity → save → load → entity stays destroyed; v1 save file loads with migration; empty deltas produce same behavior as no deltas.
11. **Backward compatible** — All 282 existing tests pass unchanged. v1 save files auto-migrate on load. SeedIndex is additive (no existing component queries affected).
12. **Golden fixture** — `tests/fixtures/saves/test_world_v1.ron` containing a valid v1 WorldSave for migration testing.

## Tasks / Subtasks

- [x] Task 1: Create delta types and SeedIndex component (AC: #1, #2, #3)
  - [x] 1.1 Create `src/infrastructure/save/delta.rs` with `SeededEntityId`, `ChunkDelta`, `WorldDeltas`
  - [x] 1.2 Add `SeedIndex(pub u32)` component with `#[derive(Component)]`
  - [x] 1.3 Add `pub mod delta;` to `src/infrastructure/save/mod.rs`
  - [x] 1.4 Register `WorldDeltas` resource in SavePlugin

- [x] Task 2: Assign SeedIndex during chunk entity spawning (AC: #1)
  - [x] 2.1 In `src/world/mod.rs` `update_chunks`, when spawning entities from `generate_chunk_content` blueprints, add `SeedIndex(i)` where `i` is the blueprint's index in the Vec
  - [x] 2.2 Import `SeedIndex` in world module
  - [x] 2.3 Verify existing chunk tests still pass (SeedIndex is additive, no query impact)

- [x] Task 3: Track destroyed entities in WorldDeltas (AC: #4)
  - [x] 3.1 Create `track_destroyed_entities` system in `src/infrastructure/save/delta.rs`
  - [x] 3.2 System queries entities with `(ChunkEntity, SeedIndex, Health)` where `health.current <= 0`
  - [x] 3.3 For each matching entity, insert seed index into `WorldDeltas.deltas[chunk_coord].destroyed`
  - [x] 3.4 Register system in SavePlugin: `FixedUpdate`, after `apply_damage`, before `despawn_destroyed`
  - [x] 3.5 CRITICAL: Only track health-based destruction. Do NOT record chunk-unloading despawns.

- [x] Task 4: Extend WorldSave with chunk_deltas (AC: #5, #6)
  - [x] 4.1 Add `chunk_deltas: Vec<ChunkDelta>` field to `WorldSave` with `#[serde(default)]`
  - [x] 4.2 Update `WorldSave::from_resources` signature to accept `&WorldDeltas`; serialize non-empty deltas
  - [x] 4.3 Add `WorldSave::apply_to_world_resources(&self, explored: &mut ExploredChunks, deltas: &mut WorldDeltas)` — restores both
  - [x] 4.4 Bump `SAVE_VERSION` from 1 to 2 in `schema.rs`
  - [x] 4.5 Update `save_game` to pass `WorldDeltas` to `WorldSave::from_resources`
  - [x] 4.6 Update `load_game` to restore `WorldDeltas` from loaded WorldSave

- [x] Task 5: Schema migration v1→v2 (AC: #7, #12)
  - [x] 5.1 Create `src/infrastructure/save/migration.rs`
  - [x] 5.2 Implement `migrate_world_v1_to_v2` — deserialize v1 WorldSave, add empty chunk_deltas, re-serialize as v2
  - [x] 5.3 Update `check_version` in `schema.rs`: accept v1 (needs migration) OR v2 (current). Return version found.
  - [x] 5.4 Update `WorldSave::from_ron`: if v1 detected, call migration first, then deserialize
  - [x] 5.5 Create `tests/fixtures/saves/test_world_v1.ron` golden fixture (valid v1 WorldSave)
  - [x] 5.6 Add `pub mod migration;` to `src/infrastructure/save/mod.rs`

- [x] Task 6: Apply deltas on chunk load (AC: #8)
  - [x] 6.1 In `update_chunks`, after `generate_chunk_content`, read `WorldDeltas` for the chunk
  - [x] 6.2 Filter blueprint Vec: skip entries whose index is in `destroyed` list
  - [x] 6.3 Spawn only non-destroyed blueprints (with SeedIndex assigned)
  - [x] 6.4 When chunk is unloaded: do NOT clear its delta (destroyed entities stay destroyed across sessions)

- [x] Task 7: Unit tests (AC: #10, #11)
  - [x] 7.1 `seeded_entity_id_roundtrip` — serialize → deserialize → fields match
  - [x] 7.2 `chunk_delta_roundtrip` — serialize → deserialize → destroyed list matches
  - [x] 7.3 `world_deltas_tracks_destroyed_entity` — add entry, verify in deltas
  - [x] 7.4 `world_save_with_deltas_roundtrip` — WorldSave with chunk_deltas serializes/deserializes
  - [x] 7.5 `migration_v1_to_v2` — load golden fixture, verify migration produces valid v2
  - [x] 7.6 `check_version_accepts_v1_and_v2` — both versions accepted by check_version
  - [x] 7.7 `seed_index_assigned_during_chunk_spawn` — build App, load chunk, verify entities have SeedIndex

- [x] Task 8: Integration + property-based tests (AC: #9, #10)
  - [x] 8.1 Add `proptest` dev dependency to Cargo.toml
  - [x] 8.2 Create `tests/delta_roundtrip.rs` with proptest roundtrip test
  - [x] 8.3 `destroy_entity_then_save_load_stays_destroyed` — kill asteroid → save → fresh app → load → spawn chunk → verify entity missing
  - [x] 8.4 `v1_save_loads_with_empty_deltas` — write v1 world.ron → load → verify no errors, empty WorldDeltas
  - [x] 8.5 `empty_deltas_same_as_no_deltas` — save with empty WorldDeltas → load → all entities spawn normally

## Dev Notes

### Architecture Patterns & Constraints

- **Delta-Save Pattern** — Seed reproduces base world; only deviations are persisted. `ChunkDelta.destroyed` stores indices of seed-generated entities killed by the player. [Source: game-architecture.md — Delta-Save Pattern]
- **SeededEntityId = (ChunkCoord, u32)** — Deterministic identity: same seed + chunk = same generation order = same index. [Source: game-architecture.md lines 867-881]
- **Only InfrastructurePlugin writes save files** — Architectural Boundary rule #5. Delta tracking runs in Infrastructure layer. [Source: game-architecture.md — Architectural Boundaries]
- **No unwrap()** — `#[deny(clippy::unwrap_used)]` enforced crate-wide. Use `.expect()` in tests only.
- **RON over Binary** — ADR-002: Readability beats performance at <1MB. [Source: game-architecture.md — ADR-002]
- **Schema versioning with migration** — Every save file has a version header. Migration functions handle old versions. [Source: game-architecture.md — Data Persistence]
- **Property-based testing** — `proptest` for delta roundtrip is CRITICAL for save system validation. [Source: game-architecture.md — Delta-Save Pattern, Critical Rules]
- **Graceful degradation** — Corrupt chunk deltas → discard, regenerate from seed, log warning. Never lose `player.ron`. [Source: game-architecture.md — Error Handling Strategy]

### What Changes vs What Stays

**NEW FILES:**
- `src/infrastructure/save/delta.rs` — SeededEntityId, ChunkDelta, WorldDeltas, SeedIndex, track_destroyed_entities
- `src/infrastructure/save/migration.rs` — v1→v2 migration function
- `tests/delta_roundtrip.rs` — Property-based proptest roundtrip
- `tests/fixtures/saves/test_world_v1.ron` — Golden v1 save fixture

**MODIFIED FILES:**
- `Cargo.toml` — Add `proptest` dev dependency
- `src/infrastructure/save/mod.rs` — Add `pub mod delta; pub mod migration;`, register WorldDeltas, update save_game/load_game
- `src/infrastructure/save/schema.rs` — Bump SAVE_VERSION to 2, update check_version to accept v1+v2
- `src/infrastructure/save/world_save.rs` — Add `chunk_deltas` field, update from_resources/apply signatures
- `src/world/mod.rs` — Assign SeedIndex during entity spawning, filter by WorldDeltas on load

**STAYS THE SAME:**
- `src/infrastructure/save/player_save.rs` — No changes (player.ron schema unchanged)
- All core gameplay systems — no changes
- All rendering systems — no changes
- Flight physics, camera, collision — no changes
- `despawn_destroyed` — no changes (new system reads its results, doesn't modify it)

### Implementation Guidance

**SeedIndex assignment pattern (in update_chunks):**
```rust
use crate::infrastructure::save::delta::SeedIndex;

// When spawning from blueprints:
for (i, blueprint) in blueprints.iter().enumerate() {
    // ... existing spawn code ...
    commands.spawn((
        // existing components...
        SeedIndex(i as u32),
    ));
}
```

**Delta tracking system pattern:**
```rust
pub fn track_destroyed_entities(
    query: Query<(&ChunkEntity, &SeedIndex, &Health)>,
    mut world_deltas: ResMut<WorldDeltas>,
) {
    for (chunk_entity, seed_index, health) in query.iter() {
        if health.current <= 0.0 {
            let delta = world_deltas.deltas
                .entry(chunk_entity.coord)
                .or_insert_with(|| ChunkDelta {
                    coord: chunk_entity.coord,
                    destroyed: Vec::new(),
                });
            if !delta.destroyed.contains(&seed_index.0) {
                delta.destroyed.push(seed_index.0);
            }
        }
    }
}
```

**Blueprint filtering pattern (in update_chunks):**
```rust
let world_deltas = world.resource::<WorldDeltas>();
let destroyed = world_deltas.deltas.get(&coord)
    .map(|d| &d.destroyed)
    .unwrap_or(&Vec::new());

for (i, blueprint) in blueprints.iter().enumerate() {
    if destroyed.contains(&(i as u32)) {
        continue; // Skip destroyed entities
    }
    // ... spawn entity with SeedIndex(i as u32) ...
}
```

**Migration pattern:**
```rust
pub fn migrate_world_v1_to_v2(ron_str: &str) -> Result<String, SaveError> {
    // Deserialize v1 format (no chunk_deltas field)
    #[derive(Deserialize)]
    struct WorldSaveV1 {
        schema_version: u32,
        seed: u64,
        explored_chunks: Vec<((i32, i32), String)>,
    }

    let v1: WorldSaveV1 = ron::from_str(ron_str)
        .map_err(|e| SaveError::ParseError(format!("{e}")))?;

    // Build v2 with empty deltas
    let v2 = WorldSave {
        schema_version: 2,
        seed: v1.seed,
        explored_chunks: v1.explored_chunks,
        chunk_deltas: Vec::new(),
    };

    v2.to_ron()
}
```

**WorldSave::from_ron with migration:**
```rust
pub fn from_ron(ron_str: &str) -> Result<Self, SaveError> {
    let version = check_version(ron_str)?;
    if version == 1 {
        let migrated = migration::migrate_world_v1_to_v2(ron_str)?;
        return ron::from_str(&migrated)
            .map_err(|e| SaveError::ParseError(format!("{e}")));
    }
    ron::from_str(ron_str).map_err(|e| SaveError::ParseError(format!("{e}")))
}
```

**Proptest roundtrip pattern:**
```rust
use proptest::prelude::*;
use proptest::collection::vec;

proptest! {
    #[test]
    fn delta_roundtrip(
        seed: u64,
        destroyed_indices in vec(0u32..20, 0..10),
    ) {
        let config = BiomeConfig::default();
        let coord = ChunkCoord { x: 0, y: 0 };
        let biome = determine_biome(seed, coord, &config);
        let blueprints = generate_chunk_content(seed, coord, 1000.0, biome, &config);

        // Create delta from random destruction indices (clamped to actual count)
        let valid_destroyed: Vec<u32> = destroyed_indices.into_iter()
            .filter(|&i| (i as usize) < blueprints.len())
            .collect();

        let delta = ChunkDelta { coord, destroyed: valid_destroyed.clone() };

        // Apply delta: filter blueprints
        let surviving: Vec<_> = blueprints.iter().enumerate()
            .filter(|(i, _)| !delta.destroyed.contains(&(*i as u32)))
            .map(|(_, bp)| bp.clone())
            .collect();

        // Regenerate and apply delta again
        let blueprints2 = generate_chunk_content(seed, coord, 1000.0, biome, &config);
        let surviving2: Vec<_> = blueprints2.iter().enumerate()
            .filter(|(i, _)| !delta.destroyed.contains(&(*i as u32)))
            .map(|(_, bp)| bp.clone())
            .collect();

        // Roundtrip: same survivors
        prop_assert_eq!(surviving.len(), surviving2.len());
    }
}
```

### Previous Story Intelligence (1-8: Save System)

- **`from_components`/`apply_to_components` pattern** — Added in code review. Save/load conversion uses single-source-of-truth methods. Follow same pattern for WorldDeltas.
- **`biome_to_str`/`str_to_biome` helpers** — Centralized in `world_save.rs`. Reuse for delta serialization.
- **`PathBuf::join` for paths** — Fixed in code review. All file paths use `Path::new().join()`, not `format!()`.
- **`#[serde(default)]`** — Use on `chunk_deltas` field for backward compatibility during migration.
- **MessageWriter uses `.write()` not `.send()`** — Corrected in 1.7. Same pattern if adding new events.
- **EventSeverityConfig has 8 known_keys** — If adding WorldModified variant, update known_keys, default mappings, severity_for match.
- **Test harness** — `tests/helpers/mod.rs` provides `test_app()` with full game systems. Save tests need `SaveConfig`, `SaveState`, `WorldDeltas` registered.
- **282 tests pass** — Must remain green after delta-save additions.

### Key Files to Touch

| File | Action |
|------|--------|
| `src/infrastructure/save/delta.rs` | CREATE — SeededEntityId, ChunkDelta, WorldDeltas, SeedIndex, track_destroyed_entities |
| `src/infrastructure/save/migration.rs` | CREATE — migrate_world_v1_to_v2 |
| `tests/delta_roundtrip.rs` | CREATE — proptest roundtrip test |
| `tests/fixtures/saves/test_world_v1.ron` | CREATE — golden v1 save fixture |
| `Cargo.toml` | MODIFY — add proptest dev dep |
| `src/infrastructure/save/mod.rs` | MODIFY — add delta/migration modules, register WorldDeltas, update save_game/load_game |
| `src/infrastructure/save/schema.rs` | MODIFY — bump SAVE_VERSION to 2, update check_version |
| `src/infrastructure/save/world_save.rs` | MODIFY — add chunk_deltas field, update from_resources/apply signatures |
| `src/world/mod.rs` | MODIFY — assign SeedIndex, filter by WorldDeltas on load |

### References

- [Source: game-architecture.md — Delta-Save Pattern: SeededEntityId, ChunkDelta, data flow]
- [Source: game-architecture.md — ADR-002: RON over Binary for Saves]
- [Source: game-architecture.md — Error Handling Strategy: corrupt deltas → regenerate from seed]
- [Source: game-architecture.md — Infrastructure Layer: save/ directory structure with migration.rs]
- [Source: game-architecture.md — Consistency Rules: SeededEntityId = (ChunkCoord, u32)]
- [Source: game-architecture.md — Data Persistence: split file architecture, schema versioning]
- [Source: game-architecture.md — Test Structure: delta_roundtrip.rs, save_migration.rs, fixtures/]
- [Source: epics.md — Epic 1 Story 9: "delta-based save (seed + changes) for small files"]
- [Source: gdd.md — Save File Size Under 1MB for 10+ hours, delta-based saves]
- [Source: 1-8-save-system.md — from_components pattern, biome helpers, PathBuf fix, 282 tests]
- [Source: src/world/mod.rs — update_chunks entity spawning, ChunkEntityIndex]
- [Source: src/core/collision.rs — Health component, despawn_destroyed system]

## Dev Agent Record

### Agent Model Used

Claude Opus 4.6

### Debug Log References

- Task 3.4 deviation: Story AC #4 specifies `track_destroyed_entities` runs "after `despawn_destroyed`", but entities would be gone by then. System is registered `after(apply_damage).before(despawn_destroyed)` instead — queries entities with Health <= 0 before they are despawned.
- `migrate_world_v1_to_v2` returns `Result<WorldSave, SaveError>` (not `Result<String, SaveError>` as AC #7 suggests) — avoids unnecessary double serialization.

### Completion Notes List

- Created delta-save system: SeedIndex, SeededEntityId, ChunkDelta, WorldDeltas types and track_destroyed_entities system
- Extended WorldSave with chunk_deltas field, updated from_resources/apply_to_world_resources signatures
- Bumped SAVE_VERSION from 1 to 2, check_version now accepts v1 (migration) and v2 (current)
- Created migration.rs with migrate_world_v1_to_v2, WorldSave::from_ron auto-migrates v1 saves
- Updated update_chunks to assign SeedIndex and filter destroyed entities via WorldDeltas
- Added proptest as dev dependency for property-based roundtrip testing
- Created golden v1 fixture at tests/fixtures/saves/test_world_v1.ron
- 304 total tests pass (282 original + 22 new). Zero regressions.

### File List

New files:
- `src/infrastructure/save/delta.rs`
- `src/infrastructure/save/migration.rs`
- `tests/delta_roundtrip.rs`
- `tests/fixtures/saves/test_world_v1.ron`

Modified files:
- `Cargo.lock`
- `Cargo.toml`
- `src/infrastructure/save/mod.rs`
- `src/infrastructure/save/schema.rs`
- `src/infrastructure/save/world_save.rs`
- `src/world/mod.rs`
- `tests/helpers/mod.rs`
- `tests/chunk_loading.rs`
- `tests/world_generation.rs`
- `tests/save_system.rs`

## Senior Developer Review (AI)

**Reviewer:** Simon (via adversarial code review workflow)
**Date:** 2026-02-27
**Outcome:** Approved after fixes

### Review 1 Findings (6 fixed, 4 low noted)
- **H1 FIXED:** `track_destroyed_entities` was missing from test_app — added to helpers/mod.rs chain + wrote E2E integration test `e2e_damage_track_save_load_filters_entity`
- **M1 FIXED:** Removed unused `WorldDeltas` import in delta_roundtrip.rs
- **M2 FIXED:** Added `warn!()` for missing WorldDeltas in `apply_to_world()` (world_save.rs)
- **M3 FIXED:** Documented `check_version` design pattern in schema.rs
- **M4 FIXED:** Added `chunk_x`/`chunk_y` proptest parameters to `delta_roundtrip`
- **M5 FIXED:** Added `Cargo.lock` to File List
- **L1 NOTED:** `SeededEntityId` is dead code in production (architecture placeholder)
- **L2 NOTED:** No test for `WorldSave::from_world` with non-empty deltas
- **L3 NOTED:** proptest `world_save_delta_roundtrip` can generate duplicate destroyed indices
- **L4 NOTED:** Migration unit test uses inline RON, not golden fixture

### Review 2 Findings (1 fixed, 4 low noted) — 2026-02-28
- **M1 FIXED:** `WorldSave::from_ron` now uses two-phase deserialization — corrupt chunk_deltas are discarded with `warn!()` while preserving explored_chunks (architecture: "corrupt deltas → regenerate from seed"). Added 2 tests: `world_save_corrupt_deltas_recovers_core_fields`, `world_save_fully_corrupt_still_errors`.
- **L1 NOTED:** `destroyed.contains()` in update_chunks is O(n) linear scan — HashSet would be faster at scale
- **L2 NOTED:** `destroyed` Vec within ChunkDelta is unsorted (non-deterministic serialization)
- **L3 NOTED:** E2E tests duplicate ~25 lines of app setup — could extract helper
- **L4 NOTED:** proptest `world_save_delta_roundtrip` only tests single chunk at (0,0)

## Change Log

- 2026-02-27: Story 1.9 Delta-Based Save implemented. Added delta types (SeedIndex, SeededEntityId, ChunkDelta, WorldDeltas), destroyed entity tracking, WorldSave chunk_deltas field, schema v1→v2 migration, delta filtering on chunk load, proptest roundtrip, golden v1 fixture. 22 new tests (304 total).
- 2026-02-27: Code review fixes — added track_destroyed_entities to test_app, E2E pipeline test, removed unused import, added warn!() for missing WorldDeltas, documented check_version design, expanded proptest coordinates, fixed File List. 305 total tests.
- 2026-02-28: Code review 2 fix — graceful degradation for corrupt chunk_deltas in WorldSave::from_ron (two-phase deserialization). 2 new tests. 307 total tests.
