# Story 6b-1: Companion Barks

## Goal
Companions react to game situations with contextual one-liner barks so they feel alive.

## Acceptance Criteria
- `CompanionPersonality` enum (Brave / Cautious / Sarcastic / Loyal) assigned at recruitment based on faction
- Companion barks on: `PlayerDeath`, `EnemyDestroyed`, WingmanCommand changed
- `BarkDisplay` resource holds current bark text and countdown timer
- HUD text node (bottom-center) shows active bark for 4 seconds, then clears
- `pick_bark()` pure function covered by unit tests (all 16 combinations)
- `personality_for_faction()` pure function maps each FactionId to a personality

## Implementation

### New file: `src/social/companion_personality.rs`
- `CompanionPersonality` enum (4 variants) — Component
- `BarkTrigger` enum (PlayerDied / EnemyDestroyed / CommandReceived / DamageTaken)
- `BarkDisplay` resource with `current: Option<(String, &'static str)>` and `timer: f32`
- `BARK_DISPLAY_DURATION: f32 = 4.0`
- `pick_bark(personality, trigger) -> &'static str` — pure
- `personality_for_faction(faction) -> CompanionPersonality` — pure
- `emit_barks_on_game_events` system — reads `GameEvent` messages
- `emit_bark_on_command_change` system — watches `Changed<WingmanCommand>`
- `tick_bark_display` system — ticks timer, clears expired barks

### Modify: `src/social/companion.rs`
- In `handle_recruit_companion`: insert `personality_for_faction(&faction)` on spawned entity

### Modify: `src/social/mod.rs`
- Register `BarkDisplay` resource and all 3 bark systems

### Modify: `src/rendering/mod.rs`
- `BarkHudMarker` component
- `spawn_bark_hud()` Startup — bottom-center text node
- `update_bark_hud()` Update — reads `BarkDisplay`, updates text visibility + content
