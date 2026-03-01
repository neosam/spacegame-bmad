# Story 6a-6: Companion Save

**Epic:** 6a — Companion Core
**Status:** done

## User Story
As a player, my companion roster is saved so that my crew persists between sessions.

## Acceptance Criteria
- [x] `CompanionSaveEntry { name, faction, x, y }` with Serialize/Deserialize
- [x] `PlayerSave.companions: Vec<CompanionSaveEntry>` with `#[serde(default)]`
- [x] `from_world` serializes all companion entities
- [x] `apply_to_world` re-spawns companions from save data
- [x] SAVE_VERSION bumped from 5 → 6
- [x] v5 accepted by check_version for migration

## Technical Notes
- `CompanionSaveEntry` lives in `src/social/companion.rs`
- `from_world` queries `(CompanionData, Transform)` with `With<Companion>`
- `apply_to_world` clears CompanionRoster then re-spawns each saved entry
- SAVE_VERSION 5 added to check_version allowed list
