# Story 6c-3: Target Acquisition

## Goal
Companion tracks the nearest enemy in Attack mode so combat behavior is goal-directed.

## Acceptance Criteria
- `CompanionTarget { entity: Option<Entity>, aggro_range: f32 }` component added at recruitment
- `update_companion_target` system: finds nearest AiState entity within aggro_range
- In Attack mode: rotates toward target instead of player
- In Defend/Retreat mode: target cleared, follows player
- Stale targets (entity despawned) cleared automatically
- Pure function `nearest_enemy` covered by unit test

## Implementation
Added to `src/social/companion_personality.rs`.
Registered in `src/social/mod.rs`.
