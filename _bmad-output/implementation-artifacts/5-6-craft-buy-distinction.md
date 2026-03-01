# Story 5.6: Craft-Buy Distinction

Status: done

## Story

As a player,
I want to notice that some items are craft-only and some are buy-only,
so that I have reasons for both activities.

## Completion Notes

- `AcquisitionMethod` enum: CraftOnly, BuyOnly, CraftOrBuy
- Each CraftingRecipe has `acquisition: AcquisitionMethod`
- Most upgrades: CraftOnly | Scanner: CraftOrBuy | Cargo: BuyOnly
- `can_craft()` pure function works correctly for all acquisition methods
