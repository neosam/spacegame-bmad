# Story 6c-1: Companion Ship Flight

## Goal
The companion rotates toward its target and accelerates in its facing direction like a real ship,
so it doesn't slide around like a floating object.

## Acceptance Criteria
- `CompanionFlight { angle, turn_rate, thrust_force, max_speed, drag }` component added at recruitment
- `rotate_toward_angle(current, desired, max_turn)` pure function
- `companion_desired_angle(companion_pos, target_pos)` pure function
- `update_companion_rotation` system: rotates facing toward target, updates Transform.rotation
- `update_companion_thrust_ai` system: thrusts in facing direction when roughly aligned
- `apply_companion_drag` system: drag applied each frame
- Old `update_companion_follow` (direct velocity) replaced — no more teleport-sliding
- All pure functions covered by unit tests

## Implementation
Modifies `src/social/companion.rs`: new CompanionFlight component, new pure functions, new systems.
Modifies `src/social/mod.rs`: registers new systems, removes old update_companion_follow chain.
Modifies `handle_recruit_companion` to add CompanionFlight.
