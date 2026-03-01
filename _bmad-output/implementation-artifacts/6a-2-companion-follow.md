# Story 6a-2: Companion Follow

**Epic:** 6a — Companion Core
**Status:** done

## User Story
As a player, my companion follows my ship so that I'm not alone in the void.

## Acceptance Criteria
- [x] `CompanionFollowAI` component with `follow_speed` and `follow_distance`
- [x] `update_companion_follow` system moves companion toward player
- [x] Pure function `companion_follow_velocity` is testable without App
- [x] `update_companion_positions` integrates velocity → position for companions

## Technical Notes
- Lives in `src/social/companion.rs`
- Companions with `CompanionRetreating` are excluded from follow behavior
- Velocity is blended smoothly (lerp with dt*5.0) to avoid jitter
