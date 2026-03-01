# Story 2.7: Destruction Cascade

Status: done

## Story

As a player,
I witness an epic destruction cascade when the generator explodes so that the moment feels like a breakthrough.

For this story's scope: When `TutorialPhase::GeneratorDestroyed` is entered, a `CascadeTimer` resource is inserted with a 2-second countdown. Each frame the timer ticks. When it expires, all remaining tutorial-specific entities (`TutorialWreck`, `TutorialStation`) are despawned and the phase advances to `TutorialComplete`. A `TutorialComplete` variant is added to `TutorialPhase` as the final terminal state. This signals the end of the tutorial sequence. The cascade also emits a `TutorialComplete` `GameEvent` as a Tier1 milestone.

## Acceptance Criteria

1. A `CascadeTimer` resource is inserted (with a configurable delay, default 2.0s) when `TutorialPhase::GeneratorDestroyed` is entered via an `OnEnter` system
2. A `tick_cascade_timer` system runs each `FixedUpdate` frame while the phase is `GeneratorDestroyed`, decrementing the timer
3. When the timer expires: all `TutorialWreck` entities are despawned, all `TutorialStation` entities are despawned, and `TutorialPhase` advances to `TutorialComplete`
4. `TutorialPhase` gains a `TutorialComplete` variant as the final state
5. A `TutorialComplete` variant is added to `GameEventKind` and emitted with Tier1 severity when the cascade completes
6. `TutorialConfig` gains a `cascade_delay_secs: f32` field (default 2.0) used for the `CascadeTimer` duration
7. The cascade despawn is idempotent — if wrecks or stations are already gone, no panic occurs
8. All existing 100-seed validation tests still pass

## Tasks / Subtasks

- [x] Task 1: Data structures — TutorialPhase variant, GameEventKind variant, TutorialConfig field, CascadeTimer resource (AC: #1, #4, #5, #6)
  - [x] Add `TutorialComplete` variant to `TutorialPhase` enum after `GeneratorDestroyed`
  - [x] Add `cascade_delay_secs: f32` field to `TutorialConfig` struct (default 2.0)
  - [x] Update `TutorialConfig::from_ron` test with the new field in the RON string
  - [x] Add `TutorialComplete` variant to `GameEventKind` in `src/shared/events.rs`
  - [x] Add `TutorialComplete → Tier1` to `EventSeverityConfig::default` in `src/infrastructure/events.rs`
  - [x] Add match arm for `TutorialComplete` in `EventSeverityConfig::severity_for`
  - [x] Add `TutorialComplete` to the `known_keys` slice in `EventSeverityConfig::validate`
  - [x] Update `severity_config_default_has_all_mappings` test to expect 12 mappings
  - [x] Add `CascadeTimer` resource struct to `src/core/tutorial.rs`
- [x] Task 2: `start_destruction_cascade` system (AC: #1, #6)
  - [x] Add system in `src/core/tutorial.rs` registered `OnEnter(TutorialPhase::GeneratorDestroyed)`
  - [x] Reads `TutorialConfig.cascade_delay_secs` and inserts `CascadeTimer { remaining: cascade_delay_secs }`
- [x] Task 3: `tick_cascade_timer` system (AC: #2, #3, #7)
  - [x] Add system in `src/core/tutorial.rs`
  - [x] Guards: phase must be `GeneratorDestroyed`; `CascadeTimer` resource must exist
  - [x] Decrements timer by `time.delta_secs()`
  - [x] When timer <= 0.0: despawn all `TutorialWreck` and `TutorialStation` entities, emit `GameEvent::TutorialComplete`, advance phase to `TutorialComplete`
  - [x] Remove `CascadeTimer` resource after cascade fires to prevent repeated triggering
- [x] Task 4: System registration in CorePlugin (AC: #1, #2)
  - [x] Import new types/systems in `src/core/mod.rs`
  - [x] Register `start_destruction_cascade` as `OnEnter(TutorialPhase::GeneratorDestroyed)`
  - [x] Register `tick_cascade_timer` in `CoreSet::Events`
- [x] Task 5: Unit tests in `src/core/tutorial.rs` (AC: #4, #6)
  - [x] Test: `tutorial_phase_tutorial_complete_variant_exists`
  - [x] Test: `tutorial_phase_sequence_all_variants_distinct` — updated to include 7 variants
  - [x] Test: `cascade_timer_can_be_constructed_with_positive_remaining`
  - [x] Test: `tutorial_config_cascade_delay_default_positive`
  - [x] Test: `tutorial_config_cascade_delay_default_value`
- [x] Task 6: Integration tests in `tests/tutorial_zone.rs` (AC: #1, #2, #3, #7, #8)
  - [x] Test: `cascade_timer_inserted_on_generator_destroyed_phase`
  - [x] Test: `cascade_timer_not_inserted_in_other_phases`
  - [x] Test: `phase_advances_to_tutorial_complete_after_cascade_delay`
  - [x] Test: `tutorial_wreck_despawned_after_cascade`
  - [x] Test: `tutorial_station_despawned_after_cascade`
  - [x] Test: `cascade_is_idempotent_no_panic_without_wreck_or_station`
  - [x] Test: `cascade_timer_removed_after_cascade_fires`
  - [x] Test: `hundred_seed_validation_still_passes_after_cascade`

## Dev Notes

### Architecture Patterns

- **OnEnter system:** `start_destruction_cascade` uses `OnEnter(TutorialPhase::GeneratorDestroyed)` — runs exactly once on phase entry.
- **CascadeTimer resource:** Inserted by `start_destruction_cascade`, consumed by `tick_cascade_timer`. Use `commands.remove_resource::<CascadeTimer>()` after firing to prevent re-triggering.
- **Despawn pattern:** Use `commands.entity(e).despawn()` for each wreck/station entity. Query returns empty if already gone — idempotent by design.
- **Phase guard:** `tick_cascade_timer` guards with `if *phase.get() != TutorialPhase::GeneratorDestroyed { return; }`.
- **Event emission:** Same pattern as `check_generator_destroyed` — use `MessageWriter<GameEvent>`.

### New Structures

```rust
/// Timer resource inserted when the GeneratorDestroyed phase is entered.
/// Counts down to zero, then triggers cascade despawn and phase advance.
#[derive(Resource, Debug)]
pub struct CascadeTimer {
    pub remaining: f32,
}
```

### TutorialConfig extension

```rust
pub struct TutorialConfig {
    // ... existing fields ...
    /// Duration in seconds before cascade despawn after generator is destroyed
    pub cascade_delay_secs: f32,
}

impl Default for TutorialConfig {
    fn default() -> Self {
        Self {
            // ... existing defaults ...
            cascade_delay_secs: 2.0,
        }
    }
}
```

### TutorialPhase extension

```rust
pub enum TutorialPhase {
    Flying,
    Shooting,
    SpreadUnlocked,
    Complete,
    StationVisited,
    GeneratorDestroyed,
    TutorialComplete,  // ← new: final terminal state after cascade
}
```

### start_destruction_cascade system

```rust
pub fn start_destruction_cascade(
    mut commands: Commands,
    config: Res<TutorialConfig>,
) {
    commands.insert_resource(CascadeTimer {
        remaining: config.cascade_delay_secs,
    });
}
```

### tick_cascade_timer system

```rust
pub fn tick_cascade_timer(
    mut commands: Commands,
    time: Res<Time>,
    phase: Res<State<TutorialPhase>>,
    mut next_phase: ResMut<NextState<TutorialPhase>>,
    cascade_timer: Option<ResMut<CascadeTimer>>,
    wreck_query: Query<Entity, With<TutorialWreck>>,
    station_query: Query<Entity, With<TutorialStation>>,
    mut game_events: bevy::ecs::message::MessageWriter<crate::shared::events::GameEvent>,
    time_for_event: Res<Time>,
    severity_config: Res<crate::infrastructure::events::EventSeverityConfig>,
) {
    if *phase.get() != TutorialPhase::GeneratorDestroyed {
        return;
    }
    let Some(mut timer) = cascade_timer else { return };
    timer.remaining -= time.delta_secs();
    if timer.remaining <= 0.0 {
        // Despawn tutorial entities
        for entity in wreck_query.iter() {
            commands.entity(entity).despawn();
        }
        for entity in station_query.iter() {
            commands.entity(entity).despawn();
        }
        // Emit completion event
        let kind = crate::shared::events::GameEventKind::TutorialComplete;
        game_events.write(crate::shared::events::GameEvent {
            severity: severity_config.severity_for(&kind),
            kind,
            position: Vec2::ZERO,
            game_time: time_for_event.elapsed_secs_f64(),
        });
        // Advance phase
        next_phase.set(TutorialPhase::TutorialComplete);
        // Remove timer to prevent re-triggering
        commands.remove_resource::<CascadeTimer>();
    }
}
```

### File Structure

| File | Action | Purpose |
|------|--------|---------|
| `src/core/tutorial.rs` | MODIFY | Add TutorialComplete to TutorialPhase, cascade_delay_secs to TutorialConfig, CascadeTimer resource, start_destruction_cascade and tick_cascade_timer systems, unit tests |
| `src/shared/events.rs` | MODIFY | Add TutorialComplete variant to GameEventKind |
| `src/infrastructure/events.rs` | MODIFY | Add TutorialComplete mapping, match arm, known_keys entry; update test |
| `src/core/mod.rs` | MODIFY | Import new systems; register OnEnter and FixedUpdate systems |
| `tests/tutorial_zone.rs` | MODIFY | Add integration tests |

### Testing Requirements

- **Unit tests** in `src/core/tutorial.rs`:
  - `TutorialPhase::TutorialComplete` variant exists and is distinct from all other variants
  - `CascadeTimer` can be constructed with a positive remaining value
  - `TutorialConfig.cascade_delay_secs` default is positive
- **Integration tests** in `tests/tutorial_zone.rs`:
  - `CascadeTimer` resource is inserted when entering `GeneratorDestroyed` phase
  - Phase advances to `TutorialComplete` after cascade delay ticks down (tick enough frames)
  - `TutorialWreck` entities are despawned after cascade fires
  - `TutorialStation` entities are despawned after cascade fires
  - Cascade does not panic when no wreck or station entities exist
  - All 100 seeds still pass layout validation
- **Pattern:** `#[deny(clippy::unwrap_used)]` — use `.expect()` in tests
- **Time:** `TimeUpdateStrategy::ManualDuration(Duration::from_secs_f32(1.0/60.0))`

### References

- [Source: _bmad-output/epics.md#Epic 2 — Story 7]
- [Source: src/core/tutorial.rs — TutorialPhase, TutorialConfig, GravityWellGenerator, check_generator_destroyed]
- [Source: src/core/mod.rs — CoreSet::Events, OnEnter registration pattern]
- [Source: src/shared/events.rs — GameEventKind, EventSeverity]
- [Source: src/infrastructure/events.rs — EventSeverityConfig]
- [Source: tests/tutorial_zone.rs — tutorial_test_app(), generator_destroyed_test_app()]
- [Source: _bmad-output/implementation-artifacts/2-6-destroy-generator.md — story format reference]

### Key Bevy 0.18 Notes

- `OnEnter(TutorialPhase::GeneratorDestroyed)` runs exactly once on phase entry — correct for inserting the one-shot timer
- `Option<ResMut<CascadeTimer>>` — gracefully handles the case where the resource has not yet been inserted or was removed
- `commands.remove_resource::<CascadeTimer>()` — safe even if resource doesn't exist; prevents double-fire
- `commands.entity(e).despawn()` — safe for entities that have child hierarchies (no children here)
- Timer must tick in `FixedUpdate` so `time.delta_secs()` is deterministic in tests

## Dev Agent Record

### Agent Model Used

Claude Sonnet 4.6

### Debug Log References

No failures during implementation — all tests passed on first run.

### Completion Notes List

- Task 1: Added `TutorialComplete` variant to `TutorialPhase` after `GeneratorDestroyed`. Added `cascade_delay_secs: f32` field to `TutorialConfig` with default 2.0. Updated `tutorial_config_from_ron` test to include new field. Updated `tutorial_config_default_has_valid_values` test to check the new field. Added `TutorialComplete` variant to `GameEventKind` in `src/shared/events.rs`. Updated `EventSeverityConfig` with new mapping (`Tier1`), match arm, `known_keys` entry, and updated the mapping count tests from 11 to 12. Added `CascadeTimer { remaining: f32 }` resource struct in `src/core/tutorial.rs`.
- Task 2: Implemented `start_destruction_cascade` system in `src/core/tutorial.rs`. Runs `OnEnter(TutorialPhase::GeneratorDestroyed)`. Reads `config.cascade_delay_secs` and inserts `CascadeTimer { remaining }`.
- Task 3: Implemented `tick_cascade_timer` system in `src/core/tutorial.rs`. Guards on `TutorialPhase::GeneratorDestroyed` and `Option<ResMut<CascadeTimer>>`. Decrements timer by `time.delta_secs()`. On expiry: despawns all `TutorialWreck` and `TutorialStation` entities (idempotent — empty queries are safe), emits `GameEvent::TutorialComplete` (Tier1), advances phase to `TutorialComplete`, removes `CascadeTimer` resource via `commands.remove_resource`.
- Task 4: Imported `start_destruction_cascade` and `tick_cascade_timer` in `src/core/mod.rs`. Registered `start_destruction_cascade` as `OnEnter(TutorialPhase::GeneratorDestroyed)`. Registered `tick_cascade_timer` in `CoreSet::Events` with `.after(check_generator_destroyed)`.
- Task 5: Added 4 unit tests in `src/core/tutorial.rs`: `tutorial_phase_tutorial_complete_variant_exists`, `cascade_timer_can_be_constructed_with_positive_remaining`, `tutorial_config_cascade_delay_default_positive`, `tutorial_config_cascade_delay_default_value`. Updated `tutorial_phase_sequence_all_variants_distinct` to include 7 variants.
- Task 6: Added 8 integration tests in `tests/tutorial_zone.rs` plus `cascade_test_app()` helper. All pass. Total test count: 409.
- Updated `assets/config/tutorial.ron` to include `cascade_delay_secs: 2.0`.

### File List

- `src/core/tutorial.rs` — MODIFIED: Added `TutorialComplete` to `TutorialPhase`, `cascade_delay_secs` to `TutorialConfig`, `CascadeTimer` resource, `start_destruction_cascade` system, `tick_cascade_timer` system, 4 new unit tests, updated 2 existing tests
- `src/shared/events.rs` — MODIFIED: Added `TutorialComplete` variant to `GameEventKind`
- `src/infrastructure/events.rs` — MODIFIED: Added `TutorialComplete → Tier1` mapping, match arm, known_keys entry; updated mapping count tests from 11 to 12
- `src/core/mod.rs` — MODIFIED: Imported `start_destruction_cascade` and `tick_cascade_timer`; registered `OnEnter(GeneratorDestroyed)` system and `CoreSet::Events` system
- `tests/tutorial_zone.rs` — MODIFIED: Added imports for new types/systems; added `cascade_test_app()` helper and 8 new integration tests
- `assets/config/tutorial.ron` — MODIFIED: Added `cascade_delay_secs: 2.0`
- `_bmad-output/implementation-artifacts/2-7-destruction-cascade.md` — CREATED

## Change Log

- 2026-02-28: Implemented Story 2.7 Destruction Cascade — TutorialComplete phase, CascadeTimer resource, start_destruction_cascade and tick_cascade_timer systems, TutorialComplete event. 12 new tests (4 unit + 8 integration). Total: 409 tests.
