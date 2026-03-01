# Story 6b-2: Player Opinion

## Goal
Companions have opinions of the player that change based on player actions so that choices matter.

## Acceptance Criteria
- `PlayerOpinions` resource: HashMap<Entity, i32>, range −100 to 100, starts at 0
- `opinion_delta_for_event()` pure function: EnemyDestroyed → +2, PlayerDeath → −5, StationDocked → +1
- `clamp_opinion()` pure function caps at ±100
- `update_player_opinions` system reads GameEvents each frame, updates all companions' scores
- Unit tests for all pure functions

## Implementation
Added to `src/social/companion_personality.rs` (same module as barks, part of personality system).
Systems registered in `src/social/mod.rs` under Story 6b-2 comment.
