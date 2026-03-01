# Story 2.6: Destroy Generator

Status: done

## Story

As a player,
I can attack and destroy the generator using Spread so that I experience my first "boss" moment and earn freedom.

For this story's scope: The `GravityWellGenerator` entity gains a `Collider` so that weapons can hit it. When the player shoots the generator and reduces its health to zero, `despawn_destroyed` (already implemented) removes the entity. A new system `check_generator_destroyed` detects the generator's absence and advances `TutorialPhase` from `StationVisited` to `GeneratorDestroyed`. Because the generator entity is gone, `apply_gravity_well` naturally stops pulling the player — the gravity well effect ceases without any special handling. A `GameEvent::GeneratorDestroyed` event is emitted as a Tier1 event. This is the escape from the tutorial zone.

## Acceptance Criteria

1. `GravityWellGenerator` gains a `Collider` component when spawned so that laser and spread weapons can damage it
2. When `GravityWellGenerator` health reaches 0, `despawn_destroyed` removes the entity (this is already implemented; no change needed)
3. A `check_generator_destroyed` system detects when no `GravityWellGenerator` entity exists and the phase is `StationVisited`, then advances to `GeneratorDestroyed`
4. `TutorialPhase` gains a `GeneratorDestroyed` variant that follows `StationVisited`
5. A `GeneratorDestroyed` variant is added to `GameEventKind` and emitted with Tier1 severity when the generator is destroyed
6. The gravity well pull stops naturally when the generator entity is despawned (no special code needed — `apply_gravity_well` queries the missing entity and produces no force)
7. The phase transition is idempotent — once in `GeneratorDestroyed`, `check_generator_destroyed` does nothing
8. The system is registered in `CoreSet::Events` after `despawn_destroyed` (same frame ordering)
9. `GravityWellGenerator` requires Spread weapon to destroy (`requires_projectile: true` already set in spawn)
10. All existing 100-seed validation tests still pass

## Tasks / Subtasks

- [x] Task 1: Data structures — TutorialPhase variant, GameEventKind variant, EventSeverityConfig mapping (AC: #4, #5)
  - [x] Add `GeneratorDestroyed` variant to `TutorialPhase` enum after `StationVisited`
  - [x] Add `GeneratorDestroyed` variant to `GameEventKind` in `src/shared/events.rs`
  - [x] Add `GeneratorDestroyed → Tier1` to `EventSeverityConfig::default` in `src/infrastructure/events.rs`
  - [x] Add match arm for `GeneratorDestroyed` in `EventSeverityConfig::severity_for`
  - [x] Add `GeneratorDestroyed` to the `known_keys` slice in `EventSeverityConfig::validate`
  - [x] Update `severity_config_default_has_all_mappings` test to expect 11 mappings
- [x] Task 2: Collider on generator spawn (AC: #1)
  - [x] Add `Collider { radius: 30.0 }` to the `GravityWellGenerator` spawn bundle in `spawn_tutorial_zone`
- [x] Task 3: `check_generator_destroyed` system (AC: #3, #6, #7)
  - [x] Add system in `src/core/tutorial.rs`
  - [x] System guards: phase must be `StationVisited`
  - [x] System checks: no `GravityWellGenerator` entities exist in the world
  - [x] On destruction detected: emit `GameEvent::GeneratorDestroyed`, advance phase to `GeneratorDestroyed`
- [x] Task 4: System registration in CorePlugin (AC: #8)
  - [x] Import `check_generator_destroyed` in `src/core/mod.rs`
  - [x] Register `check_generator_destroyed` in `CoreSet::Events` after `dock_at_station`
- [x] Task 5: Unit tests in `src/core/tutorial.rs` (AC: #4, #5)
  - [x] Test: `tutorial_phase_generator_destroyed_variant_exists`
  - [x] Test: `tutorial_phase_sequence_all_variants_distinct` (verifies all 6 variants distinct)
- [x] Task 6: Integration tests in `tests/tutorial_zone.rs` (AC: #1, #3, #6, #7, #10)
  - [x] Test: `generator_has_collider`
  - [x] Test: `generator_collider_radius_positive`
  - [x] Test: `phase_advances_to_generator_destroyed_when_generator_gone`
  - [x] Test: `phase_stays_station_visited_while_generator_alive`
  - [x] Test: `generator_destroyed_is_idempotent`
  - [x] Test: `check_generator_not_triggered_in_non_station_visited_phase`
  - [x] Test: `gravity_well_stops_when_generator_despawned_integration`
  - [x] Test: `hundred_seed_validation_still_passes_with_generator_collider`

## Dev Notes

### Architecture Patterns

- **Collider on generator:** The `check_laser_collisions` and `check_projectile_collisions` systems both query `(Entity, &Transform, &Collider), (With<Health>, Without<Player>)`. Adding `Collider` to the generator spawn bundle is sufficient — no other code changes needed for weapons to damage it.
- **Phase guard:** `check_generator_destroyed` guards with `if *phase.get() != TutorialPhase::StationVisited { return; }` — idempotent once past `StationVisited`.
- **Generator absence detection:** Query `Query<Entity, With<GravityWellGenerator>>`. If `.iter().next().is_none()`, the generator is gone.
- **Gravity well stops automatically:** `apply_gravity_well` iterates `generator_query: Query<(&GravityWellGenerator, &Transform)>`. When the entity is despawned, the query returns no results, so no force is applied. Zero-cost fix.
- **Event emission:** Use `bevy::ecs::message::MessageWriter<GameEvent>` (same pattern as `dock_at_station`).

### Existing Code to Reuse

- `src/core/tutorial.rs` — `TutorialPhase`, `GravityWellGenerator`, `spawn_tutorial_zone` — extend directly
- `src/core/collision.rs` — `despawn_destroyed` already handles generator despawn; `Collider` component
- `src/shared/events.rs` — `GameEventKind` — add variant
- `src/infrastructure/events.rs` — `EventSeverityConfig` — add mapping
- `src/core/mod.rs` — `CoreSet::Events` — register system
- `tests/tutorial_zone.rs` — `tutorial_test_app()`, `station_docking_test_app()` — follow same patterns

### Implementation Guidance

```rust
/// Advances TutorialPhase from StationVisited to GeneratorDestroyed when
/// the GravityWellGenerator entity no longer exists.
/// The gravity well pull stops naturally because apply_gravity_well finds
/// no generator entities to iterate.
pub fn check_generator_destroyed(
    phase: Res<State<TutorialPhase>>,
    mut next_phase: ResMut<NextState<TutorialPhase>>,
    generator_query: Query<Entity, With<GravityWellGenerator>>,
    mut game_events: bevy::ecs::message::MessageWriter<crate::shared::events::GameEvent>,
    time: Res<Time>,
    severity_config: Res<crate::infrastructure::events::EventSeverityConfig>,
) {
    if *phase.get() != TutorialPhase::StationVisited {
        return;
    }
    if generator_query.iter().next().is_none() {
        let kind = crate::shared::events::GameEventKind::GeneratorDestroyed;
        game_events.write(crate::shared::events::GameEvent {
            severity: severity_config.severity_for(&kind),
            kind,
            position: Vec2::ZERO,
            game_time: time.elapsed_secs_f64(),
        });
        next_phase.set(TutorialPhase::GeneratorDestroyed);
    }
}
```

**TutorialPhase extension:**

```rust
pub enum TutorialPhase {
    Flying,
    Shooting,
    SpreadUnlocked,
    Complete,
    StationVisited,
    GeneratorDestroyed,  // ← new: generator gone, gravity well dissolved, tutorial over
}
```

**GameEventKind extension:**

```rust
pub enum GameEventKind {
    // ... existing ...
    GeneratorDestroyed,
}
```

**spawn_tutorial_zone — add Collider to generator bundle:**

```rust
commands.spawn((
    GravityWellGenerator { ... },
    crate::core::collision::Health { ... },
    crate::core::collision::Collider { radius: 30.0 },  // ← add this
    Transform::from_translation(layout.generator_position.extend(0.0)),
));
```

### File Structure

| File | Action | Purpose |
|------|--------|---------|
| `src/core/tutorial.rs` | MODIFY | Add GeneratorDestroyed to TutorialPhase, add Collider to generator spawn, add check_generator_destroyed system, unit tests |
| `src/shared/events.rs` | MODIFY | Add GeneratorDestroyed variant to GameEventKind |
| `src/infrastructure/events.rs` | MODIFY | Add GeneratorDestroyed mapping, match arm, known_keys entry; update test |
| `src/core/mod.rs` | MODIFY | Import check_generator_destroyed; register in CoreSet::Events |
| `tests/tutorial_zone.rs` | MODIFY | Add integration tests |

### Testing Requirements

- **Unit tests** in `src/core/tutorial.rs`:
  - `TutorialPhase::GeneratorDestroyed` variant exists and is distinct from `StationVisited`
- **Integration tests** in `tests/tutorial_zone.rs`:
  - Generator has `Collider` component after spawn
  - Phase advances to `GeneratorDestroyed` when `GravityWellGenerator` is absent and phase is `StationVisited`
  - Phase stays `StationVisited` while generator entity is alive
  - Gravity well stops when generator is despawned
  - Transition is idempotent once `GeneratorDestroyed`
- **Pattern:** `#[deny(clippy::unwrap_used)]` — use `.expect()` in tests
- **Time:** `TimeUpdateStrategy::ManualDuration(Duration::from_secs_f32(1.0/60.0))`

### References

- [Source: _bmad-output/epics.md#Epic 2 — Story 6]
- [Source: src/core/tutorial.rs — TutorialPhase, GravityWellGenerator, spawn_tutorial_zone, dock_at_station]
- [Source: src/core/collision.rs — despawn_destroyed, Collider, Health]
- [Source: src/shared/events.rs — GameEventKind, EventSeverity]
- [Source: src/infrastructure/events.rs — EventSeverityConfig]
- [Source: src/core/mod.rs — CoreSet::Events, system registration]
- [Source: tests/tutorial_zone.rs — tutorial_test_app(), station_docking_test_app()]
- [Source: _bmad-output/implementation-artifacts/2-5-station-spread-weapon.md — story format reference]

### Key Bevy 0.18 Notes

- `Query<Entity, With<GravityWellGenerator>>.iter().next().is_none()` — check for entity absence
- Phase guard: `if *phase.get() != TutorialPhase::StationVisited { return; }`
- `commands.entity(entity).despawn()` — handled by `despawn_destroyed`; no special code needed
- Phase transition visible in tests after one additional `app.update()`
- The `despawn_destroyed` system queries `Without<Player>` and checks `health.current <= 0.0` — generator qualifies automatically once it has `Health`

## Dev Agent Record

### Agent Model Used

Claude Sonnet 4.6

### Debug Log References

- One test failure fixed: `phase_stays_station_visited_while_generator_alive` — the generator entity must be spawned BEFORE the `NextState` is set and `app.update()` is called. The `check_generator_destroyed` system runs in `FixedUpdate` during the same `app.update()` that applies the state transition. If the entity is spawned after the transition frame, the system sees an empty query and fires prematurely.

### Completion Notes List

- Task 1: Added `GeneratorDestroyed` variant to `TutorialPhase` after `StationVisited`. Added `GeneratorDestroyed` variant to `GameEventKind` in `src/shared/events.rs`. Updated `EventSeverityConfig` with new mapping (`Tier1`), match arm, `known_keys` entry, and updated the `severity_config_default_has_all_mappings` test from 10 to 11 mappings, and the validate test too.
- Task 2: Added `crate::core::collision::Collider { radius: 30.0 }` to the `GravityWellGenerator` spawn bundle in `spawn_tutorial_zone`. This enables both laser (ray-circle) and spread projectile (circle-circle) weapons to hit and damage the generator.
- Task 3: Implemented `check_generator_destroyed` system in `src/core/tutorial.rs`. Guards on `TutorialPhase::StationVisited`. Checks `generator_query.iter().next().is_none()`. On absence: emits `GameEvent { kind: GeneratorDestroyed, severity: Tier1 }`, advances phase to `GeneratorDestroyed`. The gravity well stops automatically because `apply_gravity_well` iterates `Query<(&GravityWellGenerator, &Transform)>` — no entities, no force.
- Task 4: Imported `check_generator_destroyed` in `src/core/mod.rs`. Registered it in `CoreSet::Events` with `.after(dock_at_station)`.
- Task 5: Added 2 unit tests in `src/core/tutorial.rs`: `tutorial_phase_generator_destroyed_variant_exists` and `tutorial_phase_sequence_all_variants_distinct`.
- Task 6: Added 8 integration tests in `tests/tutorial_zone.rs`. All pass. Total test count: 397.

### File List

- `src/core/tutorial.rs` — MODIFIED: Added `GeneratorDestroyed` to `TutorialPhase`, added `Collider { radius: 30.0 }` to generator spawn bundle, added `check_generator_destroyed` system, added 2 unit tests
- `src/shared/events.rs` — MODIFIED: Added `GeneratorDestroyed` variant to `GameEventKind`
- `src/infrastructure/events.rs` — MODIFIED: Added `GeneratorDestroyed → Tier1` mapping, match arm, known_keys entry; updated mapping count tests from 10 to 11
- `src/core/mod.rs` — MODIFIED: Imported `check_generator_destroyed`; registered in `CoreSet::Events` after `dock_at_station`
- `tests/tutorial_zone.rs` — MODIFIED: Added import for `check_generator_destroyed`; added 8 new integration tests and `generator_destroyed_test_app()` helper
- `_bmad-output/implementation-artifacts/2-6-destroy-generator.md` — CREATED

## Change Log

- 2026-02-28: Implemented Story 2.6 Destroy Generator — GeneratorDestroyed phase, Collider on generator, check_generator_destroyed system, GeneratorDestroyed event. 10 new tests (2 unit + 8 integration). Total: 397 tests.
