# Story 1.1: Seamless World Generation

Status: done

<!-- Note: Validation is optional. Run validate-create-story for quality check before dev-story. -->

## Story

As a player,
I want to fly in any direction and have the world generate seamlessly around me,
so that the universe feels infinite.

## Acceptance Criteria

1. **Chunk Grid System** — The world is divided into fixed-size square chunks identified by `ChunkCoord(i32, i32)`. Player position maps deterministically to a chunk coordinate.
2. **Player-Centered Loading** — Chunks within a configurable `load_radius` around the player's current chunk are loaded and active. Chunks outside this radius are unloaded (despawned).
3. **Seed-Deterministic Generation** — Given the same world seed and chunk coordinate, the exact same entities (asteroids, later drones) are spawned at the same positions with the same properties. Generation is pure: `fn generate_chunk(seed: u64, coord: ChunkCoord) -> Vec<EntityBlueprint>`.
4. **Seamless Transition** — No visible loading or pop-in when crossing chunk boundaries. Entities appear before the player reaches them.
5. **Entity Migration** — The existing Epic 0 fixed-spawn asteroid/drone system is replaced by chunk-based procedural spawning. The arcade feel is preserved (similar density and placement variety).
6. **Performance** — Chunk generation completes in <16ms per chunk. Total active entity count stays within 200 entity budget. Memory for loaded chunks stays bounded.
7. **World Seed Config** — A `WorldConfig` resource (loaded from `assets/config/world.ron`) holds `seed: u64`, `chunk_size: f32`, `load_radius: u32`, and generation parameters. Falls back to `Default` with `warn!` on error (same pattern as FlightConfig).
8. **WorldPlugin Integration** — New `WorldPlugin` added to `game_plugins()` return tuple. Systems run in `FixedUpdate` and integrate with existing system chain.

## Tasks / Subtasks

- [x] Task 1: Create WorldPlugin skeleton and config (AC: #7, #8)
  - [x] 1.1 Create `src/world/mod.rs` with `WorldPlugin` struct implementing `Plugin`
  - [x] 1.2 Create `src/world/chunk.rs` with `ChunkCoord`, chunk math functions (`world_to_chunk`, `chunk_to_world_center`, `chunks_in_radius`)
  - [x] 1.3 Create `WorldConfig` resource with RON deserialization, `Default` impl, `from_ron()` method
  - [x] 1.4 Create `assets/config/world.ron` with tunable values (seed, chunk_size, load_radius)
  - [x] 1.5 Register `WorldPlugin` in `game_plugins()` tuple in `src/lib.rs`

- [x] Task 2: Implement chunk lifecycle management (AC: #1, #2)
  - [x] 2.1 Create `ActiveChunks` resource tracking currently loaded `HashSet<ChunkCoord>`
  - [x] 2.2 Create `ChunkEntity` component linking entities to their parent chunk
  - [x] 2.3 Implement `update_chunks` system: compute desired chunks from player position + load_radius, diff against `ActiveChunks`, handle load/unload
  - [x] 2.4 Implement load logic: for each new chunk, run generation and spawn entities
  - [x] 2.5 Implement unload logic: for each removed chunk, despawn all entities with matching `ChunkEntity`

- [x] Task 3: Implement seed-deterministic chunk generation (AC: #3, #5)
  - [x] 3.1 Create `src/world/generation.rs` with `generate_chunk_content(seed: u64, coord: ChunkCoord, config: &WorldConfig) -> Vec<EntityBlueprint>`
  - [x] 3.2 Use seeded RNG (`rand::SeedableRng` with `StdRng` from chunk-specific seed derived from `hash(world_seed, chunk_x, chunk_y)`)
  - [x] 3.3 Generate asteroid positions, sizes, velocities, health values within chunk bounds
  - [x] 3.4 Generate scout drone positions (lower density than asteroids)
  - [x] 3.5 Ensure generation is pure and deterministic (same inputs = same outputs)

- [x] Task 4: Migrate from fixed spawning to chunk-based spawning (AC: #5)
  - [x] 4.1 Remove `spawn_initial_entities` system from CorePlugin (function kept for test compatibility)
  - [x] 4.2 Entity properties (health, radius, velocity ranges) moved to `WorldConfig`
  - [x] 4.3 Preserve existing marker components (`Asteroid`, `ScoutDrone`, `NeedsAsteroidVisual`, `NeedsDroneVisual`) — chunk generation uses same spawn pattern
  - [x] 4.4 Respawn system compatibility preserved — `RespawnTimer` works with both chunk and non-chunk entities; chunk entities regenerate from seed on reload

- [x] Task 5: System integration and ordering (AC: #8)
  - [x] 5.1 `update_chunks` runs in `FixedUpdate` with `.before(CoreSet::Collision)` ordering
  - [x] 5.2 Chunk entity despawning doesn't conflict with damage/respawn pipeline (verified by 157 passing tests)
  - [x] 5.3 Updated `tests/helpers/mod.rs` test harness with WorldConfig, ActiveChunks, update_chunks system

- [x] Task 6: Rendering integration (AC: #4)
  - [x] 6.1 Chunk-spawned entities use same marker pattern (`NeedsAsteroidVisual`, `NeedsDroneVisual`) — existing RenderingPlugin handles them automatically
  - [x] 6.2 No new rendering code needed — verified existing systems work with chunk-spawned entities

- [x] Task 7: Unit tests (AC: all)
  - [x] 7.1 `ChunkCoord` math: 7 tests for `world_to_chunk` including edge cases at chunk boundaries, negative coords
  - [x] 7.2 `chunks_in_radius` returns correct set for radius 0, 1, 2+ (4 tests)
  - [x] 7.3 `generate_chunk_content` determinism: same seed+coord = same output across multiple calls
  - [x] 7.4 `generate_chunk_content` variety: different coords produce different content
  - [x] 7.5 `WorldConfig` RON parsing (valid + invalid) and Default validation (3 tests)
  - [x] 7.6 Entity count within config bounds, positions within chunk bounds, valid entity properties (3 tests)

- [x] Task 8: Integration tests (AC: all)
  - [x] 8.1 Player at origin → chunks within load_radius are populated with entities
  - [x] 8.2 Player moves to new chunk → new chunk loads, distant chunk unloads
  - [x] 8.3 Entities have correct components (Asteroid/ScoutDrone markers, Health, Collider, Velocity, ChunkEntity)
  - [x] 8.4 Chunk determinism: reload same chunk → same entities at same positions
  - [x] 8.5 Total entity count stays within budget
  - [x] 8.6 Existing weapon/collision systems work with chunk-spawned entities (laser hits asteroid)

## Dev Notes

### Architecture Patterns & Constraints

- **Core/Rendering separation MUST be maintained:** WorldPlugin spawns entities with marker components (`NeedsAsteroidVisual`, `NeedsDroneVisual`). Existing RenderingPlugin already handles these — no rendering changes needed.
- **Asset caching pattern already exists:** `AsteroidAssets`, `DroneAssets` are cached at Startup in RenderingPlugin. Chunk-spawned entities will use the same shared mesh/material handles.
- **System chaining mandatory:** All new world systems must integrate into the existing `FixedUpdate` chain. World loading/unloading must happen BEFORE collision detection.
- **Config loading pattern:** Use the established `from_ron()` + `Default` fallback with `warn!` pattern (same as `FlightConfig`, `WeaponConfig`, `SpawningConfig`).
- **No unwrap():** `#[deny(clippy::unwrap_used)]` is enforced. Use `.expect("descriptive message")` in tests.
- **Plugin registration:** Add `WorldPlugin` to `game_plugins()` tuple — currently returns `(CorePlugin, RenderingPlugin, DevPlugin)`, will become `(CorePlugin, RenderingPlugin, DevPlugin, WorldPlugin)`.

### Chunk Design Decisions

- **Chunk size:** Start with 1000.0 world units (tunable via `world.ron`). This should contain ~4-8 asteroids and 1-2 drones at default density.
- **Load radius:** Start with 2 chunks in each direction (5x5 grid = 25 chunks loaded). This gives a smooth exploration buffer.
- **Seed derivation per chunk:** `chunk_seed = hash(world_seed, chunk_coord.x, chunk_coord.y)` — use a fast hash like `world_seed ^ (x as u64).wrapping_mul(PRIME1) ^ (y as u64).wrapping_mul(PRIME2)` or use `std::hash::Hasher` with `DefaultHasher`.
- **Entity density:** Approximately same as current `SpawningConfig` defaults per chunk area. Scale to feel right.

### Migration Strategy (Epic 0 → Epic 1)

The existing `spawn_initial_entities` system spawns fixed-position asteroids/drones from `SpawningConfig`. This story replaces that with chunk-based procedural spawning:
1. `spawn_initial_entities` is removed from production system registration (kept available for tests if needed)
2. `SpawningConfig` spawn positions become irrelevant; health/radius/velocity ranges may be absorbed into `WorldConfig` or kept as a separate reference
3. Existing `RespawnTimer` system may need adjustment — respawned entities should belong to a chunk. Consider: respawn within same chunk, or disable respawn for chunk-based entities (chunk unload/reload handles it naturally).

### Respawn Consideration

**Recommended approach:** Disable per-entity respawn timers for chunk-based entities. Instead, when a chunk is unloaded and reloaded, it regenerates from seed (destroyed entities reappear). This is simpler and aligns with the delta-save architecture (Story 1-8/1-9 will track destroyed entities as deltas). For now, destroyed entities come back when their chunk is reloaded.

### Project Structure Notes

- New files to create: `src/world/mod.rs`, `src/world/chunk.rs`, `src/world/generation.rs`
- New config: `assets/config/world.ron`
- Modified files: `src/lib.rs` (add WorldPlugin), `tests/helpers/mod.rs` (add world systems)
- Existing files untouched: `src/core/`, `src/rendering/`, `src/shared/`
- New test file: `tests/world_generation.rs`
- Alignment with architecture target: `src/world/` domain matches architecture doc exactly

### Key Libraries

- **`rand`** (already in Cargo.toml) — `rand::SeedableRng`, `rand::rngs::StdRng`, `rand::Rng` for deterministic RNG per chunk
- **`bevy` 0.18.0** — `Transform`, `Commands`, `Query`, `Res`, `Resource`, `Component`, `Plugin`, `FixedUpdate`
- **`ron`** (already in Cargo.toml) — Config deserialization
- **`serde`** (already in Cargo.toml) — Derive `Serialize`, `Deserialize` for config structs
- **NO new dependencies needed**

### References

- [Source: _bmad-output/planning-artifacts/game-architecture.md — "World Streaming System" section]
- [Source: _bmad-output/planning-artifacts/game-architecture.md — "Project Structure - World Domain"]
- [Source: _bmad-output/planning-artifacts/game-architecture.md — "Architectural Boundaries"]
- [Source: _bmad-output/planning-artifacts/game-architecture.md — "Delta-Save Pattern"]
- [Source: _bmad-output/planning-artifacts/gdd.md — "World Design Framework"]
- [Source: _bmad-output/planning-artifacts/gdd.md — "Open World (Post-Tutorial)"]
- [Source: _bmad-output/planning-artifacts/gdd.md — "Difficulty Curve & Distance-Based Scaling"]
- [Source: _bmad-output/implementation-artifacts/0-8-runtime-spawning.md — Previous story patterns]

### Previous Story Intelligence (from Story 0.8)

- **Marker component pattern works well:** `NeedsAsteroidVisual` / `NeedsDroneVisual` → rendering attaches mesh. Reuse exactly.
- **System chain is 4-way in damage pipeline:** `apply_damage → handle_player_death → spawn_respawn_timers → despawn_destroyed`. World systems must run BEFORE this chain.
- **Config loading is tested and reliable:** Same `from_ron()` + `Default` pattern used in FlightConfig, WeaponConfig, SpawningConfig.
- **Lyon mesh generation for asteroids/drones:** Already in `rendering/vector_art.rs`. No changes needed.
- **129 existing tests (69 unit + 60 integration):** Must all continue passing. Zero regressions.

### Git Intelligence

- **Commit convention:** `feat(1.1): seamless world generation — chunk-based procedural world, WorldPlugin, seed-deterministic generation. X unit + Y integration tests.`
- **Recent pattern:** Each story is one commit with all tasks included
- **VCS:** Using `jj` (Jujutsu), not raw git. Use `jj new` after code review marks story done.

## Dev Agent Record

### Agent Model Used
Claude Opus 4.6

### Debug Log References
- Fixed borrow lifetime issue in integration test (EntityMut temporary) by using scoped block
- Fixed existing `destroyed_asteroid_respawns_after_delay` test to filter by `Without<ChunkEntity>` since chunk system now spawns additional asteroids
- Removed unused `spawn_initial_entities` import from CorePlugin after removing the Startup system

### Completion Notes List
- Implemented complete `WorldPlugin` with chunk-based procedural world generation
- `update_chunks` system: single unified system handles chunk lifecycle (load/unload) in FixedUpdate, ordered before collision detection
- Seed-deterministic generation via `StdRng::seed_from_u64` with per-chunk seed derivation using prime multiplication
- `WorldConfig` loaded from `assets/config/world.ron` with established `from_ron()` + Default fallback pattern
- Entity budget enforcement: spawn count capped at `entity_budget` (default 200), counts ALL collidable entities
- Migrated from fixed `spawn_initial_entities` to chunk-based spawning — all existing marker components and rendering pipeline preserved
- 22 new unit tests + 10 new integration tests = 32 new tests; total project: 162 tests (92 unit + 70 integration), 0 regressions

### File List
- `src/world/mod.rs` — NEW: WorldPlugin, WorldConfig, ActiveChunks, ChunkEntity, update_chunks system
- `src/world/chunk.rs` — NEW: ChunkCoord, world_to_chunk, chunk_to_world_center, chunks_in_radius + 14 unit tests
- `src/world/generation.rs` — NEW: EntityBlueprint, generate_chunk_content, seed derivation + 5 unit tests
- `src/lib.rs` — MODIFIED: added `pub mod world`, WorldPlugin to game_plugins() tuple
- `src/core/mod.rs` — MODIFIED: removed spawn_initial_entities from Startup systems and unused import
- `assets/config/world.ron` — NEW: world generation config
- `tests/helpers/mod.rs` — MODIFIED: added WorldConfig, ActiveChunks, update_chunks to test harness
- `tests/world_generation.rs` — NEW: 6 integration tests for chunk system
- `tests/runtime_spawning.rs` — MODIFIED: updated destroyed_asteroid_respawns_after_delay to filter by Without<ChunkEntity>

### Architecture Deviation Notes
- **Chunk system schedule:** Architecture doc specifies `PostUpdate` for chunk load/unload execution. Implementation uses `FixedUpdate.before(CoreSet::Collision)` instead. Rationale: entities must exist before collision detection runs in the same FixedUpdate tick. PostUpdate would cause a 1-frame delay before spawned entities participate in gameplay.
- **Cross-plugin imports:** WorldPlugin imports `Health`, `Collider`, `Player`, `Asteroid`, `ScoutDrone` directly from `crate::core` instead of from `shared/`. Architecture doc mandates shared components live in `src/shared/components.rs`. Rationale: moving these components is a cross-cutting refactor affecting all existing systems and tests — deferred to a dedicated refactoring story. Tracked as known deviation.

### Change Log
- 2026-02-26: Implemented Story 1.1 — Seamless World Generation. Chunk-based procedural spawning replaces fixed spawning. 28 new tests, 0 regressions.
- 2026-02-26: Code Review #1 (AI) — 7 findings (1 HIGH, 3 MEDIUM, 3 LOW). All HIGH/MEDIUM fixed:
  - FIXED: Entity budget enforcement broken across multi-chunk loads (running counter added)
  - FIXED: Test harness system ordering now matches production (update_chunks before collision chain)
  - FIXED: ChunkCoord re-exported from world module via `pub use`
  - FIXED: laser_hits_chunk_spawned_asteroid test now verifies actual damage/destruction
  - NEW: entity_budget_enforced_across_multi_chunk_load integration test added
  - Total: 158 tests (91 unit + 67 integration), 0 regressions
- 2026-02-26: Code Review #2 (AI) — 7 findings (0 HIGH, 3 MEDIUM, 4 LOW). All MEDIUM fixed:
  - FIXED: Entity budget inflation from deferred despawns — despawned_count subtracted from total
  - FIXED: No test for entity despawn on chunk unload — chunk_unload_despawns_entities test added
  - FIXED: Architecture deviation documented in Dev Notes
  - Total: 159 tests (91 unit + 68 integration), 0 regressions
- 2026-02-26: Code Review #3 (AI) — 7 findings (0 HIGH, 3 MEDIUM, 4 LOW). All MEDIUM fixed:
  - FIXED: Entity budget now counts ALL collidable entities (not just chunk entities) — `all_collidable` query added to `update_chunks`
  - FIXED: Non-deterministic chunk load order — `to_load`/`to_unload` sorted via `Ord` on `ChunkCoord`
  - FIXED: Cross-plugin import deviation documented in Architecture Deviation Notes
  - NEW: entity_budget_accounts_for_non_chunk_collidable_entities integration test added
  - Total: 160 tests (91 unit + 69 integration), 0 regressions
- 2026-02-26: Code Review #4 (AI) — 7 findings (0 HIGH, 3 MEDIUM, 4 LOW). All MEDIUM fixed:
  - FIXED: Velocity range panic risk — `safe_speed()` helper with graceful fallback when min >= max
  - FIXED: Stale Completion Notes updated to reflect actual test counts after 4 reviews
  - FIXED: No test for player-less scenario — `update_chunks_without_player_does_not_panic` test added
  - NEW: `equal_velocity_ranges_do_not_panic` unit test added
  - Total: 162 tests (92 unit + 70 integration), 0 regressions
