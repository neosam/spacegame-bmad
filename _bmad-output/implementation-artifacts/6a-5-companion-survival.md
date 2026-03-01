# Story 6a-5: Companion Survival

**Epic:** 6a — Companion Core
**Status:** done

## User Story
As a player, my companions survive when I die and retreat to the station so that death doesn't destroy my crew.

## Acceptance Criteria
- [x] `CompanionRetreating { target: Vec2 }` component
- [x] `handle_companion_survival` reads PlayerDeath event → adds CompanionRetreating
- [x] `update_retreating_companions` moves companions to nearest station
- [x] CompanionRetreating removed when companion arrives (< 25 units from target)
- [x] Follow AI is excluded while retreating

## Technical Notes
- Uses MessageReader<GameEvent> to detect PlayerDeath
- Nearest station found by min distance from companion position
- Companion velocity blended with lerp while retreating
