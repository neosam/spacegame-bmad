# Story 5.3: Weapon Upgrades

Status: done

## Story

As a player,
I want to upgrade my weapons independently,
so that I can specialize my combat style.

## Completion Notes

- `WeaponSystem` enum: LaserDamage, LaserFireRate, SpreadDamage, SpreadFireRate, EnergyEfficiency
- `apply_upgrade_effects` applies multipliers to WeaponConfig
- EnergyEfficiency inversely scales spread_energy_cost
- Base values in `BaseStats` resource for correct recompute
