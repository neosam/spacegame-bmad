# Story 1.6: Noise Biome Distribution

Status: done

## Story

As a developer,
I want biome distribution to be determined by noise layers,
so that the world is seed-deterministic with spatially coherent biome regions.

## Acceptance Criteria

1. **Noise-based biome selection** — `determine_biome()` uses continuous noise (Perlin or Simplex) instead of per-chunk RNG hash, producing spatially coherent biome regions where adjacent chunks tend to share biomes.
2. **Seed-deterministic** — Same `(seed, chunk_coord)` always produces the same biome. All existing determinism guarantees are preserved.
3. **Noise layers module** — New file `src/world/noise_layers.rs` encapsulates all noise logic, exported via `WorldPlugin`.
4. **Architecture-specified pattern** — Noise evaluation follows `noise(chunk_x, chunk_y, seed + LAYER_OFFSET)` per the game architecture. `BIOME_NOISE_OFFSET` constant replaces `BIOME_SEED_OFFSET`.
5. **Configurable noise parameters** — `BiomeConfig` gains noise tuning fields (`noise_scale`, `noise_octaves`, `noise_persistence`, `noise_lacunarity`) with `#[serde(default)]` for backward compatibility with existing `biome.ron`.
6. **Biome thresholds preserved** — The threshold-based biome classification (`deep_space_threshold`, `asteroid_field_threshold`) continues to work, but now operates on noise output `[-1.0, 1.0]` remapped to `[0.0, 1.0]`.
7. **All 3 biome types present** — Across a 20x20 sample area, all three biomes appear and no single biome exceeds 60% (same constraint as before).
8. **Entity generation unchanged** — `generate_chunk_content()` is NOT modified. Entity placement still uses the hash-based `chunk_seed()` + `StdRng` for per-entity randomness.
9. **Backward compatible** — All 236 existing tests pass. `biome.ron` without noise fields uses defaults via `#[serde(default)]`.
10. **`noise` crate dependency** — Add `noise = "0.9"` (or latest stable) to `Cargo.toml`.

## Tasks / Subtasks

- [x] Task 1: Add `noise` dependency (AC: #10)
  - [x] 1.1 Add `noise` crate to `Cargo.toml` dependencies
  - [x] 1.2 Verify it compiles: `cargo check`

- [x] Task 2: Create `src/world/noise_layers.rs` (AC: #3, #4)
  - [x] 2.1 Create new module file `src/world/noise_layers.rs`
  - [x] 2.2 Define `BIOME_NOISE_OFFSET: u64` constant for layer separation
  - [x] 2.3 Implement `pub fn biome_noise_value(seed: u64, coord: ChunkCoord, config: &BiomeNoiseConfig) -> f64` that:
    - Creates a seeded Perlin/Fbm noise generator using `seed + BIOME_NOISE_OFFSET`
    - Evaluates at `(coord.x as f64 * scale, coord.y as f64 * scale)`
    - Returns raw noise value in `[-1.0, 1.0]`
  - [x] 2.4 Implement `pub fn noise_to_unit(value: f64) -> f32` that remaps `[-1.0, 1.0]` → `[0.0, 1.0]` with clamping
  - [x] 2.5 Add `pub mod noise_layers;` to `src/world/mod.rs`

- [x] Task 3: Extend `BiomeConfig` with noise parameters (AC: #5, #9)
  - [x] 3.1 Add `BiomeNoiseConfig` struct (or inline fields) to `BiomeConfig`:
    - `noise_scale: f64` (default: `0.37`) — controls biome region size; smaller = larger regions
    - `noise_octaves: usize` (default: `4`)
    - `noise_persistence: f64` (default: `0.5`)
    - `noise_lacunarity: f64` (default: `2.0`)
  - [x] 3.2 All new fields use `#[serde(default)]` for backward compat with existing `biome.ron`
  - [x] 3.3 Add `BiomeNoiseConfig::default()` with sensible values
  - [x] 3.4 Update `biome.ron` with noise parameters (optional — defaults are fine)

- [x] Task 4: Refactor `determine_biome()` to use noise (AC: #1, #2, #4, #6)
  - [x] 4.1 Replace hash+RNG logic with call to `biome_noise_value()` + `noise_to_unit()`
  - [x] 4.2 Keep threshold comparison: `if value < deep_space_threshold { DeepSpace } else if value < asteroid_field_threshold { AsteroidField } else { WreckField }`
  - [x] 4.3 Remove `BIOME_SEED_OFFSET` constant from `generation.rs` (moved to `noise_layers.rs` as `BIOME_NOISE_OFFSET`)
  - [x] 4.4 Update `determine_biome()` signature to accept noise config (or extract from `BiomeConfig`)
  - [x] 4.5 Keep `chunk_seed()` function in `generation.rs` — it's still used by `generate_chunk_content()`

- [x] Task 5: Update callers of `determine_biome()` (AC: #8, #9)
  - [x] 5.1 Update `update_chunks()` in `mod.rs` to pass noise config to `determine_biome()`
  - [x] 5.2 Update all test calls to `determine_biome()` with new signature
  - [x] 5.3 Verify `generate_chunk_content()` is completely unchanged

- [x] Task 6: Unit tests for noise_layers (AC: #1, #2, #3, #7)
  - [x] 6.1 `biome_noise_value_is_deterministic` — same seed+coord = same value
  - [x] 6.2 `biome_noise_value_varies_spatially` — different coords produce different values
  - [x] 6.3 `noise_to_unit_clamps_to_0_1` — edge cases (-2.0, 2.0) clamped
  - [x] 6.4 `noise_to_unit_remaps_correctly` — -1.0 → 0.0, 0.0 → 0.5, 1.0 → ~1.0
  - [x] 6.5 `adjacent_chunks_tend_to_share_biome` — sample 100 adjacent pairs, >50% same biome (spatial coherence)

- [x] Task 7: Update existing biome tests in `generation.rs` (AC: #2, #7, #9)
  - [x] 7.1 Update `determine_biome_is_deterministic` for new signature
  - [x] 7.2 Update `determine_biome_variety_across_coords` — still all 3 biomes, none >60%
  - [x] 7.3 Update threshold boundary tests (`all_deep_space`, `no_deep_space`, `all_wreck_field`)
  - [x] 7.4 Verify `generate_chunk_is_deterministic` still passes unchanged
  - [x] 7.5 Verify entity count/property tests still pass unchanged

- [x] Task 8: Integration test for spatial coherence (AC: #1, #7)
  - [x] 8.1 `noise_biomes_are_spatially_coherent` — In a loaded world, adjacent chunks share biomes more than random (>50% of adjacent pairs vs ~33% for uniform random)
  - [x] 8.2 `noise_biomes_seed_deterministic_across_reload` — Unload and reload chunks, verify same biomes assigned
  - [x] 8.3 `all_existing_integration_tests_pass` — Run full suite, no regressions

## Dev Notes

### Architecture Patterns & Constraints

- **Core/Rendering separation** — All changes in `src/world/`. No rendering changes needed. World map and minimap read `ActiveChunks`/`ExploredChunks` which still contain `BiomeType` — unchanged interface.
- **No unwrap()** — `#[deny(clippy::unwrap_used)]` enforced. Use `.expect()` in tests only.
- **Config backward compat** — Use `#[serde(default = "...")]` for all new fields (pattern from Story 1-5: `max_chunks_per_frame`).
- **Plugin isolation** — `noise_layers.rs` is a pure function module. No ECS dependencies. Import `ChunkCoord` from `chunk.rs`, noise config from `mod.rs`.

### What Changes vs What Stays

**CHANGES:**
- `src/world/noise_layers.rs` — NEW file with noise evaluation functions
- `src/world/generation.rs` — `determine_biome()` refactored to call noise instead of hash+RNG. Remove `BIOME_SEED_OFFSET`. Keep `chunk_seed()`, `generate_chunk_content()`, all blueprint types.
- `src/world/mod.rs` — Add `pub mod noise_layers;`. Extend `BiomeConfig` with noise params. Update `determine_biome()` call in `update_chunks()`.
- `Cargo.toml` — Add `noise` dependency.
- `assets/config/biome.ron` — Optionally add noise parameters.

**STAYS THE SAME:**
- `src/world/chunk.rs` — No changes (coordinate math).
- `src/world/mod.rs` — `update_chunks()` logic flow unchanged (3-phase: unload/queue/load). Only the `determine_biome()` call signature changes.
- `generate_chunk_content()` — Entity generation completely unchanged. Uses `chunk_seed()` + `StdRng` for per-entity randomness.
- All resources: `ActiveChunks`, `ExploredChunks`, `ChunkEntityIndex`, `PendingChunks`, `ChunkLoadState`, `WorldConfig`.
- Rendering systems, minimap, world map — read same data types.

### Implementation Guidance

**Noise crate usage pattern:**
```rust
use noise::{NoiseFn, Perlin, Fbm, Seedable, MultiFractal};

pub fn biome_noise_value(seed: u64, coord: ChunkCoord, config: &BiomeNoiseConfig) -> f64 {
    let noise_seed = seed.wrapping_add(BIOME_NOISE_OFFSET);
    let fbm = Fbm::<Perlin>::new(noise_seed as u32)
        .set_octaves(config.octaves)
        .set_persistence(config.persistence)
        .set_lacunarity(config.lacunarity);
    fbm.get([coord.x as f64 * config.scale, coord.y as f64 * config.scale])
}
```

**Noise output range:** Fbm with Perlin typically outputs in roughly `[-1.0, 1.0]`. Remap with `(value + 1.0) / 2.0` then clamp to `[0.0, 1.0]` before threshold comparison.

**noise_scale tuning:** A `scale` of `0.1` means biome regions span ~10 chunks. Smaller scale = larger regions. This is the most gameplay-impactful parameter — playtest to find the sweet spot.

**Seed truncation:** The `noise` crate uses `u32` seeds. Truncate `u64` with `as u32`. The `BIOME_NOISE_OFFSET` ensures this differs from any future faction/boss noise layers.

### Previous Story Intelligence (1-5: Chunk Loading)

- **3-phase chunk loading** established — unload/queue/load. Only the `determine_biome()` call inside Phase 3 changes.
- **Config backward compat pattern** — `#[serde(default = "default_fn")]` works. Follow exactly the same pattern for noise params.
- **Test harness** in `tests/helpers/mod.rs` — No changes needed (no new resources).
- **`run_until_loaded()` helper** — Available for integration tests requiring multi-frame staggered loading.
- **236 existing tests** — Must all pass. Run `cargo test` before and after.

### Key Files to Touch

| File | Action |
|------|--------|
| `Cargo.toml` | Add `noise` dependency |
| `src/world/noise_layers.rs` | CREATE — noise evaluation functions |
| `src/world/mod.rs` | Add `pub mod noise_layers`, extend `BiomeConfig`, update `determine_biome()` call |
| `src/world/generation.rs` | Refactor `determine_biome()`, remove `BIOME_SEED_OFFSET` |
| `assets/config/biome.ron` | Optionally add noise params |
| `tests/noise_biome.rs` | CREATE — integration tests for spatial coherence |

### References

- [Source: _bmad-output/planning-artifacts/game-architecture.md — Noise Library Decision #8: `noise` crate, `noise(chunk_x, chunk_y, seed + LAYER_OFFSET)` pattern]
- [Source: _bmad-output/planning-artifacts/game-architecture.md — Project Structure: `src/world/noise_layers.rs`]
- [Source: _bmad-output/planning-artifacts/game-architecture.md — Multi-layered noise: biomes + factions + bosses as overlapping noise layers]
- [Source: _bmad-output/planning-artifacts/epics.md — Epic 1 Story 6: "biome distribution is determined by noise layers so that the world is seed-deterministic"]
- [Source: src/world/generation.rs — Current `determine_biome()` using hash+StdRng]
- [Source: src/world/mod.rs — `BiomeConfig` struct with thresholds]
- [Source: 1-5-chunk-loading.md — Config backward compat pattern, test harness]

## Dev Agent Record

### Agent Model Used
Claude Opus 4.6

### Debug Log References
- FBM noise amplitude normalization: Raw FBM output concentrates around 0; contrast factor 4.0 applied after dividing by theoretical max amplitude to spread distribution for threshold-based biome selection.
- noise_scale=0.37 chosen (avoids Perlin integer lattice points where noise=0, provides ~3-chunk biome regions).
- noise_to_unit returns [0.0, 1.0) exclusive upper bound to match previous RNG [0,1) behavior for threshold comparisons.
- Integration test `chunk_unload_despawns_entities` updated to find a chunk with actual entities rather than hardcoding (-2,-2) which may be empty DeepSpace with noise.

### Completion Notes List
- Task 1: Added `noise = "0.9"` to Cargo.toml, compiles successfully.
- Task 2: Created `src/world/noise_layers.rs` with `BIOME_NOISE_OFFSET`, `BiomeNoiseConfig`, `biome_noise_value()`, `noise_to_unit()`. Added `pub mod noise_layers;` to `mod.rs`.
- Task 3: Added `noise: BiomeNoiseConfig` field to `BiomeConfig` with `#[serde(default)]`. All fields have serde defaults. `biome.ron` backward-compatible (no changes needed).
- Task 4: Refactored `determine_biome()` to use `biome_noise_value()` + `noise_to_unit()`. Removed `BIOME_SEED_OFFSET` from `generation.rs` (now `BIOME_NOISE_OFFSET` in `noise_layers.rs`). Threshold comparison unchanged. `chunk_seed()` preserved for entity generation.
- Task 5: `determine_biome()` signature unchanged (noise config extracted from `BiomeConfig`). `update_chunks()` call site unchanged. `generate_chunk_content()` completely untouched.
- Task 6: 5 unit tests in `noise_layers.rs`: determinism, spatial variation, clamp bounds, remap correctness, adjacent coherence (>50%).
- Task 7: All existing biome tests in `generation.rs` pass with noise. Threshold boundary tests pass. Determinism tests pass. Entity generation tests unchanged and passing.
- Task 8: 2 integration tests in `tests/noise_biome.rs`: spatial coherence (adjacent chunks share biomes > random) and seed determinism across reload. Full 238-test suite passes with 0 regressions.

### File List
- `Cargo.toml` — Modified: added `noise = "0.9"` dependency
- `src/world/noise_layers.rs` — New: noise evaluation functions, BiomeNoiseConfig, BIOME_NOISE_OFFSET, 5 unit tests
- `src/world/mod.rs` — Modified: added `pub mod noise_layers;`, added `noise: BiomeNoiseConfig` to `BiomeConfig` with `#[serde(default)]`
- `src/world/generation.rs` — Modified: `determine_biome()` refactored to use noise, removed `BIOME_SEED_OFFSET`, removed `rand` imports for biome (kept for entity gen)
- `tests/noise_biome.rs` — New: 2 integration tests for spatial coherence and seed determinism across reload
- `tests/world_generation.rs` — Modified: `chunk_unload_despawns_entities` updated to dynamically find chunk with entities
- `_bmad-output/implementation-artifacts/1-6-noise-biome-distribution.md` — Modified: status, tasks, dev agent record
- `_bmad-output/implementation-artifacts/sprint-status.yaml` — Modified: 1-6 status → in-progress → review

## Change Log
- 2026-02-27: Implemented noise-based biome distribution replacing hash+RNG. Added `noise = 0.9` crate, `BiomeNoiseConfig` with configurable parameters, FBM Perlin noise with amplitude normalization and contrast. All 238 tests pass (236 existing + 2 new integration tests). 7 files changed, 0 regressions.
- 2026-02-27: Code review fixes — (1) Integration test coherence assertion strengthened from >33% to >50%. (2) `biome_noise_value()` docstring corrected re: actual output range. (3) Added `BiomeNoiseConfig::validate()` for degenerate parameter detection, called at plugin startup. (4) Added RON deserialization test with explicit noise params. 239 tests pass.
