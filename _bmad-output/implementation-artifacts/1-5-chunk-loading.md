# Story 1.5: Chunk Loading

Status: done

<!-- Note: Validation is optional. Run validate-create-story for quality check before dev-story. -->

## Story

As a developer,
I want the chunk system to efficiently load and unload chunks based on player position with indexed entity tracking and staggered loading,
so that memory stays bounded during extended exploration sessions.

## Acceptance Criteria

1. **ChunkEntityIndex** — A `ChunkEntityIndex` resource (`HashMap<ChunkCoord, Vec<Entity>>`) tracks all entities per chunk. Lookup for despawn is O(1) per chunk coordinate, not O(n) over all entities.
2. **Reliable Cleanup** — When a chunk unloads, ALL its entities are despawned via the index. The `ChunkEntityIndex` entry is removed after despawn. No entity leaks.
3. **Staggered Loading** — New chunks are loaded at most `max_chunks_per_frame` (configurable, default 4) per frame. Chunks exceeding this limit are queued in `PendingChunks` and loaded in subsequent frames.
4. **Load Priority** — Queued chunks are sorted by Manhattan distance from the player chunk (nearest first). Ties broken by deterministic `ChunkCoord` ordering (Ord).
5. **Pending Queue Refresh** — When the player moves to a new chunk, `PendingChunks` is recalculated from scratch: `desired_set - active_set`, sorted by distance. Stale entries from the previous position are discarded.
6. **Immediate Unload** — Chunks outside the load radius are ALWAYS unloaded in the same frame (not deferred). Only loading is staggered.
7. **Memory Bounded** — After simulating 100+ chunk transitions (extended exploration), total entity count never exceeds `entity_budget`. No memory growth beyond active chunk entities + ExploredChunks metadata.
8. **Config-Driven** — `max_chunks_per_frame` added to `WorldConfig` (loaded from `assets/config/world.ron` via existing `from_ron()` pattern). Default: 4. Existing fields unchanged.
9. **Backward Compatible** — Chunk generation, biome assignment, entity spawning behavior, and system scheduling are identical. Only the internal load/unload mechanism changes. All 220 existing tests pass.
10. **No Gameplay Impact** — Chunk loading optimization is invisible to the player. World generation, biome distribution, and entity behavior are identical. Partially loaded worlds (pending chunks) are a brief transient state, not a visible gap.

## Tasks / Subtasks

- [x] Task 1: Add `manhattan_distance` to chunk.rs (AC: #4)
  - [x] 1.1 Add `pub fn manhattan_distance(a: ChunkCoord, b: ChunkCoord) -> u32` to `src/world/chunk.rs`
  - [x] 1.2 Formula: `(a.x - b.x).unsigned_abs() + (a.y - b.y).unsigned_abs()`

- [x] Task 2: Add `ChunkEntityIndex` resource (AC: #1, #2)
  - [x] 2.1 Define `ChunkEntityIndex` resource in `src/world/mod.rs`: `pub chunks: HashMap<ChunkCoord, Vec<Entity>>`
  - [x] 2.2 Implement `entity_count(&self) -> usize` method — sum of all vec lengths
  - [x] 2.3 Initialize as empty in `WorldPlugin::build()`
  - [x] 2.4 Export from world module

- [x] Task 3: Add `PendingChunks` resource and extend `WorldConfig` (AC: #3, #4, #5, #8)
  - [x] 3.1 Define `PendingChunks` resource in `src/world/mod.rs`: `pub chunks: VecDeque<ChunkCoord>`
  - [x] 3.2 Add `max_chunks_per_frame: usize` to `WorldConfig` (default: 4)
  - [x] 3.3 Update `WorldConfig::default()` with new field
  - [x] 3.4 Update `assets/config/world.ron` with `max_chunks_per_frame: 4`
  - [x] 3.5 Initialize `PendingChunks` as empty in `WorldPlugin::build()`

- [x] Task 4: Refactor `update_chunks` system (AC: #1-#7, #9)
  - [x] 4.1 Add `ChunkEntityIndex` and `PendingChunks` as system parameters
  - [x] 4.2 **Unload phase:** Replace linear entity scan with `chunk_entity_index.chunks.remove(&coord)` → despawn all entities in returned Vec
  - [x] 4.3 **Queue phase:** On player chunk change (or first frame), calculate `desired - active - pending_set` → sort by `manhattan_distance` → replace `PendingChunks`
  - [x] 4.4 **Load phase:** Pop up to `max_chunks_per_frame` from `PendingChunks`, generate and spawn with budget enforcement
  - [x] 4.5 **Index maintenance:** On spawn, push entity into `chunk_entity_index.chunks[coord]`
  - [x] 4.6 **Budget tracking:** Kept `all_collidable` query for accurate total budget (includes non-chunk entities). Index used for fast despawn.
  - [x] 4.7 Track `last_player_chunk: Option<ChunkCoord>` in a `ChunkLoadState` resource to detect chunk changes
  - [x] 4.8 Maintain deterministic spawn order within each chunk (sorted coord, sequential blueprint)

- [x] Task 5: Add `ChunkLoadState` resource (AC: #5)
  - [x] 5.1 Define `ChunkLoadState` resource: `pub last_player_chunk: Option<ChunkCoord>`
  - [x] 5.2 Initialize in `WorldPlugin::build()`
  - [x] 5.3 Update in `update_chunks` after computing player chunk

- [x] Task 6: Update test harness (AC: #9)
  - [x] 6.1 Add `ChunkEntityIndex`, `PendingChunks`, `ChunkLoadState` to `tests/helpers/mod.rs` test_app()
  - [x] 6.2 Verify all 220 existing tests pass (all 220 pass + 16 new = 236 total)

- [x] Task 7: Unit tests — 10 tests (AC: all)
  - [x] 7.1 `manhattan_distance`: same chunk → 0
  - [x] 7.2 `manhattan_distance`: adjacent chunk → 1
  - [x] 7.3 `manhattan_distance`: diagonal → 2
  - [x] 7.4 `manhattan_distance`: negative coords correct
  - [x] 7.5 `ChunkEntityIndex::entity_count`: empty → 0
  - [x] 7.6 `ChunkEntityIndex::entity_count`: multiple chunks summed
  - [x] 7.7 `WorldConfig` RON parsing with `max_chunks_per_frame`
  - [x] 7.8 `WorldConfig` default includes `max_chunks_per_frame: 4`
  - [x] 7.9 `PendingChunks` default is empty
  - [x] 7.10 `ChunkLoadState` default is `None`

- [x] Task 8: Integration tests — 6 tests (AC: all)
  - [x] 8.1 `chunk_entity_index_populated_on_load` — After `update_chunks`, `ChunkEntityIndex` contains entries for all active chunks with correct entity counts
  - [x] 8.2 `chunk_entity_index_cleared_on_unload` — After moving player and unloading, index entries for old chunks are gone, no leaked entities
  - [x] 8.3 `staggered_loading_respects_max_per_frame` — With `max_chunks_per_frame: 2` and 25 desired chunks, only 2 load per frame. After 13 frames all 25 loaded.
  - [x] 8.4 `load_priority_nearest_first` — With `max_chunks_per_frame: 1`, first loaded chunk is the one closest to player (Manhattan distance)
  - [x] 8.5 `extended_exploration_memory_bounded` — Simulate 100 chunk transitions (move player 100 chunks in one direction, one chunk per frame). Assert entity count never exceeds budget. Assert no ChunkEntityIndex leak.
  - [x] 8.6 `pending_queue_refreshed_on_chunk_change` — Move player to new chunk while pending queue has entries. Assert pending queue is recalculated with new distances.

## Dev Notes

### Architecture Patterns & Constraints

- **Core/Rendering separation MUST be maintained:** All chunk loading changes are in `src/world/mod.rs`. Rendering systems (`minimap.rs`, `world_map.rs`) continue to read `ActiveChunks` and `ExploredChunks` unchanged.
- **FixedUpdate scheduling:** `update_chunks` stays in `FixedUpdate.before(CoreSet::Collision)`. No schedule changes.
- **No unwrap():** `#[deny(clippy::unwrap_used)]` enforced project-wide. Use `.expect("msg")` in tests.
- **Deferred commands:** Bevy `commands.entity(e).despawn()` is deferred. The `ChunkEntityIndex` must be cleaned up in the same frame as the despawn command to avoid stale references. This is safe because the index is the authoritative source — we don't need to query despawned entities.
- **System parameters:** The refactored `update_chunks` replaces `all_collidable: Query<Entity, With<Collider>>` with `chunk_entity_index: ResMut<ChunkEntityIndex>`. Budget is tracked via the index's `entity_count()` method, which is always accurate because the index is updated synchronously (not deferred).

### Implementation Strategy

**Phase 1: Add new resources (Tasks 1-3, 5)**
- Pure additive changes. No existing code modified yet.
- `manhattan_distance` is a pure function in `chunk.rs` — easy to unit test.
- `ChunkEntityIndex`, `PendingChunks`, `ChunkLoadState` added as new resources.
- `WorldConfig` gains one new field with backward-compatible RON parsing (serde default).

**Phase 2: Refactor `update_chunks` (Task 4)**
- The core refactor. Three distinct phases in one system:
  1. **Unload** (immediate): Use `ChunkEntityIndex` for O(1) lookup instead of iterating all `chunk_entities`.
  2. **Queue** (on chunk change): Compute new pending set, sort by Manhattan distance.
  3. **Load** (staggered): Pop up to `max_chunks_per_frame` from queue, spawn with budget.

**Key design decision: Budget tracking via index, not query.**
- Current code counts `all_collidable.iter().count()` every frame — this includes non-chunk entities (player, manual spawns).
- New approach: `ChunkEntityIndex.entity_count()` tracks only chunk-spawned entities. Add a constant for the player entity (1) to get total. This is O(n_chunks) not O(n_entities).
- **Important:** Non-chunk collidable entities (player, manually spawned) are NOT tracked in the index. Budget enforcement should use `chunk_entity_index.entity_count() + non_chunk_collidable_count` where non-chunk count comes from a simple query of `Collider` entities WITHOUT `ChunkEntity` component.

**Actually — simpler approach:** Keep the `all_collidable` query for accurate budget counting but use the index ONLY for fast despawn. The budget query runs once per frame (not per-chunk) and is cheap with ~200 entities. The real performance win is in despawn, which currently does O(chunks_to_unload × all_entities).

**Revised design:**
```
// Unload: O(chunks × entities_per_chunk) via index — was O(chunks × ALL_entities)
// Budget: O(all_entities) via query — unchanged, runs once per frame
// Load: O(max_chunks_per_frame × entities_per_chunk) — was O(all_new_chunks × entities_per_chunk)
```

### Staggered Loading Design

```
Frame N (player enters new chunk):
  1. Unload immediately: chunks outside radius → despawn via index
  2. Compute pending: desired - active, sorted by manhattan_distance
  3. Load up to max_chunks_per_frame from pending

Frame N+1:
  1. Unload: nothing (player hasn't moved)
  2. Pending queue: still has remaining chunks
  3. Load next batch from pending

Frame N+K (player moves to another chunk):
  1. Unload: chunks now outside radius
  2. Pending queue: FULL RECALCULATE (old pending discarded)
  3. Load next batch from fresh pending
```

### Resource Designs

```rust
// In src/world/mod.rs

/// Tracks entities per chunk for O(1) despawn lookup.
#[derive(Resource, Default, Debug)]
pub struct ChunkEntityIndex {
    pub chunks: HashMap<ChunkCoord, Vec<Entity>>,
}

impl ChunkEntityIndex {
    /// Total entity count across all chunks.
    pub fn entity_count(&self) -> usize {
        self.chunks.values().map(|v| v.len()).sum()
    }
}

/// Queue of chunks waiting to be loaded, sorted by distance (nearest first).
#[derive(Resource, Default, Debug)]
pub struct PendingChunks {
    pub chunks: VecDeque<ChunkCoord>,
}

/// Tracks player's last known chunk for change detection.
#[derive(Resource, Default, Debug)]
pub struct ChunkLoadState {
    pub last_player_chunk: Option<ChunkCoord>,
}
```

### WorldConfig Extension

```rust
#[derive(Resource, Deserialize, Clone, Debug)]
pub struct WorldConfig {
    pub seed: u64,
    pub chunk_size: f32,
    pub load_radius: u32,
    pub entity_budget: usize,
    #[serde(default = "default_max_chunks_per_frame")]
    pub max_chunks_per_frame: usize,
}

fn default_max_chunks_per_frame() -> usize { 4 }
```

The `#[serde(default = ...)]` ensures backward compatibility with existing `world.ron` files that don't have the new field.

### Refactored update_chunks Pseudocode

```rust
fn update_chunks(
    mut commands: Commands,
    player_query: Query<&Transform, With<Player>>,
    config: Res<WorldConfig>,
    biome_config: Res<BiomeConfig>,
    mut active_chunks: ResMut<ActiveChunks>,
    mut explored_chunks: ResMut<ExploredChunks>,
    mut chunk_entity_index: ResMut<ChunkEntityIndex>,
    mut pending_chunks: ResMut<PendingChunks>,
    mut chunk_load_state: ResMut<ChunkLoadState>,
    all_collidable: Query<Entity, With<Collider>>,
) {
    let player_chunk = ...;
    let desired = chunks_in_radius(player_chunk, config.load_radius);

    // Phase 1: UNLOAD (immediate, all at once)
    let to_unload: Vec<ChunkCoord> = active_chunks...filter(!desired)...sorted;
    let mut despawned_count = 0;
    for coord in &to_unload {
        if let Some(entities) = chunk_entity_index.chunks.remove(coord) {
            for entity in &entities {
                commands.entity(*entity).despawn();
                despawned_count += entities.len();
            }
        }
        active_chunks.chunks.remove(coord);
    }

    // Phase 2: QUEUE (only on chunk change or first frame)
    let chunk_changed = chunk_load_state.last_player_chunk != Some(player_chunk);
    if chunk_changed {
        let mut new_pending: Vec<ChunkCoord> = desired.iter()
            .filter(|c| !active_chunks.chunks.contains_key(c))
            .copied().collect();
        new_pending.sort_by_key(|c| (manhattan_distance(*c, player_chunk), *c));
        pending_chunks.chunks = VecDeque::from(new_pending);
        chunk_load_state.last_player_chunk = Some(player_chunk);
    }

    // Phase 3: LOAD (staggered)
    let total_entity_count = all_collidable.iter().count() - despawned_count;
    let mut loaded = 0;
    let mut budget_used = total_entity_count;
    while loaded < config.max_chunks_per_frame {
        let Some(coord) = pending_chunks.chunks.pop_front() else { break };
        if active_chunks.chunks.contains_key(&coord) { continue; } // already loaded
        if !desired.contains(&coord) { continue; } // no longer desired

        let biome = determine_biome(...);
        let blueprints = generate_chunk_content(...);
        let remaining_budget = config.entity_budget.saturating_sub(budget_used);
        let spawn_count = blueprints.len().min(remaining_budget);

        let mut chunk_entities = Vec::with_capacity(spawn_count);
        for blueprint in blueprints.into_iter().take(spawn_count) {
            let entity = commands.spawn((...)).id();
            chunk_entities.push(entity);
        }

        chunk_entity_index.chunks.insert(coord, chunk_entities);
        budget_used += spawn_count;
        active_chunks.chunks.insert(coord, biome);
        explored_chunks.chunks.entry(coord).or_insert(biome);
        loaded += 1;
    }
}
```

### Edge Cases to Handle

1. **Player teleport:** Large jump → many chunks to unload and load. Unload is immediate. Load is staggered over multiple frames. The player briefly sees empty space — acceptable.
2. **Rapid direction changes:** Player oscillates between chunks. Pending queue is refreshed each time, discarding stale entries. No wasted work on chunks that are no longer desired.
3. **Budget exhaustion mid-batch:** If budget runs out during staggered loading, remaining pending chunks stay in queue but won't spawn entities until budget frees up (chunks unloaded).
4. **Entity destroyed mid-chunk:** If an asteroid/drone is destroyed by combat, it's despawned by the damage system but remains in `ChunkEntityIndex`. On chunk unload, `commands.entity(e).despawn()` on an already-despawned entity is a no-op in Bevy (logs warning at most). **Mitigation:** Accept the stale entry — it's harmless and avoids the complexity of cross-system index maintenance. The entity count via `all_collidable` query is always accurate.
5. **First frame:** `last_player_chunk` is `None` → always triggers queue calculation on first frame.

### Previous Story Intelligence (from Stories 1.1-1.4)

**Learnings to apply:**
1. **Config backward compatibility** — Use `#[serde(default = "...")]` for new fields so existing RON files still parse.
2. **Test harness must be updated** — Every new resource needs `app.init_resource::<T>()` or `app.insert_resource(T::default())` in `test_app()`.
3. **Existing tests may break** — Tests in `tests/world_generation.rs` build custom `App` instances. They need the new resources added.
4. **Deterministic ordering matters** — Chunks must be processed in a deterministic order. Use `sort()` + `sort_by_key()` consistently.
5. **`commands.spawn((...)).id()`** — Use `.id()` to capture spawned entity ID for the index.
6. **Clippy `too_many_arguments`** — `update_chunks` already has `#[allow(clippy::too_many_arguments)]`. Adding more params is fine.
7. **Integration tests need multiple `app.update()` cycles** — Staggered loading requires testing across frames.

**Code review findings from 1.3/1.4 to watch for:**
- Entity leak: chunk unloaded but index not cleaned → test 8.2 catches this
- Stale pending entries after player moves → test 8.6 catches this
- Budget counted incorrectly with mixed chunk/non-chunk entities
- Off-by-one in `max_chunks_per_frame` boundary

### Project Structure Notes

- **Files to create:**
  - `tests/chunk_loading.rs` — 6 integration tests for staggered loading and entity index
- **Files to modify:**
  - `src/world/mod.rs` — Add `ChunkEntityIndex`, `PendingChunks`, `ChunkLoadState` resources, refactor `update_chunks`, register in `WorldPlugin`
  - `src/world/chunk.rs` — Add `manhattan_distance` function + unit tests
  - `assets/config/world.ron` — Add `max_chunks_per_frame: 4`
  - `tests/helpers/mod.rs` — Add `ChunkEntityIndex`, `PendingChunks`, `ChunkLoadState` to test harness
  - `tests/world_generation.rs` — Add new resources to custom App tests (same pattern as 1.4 adding ExploredChunks)
- **Files NOT to touch:**
  - `src/world/generation.rs` — Generation logic unchanged
  - `src/rendering/` — No rendering changes (minimap/world_map read ActiveChunks/ExploredChunks unchanged)
  - `src/core/` — No core changes
  - `src/shared/` — No shared component changes
  - `assets/config/biome.ron` — Biome config unchanged
  - `assets/config/minimap.ron` — Minimap config unchanged
  - `assets/config/world_map.ron` — World map config unchanged

### Key Libraries

- **`bevy` 0.18.0** — `Commands`, `Entity`, `Query`, `Res`, `ResMut`, `Resource`, `With`, `Transform`, `Plugin`
- **`std::collections::HashMap`** — For `ChunkEntityIndex.chunks`
- **`std::collections::VecDeque`** — For `PendingChunks.chunks` (efficient pop_front)
- **`serde`** (already in Cargo.toml) — `#[serde(default = "...")]` for backward-compatible WorldConfig
- **NO new dependencies needed**

### References

- [Source: _bmad-output/epics.md — Epic 1 Story 5: "As a developer, the chunk system loads and unloads chunks based on player position so that memory stays bounded"]
- [Source: _bmad-output/game-architecture.md — "World Streaming" decision: Hybrid Grid-Chunks + Player-Radius-Loading]
- [Source: _bmad-output/game-architecture.md — "Memory under 500MB after 2 hours of continuous exploration"]
- [Source: _bmad-output/game-architecture.md — "WASM: Smaller load radius, per-frame generation budget"]
- [Source: _bmad-output/game-architecture.md — "System Ordering" — chunk load/unload in FixedUpdate]
- [Source: _bmad-output/game-architecture.md — "Entity Budget" — 200 simultaneously active entities]
- [Source: _bmad-output/game-architecture.md — "Graceful Degradation" — systems log errors and fall back]
- [Source: _bmad-output/implementation-artifacts/1-1-seamless-world-generation.md — Original chunk system implementation]
- [Source: _bmad-output/implementation-artifacts/1-2-biome-types.md — BiomeType, ActiveChunks as HashMap]
- [Source: _bmad-output/implementation-artifacts/1-4-world-map.md — ExploredChunks resource, ChunkEntity component]
- [Source: src/world/mod.rs:193-296 — Current update_chunks implementation with linear despawn scan]
- [Source: src/world/chunk.rs:29-41 — chunks_in_radius with square neighborhood]

### Git Intelligence

- **Last commits:**
  - `feat(1.4): world map — explored chunk tracking, toggle overlay, biome-colored tiles, WorldMapConfig. 16 new tests (10 unit + 6 integration).`
  - `fix(1.3): code review — blip color test, MinimapState cleanup on root removal. 2 new integration tests.`
  - `feat(1.3): minimap blips — scanner range, blip types, Bevy UI minimap rendering. 12 unit + 5 integration tests.`
- **Commit convention:** `feat(1.5): chunk loading — entity index, staggered loading, pending queue, manhattan priority. X new tests (Y unit + Z integration).`
- **VCS:** Using `jj` (Jujutsu). Use `jj new` after code review marks story done.
- **Total tests before this story:** 220 (137 unit + 83 integration, all passing)

## Dev Agent Record

### Agent Model Used

Claude Opus 4.6

### Debug Log References

- Bevy 0.18: `Entity::from_raw()` removed, use `Entity::from_bits()` for test entity construction.
- Staggered loading required updating all existing tests that assumed single-frame chunk loading to use `run_until_loaded()` or explicit frame loops.

### Completion Notes List

- Task 1: Added `manhattan_distance` pure function to `chunk.rs` with `unsigned_abs()` for correct handling of negative coordinates.
- Task 2: Added `ChunkEntityIndex` resource with `entity_count()` method. Registered in `WorldPlugin::build()`.
- Task 3: Added `PendingChunks` resource with `VecDeque`. Extended `WorldConfig` with `max_chunks_per_frame` using `#[serde(default)]` for backward compatibility.
- Task 4: Refactored `update_chunks` into 3 phases: immediate unload via index, queue calculation on chunk change, staggered load with priority. Kept `all_collidable` query for accurate total budget (per Dev Notes revised design).
- Task 5: Added `ChunkLoadState` resource for chunk-change detection.
- Task 6: Updated test harness and all custom `App` test instances with new resources. Added `run_until_loaded()` helper. Updated world_map test for staggered loading.
- Task 7: 10 unit tests covering manhattan_distance, ChunkEntityIndex, WorldConfig, PendingChunks, ChunkLoadState.
- Task 8: 6 integration tests covering index population, cleanup on unload, staggered loading limits, load priority, extended exploration memory bounds, and pending queue refresh.

### File List

- `src/world/chunk.rs` — Added `manhattan_distance` function + 4 unit tests
- `src/world/mod.rs` — Added `ChunkEntityIndex`, `PendingChunks`, `ChunkLoadState` resources; extended `WorldConfig` with `max_chunks_per_frame`; refactored `update_chunks` to 3-phase system; added 6 unit tests
- `assets/config/world.ron` — Added `max_chunks_per_frame: 4`
- `tests/helpers/mod.rs` — Added new resources to test harness
- `tests/chunk_loading.rs` — New file: 6 integration tests for staggered loading
- `tests/world_generation.rs` — Updated imports, added new resources to custom App tests, replaced single `app.update()` with `run_until_loaded()` for staggered loading compatibility
- `tests/world_map.rs` — Updated `tile_count_matches_visible_explored_chunks_when_map_opened` for staggered loading

### Change Log

- 2026-02-27: Implemented Story 1.5 — ChunkEntityIndex for O(1) despawn, PendingChunks for staggered loading with Manhattan distance priority, ChunkLoadState for chunk-change detection, WorldConfig.max_chunks_per_frame config. 16 new tests (10 unit + 6 integration). All 236 tests pass.
- 2026-02-27: Code review fixes — (H1) saturating_sub for entity count to prevent usize underflow when combat-destroyed entities cause stale index entries; (M1) defensive get_entity() for despawn on potentially non-existent entities; (M2) run_until_loaded() moved to shared test helpers; (M3) added serde default assertion to world_config_from_ron test; (M4) added max_chunks_per_frame==0 warning in WorldPlugin::build().
