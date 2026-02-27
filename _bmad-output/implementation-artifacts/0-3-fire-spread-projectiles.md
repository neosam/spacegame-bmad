# Story 0.3: Fire Spread Projectiles

Status: done

<!-- Note: Validation is optional. Run validate-create-story for quality check before dev-story. -->

## Story

As a player,
I want to fire spread projectiles that consume energy,
so that I have a tactical weapon choice.

## Acceptance Criteria

1. When spread is the active weapon and fire input is pressed, multiple projectiles spawn in a configurable arc pattern (e.g., 5 projectiles across ~30°)
2. Spread projectiles are physical entities that fly through space at a configurable speed — visible, moving, not hitscan
3. Each projectile entity despawns after a configurable lifetime (range/time)
4. Firing spread consumes energy from the player's energy bar; if insufficient energy, spread does NOT fire
5. The player has an energy component with a maximum capacity, current value, and configurable regeneration rate per second
6. Energy regenerates passively over time, clamped at max capacity
7. All spread/energy balance values are configurable via `WeaponConfig` in `assets/config/weapons.ron` (same RON + fallback pattern)
8. Projectile entities store origin, direction, speed, and damage for future collision detection (Story 0.5) — but NO collision in this story
9. An `ActiveWeapon` enum component on the player determines which weapon fires on input (Laser or Spread); defaults to Laser
10. All projectile and energy systems run in `FixedUpdate` within the existing `CoreSet` ordering
11. Visible projectile rendering: small colorful meshes (bright magenta/orange) moving through space
12. No `unwrap()` in game code — `#[deny(clippy::unwrap_used)]` enforced

## Tasks / Subtasks

- [x] Task 1: Energy System (AC: #4, #5, #6, #7)
  - [x] 1.1 Add `Energy` component to `src/core/weapons.rs`: `current: f32, max_capacity: f32`
  - [x] 1.2 Add energy config fields to `WeaponConfig`: `energy_max: f32`, `energy_regen_rate: f32`, `spread_energy_cost: f32`
  - [x] 1.3 Update `assets/config/weapons.ron` with energy balance values
  - [x] 1.4 Update `WeaponConfig::default()` with sensible energy defaults
  - [x] 1.5 Create `regenerate_energy` system: increments `Energy.current` by `regen_rate * dt`, clamped at `max_capacity`
  - [x] 1.6 Add `Energy` component to player entity spawn in `src/rendering/mod.rs` `setup_player`
  - [x] 1.7 Register `regenerate_energy` in `CorePlugin` FixedUpdate (in `CoreSet::Input` alongside `tick_fire_cooldown`)
  - [x] 1.8 Add unit tests: energy regenerates, energy clamps at max, energy defaults are valid

- [x] Task 2: ActiveWeapon Enum and Spread Projectile Components (AC: #8, #9)
  - [x] 2.1 Define `ActiveWeapon` enum in `src/core/weapons.rs`: `Laser`, `Spread` — derive `Component, Default` (default = Laser)
  - [x] 2.2 Define `SpreadProjectile` component: `origin: Vec2, direction: Vec2, speed: f32, damage: f32, timer: f32` (remaining lifetime)
  - [x] 2.3 Define `SpreadFired` message (Bevy 0.18 Message system): `origin: Vec2, direction: Vec2, count: u32` — for future systems
  - [x] 2.4 Add spread config fields to `WeaponConfig`: `spread_projectile_count: u32`, `spread_arc_degrees: f32`, `spread_projectile_speed: f32`, `spread_projectile_lifetime: f32`, `spread_damage: f32`, `spread_fire_rate: f32`
  - [x] 2.5 Update `WeaponConfig::default()` and `weapons.ron` with spread values
  - [x] 2.6 Add `ActiveWeapon` component to player entity spawn
  - [x] 2.7 Register `SpreadFired` message in `CorePlugin` with `app.add_message::<SpreadFired>()`

- [x] Task 3: Spread Firing System (AC: #1, #2, #4, #10)
  - [x] 3.1 Modify `fire_laser` system → rename to `fire_weapon` or add an `ActiveWeapon` check: if `ActiveWeapon::Laser` → existing laser logic; if `ActiveWeapon::Spread` → spread logic
  - [x] 3.2 Implement spread firing logic:
    - Check `ActiveWeapon::Spread` on player
    - Check `FireCooldown` (reuse existing, set to `1.0 / config.spread_fire_rate`)
    - Check `Energy.current >= config.spread_energy_cost` — skip if insufficient
    - Deduct `config.spread_energy_cost` from `Energy.current`
    - Calculate spread directions: evenly distributed across `spread_arc_degrees` centered on facing direction
    - Spawn `spread_projectile_count` entities, each with `SpreadProjectile`, `NeedsProjectileVisual` marker, and `Transform`
    - Emit `SpreadFired` message
  - [x] 3.3 Ensure arc calculation uses radians: `arc_rad = spread_arc_degrees.to_radians()`; distribute N projectiles from `-arc_rad/2` to `+arc_rad/2` around facing direction

- [x] Task 4: Projectile Movement and Lifetime (AC: #2, #3, #10)
  - [x] 4.1 Create `move_spread_projectiles` system: for each `SpreadProjectile`, update `Transform.translation` by `direction * speed * dt`
  - [x] 4.2 Create `tick_spread_projectiles` system: decrement `SpreadProjectile.timer` by `dt`, despawn when timer <= 0
  - [x] 4.3 Register both systems in `FixedUpdate` after Physics, alongside existing weapon systems (chain: fire_weapon, tick_laser_pulses, move_spread_projectiles, tick_spread_projectiles)

- [x] Task 5: Spread Projectile Rendering (AC: #11)
  - [x] 5.1 In `src/rendering/vector_art.rs`, create `generate_projectile_mesh(radius: f32) -> Mesh` — small circle or diamond shape
  - [x] 5.2 In `src/rendering/mod.rs`, add `ProjectileAssets` resource (cached mesh + material handles), init in `setup_projectile_assets` Startup system
  - [x] 5.3 Create `render_spread_projectiles` system: attaches cached `Mesh2d` + `MeshMaterial2d` (bright magenta/orange `srgb(1.0, 0.4, 0.2)`) to entities with `NeedsProjectileVisual` marker, then removes marker
  - [x] 5.4 Register `render_spread_projectiles` in `RenderingPlugin` Update schedule

- [x] Task 6: Tests (AC: #1, #2, #3, #4, #5, #6, #8, #12)
  - [x] 6.1 Unit tests in `src/core/weapons.rs`:
    - Energy regenerates correctly per tick
    - Energy clamps at max capacity
    - WeaponConfig defaults include valid spread/energy values
    - WeaponConfig from_ron parses spread/energy fields
  - [x] 6.2 Integration tests in `tests/spread_firing.rs`:
    - Spread projectiles spawn when fire input active + ActiveWeapon::Spread + sufficient energy
    - Spread does NOT fire when energy insufficient
    - Spread does NOT fire when ActiveWeapon::Laser
    - Correct number of projectiles spawned (spread_projectile_count)
    - Projectiles move in facing direction over frames
    - Projectiles despawn after lifetime expires
    - Energy is deducted on spread fire
    - Energy regenerates over time
    - SpreadFired message emitted on fire
    - Fire rate cooldown limits spread fire rate
  - [x] 6.3 Update `tests/helpers/mod.rs`:
    - `spawn_player` includes `Energy` and `ActiveWeapon` components
    - Add `move_spread_projectiles`, `tick_spread_projectiles`, `regenerate_energy` to test_app FixedUpdate chain
    - Register `SpreadFired` message
    - Add helper: `fn set_active_weapon_spread(app: &mut App, entity: Entity)` sets `ActiveWeapon::Spread`

## Dev Notes

### Architecture Patterns and Constraints

- **Projectile, not hitscan** — spread fires physical entities that MOVE through space with velocity. Unlike the laser (instant ray, visual-only entity), spread projectiles are real moving entities with `Transform` updates each frame. [Source: gdd.md#Weapon Systems, Category 2]
- **Energy system** — single energy bar, regenerates over time. Laser is energy-free (Story 0.2). Spread costs energy. If energy is depleted, spread cannot fire but laser still works. [Source: gdd.md#Energy system]
- **ActiveWeapon enum** — introduced early to prepare for weapon switching (Story 0.4). Default is Laser. Story 0.4 adds the input mapping to cycle it. No throwaway code.
- **No object pooling yet** — architecture specifies `ObjectPool` for projectiles (80 pre-allocated) in `src/infrastructure/pool.rs`. This module does NOT exist yet. For Story 0.3, use simple `Commands::spawn/despawn`. Pooling optimization deferred to Epic 1+. [Source: game-architecture.md#Object Pooling]
- **No collision detection** — projectile entities store data for future ray/circle intersection in Story 0.5. Do NOT implement collision. [Source: epics.md#Epic 0, Story 5]
- **No screen shake / audio** — visual feedback (Story 0.6), audio deferred. [Source: gdd.md#Weapon Feel]
- **Custom physics only** — no physics engine crate. Movement is `translation += direction * speed * dt`. [Source: game-architecture.md#Physics/Flight]
- **No `unwrap()` in game code** — enforced via `#[deny(clippy::unwrap_used)]`. Use `.expect()` in tests only.
- **Graceful degradation** — config loading falls back to defaults on error.

### System Ordering (Critical)

Weapon systems integrate into the existing `CoreSet` chain in `FixedUpdate`:

```
PreUpdate: read_input (existing)
FixedUpdate:
  CoreSet::Input    → tick_fire_cooldown, regenerate_energy
  CoreSet::Physics  → apply_thrust, apply_rotation, apply_drag, apply_velocity (existing)
  (after Physics)   → fire_weapon, tick_laser_pulses, move_spread_projectiles, tick_spread_projectiles
  CoreSet::Collision → (Story 0.5)
  CoreSet::Damage   → (Story 0.5)
  CoreSet::Events   → (Epic 1)
PostUpdate: camera_follow_player (existing)
```

### Spread Arc Calculation

```rust
let arc_rad = config.spread_arc_degrees.to_radians();
let step = if config.spread_projectile_count > 1 {
    arc_rad / (config.spread_projectile_count - 1) as f32
} else {
    0.0
};
let start_angle = -arc_rad / 2.0;

for i in 0..config.spread_projectile_count {
    let offset_angle = start_angle + step * i as f32;
    let cos = offset_angle.cos();
    let sin = offset_angle.sin();
    let proj_direction = Vec2::new(
        facing.x * cos - facing.y * sin,
        facing.x * sin + facing.y * cos,
    );
    // Spawn projectile with proj_direction...
}
```

### Energy Component

```rust
#[derive(Component)]
pub struct Energy {
    pub current: f32,
    pub max_capacity: f32,
}

impl Default for Energy {
    fn default() -> Self {
        Self { current: 100.0, max_capacity: 100.0 }
    }
}
```

### SpreadProjectile Entity Structure

```rust
#[derive(Component)]
pub struct SpreadProjectile {
    pub origin: Vec2,
    pub direction: Vec2,
    pub speed: f32,
    pub damage: f32,
    pub timer: f32, // remaining lifetime in seconds
}

/// Marker for projectile entities needing visual mesh.
#[derive(Component)]
pub struct NeedsProjectileVisual;
```

### WeaponConfig Additions

```rust
// Add to existing WeaponConfig:
pub energy_max: f32,              // e.g., 100.0
pub energy_regen_rate: f32,       // per second, e.g., 15.0
pub spread_energy_cost: f32,      // per shot, e.g., 20.0
pub spread_projectile_count: u32, // e.g., 5
pub spread_arc_degrees: f32,      // e.g., 30.0
pub spread_projectile_speed: f32, // world units/sec, e.g., 600.0
pub spread_projectile_lifetime: f32, // seconds, e.g., 0.8
pub spread_damage: f32,           // per projectile, e.g., 5.0
pub spread_fire_rate: f32,        // pulses/sec, e.g., 2.0
```

### Refactoring fire_laser → fire_weapon

The existing `fire_laser` system checks `action_state.fire` and spawns laser pulses. For Story 0.3, this system needs to branch on `ActiveWeapon`:

```rust
pub fn fire_weapon(
    action_state: Res<ActionState>,
    config: Res<WeaponConfig>,
    mut player_query: Query<(&Transform, &mut FireCooldown, &mut Energy, &ActiveWeapon), With<Player>>,
    mut commands: Commands,
    mut laser_events: MessageWriter<LaserFired>,
    mut spread_events: MessageWriter<SpreadFired>,
) {
    if !action_state.fire { return; }
    for (transform, mut cooldown, mut energy, active_weapon) in player_query.iter_mut() {
        if cooldown.timer > 0.0 { continue; }
        match active_weapon {
            ActiveWeapon::Laser => { /* existing laser logic */ }
            ActiveWeapon::Spread => { /* new spread logic with energy check */ }
        }
    }
}
```

**IMPORTANT:** When renaming `fire_laser` → `fire_weapon`, update ALL references:
- `src/core/mod.rs` imports and system registration
- `tests/helpers/mod.rs` imports and system registration
- `tests/laser_firing.rs` imports (if any reference `fire_laser` directly)

### NeedsProjectileVisual Pattern

Follow the same marker pattern established in Story 0.2 for laser rendering:
- Core spawns entity with `SpreadProjectile` + `NeedsProjectileVisual` + `Transform`
- Rendering system (in `src/rendering/mod.rs`) adds `Mesh2d` + `MeshMaterial2d` and removes marker
- This keeps core systems testable without rendering infrastructure

### What This Story Does NOT Include

- **No collision detection** — that's Story 0.5
- **No damage dealing** — that's Story 0.5
- **No weapon switching input** — that's Story 0.4 (but ActiveWeapon enum is introduced here)
- **No screen shake** — that's Story 0.6
- **No audio** — deferred
- **No object pooling** — deferred to Epic 1+
- **No energy bar UI/HUD** — UI is outside arcade prototype scope

### Previous Story Intelligence (Story 0.2)

**Patterns established:**
- `WeaponConfig` loaded from RON in `CorePlugin::build()` with graceful fallback
- `FireCooldown` component on player for rate-limiting
- `NeedsLaserVisual` marker pattern for core/rendering separation
- `LaserAssets` resource caches mesh/material handles (avoid per-frame allocation)
- `LaserFired` message (Bevy 0.18 `#[derive(Message)]`, `MessageWriter`, `.write()`)
- Weapon systems registered after `CoreSet::Physics`, before `CoreSet::Collision`

**Bevy 0.18 gotchas:**
- Events → Messages: `#[derive(Message)]`, `MessageWriter`, `app.add_message::<T>()`
- `MessageWriter.write()` not `.send()`
- `MinimalPlugins` has no `Assets<Mesh>` — use marker pattern for testability
- First `app.update()` has dt=0 — prime in tests
- `TimeUpdateStrategy::ManualDuration` for deterministic tests

**Code review fixes applied in 0.2:**
- `generate_laser_mesh()` added to `vector_art.rs` per spec
- `render_laser_pulses` moved from `core/weapons.rs` to `rendering/mod.rs` (correct boundary)
- `LaserAssets` caching prevents per-pulse allocation
- Test fire_rate assertion strengthened (verifies cooldown blocks, then allows fire)

### Project Structure Notes

| File | Action | Purpose |
|------|--------|---------|
| `src/core/weapons.rs` | MODIFY | Add Energy, ActiveWeapon, SpreadProjectile, SpreadFired, fire_weapon (replaces fire_laser), move/tick projectile systems, regenerate_energy |
| `src/core/mod.rs` | MODIFY | Update imports (fire_laser → fire_weapon), register new systems, register SpreadFired message |
| `src/rendering/mod.rs` | MODIFY | Add Energy + ActiveWeapon to player spawn, ProjectileAssets, setup/render projectile systems |
| `src/rendering/vector_art.rs` | MODIFY | Add `generate_projectile_mesh()` |
| `assets/config/weapons.ron` | MODIFY | Add spread and energy config fields |
| `tests/helpers/mod.rs` | MODIFY | Add Energy, ActiveWeapon, spread systems, SpreadFired to test_app and spawn helpers |
| `tests/spread_firing.rs` | CREATE | Integration tests for spread firing and energy |
| `tests/laser_firing.rs` | MODIFY | Update if fire_laser is renamed to fire_weapon |

### References

- [Source: gdd.md#Weapon Systems] — Spread is projectile, medium energy cost, wide arc
- [Source: gdd.md#Weapon Feel Differentiation] — Colorful burst, scatter sound, light screen shake
- [Source: gdd.md#Hit Detection] — Projectiles: physics-based, spawned as entities with velocity
- [Source: gdd.md#Energy system] — Single energy bar, regen over time, spread costs medium
- [Source: game-architecture.md#Object Pooling] — Projectiles pooled (80), but pool.rs not yet implemented
- [Source: game-architecture.md#System Ordering] — CoreSet chain in FixedUpdate
- [Source: game-architecture.md#System Location Mapping] — Weapon System in src/core/weapons.rs
- [Source: game-architecture.md#Configuration Management] — *Config naming, RON assets
- [Source: game-architecture.md#Cross-cutting Concerns] — #[deny(clippy::unwrap_used)], graceful degradation
- [Source: epics.md#Epic 0] — Story 3: "fire spread projectiles that consume energy"
- [Source: 0-2-fire-laser.md#Dev Agent Record] — Bevy 0.18 Message system, marker pattern, LaserAssets caching

## Dev Agent Record

### Agent Model Used

Claude Opus 4.6

### Debug Log References

None

### Completion Notes List

- All 6 tasks completed: Energy system, ActiveWeapon enum, spread firing, projectile movement/lifetime, rendering, tests
- Refactored `fire_laser` → `fire_weapon` with `ActiveWeapon` branching
- 44 total tests passing (21 unit + 5 flight + 8 laser + 10 spread), 0 clippy warnings
- `NeedsProjectileVisual` marker pattern follows established `NeedsLaserVisual` pattern
- `ProjectileAssets` caching follows `LaserAssets` pattern
- Arc calculation distributes projectiles evenly from `-arc/2` to `+arc/2`

### File List

- `src/core/weapons.rs` — Energy, ActiveWeapon, SpreadProjectile, SpreadFired, fire_weapon, move/tick systems, regenerate_energy
- `src/core/mod.rs` — Updated imports and system registration
- `src/rendering/mod.rs` — ProjectileAssets, setup/render projectile systems, Energy+ActiveWeapon on player
- `src/rendering/vector_art.rs` — generate_projectile_mesh()
- `assets/config/weapons.ron` — All spread/energy config fields
- `tests/helpers/mod.rs` — Updated with Energy, ActiveWeapon, spread systems, set_active_weapon_spread helper
- `tests/laser_firing.rs` — Updated fire_laser→fire_weapon references
- `tests/spread_firing.rs` — 10 integration tests for spread firing and energy
