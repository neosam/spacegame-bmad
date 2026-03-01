# Story 5.4: Recipe Discovery

Status: done

## Story

As a player,
I want to discover new recipes through exploration and station purchase,
so that crafting rewards curiosity.

## Completion Notes

- `DiscoveredRecipes` starts with 13 tier-1 recipes by default
- `discover_recipe_for_chunk` system adds higher-tier recipes on chunk load
- Deduplication prevents duplicate recipe entries
- Station-based recipe discovery via exploration/purchase
