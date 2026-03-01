# Story 5.5: Visual Ship Changes

Status: done

## Story

As a player,
I want my ship to visually change as I upgrade,
so that my progression is visible.

## Completion Notes

- `NeedsShipUpgradeVisual` marker in `src/shared/components.rs`
- Core: `mark_player_needs_upgrade_visual` triggers on upgrade change
- Rendering: `update_ship_upgrade_visual` applies hull-tier color:
  - Tier 0: gold | Tier 1-2: blue | Tier 3-4: bright gold | Tier 5: silver
- Follows Core/Rendering separation
