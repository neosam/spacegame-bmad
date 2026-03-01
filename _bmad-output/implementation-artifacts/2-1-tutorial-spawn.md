# Story 2.1: Tutorial Spawn

Status: review

<!-- Note: Validation is optional. Run validate-create-story for quality check before dev-story. -->

## Story

As a player,
I want to spawn in a contained tutorial zone with a visibly defective station,
so that I have an immediate point of interest and implicit guidance to explore.

## Acceptance Criteria

1. Player spawns at a defined position inside the tutorial zone (not at world origin)
2. A defective tutorial station entity spawns within visible range of the player
3. A gravity well generator entity spawns, defining the tutorial boundary
4. The tutorial zone layout is procedurally generated from the world seed
5. The safe radius area is free of gravity pull — player can practice flying unimpeded
6. All tutorial entities (station, generator) spawn within the safe radius
7. The tutorial zone coexists with the existing chunk/biome system without conflicts
8. A `TutorialPhase` state machine starts in `Flying` phase (only thrust + rotate available)
9. Constraint validation: station must be reachable from player spawn
10. 100-seed automated test: all seeds produce valid, completable tutorial layouts

## Tasks / Subtasks

- [x] Task 1: Tutorial zone config and data structures (AC: #1, #3, #4, #6)
  - [x] Create `assets/config/tutorial.ron` with `TutorialConfig` (safe_radius, pull_strength, station_offset_range, generator_offset_range)
  - [x] Add `TutorialConfig` resource with `from_ron()` + `Default` fallback pattern
  - [x] Add `GravityWellGenerator` component (safe_radius, pull_strength, health, requires_projectile)
  - [x] Add `TutorialStation` marker component (defective: bool)
  - [x] Add `TutorialZone` resource tracking zone center, seed, and layout metadata
- [x] Task 2: Tutorial phase state machine (AC: #8)
  - [x] Define `TutorialPhase` as Bevy `States` enum: `Flying`, `Shooting`, `SpreadUnlocked`, `Complete`
  - [x] Register state with `init_state::<TutorialPhase>()` defaulting to `Flying`
  - [x] Weapon systems must gate on `TutorialPhase` — laser unavailable in `Flying`, spread unavailable until `SpreadUnlocked`
- [x] Task 3: Tutorial zone generation (AC: #1, #2, #3, #4, #6, #9)
  - [x] `generate_tutorial_zone(seed: u64, config: &TutorialConfig) -> TutorialLayout` — deterministic layout from seed
  - [x] Place station within `station_offset_range` of zone center
  - [x] Place generator at zone boundary edge
  - [x] Player spawn point near zone center (slight offset for visual interest)
  - [x] Validate layout constraints internally (all entities within safe_radius, station reachable)
- [x] Task 4: Spawn systems (AC: #1, #2, #3, #7)
  - [x] `spawn_tutorial_zone` Startup system — spawns player, station, generator at layout positions
  - [x] Integrate with existing `WorldPlugin` — tutorial zone occupies chunk(0,0) area, chunk generation skips tutorial chunks
  - [x] Emit `GameEvent` on tutorial zone spawn for logbook
- [x] Task 5: Constraint validation and 100-seed test (AC: #9, #10)
  - [x] `TutorialConstraints` struct with validation logic
  - [x] `validate_tutorial_seed(seed) -> Result<(), Vec<ConstraintViolation>>`
  - [x] Integration test: iterate seeds 0..100, assert all pass validation
  - [x] Unit tests for individual constraint checks

## Dev Notes

### Architecture Patterns

- **Core/Rendering separation:** Tutorial zone logic goes in `src/core/` (or new `src/tutorial/`). Rendering (station visual, generator visual) will be separate stories. This story is logic-only.
- **Config loading pattern:** Follow existing `FlightConfig`/`WeaponConfig` pattern — RON file, `from_ron()` with `warn!` fallback to `Default`.
- **State machine:** Use Bevy 0.18 `States` derive macro. See existing `GameState` in `src/game_states.rs` for pattern.
- **Component markers:** Follow `NeedsLaserVisual` pattern from rendering — add marker components that rendering stories will consume later.

### Existing Code to Reuse (DO NOT Reinvent)

- `src/world/generation.rs` — `deterministic_rng(seed, chunk)` pattern for seed-based RNG. Reuse the seeded RNG approach.
- `src/world/mod.rs` — `WorldConfig.seed` is the world seed. Tutorial zone derives its seed from this.
- `src/core/spawning.rs` — `SpawnPoint { x, y }` struct already exists. Reuse for tutorial entity positions.
- `src/infrastructure/events.rs` — `GameEvent` emission pattern. Use for tutorial spawn event.
- `src/core/collision.rs` — `Health` component for generator health.
- `tests/helpers/mod.rs` — `test_app()` harness. Extend with tutorial config resource.

### Integration Points

- **Chunk system interaction:** Tutorial zone must coexist with chunk loading. Options:
  1. Mark tutorial chunks as "occupied" in `ActiveChunks` so `generate_chunk_content` skips them
  2. Or generate tutorial zone as a special chunk type
  - Preferred: Option 1 — simpler, less invasive to existing chunk system
- **Save system:** Player position already saved in `PlayerSave`. Tutorial phase state should be saveable (future story concern, not this story).
- **Weapon gating:** `TutorialPhase::Flying` must disable weapon firing. Modify weapon systems to check state, OR use a `WeaponsLocked` component on player.

### File Structure

| File | Action | Purpose |
|------|--------|---------|
| `src/core/tutorial.rs` | CREATE | TutorialConfig, TutorialPhase, TutorialZone, GravityWellGenerator, TutorialStation, TutorialConstraints, spawn system, validation |
| `src/core/mod.rs` | MODIFY | Register tutorial module, add systems to CorePlugin |
| `assets/config/tutorial.ron` | CREATE | Tutorial balance config |
| `tests/helpers/mod.rs` | MODIFY | Add TutorialConfig to test_app() resources |
| `tests/tutorial_zone.rs` | CREATE | 100-seed validation test + unit tests |
| `src/core/weapons.rs` | MODIFY | Gate weapon firing on TutorialPhase state |

### Testing Requirements

- **Unit tests** in `src/core/tutorial.rs`:
  - `generate_tutorial_zone` produces deterministic layout for same seed
  - Different seeds produce different layouts
  - All generated entities fall within safe_radius
  - Station is within reachable distance from player spawn
  - TutorialConstraints validation catches invalid layouts
- **Integration tests** in `tests/tutorial_zone.rs`:
  - 100-seed validation test (iterate 0..100, all must pass)
  - Tutorial zone spawns correct entities in test_app
  - TutorialPhase starts as Flying
  - Weapon firing blocked in Flying phase
  - Tutorial zone doesn't conflict with chunk generation
- **Pattern:** Use `#[deny(clippy::unwrap_used)]` — use `.expect()` in tests
- **Time:** Use `TimeUpdateStrategy::ManualDuration` for deterministic tests

### Project Structure Notes

- New module `src/core/tutorial.rs` aligns with core domain (game logic, not rendering)
- Config file `assets/config/tutorial.ron` follows existing `assets/config/*.ron` pattern
- Integration test file `tests/tutorial_zone.rs` follows existing `tests/*.rs` pattern
- No rendering in this story — visual representation handled in later stories (2-2 through 2-7)

### References

- [Source: _bmad-output/epics.md#Epic 2 — Story 1]
- [Source: _bmad-output/game-architecture.md#Gravity Well Tutorial pattern]
- [Source: _bmad-output/gdd.md#Tutorial Zone section]
- [Source: src/world/generation.rs — deterministic RNG pattern]
- [Source: src/core/spawning.rs — SpawnPoint struct]
- [Source: src/core/mod.rs — CorePlugin system registration]
- [Source: tests/helpers/mod.rs — test_app() harness]

### Key Bevy 0.18 Notes

- `States` derive: `#[derive(States, Clone, Eq, PartialEq, Debug, Hash, Default)]`
- Use `init_state::<TutorialPhase>()` in plugin build
- State-conditional systems: `.run_if(in_state(TutorialPhase::Flying))`
- Startup systems: `.add_systems(Startup, spawn_tutorial_zone)`

## Dev Agent Record

### Agent Model Used
Claude Opus 4.6

### Debug Log References
- Fixed `gen` reserved keyword issue in test (renamed to `generator`)
- Fixed StatesPlugin missing from test_app (added `bevy::state::app::StatesPlugin`)
- Fixed Startup systems not running in tests (created `tutorial_test_app()` helper that registers systems before prime frame)
- Fixed `world()` vs `world_mut()` for query_filtered API

### Completion Notes List
- Task 1: Created `TutorialConfig` resource with RON loading, `GravityWellGenerator` component, `TutorialStation` marker, `TutorialZone` resource
- Task 2: Defined `TutorialPhase` States enum with 4 variants, registered in CorePlugin, weapon gating via `WeaponsLocked` component
- Task 3: Implemented `generate_tutorial_zone()` pure function with seeded RNG, deterministic layout placement within config ranges
- Task 4: `spawn_tutorial_zone` Startup system spawns player/station/generator, marks tutorial chunks as occupied in ActiveChunks
- Task 5: `validate_tutorial_layout()` and `validate_tutorial_seed()` with ConstraintViolation reporting, 100-seed integration test

### File List
- `src/core/tutorial.rs` — NEW: Tutorial module (config, components, state machine, generation, validation, spawn system)
- `src/core/mod.rs` — MODIFIED: Register tutorial module, systems, config loading
- `src/core/weapons.rs` — MODIFIED: Added WeaponsLocked check in fire_weapon
- `assets/config/tutorial.ron` — NEW: Tutorial balance configuration
- `tests/helpers/mod.rs` — MODIFIED: Added TutorialConfig, TutorialPhase, StatesPlugin
- `tests/tutorial_zone.rs` — NEW: Integration tests (100-seed validation, entity spawning, phase state, weapon gating, chunk coexistence)

## Change Log
- 2026-02-28: Implemented Story 2.1 Tutorial Spawn — tutorial zone config, TutorialPhase state machine, deterministic zone generation, spawn system, chunk coexistence, weapon gating, constraint validation. 344 tests passing (25 new tutorial tests).
