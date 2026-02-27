# Story 0.8: Runtime Spawning

Status: done

<!-- Note: Validation is optional. Run validate-create-story for quality check before dev-story. -->

## Story

As a player,
I encounter asteroids and Scout Drones in the world,
so that the arcade loop has targets to shoot and threats to avoid.

## Acceptance Criteria

1. Asteroids spawn at configurable positions around the player's starting area on game start
2. Scout Drones spawn at configurable positions on game start
3. Asteroids have procedural visual representation (irregular polygon, grey/brown tone)
4. Scout Drones have procedural visual representation (distinct from asteroids, red/hostile tone)
5. Asteroids drift with slow random velocity (not stationary)
6. Scout Drones drift with slow random velocity (no AI behavior in Epic 0)
7. Destroyed asteroids and drones respawn after a configurable delay (world stays populated)
8. Spawning parameters are configurable via RON file (`assets/config/spawning.ron`)
9. Entity count stays within budget: 3-5 asteroids, 2-3 drones for Epic 0 scope
10. All spawned entities interact correctly with existing collision/damage systems (laser, projectile, contact)
11. All existing tests continue to pass (no regression)
12. No `unwrap()` in game code — `#[deny(clippy::unwrap_used)]` enforced

## Tasks / Subtasks

- [x] Task 1: Marker Components and Spawning Config (AC: #1, #2, #8, #9)
  - [x] 1.1 Create `Asteroid` marker component in `src/core/spawning.rs`
  - [x] 1.2 Create `ScoutDrone` marker component in `src/core/spawning.rs`
  - [x] 1.3 Create `NeedsAsteroidVisual` marker component in `src/core/spawning.rs` (for rendering pipeline)
  - [x] 1.4 Create `NeedsDroneVisual` marker component in `src/core/spawning.rs` (for rendering pipeline)
  - [x] 1.5 Create `RespawnTimer { pub timer: f32, pub spawn_type: SpawnType, pub position: Vec2 }` component in `src/core/spawning.rs`
  - [x] 1.6 Create `SpawnType` enum: `Asteroid`, `ScoutDrone`
  - [x] 1.7 Create `SpawningConfig` struct with RON deserialization: asteroid positions/count, drone positions/count, asteroid health/radius, drone health/radius, respawn delay, velocity range
  - [x] 1.8 Create `assets/config/spawning.ron` with default values (3-5 asteroids, 2-3 drones, positions spread around origin)
  - [x] 1.9 Load `SpawningConfig` via `from_ron()` with `Default` fallback (same pattern as `FlightConfig`/`WeaponConfig`)

- [x] Task 2: Spawning Systems (AC: #1, #2, #5, #6)
  - [x] 2.1 Create `spawn_initial_entities` Startup system: reads `SpawningConfig`, spawns asteroids and drones with `Transform`, `Velocity` (random within range), `Collider`, `Health`, `Asteroid`/`ScoutDrone` marker, and `NeedsAsteroidVisual`/`NeedsDroneVisual` marker
  - [x] 2.2 Asteroids: radius from config (default ~20.0), health from config (default ~50.0), random velocity magnitude 5-15 units/s
  - [x] 2.3 Scout Drones: radius from config (default ~10.0), health from config (default ~30.0), random velocity magnitude 10-25 units/s

- [x] Task 3: Respawn System (AC: #7)
  - [x] 3.1 Modify `despawn_destroyed` or create a parallel system: when an entity with `Asteroid` or `ScoutDrone` marker reaches zero health, spawn a `RespawnTimer` entity at its position before despawning
  - [x] 3.2 Create `tick_respawn_timers` system in FixedUpdate: decrements timer by dt, when expired spawns a new entity of the appropriate type at the stored position (with new random velocity) and despawns the timer entity
  - [x] 3.3 Respawn delay configurable in `SpawningConfig` (default: 5.0 seconds)

- [x] Task 4: Visual Representation (AC: #3, #4)
  - [x] 4.1 Create `generate_asteroid_mesh(radius: f32)` in `src/rendering/vector_art.rs`: irregular polygon using lyon (6-8 vertices with slight random offset from circle), or simple Circle mesh for MVP
  - [x] 4.2 Create `generate_drone_mesh(radius: f32)` in `src/rendering/vector_art.rs`: diamond or small triangle shape to distinguish from asteroids
  - [x] 4.3 Create `AsteroidAssets` resource in `src/rendering/mod.rs` with cached mesh + material (grey/brown: `Color::srgb(0.6, 0.5, 0.4)`)
  - [x] 4.4 Create `DroneAssets` resource in `src/rendering/mod.rs` with cached mesh + material (red/hostile: `Color::srgb(0.9, 0.2, 0.2)`)
  - [x] 4.5 Create `setup_asteroid_assets` and `setup_drone_assets` Startup systems
  - [x] 4.6 Create `render_asteroids` Update system: queries `With<NeedsAsteroidVisual>`, attaches `Mesh2d` + `MeshMaterial2d`, removes marker
  - [x] 4.7 Create `render_drones` Update system: queries `With<NeedsDroneVisual>`, attaches `Mesh2d` + `MeshMaterial2d`, removes marker

- [x] Task 5: System Registration (AC: #10, #11)
  - [x] 5.1 Create `src/core/spawning.rs` module, add `pub mod spawning;` to `src/core/mod.rs`
  - [x] 5.2 Register `spawn_initial_entities` in Startup (CorePlugin)
  - [x] 5.3 Register `tick_respawn_timers` in FixedUpdate, after `CoreSet::Damage` (so respawn happens after destruction)
  - [x] 5.4 Register `setup_asteroid_assets`, `setup_drone_assets` in Startup (RenderingPlugin)
  - [x] 5.5 Register `render_asteroids`, `render_drones` in Update (RenderingPlugin)
  - [x] 5.6 Export new types from `src/core/spawning.rs` for use in tests

- [x] Task 6: Tests (AC: #1-12)
  - [x] 6.1 Unit tests in `src/core/spawning.rs` (10 tests):
    - SpawningConfig loads from RON with defaults
    - spawn_initial_entities creates correct number of asteroid entities
    - spawn_initial_entities creates correct number of drone entities
    - Respawn timer ticks down and spawns new entity when expired
    - Respawn timer spawns drone type
    - Spawned entities have correct components (Health, Collider, Velocity, marker)
    - Spawned asteroids have NeedsAsteroidVisual marker
    - Spawned drones have NeedsDroneVisual marker
    - spawn_respawn_timers creates timer for destroyed asteroid
    - drift_entities moves asteroids
  - [x] 6.2 Integration tests in `tests/runtime_spawning.rs` (7 tests):
    - Asteroid can be destroyed by laser (full pipeline: spawn → fire → collision → damage → despawn)
    - Drone can be destroyed by spread projectiles
    - Destroyed asteroid creates respawn timer
    - Destroyed asteroid respawns after delay
    - Spawned asteroid deals contact damage to player
    - Scout drone has correct marker
    - Asteroid has correct marker
  - [x] 6.3 Update `tests/helpers/mod.rs`: add `SpawningConfig` import and new system registrations for test_app
  - [x] 6.4 Verify no regression: all 106 existing tests pass (now 124 total: 65 unit + 59 integration)

## Dev Notes

### Architecture Patterns and Constraints

- **Core/Rendering Split:** Core spawns entities with marker components (`NeedsAsteroidVisual`, `NeedsDroneVisual`). Rendering detects markers, attaches `Mesh2d` + `MeshMaterial2d`, removes marker. This is the established pattern from `NeedsLaserVisual` and `NeedsProjectileVisual`. [Source: game-architecture.md#Architectural Boundaries]
- **RON Config Pattern:** Load config with `from_ron()` fallback to `Default::default()` with `warn!`. See `FlightConfig` and `WeaponConfig` for the exact pattern. [Source: src/core/flight.rs, src/core/weapons.rs]
- **Entity Composition:** All damageable entities need `Collider` + `Health` + `Transform`. Contact-damageable entities (that can hurt the player) need no special component — anything with `Collider` that isn't `Player` will trigger contact damage. [Source: src/core/collision.rs check_contact_collisions]
- **Deterministic Spawning (Epic 0):** Simple fixed-position spawning. NOT procedural/noise-based — that's Epic 1 (seamless world generation). Keep it simple: a list of spawn points in the RON config. [Source: epics.md#Epic 0 vs Epic 1]
- **Respawn on Destruction:** When entities die, they should eventually come back so the world stays populated. A simple timer-based respawn at the original position is sufficient for Epic 0.
- **No `unwrap()`** — enforced via `#[deny(clippy::unwrap_used)]` in lib.rs. [Source: game-architecture.md#Error Handling]

### Existing Infrastructure (from Stories 0.1-0.7)

**Already implemented — DO NOT recreate:**
- `Health { current: f32, max: f32 }` component — `src/core/collision.rs`
- `Collider { radius: f32 }` component — `src/core/collision.rs`
- `Velocity(pub Vec2)` component — `src/shared/components.rs`
- `DamageQueue` resource — `src/core/collision.rs`
- `DestroyedPositions` resource — `src/core/collision.rs`
- `check_laser_collisions` — hitscan ray-circle against all non-Player entities with Collider+Health
- `check_projectile_collisions` — circle-circle against all non-Player entities with Collider+Health
- `check_contact_collisions` — player body vs all non-Player entities with Collider (+ With<Health>)
- `apply_damage` — drains DamageQueue, subtracts from Health, inserts JustDamaged
- `despawn_destroyed` — removes non-Player entities with health <= 0, records in DestroyedPositions
- `spawn_destruction_effects` — reads DestroyedPositions, spawns expanding circles
- `trigger_screen_shake` — reads JustDamaged on Player for trauma
- `Player` marker component — `src/core/flight.rs`
- `NeedsLaserVisual`, `NeedsProjectileVisual` marker pattern — `src/core/weapons.rs`
- Asset caching pattern: `LaserAssets`, `ProjectileAssets` resources — `src/rendering/mod.rs`
- `generate_projectile_mesh(radius)` for Circle mesh — `src/rendering/vector_art.rs`
- Test helpers: `spawn_asteroid()`, `spawn_drone()`, `spawn_player()` — `tests/helpers/mod.rs`
- `test_app()` with all systems chained in FixedUpdate — `tests/helpers/mod.rs`

**What's missing (implement in this story):**
1. No marker components for asteroids/drones (currently bare Collider+Health+Transform in tests)
2. No runtime spawning system — entities only exist in test helpers
3. No visual representation for asteroids/drones — no mesh/material
4. No respawn mechanism after destruction
5. No spawning configuration

### System Ordering (Updated for Story 0.8)

```
Startup:
  CorePlugin:      spawn_initial_entities ← NEW
  RenderingPlugin: setup_player, setup_laser_assets, setup_projectile_assets,
                   setup_asteroid_assets ← NEW, setup_drone_assets ← NEW,
                   setup_flash_materials, setup_destruction_assets, setup_impact_flash_assets,
                   setup_starfield
FixedUpdate:
  CoreSet::Input    → tick_fire_cooldown, regenerate_energy, switch_weapon
  CoreSet::Physics  → apply_thrust, apply_rotation, apply_drag, apply_velocity
  (after Physics)   → fire_weapon, tick_laser_pulses, move_spread_projectiles, tick_spread_projectiles
  CoreSet::Collision → check_laser_collisions, check_projectile_collisions, check_contact_collisions
  CoreSet::Damage    → apply_damage → handle_player_death → despawn_destroyed (chained)
  (after Damage)     → tick_contact_cooldown, tick_invincibility, tick_respawn_timers ← NEW
Update:
  render_laser_pulses, render_spread_projectiles
  render_asteroids ← NEW, render_drones ← NEW
  trigger_damage_flash, update_damage_flash
  spawn_destruction_effects, update_destruction_effects
  spawn_laser_impact_flash, update_impact_flashes
  trigger_screen_shake, blink_invincible, update_starfield
PostUpdate:
  camera_follow_player → apply_screen_shake
```

### Bevy 0.18 Patterns

- **Marker Components:** `#[derive(Component)]` with no fields, used as query filters
- **Asset Caching:** Resources holding `Handle<Mesh>` + `Handle<ColorMaterial>`, initialized at Startup
- **Mesh2d + MeshMaterial2d:** Bevy 0.18's 2D rendering components (not `Sprite` or old `SpriteBundle`)
- **lyon_tessellation:** Used for procedural mesh generation (player ship already uses this)
- **Circle::new(radius):** Simple circle mesh via `Mesh::from(Circle::new(r))` (used for projectiles)
- **System Chaining:** `.chain()` for ordered execution; `.after(set)` for cross-set ordering
- **`from_ron()`** pattern with `warn!` fallback for config loading

### What This Story Does NOT Include

- **No enemy AI** — drones just drift. AI behavior is Epic 4 (Combat Depth).
- **No procedural world generation** — fixed spawn positions. Noise-based spawning is Epic 1.
- **No chunk-based spawning** — all entities spawn at game start. Chunk loading is Epic 1.
- **No asteroid size variation** — all same radius. Visual variety deferred to Epic 10.
- **No enemy types beyond Scout Drone** — Fighters, Heavy Cruisers, etc. are Epic 4.
- **No wave-based spawning** — no escalation, no difficulty ramp. Just persistent population.
- **No spawn distance from player** — entities spawn at fixed positions regardless of player location.

### Previous Story Intelligence (Story 0.7)

**Patterns established:**
- `ContactDamageCooldown` and `Invincible` cross-domain components in shared/components.rs
- `handle_player_death` runs between apply_damage and despawn_destroyed (3-way chain)
- `despawn_destroyed` has `Without<Player>` — only despawns non-player entities with zero health
- 106 tests passing (46 unit + 60 integration)

**Code review fixes from 0.7:**
- `#[allow(clippy::type_complexity)]` on 3 collision system functions
- `With<Health>` added to contact collision target query for consistency
- Bevy timer tests need small time steps (1/60s) with many iterations, not large single deltas

### Project Structure Notes

| File | Action | Purpose |
|------|--------|---------|
| `src/core/spawning.rs` | CREATE | Marker components (Asteroid, ScoutDrone), NeedsVisual markers, SpawningConfig, spawn_initial_entities, tick_respawn_timers, RespawnTimer |
| `src/core/mod.rs` | MODIFY | Add `pub mod spawning;`, register spawn_initial_entities (Startup) and tick_respawn_timers (after Damage) |
| `src/rendering/vector_art.rs` | MODIFY | Add generate_asteroid_mesh and generate_drone_mesh functions |
| `src/rendering/mod.rs` | MODIFY | Add AsteroidAssets/DroneAssets resources, setup systems, render systems; register in Startup and Update |
| `assets/config/spawning.ron` | CREATE | Spawning configuration (positions, health, radius, respawn delay, velocity range) |
| `tests/helpers/mod.rs` | MODIFY | Add spawning imports, update spawn_asteroid/spawn_drone to include marker components, register new systems in test_app |
| `tests/runtime_spawning.rs` | CREATE | Integration tests for spawning, destruction, and respawn |

### References

- [Source: epics.md#Epic 0] — "Basic asteroid spawning (destructible)", "Basic enemy spawning (Scout Drone)"
- [Source: epics.md#Epic 0, Deliverable] — "A playable arcade loop: fly, shoot asteroids and drones, die, respawn, repeat"
- [Source: gdd.md#Inner Loop] — "Fly → Encounter → React → Survive → Fly"
- [Source: game-architecture.md#Entity Budget] — 60-80 asteroids, 15-25 enemies (full game); Epic 0 uses minimal subset
- [Source: game-architecture.md#Spawning] — Non-pooled: Player, enemies, asteroids (direct commands.spawn())
- [Source: game-architecture.md#Architectural Boundaries] — Core spawns with markers, rendering attaches visuals
- [Source: 0-7-instant-respawn.md] — Player death/respawn, contact damage, invincibility, Without<Player> patterns
- [Source: 0-5-destroy-asteroids-and-drones.md] — Collision and damage pipeline, DamageQueue pattern

## Dev Agent Record

### Agent Model Used

Claude Opus 4.6

### Debug Log References

- No blocking issues encountered

### Completion Notes List

- Created `src/core/spawning.rs` with all marker components (`Asteroid`, `ScoutDrone`, `NeedsAsteroidVisual`, `NeedsDroneVisual`), `SpawnType` enum, `RespawnTimer` component, `SpawningConfig` with RON deserialization
- Created `spawn_initial_entities` Startup system reading from SpawningConfig
- Created `spawn_respawn_timers` system that detects destroyed asteroid/drone entities (health <= 0) and creates RespawnTimer entities — runs BEFORE despawn_destroyed in the Damage chain
- Created `tick_respawn_timers` system that counts down and spawns replacement entities with new random velocity
- Created `drift_entities` system to apply Velocity to non-Player entities (asteroids/drones)
- Created `generate_asteroid_mesh` (lyon irregular 8-vertex polygon) and `generate_drone_mesh` (lyon diamond shape) in vector_art.rs
- Created `AsteroidAssets`/`DroneAssets` resources with cached mesh+material handles in rendering/mod.rs
- Created `render_asteroids`/`render_drones` systems following established NeedsVisual pattern
- Added `rand = "0.9"` dependency for random velocity generation
- Updated test helpers: `spawn_asteroid`/`spawn_drone` now include marker components and Velocity
- Updated `test_app()` to include spawning systems in FixedUpdate chain
- Damage chain expanded: apply_damage → handle_player_death → spawn_respawn_timers → despawn_destroyed
- All 124 tests pass (65 unit + 59 integration), 18 new tests added (10 unit + 7 integration + 1 in existing test updates)

### Change Log

- 2026-02-26: Implemented Story 0.8 — Runtime Spawning (all 6 tasks complete)
- 2026-02-26: Code review fixes — fixed stale docstring on spawn_respawn_timers, added 4 missing unit tests (drift_entities_moves_drones, spawn_respawn_timers_creates_timer_for_destroyed_drone, generate_asteroid_mesh_produces_vertices, generate_drone_mesh_produces_vertices)

### File List

- `src/core/spawning.rs` — NEW: Marker components, SpawningConfig, spawn/respawn/drift systems, 10 unit tests
- `src/core/mod.rs` — MODIFIED: Added `pub mod spawning;`, SpawningConfig loading, system registration
- `src/rendering/vector_art.rs` — MODIFIED: Added generate_asteroid_mesh and generate_drone_mesh
- `src/rendering/mod.rs` — MODIFIED: Added AsteroidAssets/DroneAssets, setup/render systems
- `assets/config/spawning.ron` — NEW: Spawning configuration (4 asteroids, 2 drones, velocities, respawn delay)
- `tests/helpers/mod.rs` — MODIFIED: Added spawning imports, markers to spawn_asteroid/spawn_drone, new systems in test_app
- `tests/runtime_spawning.rs` — NEW: 7 integration tests for spawning, destruction, respawn, contact damage
- `Cargo.toml` — MODIFIED: Added `rand = "0.9"` dependency
