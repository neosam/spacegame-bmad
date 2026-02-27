# Story 0.4: Switch Weapons

Status: done

<!-- Note: Validation is optional. Run validate-create-story for quality check before dev-story. -->

## Story

As a player,
I want to switch between weapons instantly,
so that combat has tactical variety.

## Acceptance Criteria

1. Pressing Tab (keyboard) or Left Bumper (gamepad) cycles the active weapon
2. Weapon switching is instant — no delay, no animation, no cooldown
3. The `ActiveWeapon` component on the player toggles between `Laser` and `Spread`
4. Switching only triggers on key press (rising edge), NOT while held — exactly one switch per press
5. After switching, the next fire input uses the new weapon type
6. Switching does NOT reset fire cooldown or energy — those persist across switches
7. The `switch_weapon` field in `ActionState` is populated by `read_input`
8. All existing laser and spread tests continue to pass (no regression)
9. No `unwrap()` in game code — `#[deny(clippy::unwrap_used)]` enforced

## Tasks / Subtasks

- [x] Task 1: Input Mapping (AC: #1, #4, #7)
  - [x] 1.1 In `src/core/input.rs` `read_input`, add Tab key mapping: `keyboard.just_pressed(KeyCode::Tab)` → `action_state.switch_weapon = true`
  - [x] 1.2 Add gamepad Left Bumper mapping: `gamepad.just_pressed(GamepadButton::LeftTrigger)` → `action_state.switch_weapon = true`
  - [x] 1.3 CRITICAL: Use `just_pressed` (NOT `pressed`) for rising-edge detection — ensures exactly one switch per key press

- [x] Task 2: Weapon Switch System (AC: #2, #3, #5, #6)
  - [x] 2.1 Create `switch_weapon` system in `src/core/weapons.rs`: reads `ActionState.switch_weapon`, toggles `ActiveWeapon` on player entity
  - [x] 2.2 Toggle logic: `Laser → Spread`, `Spread → Laser` (simple two-weapon cycle)
  - [x] 2.3 System must NOT touch `FireCooldown` or `Energy` — those persist across switches
  - [x] 2.4 Register `switch_weapon` system in `CorePlugin` in `FixedUpdate` `CoreSet::Input` set (alongside `tick_fire_cooldown` and `regenerate_energy`)

- [x] Task 3: Tests (AC: #1, #2, #3, #4, #5, #6, #7, #8)
  - [x] 3.1 Unit tests in `src/core/input.rs`:
    - Tab key sets `switch_weapon` to true
    - `switch_weapon` is false when Tab is not pressed
  - [x] 3.2 Unit test in `src/core/weapons.rs`:
    - `switch_weapon` toggles `ActiveWeapon` from Laser to Spread
    - `switch_weapon` toggles `ActiveWeapon` from Spread back to Laser
  - [x] 3.3 Integration tests in `tests/weapon_switching.rs`:
    - Switching to Spread then firing spawns SpreadProjectile (not LaserPulse)
    - Switching back to Laser then firing spawns LaserPulse (not SpreadProjectile)
    - Fire cooldown persists across weapon switch
    - Energy persists across weapon switch
    - Multiple rapid switches settle on correct weapon
  - [x] 3.4 Verify all existing tests still pass (no regression)

## Dev Notes

### Architecture Patterns and Constraints

- **Instant switching** — no animation delay, no state machine, just toggle the `ActiveWeapon` enum component. [Source: gdd.md#Weapon Switching]
- **Rising edge only** — use `just_pressed` in `read_input` so holding Tab doesn't rapid-fire toggle every frame. The `ActionState.switch_weapon` bool is set for exactly one frame per press. [Source: gdd.md#Controls]
- **Two weapons only** — for Epic 0, only Laser and Spread exist. Simple toggle. Future stories may add more weapons → will need cycle/list logic then. Keep the toggle simple now.
- **No HUD** — active weapon indicator UI is NOT in scope for the arcade prototype. [Source: Story 0.3 Dev Notes]
- **No `unwrap()`** — enforced via `#[deny(clippy::unwrap_used)]`.

### Existing Infrastructure (from Stories 0.2 + 0.3)

**Already implemented — DO NOT recreate:**
- `ActiveWeapon` enum: `Laser` (default), `Spread` — in `src/core/weapons.rs:87-92`
- `ActiveWeapon` component on player — spawned in `src/rendering/mod.rs:112`
- `ActionState.switch_weapon: bool` field — in `src/core/input.rs:12` (exists but NOT populated by `read_input`)
- `fire_weapon` system — already branches on `ActiveWeapon` in `src/core/weapons.rs:170-274`
- `Energy` component — in `src/core/weapons.rs:71-84`
- `FireCooldown` component — in `src/core/weapons.rs:126-130`
- Test helper `set_active_weapon_spread` — in `tests/helpers/mod.rs:86-91`

**What's missing (implement in this story):**
1. `read_input` does NOT map any key/button to `switch_weapon` — add Tab + Left Bumper
2. No system reads `ActionState.switch_weapon` to toggle `ActiveWeapon` — create it
3. No tests for weapon switching behavior

### System Ordering

```
PreUpdate: read_input (sets switch_weapon = true on Tab press)
FixedUpdate:
  CoreSet::Input    → tick_fire_cooldown, regenerate_energy, switch_weapon (NEW)
  CoreSet::Physics  → apply_thrust, apply_rotation, apply_drag, apply_velocity
  (after Physics)   → fire_weapon, tick_laser_pulses, move_spread_projectiles, tick_spread_projectiles
```

The `switch_weapon` system runs in `CoreSet::Input` so the weapon is toggled BEFORE `fire_weapon` runs in the same frame.

### Switch Weapon System Implementation

```rust
/// Toggles the player's active weapon when switch input is active.
pub fn switch_weapon(
    action_state: Res<ActionState>,
    mut query: Query<&mut ActiveWeapon, With<Player>>,
) {
    if !action_state.switch_weapon {
        return;
    }
    for mut weapon in query.iter_mut() {
        *weapon = match *weapon {
            ActiveWeapon::Laser => ActiveWeapon::Spread,
            ActiveWeapon::Spread => ActiveWeapon::Laser,
        };
    }
}
```

### Input Mapping Addition

In `read_input`, add AFTER the fire mapping:

```rust
// Keyboard: switch weapon (rising edge only)
if keyboard.just_pressed(KeyCode::Tab) {
    action_state.switch_weapon = true;
}
```

And in the gamepad loop:

```rust
// Switch weapon: Left Bumper
if gamepad.just_pressed(GamepadButton::LeftTrigger) {
    action_state.switch_weapon = true;
}
```

**CRITICAL:** Use `just_pressed` NOT `pressed` — `pressed` would toggle every frame while held, causing rapid flickering between weapons.

### Bevy 0.18 Gotchas

- `ButtonInput::just_pressed()` returns true for exactly one frame on key down — this is the rising-edge detector we need
- `GamepadButton::LeftTrigger` is the Left Bumper (LB) in Bevy 0.18 naming convention
- Gamepad tests are NOT feasible with `MinimalPlugins` — test keyboard mapping only, document gamepad as manual QA (same pattern as Stories 0.2/0.3)
- Messages: `#[derive(Message)]`, `MessageWriter`, `.write()`, `app.add_message::<T>()` — NOT events
- First `app.update()` has dt=0 — prime in tests
- `TimeUpdateStrategy::ManualDuration` for deterministic tests

### What This Story Does NOT Include

- **No weapon HUD/indicator** — UI is outside arcade prototype scope
- **No weapon switch sound** — audio deferred
- **No weapon switch animation** — instant toggle per GDD
- **No more than 2 weapons** — only Laser + Spread in Epic 0
- **No key rebinding** — post-MVP feature

### Previous Story Intelligence (Story 0.3)

**Patterns established:**
- `ActiveWeapon` enum with `#[derive(Component, Default, Debug, Clone, Copy, PartialEq, Eq)]`
- `fire_weapon` system branches on `match active_weapon { Laser => ..., Spread => ... }`
- `Energy` initialized from `WeaponConfig.energy_max` in `setup_player`
- Test helper `set_active_weapon_spread(app, entity)` available for integration tests
- `test_app()` includes all weapon systems + flight systems

**Code review fixes from 0.3:**
- `Energy` now initialized from `WeaponConfig.energy_max` (not hardcoded)
- Spread arc direction test added for correctness
- Fire rate test verifies both block AND recovery
- Entity-ID-based position comparison for stable test ordering

### Project Structure Notes

| File | Action | Purpose |
|------|--------|---------|
| `src/core/input.rs` | MODIFY | Add Tab + Left Bumper mapping to `switch_weapon` in `read_input` |
| `src/core/weapons.rs` | MODIFY | Add `switch_weapon` system |
| `src/core/mod.rs` | MODIFY | Import + register `switch_weapon` in CoreSet::Input |
| `tests/weapon_switching.rs` | CREATE | Integration tests for weapon switching |
| `tests/helpers/mod.rs` | MODIFY | Add `switch_weapon` system to test_app if needed |

### References

- [Source: gdd.md#Controls] — Switch weapon: Cycle through available weapons
- [Source: gdd.md#Controller Layout] — Bumpers: Weapon switch
- [Source: gdd.md#Weapon Switching] — Switching is INSTANT — no animation delay
- [Source: game-architecture.md#GameAction] — `SwitchWeapon` in GameAction enum
- [Source: game-architecture.md#System Ordering] — Input in PreUpdate
- [Source: epics.md#Epic 0, Story 4] — "switch between weapons instantly"
- [Source: 0-3-fire-spread-projectiles.md#Dev Notes] — ActiveWeapon enum, fire_weapon branching

## Dev Agent Record

### Agent Model Used

Claude Opus 4.6

### Debug Log References

- Integration tests initially failed because `switch_weapon` flag persisted across frames in test harness (no `read_input` to auto-reset). Fixed by explicitly resetting `switch_weapon = false` before fire step in tests.

### Completion Notes List

- Implemented Tab key + Gamepad Left Bumper mapping using `just_pressed` for rising-edge detection
- Created `switch_weapon` system that toggles `ActiveWeapon` between Laser and Spread
- Registered system in `CoreSet::Input` (runs before `fire_weapon`)
- Added `switch_weapon` to test helper `test_app()` for integration tests
- 2 unit tests for input mapping (Tab sets switch_weapon, no-Tab leaves false)
- 3 unit tests for switch_weapon system (Laser→Spread, Spread→Laser, no-input keeps current)
- 5 integration tests (fire after switch to Spread, fire after switch back to Laser, cooldown persists, energy persists, rapid switches)
- Full regression suite: 59 tests passing, 0 failures
- Clippy clean with no warnings

### Code Review (AI)

**Reviewer:** AI Code Reviewer  
**Date:** 2026-02-26  
**Status:** APPROVED

**Findings:**
- ✅ All Acceptance Criteria met
- ✅ All tasks completed and verified
- ✅ All tests passing (26 unit + 5 integration)
- ✅ No security issues
- ✅ Code quality excellent

**Fixes Applied:**
- Removed unused import `use bevy::prelude::*;` from `tests/weapon_switching.rs`
- Added clarifying comment for Bevy 0.18 gamepad button mapping in `src/core/input.rs`

**Outcome:** Story approved and marked as done.

### Change Log

- 2026-02-26: Implemented weapon switching — Tab/LB toggles between Laser and Spread. 10 new tests added.
- 2026-02-26: Code review fixes — Removed unused import in test file, added clarifying comment for gamepad button mapping.

### File List

- `src/core/input.rs` — MODIFIED: Added Tab key + Gamepad Left Bumper mapping to `switch_weapon` in `read_input`, 2 new unit tests, added clarifying comment for Bevy 0.18 gamepad button mapping
- `src/core/weapons.rs` — MODIFIED: Added `switch_weapon` system, 3 new unit tests
- `src/core/mod.rs` — MODIFIED: Import + register `switch_weapon` in CoreSet::Input
- `tests/weapon_switching.rs` — CREATED: 5 integration tests for weapon switching, removed unused import
- `tests/helpers/mod.rs` — MODIFIED: Import + register `switch_weapon` in test_app()
