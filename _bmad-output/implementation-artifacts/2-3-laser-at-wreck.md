# Story 2.3: Laser at Wreck

Status: done

## Story

As a player,
I find a laser at a nearby wreck that auto-docks on approach,
so that I gain my first weapon without UI complexity.

For this story's scope: After the player fires their first laser shot that hits the `TutorialWreck` entity, the tutorial phase advances from `Shooting` to the next phase. The wreck tracks whether it has been shot.

## Acceptance Criteria

1. A `TutorialWreck` component exists, spawned in the tutorial zone at a deterministic position within `safe_radius`
2. The `TutorialWreck` entity has a `WreckShotState` component tracking `has_been_shot: bool` (initially false)
3. The `TutorialWreck` entity has a `Collider` component so laser collision detection can hit it
4. When a laser hits the `TutorialWreck` entity, `WreckShotState::has_been_shot` is set to `true`
5. When `WreckShotState::has_been_shot` transitions to `true`, the `TutorialPhase` advances from `Shooting` to `SpreadUnlocked`
6. The phase advance only occurs once (idempotent — hitting the wreck again does nothing)
7. The wreck detection system runs in `FixedUpdate` in `CoreSet::Damage` (after damage is applied)
8. The wreck position is included in `TutorialLayout` and stored in `TutorialZone`
9. The `TutorialConfig` has `wreck_offset_min` and `wreck_offset_max` fields for wreck placement range
10. The wreck is placed within `safe_radius` of the zone center for all seeds 0..100

## Tasks / Subtasks

- [x] Task 1: Data structures — TutorialWreck, WreckShotState, TutorialConfig extensions (AC: #1, #2, #3, #8, #9)
  - [x] Add `TutorialWreck` marker component in `src/core/tutorial.rs`
  - [x] Add `WreckShotState { has_been_shot: bool }` component in `src/core/tutorial.rs`
  - [x] Add `wreck_offset_min: f32` and `wreck_offset_max: f32` to `TutorialConfig`
  - [x] Update `Default` for `TutorialConfig` with sensible wreck range (400–700)
  - [x] Update `TutorialLayout` to include `wreck_position: Vec2`
  - [x] Update `generate_tutorial_zone()` to generate a `wreck_position` within config range
- [x] Task 2: Spawn wreck entity in tutorial zone (AC: #1, #2, #3, #8)
  - [x] In `spawn_tutorial_zone`, spawn a `TutorialWreck` entity with `WreckShotState`, `Collider`, `Transform`, `Health`
  - [x] Store wreck position in the returned `TutorialZone` resource's layout
- [x] Task 3: Wreck shot detection system (AC: #4, #5, #6, #7)
  - [x] Add `advance_phase_on_wreck_shot` system in `src/core/tutorial.rs`
  - [x] Query `TutorialWreck` entities with `WreckShotState` + `JustDamaged` filter
  - [x] Use `JustDamaged` component (inserted by `apply_damage`) as the per-frame hit signal
  - [x] On first damage received: set `WreckShotState::has_been_shot = true`
  - [x] If phase is `Shooting` and `has_been_shot` is now `true`, transition to `SpreadUnlocked`
  - [x] Register system in `CoreSet::Damage` after `apply_damage`
- [x] Task 4: System registration in CorePlugin (AC: #7)
  - [x] Import `advance_phase_on_wreck_shot` in `src/core/mod.rs`
  - [x] Register system in the `.chain()` after `apply_damage` in `CoreSet::Damage`
- [x] Task 5: Unit tests for wreck detection logic (AC: #4, #5, #6)
  - [x] Test: `WreckShotState` starts with `has_been_shot = false`
  - [x] Test: wreck offset within config range for seeds 0..50
  - [x] Test: wreck within safe_radius for seeds 0..100
  - [x] Test: wreck position is deterministic
  - [x] Test: `TutorialConfig` from_ron includes new wreck offset fields
  - [x] Test: `validate_tutorial_layout` catches out-of-bounds wreck
- [x] Task 6: Integration tests (AC: #4, #5, #6, #7, #10)
  - [x] Test: tutorial_zone_spawns_wreck_entity — wreck spawns with tutorial zone
  - [x] Test: tutorial_wreck_spawns_with_shot_state_false
  - [x] Test: wreck_spawns_within_safe_radius_all_seeds (seeds 0..100)
  - [x] Test: phase_advances_shooting_to_spread_unlocked_when_wreck_hit
  - [x] Test: phase_does_not_advance_when_wreck_not_hit
  - [x] Test: phase_advance_is_idempotent_once_spread_unlocked
  - [x] Test: wreck_shot_state_set_true_on_first_hit

## Dev Notes

### Architecture Patterns

- **JustDamaged as hit detector:** The existing `apply_damage` system inserts a `JustDamaged` component on every entity that receives damage in the current frame. The `advance_phase_on_wreck_shot` system should query `TutorialWreck` entities that also have `JustDamaged` — this is the canonical signal that damage happened this frame.
- **TutorialPhase transition:** Use Bevy's `NextState<TutorialPhase>` resource to trigger state transitions. Do NOT set state directly on `State<TutorialPhase>`.
- **Idempotency:** Once `has_been_shot` is set to true, the system only advances the phase if `phase == TutorialPhase::Shooting`. Since Bevy states are exclusive, being in `SpreadUnlocked` or beyond means the condition is already false.
- **System ordering:** The system must run AFTER `apply_damage` (which sets `JustDamaged`) but within the same `CoreSet::Damage` chain frame. Use `.after(apply_damage)` within the Damage set.

### Existing Code to Reuse (DO NOT Reinvent)

- `src/core/tutorial.rs` — `TutorialConfig`, `TutorialLayout`, `generate_tutorial_zone`, `TutorialPhase`, `spawn_tutorial_zone` — all need extending
- `src/core/collision.rs` — `Collider`, `Health`, `JustDamaged` — use `JustDamaged` as the hit signal
- `src/shared/components.rs` — `JustDamaged { amount: f32 }` component (added by `apply_damage`)
- `src/core/mod.rs` — `CoreSet::Damage` set, existing `.chain()` pattern for system ordering
- `tests/helpers/mod.rs` — `test_app()` harness used in integration tests
- `tests/tutorial_zone.rs` — existing `tutorial_test_app()` helper

### Implementation Guidance

```rust
/// Advances TutorialPhase from Shooting to SpreadUnlocked when the TutorialWreck
/// is first hit by a laser. Uses JustDamaged as the per-frame damage signal.
pub fn advance_phase_on_wreck_shot(
    mut wreck_query: Query<&mut WreckShotState, (With<TutorialWreck>, With<JustDamaged>)>,
    phase: Res<State<TutorialPhase>>,
    mut next_phase: ResMut<NextState<TutorialPhase>>,
) {
    for mut shot_state in wreck_query.iter_mut() {
        if !shot_state.has_been_shot {
            shot_state.has_been_shot = true;
            if *phase.get() == TutorialPhase::Shooting {
                next_phase.set(TutorialPhase::SpreadUnlocked);
            }
        }
    }
}
```

Note: `JustDamaged` is inserted by `apply_damage`. It is NOT automatically removed each frame — it is an "event component" that persists until the entity is destroyed or the component is explicitly removed. For this story, the wreck will only ever transition once (idempotent via `has_been_shot` flag), so this is safe.

### File Structure

| File | Action | Purpose |
|------|--------|---------|
| `src/core/tutorial.rs` | MODIFY | Add TutorialWreck, WreckShotState, extend TutorialConfig/Layout/spawn, add advance_phase_on_wreck_shot system, add unit tests |
| `src/core/mod.rs` | MODIFY | Import and register advance_phase_on_wreck_shot in Damage set |
| `assets/config/tutorial.ron` | MODIFY | Add wreck_offset_min, wreck_offset_max fields |
| `tests/tutorial_zone.rs` | MODIFY | Add wreck integration tests |

### Testing Requirements

- **Unit tests** in `src/core/tutorial.rs`:
  - `WreckShotState` initializes `has_been_shot = false`
  - Wreck positions in generated layout are within config range for seeds 0..50
  - `TutorialConfig::from_ron()` round-trips with new wreck fields
- **Integration tests** in `tests/tutorial_zone.rs`:
  - Phase advances Shooting → SpreadUnlocked when wreck receives a `JustDamaged` component
  - Phase does NOT advance when wreck has no damage
  - Phase advance is idempotent
  - Wreck spawns within `safe_radius` for seeds 0..100
- **Pattern:** Use `#[deny(clippy::unwrap_used)]` — use `.expect()` in tests
- **Time:** Use `TimeUpdateStrategy::ManualDuration(Duration::from_secs_f32(1.0/60.0))` for deterministic tests

### Project Structure Notes

- No new files needed — extends existing `src/core/tutorial.rs` and `tests/tutorial_zone.rs`
- `assets/config/tutorial.ron` must be updated to add the two new offset fields
- System ordering: `advance_phase_on_wreck_shot` runs after `apply_damage` in `CoreSet::Damage`

### References

- [Source: _bmad-output/epics.md#Epic 2 — Story 3]
- [Source: src/core/tutorial.rs — TutorialConfig, TutorialPhase, spawn_tutorial_zone]
- [Source: src/core/collision.rs — JustDamaged, apply_damage, Collider, Health]
- [Source: src/core/mod.rs — CoreSet::Damage, system registration]
- [Source: tests/tutorial_zone.rs — tutorial_test_app() pattern]

### Key Bevy 0.18 Notes

- State transitions: use `ResMut<NextState<TutorialPhase>>` + `.set(TutorialPhase::SpreadUnlocked)`
- Query filter combining With<Component>: `Query<&mut WreckShotState, (With<TutorialWreck>, With<JustDamaged>)>`
- System ordering within a set: `.after(apply_damage)` inside `.in_set(CoreSet::Damage)` — implemented as `.chain()` in the damage chain
- `State<S>.get()` returns `&S` in Bevy 0.18
- State transitions require one additional `app.update()` to be visible in tests (NextState is applied at the start of the next frame)

## Dev Agent Record

### Agent Model Used

Claude Sonnet 4.6

### Debug Log References

- One test failure on first run: `phase_advances_shooting_to_spread_unlocked_when_wreck_hit` — the `NextState` transition set by `advance_phase_on_wreck_shot` is only visible after an additional `app.update()`. Fixed by adding a second `app.update()` call after the system runs.
- The `validate_tutorial_layout_catches_out_of_bounds` unit test required adding `wreck_position: Vec2::ZERO` since `TutorialLayout` gained a new field.

### Completion Notes List

- Task 1: Added `TutorialWreck` marker and `WreckShotState { has_been_shot: bool }` components. Extended `TutorialConfig` with `wreck_offset_min`/`wreck_offset_max` (default 400–700). Extended `TutorialLayout` with `wreck_position`. Updated `generate_tutorial_zone` to sample wreck position using seeded RNG after existing positions.
- Task 2: Spawned `TutorialWreck` entity in `spawn_tutorial_zone` with `WreckShotState`, `Collider { radius: 20.0 }`, `Health { current: 50.0, max: 50.0 }`, and `Transform` at the generated wreck position.
- Task 3: Implemented `advance_phase_on_wreck_shot` system using `JustDamaged` component (inserted by `apply_damage`) as the per-frame hit signal. Sets `has_been_shot = true` and transitions phase via `NextState`.
- Task 4: Added `advance_phase_on_wreck_shot` to the Damage chain in `CorePlugin`, positioned after `apply_damage` using `.chain()`.
- Task 5: 6 unit tests in `src/core/tutorial.rs` covering: wreck_shot_state_starts_not_shot, wreck_offset_within_config_range, wreck_within_safe_radius_all_seeds, wreck_position_is_deterministic, validate_tutorial_layout_catches_out_of_bounds_wreck, plus updated tutorial_config_from_ron and tutorial_config_default_has_valid_values to include new fields.
- Task 6: 6 integration tests in `tests/tutorial_zone.rs`: tutorial_zone_spawns_wreck_entity, tutorial_wreck_spawns_with_shot_state_false, wreck_spawns_within_safe_radius_all_seeds, phase_advances_shooting_to_spread_unlocked_when_wreck_hit, phase_does_not_advance_when_wreck_not_hit, phase_advance_is_idempotent_once_spread_unlocked, wreck_shot_state_set_true_on_first_hit.

### File List

- `src/core/tutorial.rs` — MODIFIED: Added TutorialWreck, WreckShotState components; extended TutorialConfig with wreck offset fields; extended TutorialLayout with wreck_position; updated generate_tutorial_zone; added advance_phase_on_wreck_shot system; updated validate_tutorial_layout; fixed TutorialLayout literal in test; added 6 new unit tests
- `src/core/mod.rs` — MODIFIED: Imported advance_phase_on_wreck_shot; added to Damage chain after apply_damage
- `assets/config/tutorial.ron` — MODIFIED: Added wreck_offset_min and wreck_offset_max fields
- `tests/tutorial_zone.rs` — MODIFIED: Added TutorialWreck, WreckShotState, JustDamaged imports; added wreck_phase_test_app helper; added 7 integration tests

## Change Log

- 2026-02-28: Implemented Story 2.3 Laser at Wreck — TutorialWreck entity, WreckShotState component, advance_phase_on_wreck_shot system, phase transition Shooting→SpreadUnlocked on first laser hit. 12 new tests. 366 total tests passing.
