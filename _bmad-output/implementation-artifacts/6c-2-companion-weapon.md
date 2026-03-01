# Story 6c-2: Companion Weapon

## Goal
Companion fires spread projectiles at acquired targets in Attack mode so it actively contributes to combat.

## Acceptance Criteria
- `CompanionWeapon { damage, range, cooldown_secs, timer }` component added at recruitment
- `fire_companion_weapon` system: fires SpreadProjectile when Attack mode + target in range + cooldown ready
- Projectile spawned at companion position in direction of target
- Cooldown resets after firing
- Unit tests for fire condition logic

## Implementation
Added to `src/social/companion_personality.rs`.
Registered in `src/social/mod.rs`.
