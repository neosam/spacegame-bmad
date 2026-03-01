# Story 6c-5: Opinion HUD

## Goal
The player sees the companion's current opinion score alongside the bark so opinions feel tangible.

## Acceptance Criteria
- Bark HUD text includes opinion score: `Wing-1 (+12): "Target down!"`
- `update_bark_hud` reads `PlayerOpinions` resource
- Positive score shown as `(+N)`, negative as `(-N)`, zero as `(=0)`
- Unit test: format_opinion_score pure function

## Implementation
Modified `src/rendering/mod.rs` — `update_bark_hud` reads `Option<Res<PlayerOpinions>>`.
New pure function `format_opinion_score(score: i32) -> String`.
