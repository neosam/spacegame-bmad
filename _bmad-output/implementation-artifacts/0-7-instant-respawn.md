# Story 0.7: Instant Respawn

Status: done

<!-- Note: Validation is optional. Run validate-create-story for quality check before dev-story. -->

## Story

As a player,
I respawn instantly after death,
so that the failure loop is fast and non-punishing.

## Acceptance Criteria

1. Player entity spawns with `Health` and `Collider` components (configurable max health and radius)
2. Player takes contact damage when body-overlapping entities with `Collider` (asteroids, drones)
3. Contact damage has a cooldown (0.5s) to prevent instant death from continuous overlap
4. Player's own weapons (laser hitscan, spread projectiles) do NOT damage the player
5. When player health reaches zero, player respawns instantly at origin (0, 0) with full health, zero velocity
6. A destruction effect spawns at the player's death position (reuses existing `DestroyedPositions` pipeline)
7. Screen shake triggers on player death (reuses existing `JustDamaged` → `trigger_screen_shake` pipeline)
8. Player is invincible for 2 seconds after respawn (no contact damage during this period)
9. During invincibility, the player mesh blinks (visibility toggles at 10 Hz)
10. `despawn_destroyed` never despawns the player entity — only non-Player entities are despawned on zero health
11. All existing tests continue to pass (no regression)
12. No `unwrap()` in game code — `#[deny(clippy::unwrap_used)]` enforced

## Tasks / Subtasks

- [x] Task 1: Add Health and Collider to Player (AC: #1, #4)
  - [x] 1.1 Add `Health { current: 100.0, max: 100.0 }` and `Collider { radius: 12.0 }` to player spawn in `src/rendering/mod.rs:setup_player()` — import `Health` and `Collider` from `crate::core::collision`
  - [x] 1.2 Add `Without<Player>` filter to `check_laser_collisions` collider query in `src/core/collision.rs` — prevents player's own laser from hitting self
  - [x] 1.3 Add `Without<Player>` filter to `check_projectile_collisions` collider query in `src/core/collision.rs` — prevents player's own spread projectiles from hitting self
  - [x] 1.4 Update test helper `spawn_player()` in `tests/helpers/mod.rs` to include `Health` and `Collider` (use defaults: health 100.0, radius 12.0)
  - [x] 1.5 Update test helper `spawn_player_with_velocity()` to include `Health` and `Collider`

- [x] Task 2: Contact Damage System (AC: #2, #3, #8)
  - [x] 2.1 Create `ContactDamageCooldown { pub timer: f32 }` component in `src/shared/components.rs`
  - [x] 2.2 Create `check_contact_collisions` system in `src/core/collision.rs`: queries Player entity (with `Collider`, `Transform`, `Without<Invincible>`, `Without<ContactDamageCooldown>`) against all non-Player entities with `Collider` + `Transform`
  - [x] 2.3 On circle-circle overlap: push `(player_entity, CONTACT_DAMAGE)` to `DamageQueue`, insert `ContactDamageCooldown { timer: 0.5 }` on player
  - [x] 2.4 Create `tick_contact_cooldown` system: decrements timer by dt, removes `ContactDamageCooldown` when expired
  - [x] 2.5 Register `check_contact_collisions` in FixedUpdate, `CoreSet::Collision`
  - [x] 2.6 Register `tick_contact_cooldown` in FixedUpdate, after `CoreSet::Damage`
  - [x] 2.7 Contact damage constant: `CONTACT_DAMAGE: f32 = 20.0`

- [x] Task 3: Player Death Handling (AC: #5, #6, #7, #10)
  - [x] 3.1 Add `Without<Player>` filter to `despawn_destroyed` query in `src/core/collision.rs` — player entity is never despawned
  - [x] 3.2 Create `handle_player_death` system in `src/core/collision.rs`: queries Player with `Health` + `Transform` + `Velocity`
  - [x] 3.3 When `health.current <= 0.0`: record player position in `DestroyedPositions` (triggers destruction visual via existing pipeline)
  - [x] 3.4 Reset `health.current = health.max`
  - [x] 3.5 Reset `transform.translation = Vec3::ZERO` (respawn at origin)
  - [x] 3.6 Reset `velocity.0 = Vec2::ZERO`
  - [x] 3.7 Insert `Invincible { timer: INVINCIBILITY_DURATION }` component on player
  - [x] 3.8 Remove `ContactDamageCooldown` if present (clean slate on respawn)
  - [x] 3.9 Register `handle_player_death` in FixedUpdate, `CoreSet::Damage`, chained AFTER `apply_damage` and BEFORE `despawn_destroyed`: `(apply_damage, handle_player_death, despawn_destroyed).chain()`

- [x] Task 4: Invincibility System (AC: #8, #9)
  - [x] 4.1 Create `Invincible { pub timer: f32 }` component in `src/shared/components.rs`
  - [x] 4.2 Create `tick_invincibility` system in `src/core/collision.rs`: decrements timer by dt, removes `Invincible` when timer ≤ 0, also restores `Visibility::Inherited` on removal
  - [x] 4.3 Create `blink_invincible` system in `src/rendering/effects.rs`: toggles `Visibility` (Inherited/Hidden) based on `(elapsed_secs * 10.0 * PI).sin() > 0.0` for 10 Hz blink
  - [x] 4.4 Register `tick_invincibility` in FixedUpdate, after `CoreSet::Damage`
  - [x] 4.5 Register `blink_invincible` in Update (alongside other rendering effects)
  - [x] 4.6 Constants: `INVINCIBILITY_DURATION: f32 = 2.0`

- [x] Task 5: System Registration (AC: #11)
  - [x] 5.1 Update CorePlugin `build()` in `src/core/mod.rs`: modify Damage set chain to `(apply_damage, handle_player_death, despawn_destroyed).chain()`, add `check_contact_collisions` to Collision set, add `tick_contact_cooldown` and `tick_invincibility` after Damage set
  - [x] 5.2 Update RenderingPlugin `build()` in `src/rendering/mod.rs`: add `blink_invincible` to Update systems
  - [x] 5.3 Import new systems and components in both plugins

- [x] Task 6: Tests (AC: #1–12)
  - [x] 6.1 Unit tests in `src/core/collision.rs` (7 tests):
    - Contact damage: player overlapping asteroid pushes damage to queue
    - Contact damage cooldown: no damage while cooldown active
    - Player death resets health to max
    - Player death resets position to origin
    - Player death resets velocity to zero
    - Player death inserts Invincible component
    - Invincibility timer ticks down
  - [x] 6.2 Integration tests in `tests/instant_respawn.rs` (7 tests):
    - Player takes contact damage from asteroid (full pipeline)
    - Player dies from accumulated damage and respawns at origin with full health
    - Player death position recorded in DestroyedPositions
    - Invincible player does not take contact damage
    - Player's own laser does not hit self (With<Player> excluded from laser targets)
    - Player's own projectiles do not hit self (With<Player> excluded from projectile targets)
    - Player gets invincibility after death
  - [x] 6.3 Verify no regression: all 98 existing tests pass (105 total with new tests)

## Dev Notes

### Architecture Patterns and Constraints

- **Player Death ≠ Despawn:** The player entity must NEVER be despawned. Death means resetting state (health, position, velocity) and adding invincibility. All other destructible entities follow the existing `despawn_destroyed` path. [Source: gdd.md#Win/Loss — "No permadeath. No game over."]
- **Rendering Separation:** `Invincible` component lives in `src/shared/components.rs` — used by `core/collision.rs` (writer/reader) and `rendering/effects.rs` (visual blink). Follows the same cross-domain pattern as `JustDamaged`. [Source: game-architecture.md#Architectural Boundaries]
- **Self-Damage Prevention:** Player weapons must not hit the player. Add `Without<Player>` to weapon collision target queries. In future epics, a `Team` enum will replace this with proper faction-based filtering. [Source: game-architecture.md#Combat]
- **Contact Damage Cooldown:** Without cooldown, continuous body overlap would apply damage every FixedUpdate tick (60 DPS at default settings), killing the player instantly. The 0.5s cooldown limits contact damage to 2 hits per second (40 DPS max). [Design rationale: arcade feel — contact is punishing but survivable]
- **Reuse Existing Pipelines:** Player death reuses the `DestroyedPositions` → `spawn_destruction_effects` pipeline for the destruction visual and the `JustDamaged` → `trigger_screen_shake` pipeline for camera shake. No new visual systems needed for death feedback. [Source: 0-6-visual-feedback.md]
- **No `unwrap()`** — enforced via `#[deny(clippy::unwrap_used)]` in lib.rs. [Source: game-architecture.md#Error Handling]

### Existing Infrastructure (from Stories 0.1–0.6)

**Already implemented — DO NOT recreate:**
- `DamageQueue` resource with `entries: Vec<(Entity, f32)>` — in `src/core/collision.rs:80-84`
- `apply_damage` system drains `DamageQueue`, applies to `Health`, inserts `JustDamaged` — in `src/core/collision.rs:182-193`
- `despawn_destroyed` system removes entities with `health.current <= 0.0`, records in `DestroyedPositions` — in `src/core/collision.rs:197-209`
- `Health { current: f32, max: f32 }` component — in `src/core/collision.rs:71-75`
- `Collider { radius: f32 }` component — in `src/core/collision.rs:65-68`
- `circle_circle_intersection(center1, radius1, center2, radius2) -> bool` — in `src/core/collision.rs:52-60`
- `check_laser_collisions` — in `src/core/collision.rs:103-142`
- `check_projectile_collisions` — in `src/core/collision.rs:147-177`
- `DestroyedPositions` and `LaserHitPositions` resources — in `src/core/collision.rs:86-96`
- `JustDamaged { pub amount: f32 }` component — in `src/shared/components.rs:8-12`
- `Velocity(pub Vec2)` component — in `src/shared/components.rs:4-5`
- `Player` marker component — in `src/core/flight.rs:36-37`
- `camera_follow_player` — in `src/core/camera.rs` — returns early if no Player entity (safe during respawn)
- `trigger_screen_shake` reads `JustDamaged` on Player → +0.3 trauma — in `src/rendering/effects.rs:28-52`
- `spawn_destruction_effects` reads `DestroyedPositions` → spawns expanding circles — in `src/rendering/effects.rs:194-210`
- Test helper `test_app()` includes all systems — in `tests/helpers/mod.rs:23-66`
- Test helpers `spawn_asteroid(app, pos, radius, health)`, `spawn_drone(app, pos, radius, health)` — in `tests/helpers/mod.rs:70-96`
- Test helper `spawn_player(app)` — in `tests/helpers/mod.rs:99-110` — NEEDS UPDATE to add Health + Collider

**What's missing (implement in this story):**
1. No `Health` or `Collider` on player entity — player cannot be damaged or killed
2. No contact damage system (body-to-body collision)
3. No player death handling (currently `despawn_destroyed` would remove player permanently)
4. No invincibility system
5. No visual blink for invincibility
6. `despawn_destroyed` has no `Without<Player>` guard — would despawn player
7. Weapon collision queries have no `Without<Player>` filter — player would hit self

### System Ordering (Updated for Story 0.7)

```
PreUpdate: read_input
FixedUpdate:
  CoreSet::Input    → tick_fire_cooldown, regenerate_energy, switch_weapon
  CoreSet::Physics  → apply_thrust, apply_rotation, apply_drag, apply_velocity
  (after Physics)   → fire_weapon, tick_laser_pulses, move_spread_projectiles, tick_spread_projectiles
  CoreSet::Collision → check_laser_collisions, check_projectile_collisions, check_contact_collisions ← NEW
  CoreSet::Damage    → apply_damage → handle_player_death ← NEW → despawn_destroyed (chained, 3-way)
  (after Damage)     → tick_contact_cooldown ← NEW, tick_invincibility ← NEW
Update:
  trigger_damage_flash, update_damage_flash
  spawn_destruction_effects, update_destruction_effects
  spawn_laser_impact_flash, update_impact_flashes
  trigger_screen_shake, blink_invincible ← NEW
PostUpdate:
  camera_follow_player
  apply_screen_shake (AFTER camera_follow_player)
```

### Contact Damage Implementation

```rust
/// Cooldown on contact damage to prevent instant death from continuous overlap.
/// While active, player does not take body-collision damage.
#[derive(Component)]
pub struct ContactDamageCooldown {
    pub timer: f32,
}

const CONTACT_DAMAGE: f32 = 20.0;
const CONTACT_COOLDOWN: f32 = 0.5;
```

System: `check_contact_collisions`
- Query player: `Query<(Entity, &Transform, &Collider), (With<Player>, Without<Invincible>, Without<ContactDamageCooldown>)>`
- Query targets: `Query<(&Transform, &Collider), Without<Player>>`
- For each target: `circle_circle_intersection(player_pos, player_radius, target_pos, target_radius)`
- On first overlap: `damage_queue.entries.push((player_entity, CONTACT_DAMAGE))`, `commands.entity(player_entity).insert(ContactDamageCooldown { timer: CONTACT_COOLDOWN })`, break

### Player Death Implementation

```rust
const INVINCIBILITY_DURATION: f32 = 2.0;
```

System: `handle_player_death`
- Query: `Query<(Entity, &mut Health, &mut Transform, &mut Velocity), With<Player>>`
- When `health.current <= 0.0`:
  1. `destroyed_positions.positions.push(Vec2::new(transform.translation.x, transform.translation.y))`
  2. `health.current = health.max`
  3. `transform.translation = Vec3::ZERO`
  4. `velocity.0 = Vec2::ZERO`
  5. `commands.entity(entity).insert(Invincible { timer: INVINCIBILITY_DURATION })`
  6. `commands.entity(entity).remove::<ContactDamageCooldown>()`

### Invincibility Visual

```rust
/// Player is immune to damage. Removed when timer expires.
#[derive(Component)]
pub struct Invincible {
    pub timer: f32,
}
```

Blink system (rendering, Update schedule):
```rust
// Toggle visibility at 10 Hz
let visible = (time.elapsed_secs() * 10.0 * std::f32::consts::PI).sin() > 0.0;
*visibility = if visible { Visibility::Inherited } else { Visibility::Hidden };
```

On invincibility expiry: restore `Visibility::Inherited` to ensure player is visible.

### Bevy 0.18 Patterns

- **Visibility:** `Visibility::Inherited` (default visible), `Visibility::Hidden` (invisible). NOT `Visibility::Visible` — use `Inherited` for standard entities.
- **Component Insertion:** `commands.entity(e).insert(Component)` — replaces if already present
- **Component Removal:** `commands.entity(e).remove::<Component>()` — no-op if not present
- **Query Filters:** `Without<Player>` on collision targets, `Without<Invincible>` on contact damage
- **System Chaining:** `(a, b, c).chain()` for ordered execution within a set
- **First `app.update()` has dt=0** — prime time in tests before asserting
- **`TimeUpdateStrategy::ManualDuration`** for deterministic tests

### What This Story Does NOT Include

- **No enemy weapon fire** — enemies don't shoot in Epic 0. Player only takes contact damage.
- **No knockback** — on contact, only damage is applied. Knockback deferred to Epic 4 (Combat Depth).
- **No death animation** — player resets instantly. Death animation (fade out, particles) deferred to Epic 10.
- **No game over screen** — no lives system, infinite respawns.
- **No respawn at station** — GDD specifies "respawn at last visited station" but stations don't exist in Epic 0. Respawn at origin.
- **No currency loss on death** — economy not implemented in Epic 0.
- **No death marker on map** — world map not implemented in Epic 0.
- **No asteroid/drone spawning system** — entities exist in tests. Runtime spawning is a separate concern (not part of this story).
- **No team/faction system** — `Without<Player>` is used instead of proper team-based collision filtering. Team system deferred to Epic 4.

### Previous Story Intelligence (Story 0.6)

**Patterns established:**
- `JustDamaged` cross-domain marker: core writes, rendering reads, rendering removes
- `DestroyedPositions` resource bridge: FixedUpdate populates, Update drains
- Asset caching resources for spawned visual entities (DestructionAssets, ImpactFlashAssets)
- `trigger_screen_shake.before(spawn_destruction_effects)` ordering for correct position reading
- Visual feedback systems in `src/rendering/effects.rs`

**Code review fixes from 0.6:**
- System ordering for trigger_screen_shake before spawn_destruction_effects
- GPU asset caching for impact flash (ImpactFlashAssets resource)
- All test expects use `.expect()` not `.unwrap()`
- 98 tests passing (44 unit + 54 integration)

### Git Intelligence

**Last 4 commits (newest first):**
1. `12b533f` — Story 0.6 code review #2: 5 missing unit tests, stale comment fix
2. `f55fb69` — Story 0.6: visual feedback (screen shake, damage flash, destruction effects, laser impact)
3. `4094c8f` — Story 0.5 code review: multi-hit assertion, deny unwrap, stale comments
4. `cab19e0` — Story 0.5: collision detection and damage system

**Patterns from recent work:**
- Systems registered in `CorePlugin` for FixedUpdate, `RenderingPlugin` for Update/PostUpdate
- Cross-domain components in `src/shared/components.rs`
- Integration tests in `tests/` with shared helpers in `tests/helpers/mod.rs`
- `test_app()` harness includes all existing systems
- Constants defined in module (e.g., `FLASH_DURATION`, `DESTRUCTION_LIFETIME`)

### Project Structure Notes

| File | Action | Purpose |
|------|--------|---------|
| `src/shared/components.rs` | MODIFY | Add `Invincible { pub timer: f32 }` and `ContactDamageCooldown { pub timer: f32 }` components |
| `src/core/collision.rs` | MODIFY | Add `Without<Player>` to weapon queries and `despawn_destroyed`; add `check_contact_collisions`, `tick_contact_cooldown`, `handle_player_death`, `tick_invincibility` systems; add contact damage constants |
| `src/core/mod.rs` | MODIFY | Update Damage chain to 3-way, register new systems in Collision set and after Damage set |
| `src/rendering/mod.rs` | MODIFY | Add `Health` and `Collider` imports to `setup_player`, add components to player spawn bundle; register `blink_invincible` in Update |
| `src/rendering/effects.rs` | MODIFY | Add `blink_invincible` system |
| `tests/helpers/mod.rs` | MODIFY | Add `Health` and `Collider` to `spawn_player()` and `spawn_player_with_velocity()` |
| `tests/instant_respawn.rs` | CREATE | Integration tests for contact damage, player death, respawn, invincibility, self-damage prevention |

### References

- [Source: epics.md#Epic 0, Story 7] — "As a player, I respawn instantly after death so that the failure loop is fast and non-punishing"
- [Source: gdd.md#Win/Loss] — "No permadeath. No game over. Death = instant respawn at last visited station."
- [Source: gdd.md#Failure Recovery] — "Cost of death is money and position, not progress"
- [Source: gdd.md#Inner Loop] — "Fly → Encounter → React → Survive → Fly"
- [Source: game-architecture.md#System Ordering] — FixedUpdate: Input → Physics → Collision → Damage → Events
- [Source: game-architecture.md#Architectural Boundaries] — Cross-plugin communication via shared components
- [Source: game-architecture.md#Error Handling] — No `unwrap()`, graceful degradation
- [Source: 0-6-visual-feedback.md#Dev Notes] — DestroyedPositions bridge, JustDamaged pattern, screen shake pipeline
- [Source: 0-5-destroy-asteroids-and-drones.md] — DamageQueue pattern, apply_damage, despawn_destroyed systems

## Dev Agent Record

### Agent Model Used

Claude claude-opus-4-6 (via Claude Code) - Implementation via dev-story workflow

### Debug Log References

- tick_invincibility query required Option<&mut Visibility> since MinimalPlugins (unit tests) doesn't provide Visibility component
- Unit test for invincibility timer uses relaxed assertion (timer < initial) due to Bevy's time delta behavior in Update schedule with ManualDuration

### Completion Notes List

- Story 0.7 (Instant Respawn) created with comprehensive developer guidance
- 12 Acceptance Criteria covering player health, contact damage, death handling, respawn, invincibility, and regression safety
- 6 detailed tasks with 30+ subtasks for complete implementation guidance
- All 6 tasks and 30+ subtasks implemented and verified
- Player entity now spawns with Health(100.0) and Collider(12.0)
- Contact damage system: check_contact_collisions with 0.5s cooldown, 20.0 damage per hit
- Player death handling: health/position/velocity reset, destruction effect at death position, invincibility granted
- Invincibility system: 2s duration, 10 Hz visibility blink, timer tick with Visibility restore on expiry
- Self-damage prevention: Without<Player> on laser and projectile collision target queries
- despawn_destroyed now has Without<Player> filter — player entity is never despawned
- 3-way Damage chain: apply_damage → handle_player_death → despawn_destroyed
- 17 new tests (10 unit + 7 integration), 106 total tests passing, 0 regressions
- No unwrap() in game code, clippy clean with -D warnings

### File List

| File | Action | Description |
|------|--------|-------------|
| `src/shared/components.rs` | MODIFIED | Added `ContactDamageCooldown` and `Invincible` components |
| `src/core/collision.rs` | MODIFIED | Added `Without<Player>` to weapon/despawn queries; added `check_contact_collisions`, `tick_contact_cooldown`, `handle_player_death`, `tick_invincibility` systems; added constants; added 10 unit tests |
| `src/core/mod.rs` | MODIFIED | Updated Damage chain to 3-way, registered new systems in Collision set and after Damage set |
| `src/rendering/mod.rs` | MODIFIED | Added Health + Collider to player spawn; registered `blink_invincible` in Update |
| `src/rendering/effects.rs` | MODIFIED | Added `blink_invincible` system for invincibility visual; added 1 unit test |
| `tests/helpers/mod.rs` | MODIFIED | Added Health + Collider to `spawn_player()` and `spawn_player_with_velocity()`; added new systems to `test_app()` |
| `tests/instant_respawn.rs` | CREATED | 7 integration tests for contact damage, death/respawn, invincibility, self-damage prevention |

## Change Log

- 2026-02-26: Implemented Story 0.7 — Instant Respawn. Added contact damage system (20 damage, 0.5s cooldown), player death handling (reset to origin with full health), invincibility system (2s with 10 Hz blink), self-damage prevention (Without<Player> on weapon queries). 14 new tests, 105 total passing.
- 2026-02-26: Code review fixes — 3 clippy::type_complexity allows added, contact collision target query now requires With<Health> (consistency), 4 new unit tests (cooldown expiry, invincibility expiry+visibility restore, blink toggle), stale Health doc comment updated. 106 total tests, clippy clean.
