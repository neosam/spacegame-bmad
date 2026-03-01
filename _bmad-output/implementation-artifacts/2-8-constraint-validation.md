# Story 2.8: Constraint Validation

Status: done

## Story

As a developer,
the tutorial zone generates valid layouts for any seed with constraint validation so that no player gets stuck.

For this story's scope: A `validate_tutorial_config` startup system checks all `TutorialConfig` field constraints at app startup, emitting `warn!()` for each violated constraint. This follows the same pattern as `validate_speed_cap` in `flight.rs` (Story 1.11). This is a defensive programming / developer quality-of-life feature — the system never panics, it only warns.

## Acceptance Criteria

1. A `validate_tutorial_config` startup system is added to `src/core/tutorial.rs`
2. The system checks all the following constraints and emits a `warn!()` for each violation:
   - `safe_radius > 0.0`
   - `wreck_offset_max <= safe_radius` (wreck stays inside safe zone)
   - `wreck_offset_min <= wreck_offset_max`
   - `wreck_offset_min >= 0.0`
   - `dock_radius > 0.0`
   - `dock_radius <= safe_radius`
   - `tutorial_enemy_count > 0`
   - `tutorial_enemy_spawn_radius > 0.0`
   - `cascade_delay_secs > 0.0`
3. The system is registered as a `Startup` system in `CorePlugin` in `src/core/mod.rs`
4. All existing tests still pass (100-seed validation)
5. New unit tests verify the constraint logic against valid and invalid configs
6. New integration tests verify that valid config produces no panic and invalid config does not panic (only warns)

## Tasks / Subtasks

- [x] Task 1: Implement `validate_tutorial_config` system in `src/core/tutorial.rs` (AC: #1, #2)
  - [x] Add public `validate_tutorial_config` system function after the existing `TutorialConfig` impl block
  - [x] Check `safe_radius > 0.0`, warn if violated
  - [x] Check `wreck_offset_max <= safe_radius`, warn if violated
  - [x] Check `wreck_offset_min <= wreck_offset_max`, warn if violated
  - [x] Check `wreck_offset_min >= 0.0`, warn if violated
  - [x] Check `dock_radius > 0.0`, warn if violated
  - [x] Check `dock_radius <= safe_radius`, warn if violated
  - [x] Check `tutorial_enemy_count > 0`, warn if violated
  - [x] Check `tutorial_enemy_spawn_radius > 0.0`, warn if violated
  - [x] Check `cascade_delay_secs > 0.0`, warn if violated
- [x] Task 2: Register `validate_tutorial_config` as a `Startup` system in `CorePlugin` (AC: #3)
  - [x] Import `validate_tutorial_config` in `src/core/mod.rs`
  - [x] Register with `app.add_systems(Startup, validate_tutorial_config)`
- [x] Task 3: Unit tests in `src/core/tutorial.rs` (AC: #5)
  - [x] Test: `validate_tutorial_config_default_config_passes_all_constraints` — pure logic check on default config
  - [x] Test: `validate_tutorial_config_zero_safe_radius_is_invalid`
  - [x] Test: `validate_tutorial_config_negative_safe_radius_is_invalid`
  - [x] Test: `validate_tutorial_config_wreck_offset_max_exceeds_safe_radius`
  - [x] Test: `validate_tutorial_config_wreck_offset_min_exceeds_max`
  - [x] Test: `validate_tutorial_config_negative_wreck_offset_min_is_invalid`
  - [x] Test: `validate_tutorial_config_zero_dock_radius_is_invalid`
  - [x] Test: `validate_tutorial_config_dock_radius_exceeds_safe_radius`
  - [x] Test: `validate_tutorial_config_zero_enemy_count_is_invalid`
  - [x] Test: `validate_tutorial_config_zero_enemy_spawn_radius_is_invalid`
  - [x] Test: `validate_tutorial_config_zero_cascade_delay_is_invalid`
  - [x] Test: `validate_tutorial_config_negative_cascade_delay_is_invalid`
  - [x] Test: `validate_tutorial_config_system_runs_without_panic_valid_config` (Bevy app unit test)
  - [x] Test: `validate_tutorial_config_system_runs_without_panic_invalid_config` (Bevy app unit test)
- [x] Task 4: Integration tests in `tests/tutorial_zone.rs` (AC: #4, #6)
  - [x] Test: `validate_tutorial_config_system_runs_without_panic_on_valid_config`
  - [x] Test: `validate_tutorial_config_system_runs_without_panic_on_invalid_config`
  - [x] Test: `validate_tutorial_config_accepts_valid_safe_radius`
  - [x] Test: `validate_tutorial_config_wreck_offset_max_within_safe_radius_is_valid`
  - [x] Test: `validate_tutorial_config_dock_radius_within_safe_radius_is_valid`
  - [x] Test: `validate_tutorial_config_wreck_offset_ordering_valid_by_default`
  - [x] Test: `hundred_seed_validation_still_passes_after_constraint_validation`

## Dev Notes

### Architecture Patterns

- **Startup validation pattern:** Same as `validate_speed_cap` in `src/core/flight.rs` — reads config via `Res<TutorialConfig>`, emits `warn!()` per violation, never panics.
- **Pure logic helpers:** Constraint checks are pure boolean expressions on `TutorialConfig` fields, easily testable without ECS setup.
- **Graceful degradation:** Validation only warns — app continues running even with misconfigured values. Developers see the warnings, fix them before shipping.
- **No new data structures:** This story only adds a function and tests. No new resources, events, or components.

### validate_tutorial_config system

```rust
/// Startup system: validates all `TutorialConfig` field constraints.
/// Emits `warn!()` for each violated constraint. Never panics.
/// Pattern mirrors `validate_speed_cap` in flight.rs (Story 1.11).
pub fn validate_tutorial_config(config: Res<TutorialConfig>) {
    if config.safe_radius <= 0.0 {
        warn!(
            "TutorialConfig: safe_radius ({}) must be > 0.0",
            config.safe_radius
        );
    }
    if config.wreck_offset_max > config.safe_radius {
        warn!(
            "TutorialConfig: wreck_offset_max ({}) exceeds safe_radius ({}). Wrecks may generate outside safe zone.",
            config.wreck_offset_max, config.safe_radius
        );
    }
    if config.wreck_offset_min > config.wreck_offset_max {
        warn!(
            "TutorialConfig: wreck_offset_min ({}) exceeds wreck_offset_max ({}). No valid wreck placement exists.",
            config.wreck_offset_min, config.wreck_offset_max
        );
    }
    if config.wreck_offset_min < 0.0 {
        warn!(
            "TutorialConfig: wreck_offset_min ({}) is negative. Use 0.0 or higher.",
            config.wreck_offset_min
        );
    }
    if config.dock_radius <= 0.0 {
        warn!(
            "TutorialConfig: dock_radius ({}) must be > 0.0",
            config.dock_radius
        );
    }
    if config.dock_radius > config.safe_radius {
        warn!(
            "TutorialConfig: dock_radius ({}) exceeds safe_radius ({}). Station may be unreachable for docking.",
            config.dock_radius, config.safe_radius
        );
    }
    if config.tutorial_enemy_count == 0 {
        warn!("TutorialConfig: tutorial_enemy_count is 0. No tutorial enemies will spawn.");
    }
    if config.tutorial_enemy_spawn_radius <= 0.0 {
        warn!(
            "TutorialConfig: tutorial_enemy_spawn_radius ({}) must be > 0.0",
            config.tutorial_enemy_spawn_radius
        );
    }
    if config.cascade_delay_secs <= 0.0 {
        warn!(
            "TutorialConfig: cascade_delay_secs ({}) must be > 0.0",
            config.cascade_delay_secs
        );
    }
}
```

### File Structure

| File | Action | Purpose |
|------|--------|---------|
| `src/core/tutorial.rs` | MODIFY | Add `validate_tutorial_config` system + unit tests |
| `src/core/mod.rs` | MODIFY | Import and register `validate_tutorial_config` as Startup system |
| `tests/tutorial_zone.rs` | MODIFY | Add integration tests for the validation system |

### Testing Requirements

- **Unit tests** in `src/core/tutorial.rs`:
  - Default config should satisfy all constraints (check each condition directly)
  - Each constraint individually violated
  - Pattern: pure boolean logic checks, no ECS needed
- **Integration tests** in `tests/tutorial_zone.rs`:
  - `validate_tutorial_config` Startup system runs without panic on valid config
  - `validate_tutorial_config` Startup system runs without panic on invalid config (only warns)
  - All 100 seeds still pass `validate_tutorial_seed`
- **Pattern:** `#[deny(clippy::unwrap_used)]` — use `.expect()` in tests

### References

- [Source: _bmad-output/epics.md#Epic 2 — Story 8]
- [Source: src/core/flight.rs — validate_speed_cap (Story 1.11 reference pattern)]
- [Source: src/core/tutorial.rs — TutorialConfig, existing constraints]
- [Source: src/core/mod.rs — CorePlugin, Startup system registration]
- [Source: tests/tutorial_zone.rs — tutorial_test_app(), test patterns]
- [Source: _bmad-output/implementation-artifacts/2-7-destruction-cascade.md — story format reference]

### Key Bevy 0.18 Notes

- Startup systems run once after plugins are built. `Res<TutorialConfig>` is available because `CorePlugin::build` inserts it before `add_systems(Startup, ...)`.
- `warn!()` is the Bevy/tracing macro — no import needed beyond `bevy::prelude::*`.
- No log capture in unit tests — verify system "runs without panic" only; warning text is not asserted.

## Dev Agent Record

### Agent Model Used

Claude Sonnet 4.6

### Debug Log References

No failures during implementation — all tests passed on first run.

### Completion Notes List

- Task 1: Implemented `validate_tutorial_config` startup system in `src/core/tutorial.rs`. The function reads `Res<TutorialConfig>` and emits `warn!()` for each of 9 constraint violations: `safe_radius > 0.0`, `wreck_offset_max <= safe_radius`, `wreck_offset_min <= wreck_offset_max`, `wreck_offset_min >= 0.0`, `dock_radius > 0.0`, `dock_radius <= safe_radius`, `tutorial_enemy_count > 0`, `tutorial_enemy_spawn_radius > 0.0`, `cascade_delay_secs > 0.0`. Pattern mirrors `validate_speed_cap` in `flight.rs` exactly.
- Task 2: Imported `validate_tutorial_config` in `src/core/mod.rs` and registered it as `app.add_systems(Startup, validate_tutorial_config)` alongside `validate_speed_cap` and `spawn_tutorial_zone`.
- Task 3: Added 14 unit tests in `src/core/tutorial.rs`: 1 test verifying all constraints pass on default config (pure logic, no ECS), 11 tests each verifying a specific invalid value triggers the expected condition, and 2 Bevy app unit tests verifying the system runs without panic on both valid and invalid configs.
- Task 4: Added 7 integration tests in `tests/tutorial_zone.rs` plus `constraint_validation_test_app()` helper. All tests verify no-panic behavior on valid and invalid configs, plus the 100-seed layout validation still passes.

### File List

- `src/core/tutorial.rs` — MODIFIED: Added `validate_tutorial_config` public system function before the Components section; added 14 unit tests in the existing `tests` module
- `src/core/mod.rs` — MODIFIED: Imported `validate_tutorial_config`; registered as `Startup` system
- `tests/tutorial_zone.rs` — MODIFIED: Imported `validate_tutorial_config`; added `constraint_validation_test_app()` helper and 7 integration tests
- `_bmad-output/implementation-artifacts/2-8-constraint-validation.md` — CREATED

## Change Log

- 2026-02-28: Implemented Story 2.8 Constraint Validation — `validate_tutorial_config` startup system with 9 constraint checks, 14 unit tests, 7 integration tests. Total test count: 430 (was 409, +21).
