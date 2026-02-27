# Story 1.2: Biome Types

Status: done

<!-- Note: Validation is optional. Run validate-create-story for quality check before dev-story. -->

## Story

As a player,
I want to encounter different biome types (deep space, asteroid fields, wreck fields),
so that the world has visual and tactical variety.

## Acceptance Criteria

1. **BiomeType Enum** — A `BiomeType` enum with three variants (`DeepSpace`, `AsteroidField`, `WreckField`) defines biome taxonomy. Each chunk is assigned exactly one biome type.
2. **Deterministic Biome Assignment** — Given the same world seed and chunk coordinate, the same `BiomeType` is always assigned. Biome selection is a pure function of `(seed, ChunkCoord)`.
3. **Per-Biome Entity Density** — Each biome has distinct spawn parameters (entity counts, types, health, velocity ranges) configured via `BiomeConfig` in `assets/config/biome.ron`:
   - **Deep Space:** Sparse entities (0-2 asteroids, 0-1 drones). Open, empty feel.
   - **Asteroid Field:** Dense asteroids (6-12), moderate drones (1-3). Tight maneuvering.
   - **Wreck Field:** Moderate asteroids (2-5), higher drones (2-4). Static debris feel (higher health, lower velocity).
4. **BiomeType Component** — Spawned entities carry a `BiomeType` component indicating their origin biome, enabling biome-aware rendering and logic.
5. **Chunk-Biome Tracking** — `ActiveChunks` tracks the `BiomeType` for each loaded chunk. `ChunkEntity` or a separate component links entities to their chunk's biome.
6. **Biome Distribution Variety** — Across a 10x10 chunk grid from origin, all three biome types appear. No single biome dominates >60% of chunks at default config.
7. **Entity Budget Preserved** — The existing entity budget enforcement still works correctly. Biome-specific density does not bypass the budget cap.
8. **Seed Determinism Verified** — Reloading a chunk produces the same biome type and the same entities as the first load.
9. **Config-Driven** — All biome parameters are tunable via `assets/config/biome.ron` using the established `from_ron()` + `Default` fallback pattern.

## Tasks / Subtasks

- [x] Task 1: Define BiomeType and BiomeConfig (AC: #1, #9)
  - [x] 1.1 Create `BiomeType` enum (`DeepSpace`, `AsteroidField`, `WreckField`) with `Component` derive in `src/world/generation.rs`
  - [x] 1.2 Create `BiomeConfig` resource with per-biome spawn parameters (asteroid count min/max, drone count min/max, health, velocity ranges, biome thresholds)
  - [x] 1.3 Create `assets/config/biome.ron` with default tuning values
  - [x] 1.4 Load `BiomeConfig` via `from_ron()` + `Default` fallback in `WorldPlugin` startup

- [x] Task 2: Implement deterministic biome selection (AC: #2, #6)
  - [x] 2.1 Create `determine_biome(seed: u64, coord: ChunkCoord, config: &BiomeConfig) -> BiomeType` pure function in `generation.rs`
  - [x] 2.2 Use seeded RNG (same `chunk_seed()` pattern) to produce a `f32` in [0.0, 1.0], map to biome via configurable thresholds
  - [x] 2.3 Default thresholds: DeepSpace 0.0-0.3, AsteroidField 0.3-0.7, WreckField 0.7-1.0

- [x] Task 3: Integrate biome into chunk generation (AC: #3, #4, #5, #7)
  - [x] 3.1 Extend `generate_chunk_content()` to accept `BiomeConfig` and call `determine_biome()` first
  - [x] 3.2 Use biome-specific spawn parameters from `BiomeConfig` instead of `WorldConfig` global values
  - [x] 3.3 Add `BiomeType` to `EntityBlueprint` so spawned entities carry biome information
  - [x] 3.4 Update `update_chunks` in `mod.rs` to attach `BiomeType` component when spawning entities
  - [x] 3.5 Track biome per chunk in `ActiveChunks` (e.g., `HashMap<ChunkCoord, BiomeType>`)
  - [x] 3.6 Verify entity budget enforcement still works with variable biome densities

- [x] Task 4: Update WorldConfig and clean up (AC: #9)
  - [x] 4.1 Remove per-entity spawn parameters from `WorldConfig` that are now in `BiomeConfig` (asteroid_count_min/max, drone_count_min/max, etc.)
  - [x] 4.2 Keep seed, chunk_size, load_radius, entity_budget in `WorldConfig`
  - [x] 4.3 Update `assets/config/world.ron` to remove migrated fields

- [x] Task 5: Unit tests (AC: all)
  - [x] 5.1 `determine_biome` determinism: same seed+coord = same biome across calls (2 tests)
  - [x] 5.2 `determine_biome` variety: different coords produce mix of biome types (1 test)
  - [x] 5.3 `generate_chunk_content` respects biome-specific density: AsteroidField chunks have more asteroids than DeepSpace (2 tests)
  - [x] 5.4 `BiomeConfig` RON parsing and Default validation (2 tests)
  - [x] 5.5 Biome threshold edge cases: values at exact boundaries (1 test)

- [x] Task 6: Integration tests (AC: all)
  - [x] 6.1 Chunk spawned with biome: entities have `BiomeType` component matching chunk's biome (1 test)
  - [x] 6.2 Chunk reload produces same biome: unload + reload → same BiomeType (1 test)
  - [x] 6.3 Multiple biomes appear: load 25+ chunks → at least 2 different biome types present (1 test)
  - [x] 6.4 Entity budget with high-density biome: AsteroidField chunks don't exceed budget (1 test)
  - [x] 6.5 Laser hits biome-tagged asteroid: gameplay still works with BiomeType component (1 test)

## Dev Notes

### Architecture Patterns & Constraints

- **Core/Rendering separation MUST be maintained:** BiomeType is a core component. Rendering will use it later for visual differentiation (different asteroid colors/shapes per biome), but Story 1.2 focuses on core logic only. Visual biome distinction is deferred.
- **Config pattern:** `BiomeConfig` follows the exact same pattern as `WorldConfig`, `FlightConfig`, `WeaponConfig`: `Asset + Deserialize + TypePath`, `from_ron()` method, `Default` impl, `warn!` on parse error.
- **Deterministic generation:** Biome selection MUST be a pure function. Use the existing `chunk_seed()` derivation. Do NOT use `thread_rng()` or any non-deterministic source.
- **Entity budget stays global:** The budget caps ALL collidable entities regardless of biome. The `update_chunks` system already counts `all_collidable` entities.
- **No unwrap():** `#[deny(clippy::unwrap_used)]` is enforced project-wide. Use `.expect("msg")` in tests.
- **System scheduling unchanged:** `update_chunks` stays in `FixedUpdate.before(CoreSet::Collision)`.

### Implementation Strategy

- **Biome selection via threshold ranges:** Map a seeded random `f32` in [0.0, 1.0) to biome type via configurable thresholds. This is simpler than adding the `noise` crate (Story 1.6 will add noise-based distribution). For now, seeded RNG provides adequate deterministic variety.
- **Note about Story 1.6 (Noise-Based Distribution):** Story 1.6 will replace the simple threshold-based biome selection with proper noise layers using the `noise` crate. Story 1.2 should design `determine_biome()` so it can be swapped out cleanly. Keep the function signature stable: `fn determine_biome(seed, coord, config) -> BiomeType`.
- **Extend, don't replace:** Keep `generate_chunk_content()` signature compatible. Add `BiomeConfig` as a parameter. The function now calls `determine_biome()` first, then uses biome-specific spawn parameters.

### Code Patterns from Story 1.1 (MUST REUSE)

```rust
// Seed derivation pattern (generation.rs)
fn chunk_seed(world_seed: u64, coord: ChunkCoord) -> u64 {
    world_seed
        ^ (coord.x as u64).wrapping_mul(0x517cc1b727220a95)
        ^ (coord.y as u64).wrapping_mul(0x6c62272e07bb0142)
}

// Config loading pattern
impl BiomeConfig {
    pub fn from_ron(data: &str) -> Self {
        ron::from_str(data).unwrap_or_else(|e| {
            warn!("Failed to parse biome.ron: {e}, using defaults");
            Self::default()
        })
    }
}
```

### Previous Story Intelligence (from Story 1.1)

- **Entity budget enforcement requires running counter:** When loading multiple chunks in one tick, count must update after each chunk's spawns. Fixed in code review #1.
- **Deferred despawns inflate count:** `despawned_count` must be subtracted from total entity count. Fixed in code review #2.
- **Non-chunk collidables count:** Budget must count ALL collidable entities, not just chunk entities. Fixed in code review #3.
- **Velocity range panic:** When `min >= max` for velocity ranges, use `safe_speed()` helper. Fixed in code review #4.
- **Non-deterministic chunk order:** Sort `to_load`/`to_unload` vectors via `Ord` on `ChunkCoord`. Fixed in code review #3.
- **Test harness setup:** `tests/helpers/mod.rs` already includes `WorldConfig`, `ActiveChunks`, `update_chunks`. Add `BiomeConfig` resource to the test harness.

### Project Structure Notes

- **Files to modify:**
  - `src/world/generation.rs` — Add `BiomeType` enum, `determine_biome()`, extend `generate_chunk_content()`
  - `src/world/mod.rs` — Add `BiomeConfig` resource, load biome.ron, pass to generation, track biome in `ActiveChunks`
  - `assets/config/world.ron` — Remove per-entity spawn params (now in biome.ron)
  - `tests/helpers/mod.rs` — Add `BiomeConfig` to test harness
  - `tests/world_generation.rs` — Add biome-specific integration tests
- **Files to create:**
  - `assets/config/biome.ron` — Biome configuration
- **Files NOT to touch:**
  - `src/world/chunk.rs` — ChunkCoord math is biome-agnostic
  - `src/core/` — No core changes needed
  - `src/rendering/` — Visual biome distinction deferred
  - `src/shared/` — No shared component changes

### Key Libraries

- **`rand`** (already in Cargo.toml) — `StdRng::seed_from_u64`, `Rng::random::<f32>()` for biome threshold selection
- **`ron`** + **`serde`** (already in Cargo.toml) — Config deserialization for BiomeConfig
- **NO new dependencies needed** — `noise` crate deferred to Story 1.6

### References

- [Source: _bmad-output/planning-artifacts/game-architecture.md — "Noise Library Decision" and "Multi-Layered Noise Generation"]
- [Source: _bmad-output/planning-artifacts/game-architecture.md — "Project Structure - World Domain" (biomes.rs, noise_layers.rs)]
- [Source: _bmad-output/planning-artifacts/game-architecture.md — "Configuration Management Pattern"]
- [Source: _bmad-output/planning-artifacts/gdd.md — "Combat Environments: Arena and Level Design" (Deep Space, Asteroid Fields, Wreck Fields)]
- [Source: _bmad-output/planning-artifacts/gdd.md — "Open World (Post-Tutorial)" region types table]
- [Source: _bmad-output/planning-artifacts/gdd.md — "Visual Direction" for asteroids, wreck fields, background]
- [Source: _bmad-output/planning-artifacts/epics.md — Epic 1 stories, Story 1.2 user story]
- [Source: _bmad-output/implementation-artifacts/1-1-seamless-world-generation.md — Previous story patterns, code review fixes]

### Previous Story Intelligence (from Story 1.1)

**Learnings to apply:**
1. Start with unit tests for pure functions (determine_biome, generate_chunk_content) before integration tests
2. Test determinism early — it's the most common source of subtle bugs
3. Update test harness (helpers/mod.rs) FIRST before writing integration tests
4. Entity budget enforcement must be verified with biome-specific densities
5. Code reviews found 4 rounds of fixes in Story 1.1 — common patterns: budget enforcement, ordering, edge cases

**Code review patterns to watch for:**
- Biome threshold boundary conditions (exact 0.3 or 0.7 — which biome wins?)
- Budget enforcement when AsteroidField chunks dominate (high density)
- Config struct missing Default values causing panic
- Seeded RNG consumed in wrong order breaking determinism

### Git Intelligence

- **Last commit:** `feat(1.1): seamless world generation — chunk-based procedural world, WorldPlugin, seed-deterministic generation, entity budget enforcement. 32 new tests (22 unit + 10 integration).`
- **Commit convention:** `feat(1.2): biome types — deep space, asteroid fields, wreck fields, deterministic biome assignment, per-biome entity density. X new tests (Y unit + Z integration).`
- **VCS:** Using `jj` (Jujutsu). Use `jj new` after code review marks story done.
- **Total tests before this story:** 162 (92 unit + 70 integration)

## Dev Agent Record

### Agent Model Used

Claude Opus 4.6

### Debug Log References

- No debug issues encountered — clean implementation

### Completion Notes List

- Implemented `BiomeType` enum with 3 variants (`DeepSpace`, `AsteroidField`, `WreckField`) as Component in `generation.rs`
- Created `BiomeConfig` resource with `BiomeSpawnParams` per biome and configurable thresholds (0.3/0.7 default split)
- Created `determine_biome()` pure function using independent seed offset (`BIOME_SEED_OFFSET`) — biome selection is fully independent from entity generation RNG
- Extended `generate_chunk_content()` to accept `BiomeConfig` and `BiomeType`, uses biome-specific spawn parameters
- Migrated per-entity spawn parameters from `WorldConfig` to `BiomeConfig` — `WorldConfig` now only holds `seed`, `chunk_size`, `load_radius`, `entity_budget`
- Changed `ActiveChunks` from `HashSet<ChunkCoord>` to `HashMap<ChunkCoord, BiomeType>` for chunk-biome tracking
- `update_chunks` now calls `determine_biome()` per chunk, attaches `BiomeType` component to all spawned entities
- Created `assets/config/biome.ron` with per-biome tuning values
- Updated test harness with `BiomeConfig` resource
- 18 new tests (13 unit + 5 integration); total project: 180 tests (105 unit + 75 integration), 0 regressions

### File List

- `src/world/generation.rs` — MODIFIED: Added `BiomeType` enum, `determine_biome()`, `BIOME_SEED_OFFSET`, `biome` field on `EntityBlueprint`, updated `generate_chunk_content()` signature, 10 new unit tests
- `src/world/mod.rs` — MODIFIED: Added `BiomeSpawnParams`, `BiomeConfig` resource, `params_for()` method, `from_ron()` + `Default` for BiomeConfig; slimmed `WorldConfig` to 4 fields; changed `ActiveChunks` from `HashSet` to `HashMap`; updated `update_chunks` with biome determination and BiomeType component; added BiomeConfig loading in WorldPlugin; 4 new unit tests
- `assets/config/biome.ron` — NEW: Biome configuration with per-biome spawn parameters and thresholds
- `assets/config/world.ron` — MODIFIED: Removed per-entity spawn parameters (now in biome.ron), kept seed/chunk_size/load_radius/entity_budget
- `tests/helpers/mod.rs` — MODIFIED: Added `BiomeConfig` import and `app.insert_resource(BiomeConfig::default())`
- `tests/world_generation.rs` — MODIFIED: Updated existing tests for BiomeConfig/BiomeType API changes; 5 new integration tests (biome component, biome reload, biome variety, budget with biome, laser+biome)

### Change Log

- 2026-02-26: Implemented Story 1.2 — Biome Types. Three biome types with deterministic assignment, per-biome entity density, BiomeConfig, chunk-biome tracking. 15 new tests, 0 regressions.
- 2026-02-26: Code Review fixes — Strengthened AC6 tests (all 3 biomes + 60% dominance check), replaced no-op boundary test with 3 targeted boundary tests, added BiomeConfig threshold validation with warn!, added threshold validity unit test.
- 2026-02-26: Code Review #2 — Fixed 5 Clippy `field_reassign_with_default` warnings in integration tests (struct init syntax), added budget truncation bias comment in generation.rs, corrected test count in completion notes (180 total: 105 unit + 75 integration).
