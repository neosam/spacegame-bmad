# Story 2.10: Flying → Shooting Trigger

Status: ready-for-dev

## Story

As a player,
I want to automatically receive the laser weapon when I approach the wreck,
so that I gain my first weapon without any UI interaction.

## Acceptance Criteria

1. When the player is within `wreck_dock_radius` of `TutorialWreck` AND phase is `Flying`, the phase advances to `Shooting`
2. `wreck_dock_radius` is a config field in `TutorialConfig` (default: 120.0)
3. The `WeaponsLocked` component is removed from the player when transitioning to `Shooting` phase (this already happens via `update_weapons_lock` — verify it works)
4. The trigger fires only once (idempotent — `TutorialPhase::Flying` guard prevents re-triggering)
5. The system runs in `FixedUpdate` in `CoreSet::Events`
6. If no `TutorialWreck` entity exists, the system does nothing (no crash)
7. After phase advances to `Shooting`, the player can fire the laser

## Tasks / Subtasks

- [ ] Task 1: Add config field
  - [ ] Add `wreck_dock_radius: f32` to `TutorialConfig` (default: 120.0)
  - [ ] Add to `assets/config/tutorial.ron`
  - [ ] Add constraint validation in `validate_tutorial_config`: `wreck_dock_radius > 0.0`

- [ ] Task 2: Implement `unlock_laser_at_wreck` system
  - [ ] Query `TutorialWreck` + `Transform` for wreck position
  - [ ] Query `Player` + `Transform` for player position
  - [ ] Check phase == `TutorialPhase::Flying`
  - [ ] If distance < `wreck_dock_radius`: set `NextState(TutorialPhase::Shooting)`
  - [ ] Register in `CoreSet::Events` in `FixedUpdate`

- [ ] Task 3: Unit tests in `src/core/tutorial.rs`
  - [ ] Test: no trigger when player outside wreck_dock_radius
  - [ ] Test: trigger when player inside wreck_dock_radius in Flying phase
  - [ ] Test: no trigger when phase != Flying (idempotent)
  - [ ] Test: no trigger when no wreck entity

- [ ] Task 4: Integration tests in `tests/tutorial_zone.rs`
  - [ ] Test: WeaponsLocked removed after approaching wreck
  - [ ] Test: phase advances Flying → Shooting on approach

## Dev Notes

### Architecture Pattern
- Same proximity pattern as `dock_at_station` in `src/core/tutorial.rs`
- System guard: `if *phase.get() != TutorialPhase::Flying { return; }`
- After NextState is set, `update_weapons_lock` (already registered) removes `WeaponsLocked` automatically

### Existing Code to Reuse
- `dock_at_station` — exact same proximity + phase guard pattern
- `update_weapons_lock` — already handles `Flying → non-Flying` by removing `WeaponsLocked`
- `TutorialConfig` — add `wreck_dock_radius` field

### File Structure

| File | Action | Purpose |
|------|--------|---------|
| `src/core/tutorial.rs` | MODIFY | Add `unlock_laser_at_wreck` system + config field |
| `src/core/mod.rs` | MODIFY | Import and register system |
| `assets/config/tutorial.ron` | MODIFY | Add `wreck_dock_radius` |
| `tests/tutorial_zone.rs` | MODIFY | Integration tests |
