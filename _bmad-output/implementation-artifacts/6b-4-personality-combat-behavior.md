# Story 6b-4: Personality Combat Behavior

## Goal
Companions behave differently in combat based on their personality so that each feels unique.

## Acceptance Criteria
- `update_personality_behavior` system modifies `CompanionFollowAI` each frame based on personality + WingmanCommand
- Brave: follow_distance 35/55/90 (attack/defend/retreat), follow_speed 175
- Cautious: follow_distance 65/85/120, follow_speed 155
- Sarcastic: pulsing speed variation (sinusoidal hesitation), follow_distance 60
- Loyal: exact default values (follow_distance 60, follow_speed 150)
- Runs before `update_companion_follow` so behavior is applied before movement

## Implementation
Added to `src/social/companion_personality.rs`.
Registered in `src/social/mod.rs` under Story 6b-4 comment with `.before(update_companion_follow)`.
