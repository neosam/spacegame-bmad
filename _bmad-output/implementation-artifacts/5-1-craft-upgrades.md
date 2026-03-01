# Story 5.1: Craft Upgrades

Status: done

## Story

As a player,
I want to craft upgrades from materials at a station,
so that exploration resources turn into ship improvements.

## Acceptance Criteria

1. A `UpgradeSystem` enum exists in `src/core/upgrades.rs` with 8 variants: `Thrust`, `MaxSpeed`, `Rotation`, `EnergyCapacity`, `EnergyRegen`, `ScannerRange`, `HullStrength`, `CargoCapacity`
2. A `CraftingRecipe` struct exists with fields: `upgrade_system: UpgradeSystem`, `tier: u8` (1–5), `cost_common_scrap: u32`, `cost_rare_alloy: u32`, `cost_energy_core: u32`, `credit_cost: u32`
3. A `AvailableRecipes` resource exists: `Vec<CraftingRecipe>` — initially contains all tier-1 recipes for all 8 systems
4. A `InstalledUpgrades` resource exists: `HashMap<UpgradeSystem, u8>` (current tier per system, 0 = not upgraded)
5. A pure function `can_craft(recipe: &CraftingRecipe, inventory: &PlayerInventory, credits: &Credits) -> bool` returns true only if all material + credit costs are met
6. A system `craft_upgrade(recipe)` deducts materials and credits, increments `InstalledUpgrades` for that system, emits `GameEventKind::UpgradeCrafted { system, tier }`
7. Crafting is only possible when the player is `Docked` at a station
8. `InstalledUpgrades` is saved/loaded via `PlayerSave` (new fields with `#[serde(default)]`), `SAVE_VERSION` bumped to 5
9. All existing 605 tests remain green
10. New tests: `can_craft_with_sufficient_materials`, `can_craft_fails_insufficient_scrap`, `craft_deducts_materials`, `craft_increments_tier`, `upgrade_save_load_roundtrip`

## Tasks / Subtasks

- [ ] Task 1: Create `src/core/upgrades.rs` — UpgradeSystem, CraftingRecipe, AvailableRecipes, InstalledUpgrades, can_craft
- [ ] Task 2: Add `GameEventKind::UpgradeCrafted` to events
- [ ] Task 3: Save/load integration — bump SAVE_VERSION 4→5
- [ ] Task 4: Register resources/systems in CorePlugin
- [ ] Task 5: Integration tests `tests/craft_upgrades.rs`

## Dev Notes

### Architecture Rules

- All crafting logic in `src/core/upgrades.rs` — NO rendering code in core
- Crafting only works when `Docked` component is on player
- Use B0002-safe pattern: no MessageReader + MessageWriter in same system

### Key Recipe Costs (Tier 1)

| System | CommonScrap | RareAlloy | EnergyCore | Credits |
|--------|-------------|-----------|------------|---------|
| Thrust | 3 | 0 | 0 | 20 |
| MaxSpeed | 3 | 0 | 0 | 20 |
| Rotation | 2 | 1 | 0 | 25 |
| EnergyCapacity | 0 | 2 | 1 | 30 |
| EnergyRegen | 1 | 1 | 1 | 35 |
| ScannerRange | 2 | 0 | 0 | 15 |
| HullStrength | 4 | 1 | 0 | 40 |
| CargoCapacity | 2 | 0 | 0 | 15 |

### Save Schema v4 → v5

Add to `PlayerSave`:
```rust
#[serde(default)] pub upgrade_thrust: u8,
#[serde(default)] pub upgrade_max_speed: u8,
#[serde(default)] pub upgrade_rotation: u8,
#[serde(default)] pub upgrade_energy_capacity: u8,
#[serde(default)] pub upgrade_energy_regen: u8,
#[serde(default)] pub upgrade_scanner_range: u8,
#[serde(default)] pub upgrade_hull_strength: u8,
#[serde(default)] pub upgrade_cargo_capacity: u8,
```

## Dev Agent Record

### Agent Model Used
claude-sonnet-4-6
