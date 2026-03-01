# Story 2.5: Station Spread Weapon

Status: done

## Story

As a player,
I dock at the station and receive the Spread weapon so that I learn weapon switching and energy management.

For this story's scope: When the player approaches within `dock_radius` of the `TutorialStation` entity after the tutorial wave is complete (`TutorialPhase::Complete`), the station grants the Spread weapon by marking it unlocked on the player entity. The tutorial phase advances from `Complete` to `StationVisited`. The station transforms from defective to functional (`defective: false`). A `GameEvent::StationDocked` event is emitted.

## Acceptance Criteria

1. `TutorialConfig` gains a `dock_radius: f32` field (default: 150.0) — the proximity threshold for docking
2. `TutorialPhase` gains a `StationVisited` variant that follows `Complete` in the sequence
3. A `SpreadUnlocked` marker component exists to indicate the player has received the Spread weapon from the station
4. A `dock_at_station` system runs each `FixedUpdate` frame that detects player proximity to `TutorialStation` within `dock_radius` when phase is `Complete`
5. When docking occurs: the player receives a `SpreadUnlocked` marker component, the phase advances from `Complete` to `StationVisited`, and the station's `defective` field is set to `false`
6. The dock transition is idempotent — if the phase is already `StationVisited`, no further transitions occur
7. The dock system is registered in `CoreSet::Events` (after Damage) in `FixedUpdate`
8. A `StationDocked` event variant is added to `GameEventKind` and emitted when docking occurs
9. `SpreadUnlocked` component is added to the player on docking — tests verify the component is present after docking
10. All 100 seeds produce valid tutorial layouts (dock_radius fits within station placement range)

## Tasks / Subtasks

- [x] Task 1: Data structures — TutorialConfig extension, TutorialPhase variant, SpreadUnlocked marker, GameEventKind variant (AC: #1, #2, #3, #8)
  - [x] Add `dock_radius: f32` to `TutorialConfig` (default: 150.0)
  - [x] Update `TutorialConfig::Default` with new field
  - [x] Update `tutorial_config_default_has_valid_values` unit test to assert dock_radius > 0
  - [x] Update `tutorial_config_from_ron` unit test to include dock_radius field
  - [x] Add `StationVisited` variant to `TutorialPhase` enum after `Complete`
  - [x] Add `SpreadUnlocked` marker component in `src/core/tutorial.rs`
  - [x] Add `StationDocked` variant to `GameEventKind` in `src/shared/events.rs`
- [x] Task 2: Dock system implementation (AC: #4, #5, #6, #8, #9)
  - [x] Add `dock_at_station` system in `src/core/tutorial.rs`
  - [x] System reads `TutorialConfig` for `dock_radius`
  - [x] System queries `TutorialStation` Transform for station position
  - [x] System queries `Player` Transform + `Entity` for distance check
  - [x] System checks phase is `Complete` before acting
  - [x] On proximity match: insert `SpreadUnlocked` on player, set `TutorialStation.defective = false`, advance phase to `StationVisited`
  - [x] Emit `GameEvent { kind: StationDocked, severity: Tier1, ... }`
- [x] Task 3: System registration in CorePlugin (AC: #7)
  - [x] Import `dock_at_station`, `SpreadUnlocked` in `src/core/mod.rs`
  - [x] Register `dock_at_station` in `CoreSet::Events` in `FixedUpdate`
- [x] Task 4: Unit tests for new data structures (AC: #1, #2, #3)
  - [x] Test: `tutorial_config_dock_radius_default_positive`
  - [x] Test: `tutorial_config_from_ron_includes_dock_radius`
  - [x] Test: `tutorial_phase_station_visited_variant_exists`
  - [x] Test: `spread_unlocked_component_is_marker`
- [x] Task 5: Integration tests (AC: #4, #5, #6, #9, #10)
  - [x] Test: `station_docking_advances_phase_to_station_visited_when_complete`
  - [x] Test: `station_docking_sets_station_not_defective`
  - [x] Test: `station_docking_adds_spread_unlocked_to_player`
  - [x] Test: `station_docking_is_idempotent_when_already_station_visited`
  - [x] Test: `station_no_dock_when_not_in_complete_phase`
  - [x] Test: `station_no_dock_when_too_far`
  - [x] Test: `hundred_seed_validation_still_passes_with_dock_radius`

## Dev Notes

### Architecture Patterns

- **Proximity detection:** Use Bevy Query to get both `TutorialStation` and `Player` transforms. Compute distance as `(station_pos - player_pos).length()`. If `distance <= dock_radius`, trigger docking. No collision component needed — dock_radius is a logical threshold.
- **Idempotency:** Guard with `if *phase.get() != TutorialPhase::Complete { return; }` — once `StationVisited`, the system becomes a no-op immediately.
- **Phase transition:** Uses `ResMut<NextState<TutorialPhase>>` and `.set(TutorialPhase::StationVisited)`. Becomes visible in the next `app.update()` call in tests.
- **SpreadUnlocked component:** Marker component on the Player entity. The rendering / future weapon UI systems can read this to display "Spread Unlocked" feedback.
- **Defective flag mutation:** Query the `TutorialStation` with `&mut TutorialStation` to set `defective = false`. No archetype change — just field mutation.
- **EventSeverityConfig:** Station docking is a major milestone — use `Tier1` severity. Add `StationDocked` mapping to `EventSeverityConfig`.

### Existing Code to Reuse (DO NOT Reinvent)

- `src/core/tutorial.rs` — `TutorialConfig`, `TutorialPhase`, `TutorialStation`, `WeaponsLocked` — same pattern for marker components
- `src/core/flight.rs` — `Player` marker component — use as filter
- `src/shared/events.rs` — `GameEventKind`, `EventSeverity` — extend enum
- `src/core/mod.rs` — `CoreSet::Events` — register system there
- `tests/tutorial_zone.rs` — `tutorial_test_app()` and `wreck_phase_test_app()` — follow same app builder pattern

### Implementation Guidance

```rust
/// Marker component added to the player when the station grants the Spread weapon.
#[derive(Component, Debug)]
pub struct SpreadUnlocked;

/// System: detect player proximity to TutorialStation in Complete phase → advance to StationVisited.
pub fn dock_at_station(
    config: Res<TutorialConfig>,
    phase: Res<State<TutorialPhase>>,
    mut next_phase: ResMut<NextState<TutorialPhase>>,
    mut station_query: Query<(&mut TutorialStation, &Transform)>,
    player_query: Query<(Entity, &Transform), With<crate::core::flight::Player>>,
    mut commands: Commands,
    mut game_events: bevy::ecs::message::MessageWriter<crate::shared::events::GameEvent>,
    time: Res<Time>,
    severity_config: Res<crate::infrastructure::events::EventSeverityConfig>,
) {
    if *phase.get() != TutorialPhase::Complete {
        return;
    }
    let Ok((player_entity, player_transform)) = player_query.single() else { return };
    let player_pos = player_transform.translation.truncate();

    for (mut station, station_transform) in station_query.iter_mut() {
        let station_pos = station_transform.translation.truncate();
        let distance = (station_pos - player_pos).length();
        if distance <= config.dock_radius {
            // Grant spread weapon
            commands.entity(player_entity).insert(SpreadUnlocked);
            // Station becomes functional
            station.defective = false;
            // Advance phase
            next_phase.set(TutorialPhase::StationVisited);
            // Emit game event
            let kind = crate::shared::events::GameEventKind::StationDocked;
            game_events.write(crate::shared::events::GameEvent {
                severity: severity_config.severity_for(&kind),
                kind,
                position: station_pos,
                game_time: time.elapsed_secs_f64(),
            });
        }
    }
}
```

**TutorialConfig extension:**

```rust
pub dock_radius: f32,  // default: 150.0
```

**TutorialPhase extension:**

```rust
pub enum TutorialPhase {
    Flying,
    Shooting,
    SpreadUnlocked,
    Complete,
    StationVisited,  // ← new: tutorial fully complete, station functional
}
```

**GameEventKind extension (src/shared/events.rs):**

```rust
pub enum GameEventKind {
    // ... existing ...
    StationDocked,
}
```

**EventSeverityConfig:** Add `StationDocked → Tier1` to the default severity mapping.

### File Structure

| File | Action | Purpose |
|------|--------|---------|
| `src/core/tutorial.rs` | MODIFY | Add dock_radius to TutorialConfig, StationVisited to TutorialPhase, SpreadUnlocked marker, dock_at_station system, unit tests |
| `src/shared/events.rs` | MODIFY | Add StationDocked variant to GameEventKind |
| `src/core/mod.rs` | MODIFY | Import dock_at_station and SpreadUnlocked, register in CoreSet::Events |
| `assets/config/tutorial.ron` | MODIFY | Add dock_radius field |
| `tests/tutorial_zone.rs` | MODIFY | Add import for SpreadUnlocked, dock_at_station; add integration tests |

### Testing Requirements

- **Unit tests** in `src/core/tutorial.rs`:
  - `TutorialConfig` default includes `dock_radius > 0`
  - `TutorialConfig::from_ron()` round-trips with `dock_radius`
  - `TutorialPhase::StationVisited` variant exists and is distinct from `Complete`
- **Integration tests** in `tests/tutorial_zone.rs`:
  - Phase advances to `StationVisited` when player is within `dock_radius` and phase is `Complete`
  - Station becomes `defective: false` on docking
  - Player receives `SpreadUnlocked` component on docking
  - Docking is idempotent (no double-transition from `StationVisited`)
  - No dock when phase is not `Complete`
  - No dock when player is too far from station
- **Pattern:** `#[deny(clippy::unwrap_used)]` — use `.expect()` in tests
- **Time:** `TimeUpdateStrategy::ManualDuration(Duration::from_secs_f32(1.0/60.0))`
- **State transitions:** Require additional `app.update()` call to be visible

### Project Structure Notes

- `dock_at_station` runs in `CoreSet::Events` — after all damage/despawn logic has settled
- `SpreadUnlocked` is a marker component, not related to `ActiveWeapon::Spread` enum variant — the enum variant already exists. This component signals that the tutorial station granted it.
- The `EventSeverityConfig::severity_for` must recognize `StationDocked` — add a match arm

### References

- [Source: _bmad-output/epics.md#Epic 2 — Story 5]
- [Source: src/core/tutorial.rs — TutorialConfig, TutorialPhase, TutorialStation, WeaponsLocked pattern]
- [Source: src/shared/events.rs — GameEventKind, EventSeverity]
- [Source: src/core/mod.rs — CoreSet::Events, system registration]
- [Source: tests/tutorial_zone.rs — tutorial_test_app(), wreck_phase_test_app() patterns]
- [Source: _bmad-output/implementation-artifacts/2-4-enemies-after-laser.md — story format reference]

### Key Bevy 0.18 Notes

- `Query::single()` returns `Result` — use `let Ok(...) = query.single() else { return };`
- Mutating a component field: `(&mut TutorialStation, &Transform)` in query; use `station.defective = false;`
- State guard: `if *phase.get() != TutorialPhase::Complete { return; }`
- `commands.entity(entity).insert(Component)` for adding marker component
- Phase transition visible in tests after one additional `app.update()`

## Dev Agent Record

### Agent Model Used

Claude Sonnet 4.6

### Debug Log References

- None — implemented cleanly on first pass.

### Completion Notes List

- Task 1: Added `dock_radius: f32` (default: 150.0) to `TutorialConfig`. Added `StationVisited` variant to `TutorialPhase`. Added `SpreadUnlocked` marker component. Added `StationDocked` variant to `GameEventKind` in `src/shared/events.rs`. Added `StationDocked → Tier1` to `EventSeverityConfig::severity_for`. Updated existing unit tests for `TutorialConfig::default` and `from_ron` to cover new field.
- Task 2: Implemented `dock_at_station` system. Reads `dock_radius` from config. Queries player and station transforms. Guards on `TutorialPhase::Complete`. On docking: inserts `SpreadUnlocked` on player, sets `station.defective = false`, advances phase to `StationVisited`, emits `GameEvent::StationDocked`.
- Task 3: Imported `dock_at_station` and `SpreadUnlocked` in `src/core/mod.rs`. Registered `dock_at_station` in `CoreSet::Events`.
- Task 4: Added 4 new unit tests in `src/core/tutorial.rs`.
- Task 5: Added 7 integration tests in `tests/tutorial_zone.rs`. All tests pass.

### File List

- `src/core/tutorial.rs` — MODIFIED: Added `dock_radius` to `TutorialConfig`, `StationVisited` to `TutorialPhase`, `SpreadUnlocked` marker component, `dock_at_station` system, new unit tests
- `src/shared/events.rs` — MODIFIED: Added `StationDocked` to `GameEventKind`
- `src/infrastructure/events.rs` — MODIFIED: Added `StationDocked → Tier1` severity mapping
- `src/core/mod.rs` — MODIFIED: Imported `dock_at_station` and `SpreadUnlocked`; registered system in `CoreSet::Events`
- `assets/config/tutorial.ron` — MODIFIED: Added `dock_radius: 150.0`
- `tests/tutorial_zone.rs` — MODIFIED: Added imports for `SpreadUnlocked`, `dock_at_station`; added 7 new integration tests
- `_bmad-output/implementation-artifacts/2-5-station-spread-weapon.md` — CREATED

## Change Log

- 2026-02-28: Implemented Story 2.5 Station Spread Weapon — dock_radius config, StationVisited phase, SpreadUnlocked marker, dock_at_station proximity system, StationDocked event. 11 new tests.
