# Story 6a-3: Wingman Commands

**Epic:** 6a — Companion Core
**Status:** done

## User Story
As a player, I can issue Attack/Defend/Retreat commands so that I have tactical control over my companion.

## Acceptance Criteria
- [x] `WingmanCommand` enum: Attack, Defend, Retreat
- [x] `WingmanCommand::next()` cycles Attack → Defend → Retreat → Attack
- [x] `handle_wingman_commands` system cycles on `action_state.wingman_command`
- [x] G key sets `action_state.wingman_command = true`
- [x] All companions cycle simultaneously

## Technical Notes
- `wingman_command: bool` field already existed in ActionState
- G key binding added to `read_input` in `src/core/input.rs`
