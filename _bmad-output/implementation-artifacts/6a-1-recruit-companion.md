# Story 6a-1: Recruit Companion

**Epic:** 6a — Companion Core
**Status:** done

## User Story
As a player, I can recruit a companion at a station so that I gain a wingman.

## Acceptance Criteria
- [ ] A `Companion` marker component exists
- [ ] A `CompanionData` component stores name and faction
- [ ] A `CompanionRoster` resource tracks all recruited companions
- [ ] Player can recruit via `recruit` action while docked at a station
- [ ] `GameEventKind::CompanionRecruited { name }` is emitted on recruit
- [ ] Integration test: recruiting at a station adds companion to roster

## Technical Notes
- Lives in `src/social/companion.rs`
- Core/Rendering separation: Core spawns companion with `NeedsCompanionVisual` marker
- Input: `ActionState.recruit` (new field)
- Recruit adds entity with `Companion + CompanionData + NeedsCompanionVisual`
