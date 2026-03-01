# Story 6b-3: Companion-to-Companion Opinions

## Goal
Companions have opinions of each other so that crew dynamics emerge.

## Acceptance Criteria
- `PeerOpinions` resource: HashMap<(Entity, Entity), i32>, range −100 to 100
- Key = (observer, subject) — "observer's opinion of subject"
- EnemyDestroyed → all companion pairs gain +1 (fought together)
- PlayerDeath → all companion pairs lose −1 (chaos, retreat)
- `update_peer_opinions` system handles both triggers
- Unit test confirms resource starts empty

## Implementation
Added to `src/social/companion_personality.rs`.
Registered in `src/social/mod.rs` under Story 6b-3 comment.
