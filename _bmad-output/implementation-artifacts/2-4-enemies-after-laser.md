# Story 2.4: Enemies After Laser

Status: done

## Story

As a player,
enemies appear after I have the laser so that I learn combat with a weapon available.

For this story's scope: When the tutorial phase transitions to `SpreadUnlocked` (i.e. after the player shoots the wreck), a wave of `TutorialEnemy` Scout Drones spawns near the wreck position. The tutorial tracks how many are alive; when all are destroyed, the tutorial phase advances to `Complete`.

## Acceptance Criteria

1. A `TutorialEnemy` marker component exists to distinguish tutorial enemies from normal spawn enemies
2. When `TutorialPhase` enters `SpreadUnlocked`, a configurable number of `TutorialEnemy` Scout Drones spawn near the wreck position
3. The number of enemies to spawn is defined in `TutorialConfig` as `tutorial_enemy_count: usize` (default: 3)
4. Each spawned `TutorialEnemy` entity has `ScoutDrone`, `Collider`, `Health`, `Velocity`, and `Transform` components
5. A `TutorialEnemyWave` resource tracks how many tutorial enemies remain alive
6. When all `TutorialEnemy` entities are destroyed (health <= 0 and despawned), the `TutorialPhase` advances from `SpreadUnlocked` to `Complete`
7. The phase advance only occurs once (idempotent — reaching `Complete` prevents further transitions)
8. The enemy spawn system runs on the `OnEnter(TutorialPhase::SpreadUnlocked)` schedule
9. The enemy wave completion check system runs in `FixedUpdate` in `CoreSet::Damage` (after despawn_destroyed)
10. `TutorialEnemy` entities do NOT trigger the normal `spawn_respawn_timers` system (they should not respawn)
11. Enemies spawn at randomized offsets around the wreck position (within a configurable radius `tutorial_enemy_spawn_radius`)

## Tasks / Subtasks

- [x] Task 1: Data structures — TutorialEnemy, TutorialEnemyWave, TutorialConfig extensions (AC: #1, #3, #5, #11)
  - [x] Add `TutorialEnemy` marker component in `src/core/tutorial.rs`
  - [x] Add `TutorialEnemyWave { remaining: usize }` resource in `src/core/tutorial.rs`
  - [x] Add `tutorial_enemy_count: usize` to `TutorialConfig` (default: 3)
  - [x] Add `tutorial_enemy_spawn_radius: f32` to `TutorialConfig` (default: 150.0)
  - [x] Update `Default` for `TutorialConfig` with new fields
  - [x] Update `TutorialConfig::from_ron()` (no change needed — serde handles it)
- [x] Task 2: Enemy spawn system (AC: #2, #4, #8, #11)
  - [x] Add `spawn_tutorial_enemies` system in `src/core/tutorial.rs`
  - [x] System runs `OnEnter(TutorialPhase::SpreadUnlocked)`
  - [x] Reads `TutorialZone` resource for wreck position
  - [x] Reads `TutorialConfig` for enemy count and spawn radius
  - [x] Reads `SpawningConfig` for drone health and radius values
  - [x] Spawns `tutorial_enemy_count` entities with: `TutorialEnemy`, `ScoutDrone`, `Collider`, `Health`, `Velocity::default()`, `Transform` at randomized offset from wreck
  - [x] Inserts `TutorialEnemyWave { remaining: count }` resource
- [x] Task 3: Wave completion detection system (AC: #5, #6, #7, #9)
  - [x] Add `check_tutorial_wave_complete` system in `src/core/tutorial.rs`
  - [x] Query count of alive `TutorialEnemy` entities (health > 0)
  - [x] When alive count == 0 and phase is `SpreadUnlocked`, transition to `Complete`
  - [x] System is idempotent — if phase is already `Complete`, returns immediately
  - [x] Register system in `CoreSet::Damage` after `despawn_destroyed`
- [x] Task 4: Exclude TutorialEnemy from respawn timers (AC: #10)
  - [x] Modify `spawn_respawn_timers` in `src/core/spawning.rs` to exclude entities with `TutorialEnemy` component
  - [x] Use `Without<crate::core::tutorial::TutorialEnemy>` filter on the existing query
- [x] Task 5: System registration in CorePlugin (AC: #8, #9)
  - [x] Import `spawn_tutorial_enemies`, `check_tutorial_wave_complete` in `src/core/mod.rs`
  - [x] Register `spawn_tutorial_enemies` in `OnEnter(TutorialPhase::SpreadUnlocked)`
  - [x] Register `check_tutorial_wave_complete` in `CoreSet::Damage` after `despawn_destroyed`
- [x] Task 6: Unit tests for new components and config (AC: #1, #3, #5, #11)
  - [x] Test: `tutorial_config_default_has_enemy_count_and_radius`
  - [x] Test: `tutorial_config_from_ron` updated to round-trip new fields
  - [x] Test: `tutorial_enemy_wave_default_has_zero_remaining`
  - [x] Test: `tutorial_enemy_wave_can_be_set`
- [x] Task 7: Integration tests (AC: #2, #4, #6, #7, #9, #10)
  - [x] Test: `tutorial_enemies_spawn_on_spread_unlocked_phase`
  - [x] Test: `tutorial_enemies_have_correct_components`
  - [x] Test: `tutorial_enemy_wave_resource_tracks_count`
  - [x] Test: `phase_advances_to_complete_when_all_tutorial_enemies_destroyed`
  - [x] Test: `phase_stays_spread_unlocked_while_enemies_alive`
  - [x] Test: `tutorial_enemies_do_not_trigger_respawn_timers`
  - [x] Test: `normal_scout_drone_still_triggers_respawn_timer` (regression)

## Dev Notes

### Architecture Patterns

- **OnEnter schedule:** Bevy's `OnEnter(TutorialPhase::SpreadUnlocked)` schedule runs exactly once when the state transitions. Register `spawn_tutorial_enemies` there — it will fire automatically when the wreck-shot system triggers the phase change.
- **TutorialEnemyWave resource:** Inserted by `spawn_tutorial_enemies` when enemies spawn. The `check_tutorial_wave_complete` system reads it to decide whether to advance the phase. Use `Option<Res<TutorialEnemyWave>>` to handle the case where the resource doesn't exist yet (before SpreadUnlocked phase).
- **Wave completion via entity count:** The simplest approach is to query for `TutorialEnemy` entities with health > 0, count them, and compare to the resource. When count drops to 0, the wave is done.
- **Excluding from respawn:** Add `Without<TutorialEnemy>` to the `spawn_respawn_timers` query filter — this ensures tutorial drones don't respawn like normal map drones.
- **Wreck position:** Retrieved from `TutorialZone` resource (`zone.layout.wreck_position`). Available because `spawn_tutorial_zone` inserts it as a Startup system.
- **Random offsets without seeded RNG:** Tutorial enemy positions within the wave can use `rand::random::<f32>()` (not seeded) since exact positions don't need to be deterministic across sessions — only the layout positions need to be seed-deterministic.

### Existing Code to Reuse (DO NOT Reinvent)

- `src/core/tutorial.rs` — `TutorialConfig`, `TutorialZone`, `TutorialPhase`, `spawn_tutorial_zone` — extend config and use zone resource
- `src/core/spawning.rs` — `ScoutDrone`, `SpawningConfig` (reuse `drone_health` and `drone_radius`), `spawn_respawn_timers` (add filter)
- `src/core/collision.rs` — `Collider`, `Health`
- `src/shared/components.rs` — `Velocity`
- `src/core/mod.rs` — `CoreSet::Damage` chain, `OnEnter` schedule registration

### Implementation Guidance

```rust
/// Marker for tutorial wave enemies — excluded from normal respawn logic.
#[derive(Component, Debug)]
pub struct TutorialEnemy;

/// Tracks the number of remaining tutorial wave enemies.
/// Inserted when enemies spawn; used to detect wave completion.
#[derive(Resource, Debug, Default)]
pub struct TutorialEnemyWave {
    pub remaining: usize,
}

/// Spawns tutorial enemies near the wreck when SpreadUnlocked phase begins.
/// Runs OnEnter(TutorialPhase::SpreadUnlocked).
pub fn spawn_tutorial_enemies(
    mut commands: Commands,
    tutorial_zone: Res<TutorialZone>,
    tutorial_config: Res<TutorialConfig>,
    spawning_config: Res<SpawningConfig>,
) {
    let wreck_pos = tutorial_zone.layout.wreck_position;
    let count = tutorial_config.tutorial_enemy_count;
    let radius = tutorial_config.tutorial_enemy_spawn_radius;

    for _ in 0..count {
        let angle = rand::random::<f32>() * std::f32::consts::TAU;
        let dist = rand::random::<f32>() * radius;
        let offset = Vec2::new(angle.cos() * dist, angle.sin() * dist);
        let pos = wreck_pos + offset;

        commands.spawn((
            TutorialEnemy,
            ScoutDrone,
            NeedsDroneVisual,
            Collider { radius: spawning_config.drone_radius },
            Health { current: spawning_config.drone_health, max: spawning_config.drone_health },
            Velocity::default(),
            Transform::from_translation(pos.extend(0.0)),
        ));
    }

    commands.insert_resource(TutorialEnemyWave { remaining: count });
}

/// Checks if all tutorial wave enemies are destroyed; advances phase to Complete.
/// Runs in CoreSet::Damage after despawn_destroyed.
pub fn check_tutorial_wave_complete(
    phase: Res<State<TutorialPhase>>,
    mut next_phase: ResMut<NextState<TutorialPhase>>,
    wave: Option<Res<TutorialEnemyWave>>,
    enemy_query: Query<&Health, With<TutorialEnemy>>,
    mut commands: Commands,
) {
    // Only check during SpreadUnlocked phase and when wave resource exists
    if *phase.get() != TutorialPhase::SpreadUnlocked {
        return;
    }
    let Some(_wave) = wave else { return };

    // Count tutorial enemies still alive (positive health — not yet despawned)
    let alive = enemy_query.iter().filter(|h| h.current > 0.0).count();

    if alive == 0 {
        next_phase.set(TutorialPhase::Complete);
    }
}
```

**Note on `spawn_respawn_timers` exclusion:**

```rust
// In src/core/spawning.rs — add Without<TutorialEnemy> to the query filter
pub fn spawn_respawn_timers(
    mut commands: Commands,
    config: Res<SpawningConfig>,
    query: Query<
        (&Health, &Transform, Option<&Asteroid>, Option<&ScoutDrone>),
        (Without<Player>, Without<TutorialEnemy>, Or<(With<Asteroid>, With<ScoutDrone>)>),
    >,
) {
    // ... same body ...
}
```

### File Structure

| File | Action | Purpose |
|------|--------|---------|
| `src/core/tutorial.rs` | MODIFY | Add TutorialEnemy, TutorialEnemyWave, extend TutorialConfig, spawn_tutorial_enemies, check_tutorial_wave_complete, unit tests |
| `src/core/spawning.rs` | MODIFY | Add Without<TutorialEnemy> to spawn_respawn_timers query |
| `src/core/mod.rs` | MODIFY | Import new types/systems, register OnEnter and Damage set systems |
| `assets/config/tutorial.ron` | MODIFY | Add tutorial_enemy_count and tutorial_enemy_spawn_radius fields |
| `tests/tutorial_zone.rs` | MODIFY | Add integration tests for enemy wave |

### Testing Requirements

- **Unit tests** in `src/core/tutorial.rs`:
  - `TutorialConfig` default includes `tutorial_enemy_count` and `tutorial_enemy_spawn_radius`
  - `TutorialConfig::from_ron()` round-trips with new fields
  - `TutorialEnemyWave` default has `remaining: 0`
- **Integration tests** in `tests/tutorial_zone.rs`:
  - Enemies spawn with correct count when phase enters `SpreadUnlocked`
  - Each enemy has `ScoutDrone`, `Collider`, `Health`, `Velocity`, `Transform`
  - `TutorialEnemyWave` resource exists after spawn with correct remaining count
  - Phase advances to `Complete` when all enemies despawned
  - Phase stays `SpreadUnlocked` while enemies alive
  - Tutorial enemies do not trigger respawn timers
- **Pattern:** Use `#[deny(clippy::unwrap_used)]` — use `.expect()` in tests
- **Time:** Use `TimeUpdateStrategy::ManualDuration(Duration::from_secs_f32(1.0/60.0))` for deterministic tests
- **State transitions:** Require additional `app.update()` calls to be visible in tests

### Project Structure Notes

- No new files needed — extends existing `src/core/tutorial.rs`, `src/core/spawning.rs`, and `tests/tutorial_zone.rs`
- `assets/config/tutorial.ron` must be updated to add the two new config fields
- System ordering: `check_tutorial_wave_complete` runs after `despawn_destroyed` in `CoreSet::Damage`

### References

- [Source: _bmad-output/epics.md#Epic 2 — Story 4]
- [Source: src/core/tutorial.rs — TutorialConfig, TutorialPhase, TutorialZone, spawn_tutorial_zone]
- [Source: src/core/spawning.rs — ScoutDrone, SpawningConfig, spawn_respawn_timers]
- [Source: src/core/collision.rs — Collider, Health, despawn_destroyed]
- [Source: src/core/mod.rs — CoreSet::Damage, system registration, OnEnter patterns]
- [Source: tests/tutorial_zone.rs — tutorial_test_app(), wreck_phase_test_app() patterns]

### Key Bevy 0.18 Notes

- `OnEnter(State)` schedule: `app.add_systems(OnEnter(TutorialPhase::SpreadUnlocked), spawn_tutorial_enemies)`
- `Option<Res<T>>` pattern for optional resources: safely handle case where `TutorialEnemyWave` not yet inserted
- `Without<T>` filter in queries: `Query<..., (Without<Player>, Without<TutorialEnemy>, ...)>`
- State transitions: `ResMut<NextState<TutorialPhase>>` + `.set(TutorialPhase::Complete)`
- State visibility in tests: requires additional `app.update()` after `NextState` is set

## Dev Agent Record

### Agent Model Used

Claude Sonnet 4.6

### Debug Log References

- Two tests initially failed (`phase_advances_to_complete_when_all_tutorial_enemies_destroyed`, `phase_stays_spread_unlocked_while_enemies_alive`) because `enemy_wave_test_app` registers `spawn_tutorial_enemies` on `OnEnter(SpreadUnlocked)`, which panics when `TutorialZone` resource is absent. Fixed by creating a separate `wave_completion_test_app` helper that only registers `check_tutorial_wave_complete` — used for tests that manually control wave state without spawning enemies.

### Completion Notes List

- Task 1: Added `TutorialEnemy` marker component. Added `TutorialEnemyWave { remaining: usize }` resource with `Default` (remaining=0). Extended `TutorialConfig` with `tutorial_enemy_count: usize` (default: 3) and `tutorial_enemy_spawn_radius: f32` (default: 150.0). Updated existing unit tests `tutorial_config_default_has_valid_values` and `tutorial_config_from_ron` to cover new fields.
- Task 2: Implemented `spawn_tutorial_enemies` — runs `OnEnter(TutorialPhase::SpreadUnlocked)`, reads `TutorialZone` for wreck position, spawns `tutorial_enemy_count` entities with `TutorialEnemy`, `ScoutDrone`, `NeedsDroneVisual`, `Collider`, `Health`, `Velocity`, `Transform` at random offsets within `tutorial_enemy_spawn_radius`. Inserts `TutorialEnemyWave { remaining: count }` resource.
- Task 3: Implemented `check_tutorial_wave_complete` — queries `TutorialEnemy` health, counts alive enemies (health > 0), transitions to `Complete` when count is 0. Uses `Option<Res<TutorialEnemyWave>>` to safely skip when wave resource not yet inserted. Idempotent by early return when phase is not `SpreadUnlocked`.
- Task 4: Added `Without<crate::core::tutorial::TutorialEnemy>` to `spawn_respawn_timers` query in `src/core/spawning.rs`. Tutorial drones are now excluded from respawn logic.
- Task 5: Updated imports in `src/core/mod.rs`. Registered `check_tutorial_wave_complete` at the end of the `CoreSet::Damage` chain (after `despawn_destroyed`). Registered `spawn_tutorial_enemies` on `OnEnter(TutorialPhase::SpreadUnlocked)`.
- Task 6: Added 3 unit tests in `src/core/tutorial.rs`: `tutorial_config_default_has_enemy_count_and_radius`, `tutorial_enemy_wave_default_has_zero_remaining`, `tutorial_enemy_wave_can_be_set`. Also updated existing tests to cover new config fields.
- Task 7: Added 7 integration tests in `tests/tutorial_zone.rs` across two helper apps (`enemy_wave_test_app` for spawn tests, `wave_completion_test_app` for completion tests). All 376 tests pass.

### File List

- `src/core/tutorial.rs` — MODIFIED: Added `TutorialEnemy` marker component; added `TutorialEnemyWave` resource; extended `TutorialConfig` with `tutorial_enemy_count` and `tutorial_enemy_spawn_radius`; added `spawn_tutorial_enemies` and `check_tutorial_wave_complete` systems; updated existing unit tests; added 3 new unit tests
- `src/core/spawning.rs` — MODIFIED: Added `Without<crate::core::tutorial::TutorialEnemy>` to `spawn_respawn_timers` query filter
- `src/core/mod.rs` — MODIFIED: Imported `spawn_tutorial_enemies` and `check_tutorial_wave_complete`; registered `check_tutorial_wave_complete` in `CoreSet::Damage` chain; registered `spawn_tutorial_enemies` on `OnEnter(TutorialPhase::SpreadUnlocked)`
- `assets/config/tutorial.ron` — MODIFIED: Added `tutorial_enemy_count: 3` and `tutorial_enemy_spawn_radius: 150.0`
- `tests/tutorial_zone.rs` — MODIFIED: Added `TutorialEnemy`, `TutorialEnemyWave`, `ScoutDrone`, `SpawningConfig`, `spawn_tutorial_enemies`, `check_tutorial_wave_complete` imports; added `enemy_wave_test_app` and `wave_completion_test_app` helpers; added 7 new integration tests
- `_bmad-output/implementation-artifacts/2-4-enemies-after-laser.md` — CREATED: Story file with all tasks marked complete and Dev Agent Record

## Change Log

- 2026-02-28: Implemented Story 2.4 Enemies After Laser — TutorialEnemy marker, TutorialEnemyWave resource, spawn_tutorial_enemies OnEnter system, check_tutorial_wave_complete Damage-chain system, TutorialEnemy exclusion from respawn timers, TutorialConfig extensions. 10 new tests. 376 total tests passing.
