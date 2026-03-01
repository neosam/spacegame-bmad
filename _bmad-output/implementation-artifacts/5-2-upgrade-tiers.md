# Story 5.2: Upgrade Tiers

Status: done

## Story

As a player,
I want to see 5 tiers of upgrades per ship system,
so that I have a clear progression path.

## Completion Notes

- Implemented as part of Epic 5 cohesive feature set in `src/core/upgrades.rs`
- `compute_upgrade_multiplier` pure function: tier 0=1.0, tier 5=1.5
- `default_tier1_recipes()` returns 13 recipes (8 ship + 5 weapon)
- `DiscoveredRecipes::default()` initializes with all tier-1 recipes
