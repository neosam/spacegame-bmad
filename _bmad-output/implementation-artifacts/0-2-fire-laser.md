# Story 0.2: Fire Laser

Status: done

<!-- Note: Validation is optional. Run validate-create-story for quality check before dev-story. -->

## Story

As a player,
I want to fire a laser (hitscan pulse) in my facing direction,
so that I have a reliable baseline weapon.

## Acceptance Criteria

1. Pressing fire input (Space / Gamepad South / Right Trigger > 0.5) fires a hitscan laser pulse in the ship's facing direction
2. The laser fires as rhythmic individual pulses with a configurable fire rate (e.g., ~4 pulses/sec) — not continuous beam, not held fire
3. A visible laser pulse flash renders along the firing line (thin bright line from ship nose outward to max range) for a brief duration (~0.05-0.1s)
4. The laser pulse originates from the ship's nose position (offset in facing direction from ship center)
5. Laser range is configurable via `WeaponConfig` in `assets/config/weapons.ron`
6. The laser is energy-free — no resource cost, always available
7. Fire rate cooldown prevents firing faster than configured rate, even when holding the fire button
8. `WeaponConfig` loaded from RON at startup with graceful fallback to defaults (same pattern as `FlightConfig`)
9. All weapon systems run in `FixedUpdate` schedule within the existing `CoreSet` ordering
10. No `unwrap()` in game code — `#[deny(clippy::unwrap_used)]` enforced
11. Laser hitscan data (ray origin, direction, range) is stored as a component/event for future collision detection (Story 0.5) — but NO collision detection in this story

## Tasks / Subtasks

- [x] Task 1: Input Mapping for Fire Action (AC: #1)
  - [x]1.1 In `src/core/input.rs` `read_input` system, add keyboard mapping: `KeyCode::Space` → `action_state.fire = true`
  - [x]1.2 Add gamepad mapping: `GamepadButton::South` → `action_state.fire = true`
  - [x]1.3 Add gamepad trigger mapping: right trigger axis > 0.5 → `action_state.fire = true`
  - [x]1.4 Add unit tests for fire input mapping (Space, gamepad button, gamepad trigger)

- [x] Task 2: WeaponConfig Resource and RON File (AC: #5, #8)
  - [x]2.1 Create `assets/config/weapons.ron` with `WeaponConfig` containing: `laser_fire_rate: f32` (pulses/sec), `laser_range: f32`, `laser_damage: f32` (unused until Story 0.5), `laser_pulse_duration: f32` (visual flash time), `laser_width: f32` (visual width)
  - [x]2.2 Define `WeaponConfig` struct in `src/core/weapons.rs` (NEW file) with `#[derive(Resource, Deserialize, Clone, Debug)]` and `Asset + TypePath` for future AssetServer migration
  - [x]2.3 Add `WeaponConfig::default()` with sensible defaults
  - [x]2.4 Load `WeaponConfig` from RON in `CorePlugin::build()` using the same pattern as `FlightConfig` (try file → try deserialize → fallback to defaults with warning)
  - [x]2.5 Add unit test for RON deserialization

- [x] Task 3: Laser Components and Events (AC: #11)
  - [x]3.1 In `src/core/weapons.rs`, define `LaserPulse` component: `origin: Vec2, direction: Vec2, range: f32, timer: f32` (remaining visual lifetime)
  - [x]3.2 Define `FireCooldown` component on the Player entity: `timer: f32` (time until next fire allowed)
  - [x]3.3 Define `LaserFired` event: `origin: Vec2, direction: Vec2, range: f32` — for future collision system subscription

- [x] Task 4: Laser Firing System (AC: #1, #2, #4, #6, #7, #9)
  - [x]4.1 Create `fire_laser` system in `src/core/weapons.rs`:
    - Reads `ActionState.fire`
    - Checks `FireCooldown` on player (skip if cooldown > 0)
    - Queries player `Transform` to get position and facing direction
    - Spawns `LaserPulse` entity at ship nose (player position + facing * nose_offset)
    - Resets `FireCooldown` to `1.0 / config.laser_fire_rate`
    - Emits `LaserFired` event
  - [x]4.2 Create `tick_fire_cooldown` system: decrements `FireCooldown.timer` by `dt`, clamps at 0.0
  - [x]4.3 Create `tick_laser_pulses` system: decrements `LaserPulse.timer` by `dt`, despawns when timer <= 0
  - [x]4.4 Register all weapon systems in `CorePlugin` within `FixedUpdate`. Ordering: `tick_fire_cooldown` in `CoreSet::Input`, `fire_laser` after `CoreSet::Physics`, `tick_laser_pulses` after `fire_laser`
  - [x]4.5 Add `FireCooldown` component to player entity spawn in `src/rendering/mod.rs` `setup_player`

- [x] Task 5: Laser Pulse Rendering (AC: #3, #4)
  - [x]5.1 In `src/rendering/vector_art.rs`, create `generate_laser_mesh(length: f32, width: f32) -> Mesh` — thin bright rectangle using lyon or simple quad vertices
  - [x]5.2 In the `fire_laser` system (or a separate rendering system), spawn the `LaserPulse` entity with: `Mesh2d`, `MeshMaterial2d` (bright cyan/white color), `Transform` positioned at origin, rotated to match direction, scaled to range length
  - [x]5.3 Ensure the laser visual is a short-lived flash (despawned by `tick_laser_pulses`)

- [x] Task 6: Tests (AC: #1, #2, #3, #7, #10, #11)
  - [x]6.1 Unit tests in `src/core/weapons.rs`:
    - Test fire cooldown decrements correctly
    - Test fire cooldown prevents firing when > 0
    - Test `WeaponConfig::default()` has valid values
  - [x]6.2 Unit tests in `src/core/input.rs`:
    - Test Space key maps to `fire = true`
    - Test no fire input → `fire = false`
  - [x]6.3 Integration tests in `tests/laser_firing.rs`:
    - Test laser pulse spawns when fire input is active and cooldown is 0
    - Test laser pulse does NOT spawn when cooldown is active
    - Test laser pulse spawns at correct position (ship nose)
    - Test laser pulse has correct facing direction matching player rotation
    - Test laser pulse despawns after pulse duration expires
    - Test rapid fire respects fire rate (multiple frames, count pulses)
    - Test `LaserFired` event is emitted on fire
  - [x]6.4 Update `tests/helpers/mod.rs` `test_app()` to include:
    - `WeaponConfig` resource
    - Weapon systems registration
    - `LaserFired` event registration
    - Helper: `fn fire_laser_input(app: &mut App)` sets `action_state.fire = true`

## Dev Notes

### Architecture Patterns and Constraints

- **Hitscan, not projectile** — laser is an instant ray, not a moving entity. The `LaserPulse` entity is purely visual (brief flash). The actual hit detection (ray-circle intersection) will be added in Story 0.5 using `src/core/collision.rs`. [Source: gdd.md#Weapon Systems, game-architecture.md#Physics/Flight]
- **Pulsed fire** — rhythmic individual shots (*pew... pew... pew...*), NOT continuous beam or held fire. Each press/hold triggers pulses at the configured fire rate. [Source: gdd.md#Weapon Systems]
- **Energy-free** — laser has zero resource cost. Energy system is for Spread weapon (Story 0.3). [Source: gdd.md#Weapon Systems]
- **Custom physics only** — no physics engine crate. Ray intersection math for collision in `src/core/collision.rs` (Story 0.5). [Source: game-architecture.md#Physics/Flight]
- **No `unwrap()` in game code** — enforced via `#[deny(clippy::unwrap_used)]`. Use `.expect()` in tests only. [Source: game-architecture.md#Error Handling]
- **Graceful degradation** — config loading falls back to defaults on error. [Source: game-architecture.md#Error Handling Strategy]

### System Ordering (Critical)

Weapon systems integrate into the existing `CoreSet` chain in `FixedUpdate`:

```
PreUpdate: read_input (existing)
FixedUpdate:
  CoreSet::Input    → tick_fire_cooldown
  CoreSet::Physics  → apply_thrust, apply_rotation, apply_drag, apply_velocity (existing)
  (after Physics)   → fire_laser, tick_laser_pulses
  CoreSet::Collision → (Story 0.5)
  CoreSet::Damage   → (Story 0.5)
  CoreSet::Events   → (Epic 1)
PostUpdate: camera_follow_player (existing)
```

**Option:** Add `fire_laser` and `tick_laser_pulses` directly after `CoreSet::Physics` without a new SystemSet, OR define a new set if cleaner. Use the simplest approach that preserves ordering.

[Source: game-architecture.md#System Ordering]

### WeaponConfig Pattern (Follow FlightConfig Exactly)

```rust
#[derive(Resource, Deserialize, Clone, Debug)]
// Also derive Asset + TypePath for future AssetServer migration
pub struct WeaponConfig {
    pub laser_fire_rate: f32,      // pulses per second (e.g., 4.0)
    pub laser_range: f32,          // max range in world units (e.g., 500.0)
    pub laser_damage: f32,         // damage per pulse (e.g., 10.0) — unused until Story 0.5
    pub laser_pulse_duration: f32, // visual flash time in seconds (e.g., 0.08)
    pub laser_width: f32,          // visual width (e.g., 2.0)
}
```

Load in `CorePlugin::build()` exactly like `FlightConfig`:
```rust
let config_str = std::fs::read_to_string("assets/config/weapons.ron");
// Try parse → fallback to WeaponConfig::default() with warn!
```

[Source: game-architecture.md#Configuration Management]

### Laser Visual Design

- **Color:** Sharp white/blue flash — bright cyan (`srgb(0.4, 0.9, 1.0)`) or white
- **Shape:** Thin rectangle from ship nose to max range
- **Duration:** Very brief (~0.05-0.1s) — a flash, not a persistent line
- **No screen shake** — that's for impacts (Story 0.6)
- **No sound yet** — audio integration deferred (bevy_kira_audio ready but not wired)

[Source: gdd.md#Weapon Feel Differentiation]

### Ship Nose Offset

The player ship mesh has its nose at `(0, 20)` in local space (see `vector_art.rs`). The laser should originate from approximately this offset in the ship's facing direction:

```rust
let facing = transform.rotation * Vec3::Y;
let nose_offset = facing * 20.0; // Match ship mesh nose position
let laser_origin = transform.translation.truncate() + nose_offset.truncate();
```

### LaserPulse Entity Structure

```rust
// Component
#[derive(Component)]
pub struct LaserPulse {
    pub origin: Vec2,
    pub direction: Vec2,
    pub range: f32,
    pub timer: f32, // remaining visual lifetime
}

// Spawned with:
(
    LaserPulse { origin, direction, range, timer: config.laser_pulse_duration },
    Mesh2d(laser_mesh_handle),
    MeshMaterial2d(laser_material_handle),
    Transform {
        translation: midpoint.extend(0.0), // centered between origin and endpoint
        rotation: Quat::from_rotation_z(angle), // match direction
        scale: Vec3::new(config.laser_width, config.laser_range, 1.0),
    },
)
```

### FireCooldown Component

```rust
#[derive(Component, Default)]
pub struct FireCooldown {
    pub timer: f32, // seconds remaining until next fire allowed
}
```

Add to player entity in `setup_player` alongside existing components.

### What This Story Does NOT Include

- **No collision detection** — ray-circle intersection is Story 0.5
- **No damage dealing** — damage system is Story 0.5
- **No screen shake** — visual feedback is Story 0.6
- **No audio** — sound effects deferred
- **No energy system** — laser is energy-free; energy is for Spread (Story 0.3)
- **No weapon switching** — only one weapon exists; switching is Story 0.4

### Previous Story Intelligence (Story 0.1)

**Patterns established:**
- Plugin tuple: `(CorePlugin, RenderingPlugin, DevPlugin)` in `src/lib.rs`
- Config loading: `std::fs::read_to_string` → `ron::from_str` → fallback with `warn!`
- System registration: `CoreSet` with `.chain()` in `FixedUpdate`
- Player entity: spawned in `setup_player` in `src/rendering/mod.rs` with `Player`, `Velocity`, mesh, material, transform
- Input: `ActionState` resource reset each frame, then populated from keyboard/gamepad

**Bevy 0.18 gotchas from Story 0.1:**
- `bevy::mesh::{Indices, PrimitiveTopology}` (not `bevy::render::mesh::`)
- Plugin tuples don't have `into_group()` — return concrete tuple type
- First `app.update()` always has `delta_secs() == 0.0` — prime with one update in tests
- `TimeUpdateStrategy::ManualDuration` for deterministic tests

**Test harness (`tests/helpers/mod.rs`):**
- `test_app()` → `App` with `MinimalPlugins`, `ActionState`, `FlightConfig`, flight systems, manual time
- `spawn_player(app)` → `Entity` with `Player`, `Velocity`, `Transform`
- `spawn_player_with_velocity(app, velocity)` → `Entity`
- Must add `WeaponConfig` resource and weapon systems to `test_app()` for this story

### Project Structure Notes

| File | Action | Purpose |
|------|--------|---------|
| `src/core/weapons.rs` | CREATE | WeaponConfig, LaserPulse, FireCooldown, fire_laser, tick systems |
| `src/core/mod.rs` | MODIFY | Add `pub mod weapons;`, register weapon systems and events |
| `src/core/input.rs` | MODIFY | Add Space/gamepad fire mapping in `read_input` |
| `src/rendering/mod.rs` | MODIFY | Add `FireCooldown` to player entity spawn |
| `src/rendering/vector_art.rs` | MODIFY | Add `generate_laser_mesh()` function |
| `assets/config/weapons.ron` | CREATE | WeaponConfig balance values |
| `tests/helpers/mod.rs` | MODIFY | Add WeaponConfig, weapon systems, fire helper to test_app |
| `tests/laser_firing.rs` | CREATE | Integration tests for laser firing |

Alignment with project structure: `weapons.rs` goes in `src/core/` per architecture mapping table. [Source: game-architecture.md#System Location Mapping]

### References

- [Source: gdd.md#Weapon Systems] — Laser is hitscan pulse, energy-free, rhythmic fire
- [Source: gdd.md#Weapon Feel Differentiation] — Sharp white/blue flash, precise sound, minimal screen shake
- [Source: gdd.md#Hit Detection] — Hitscan: instant ray, hits first target in line
- [Source: gdd.md#Damage Model (MVP)] — Flat damage per hit (Story 0.5)
- [Source: game-architecture.md#Physics/Flight] — Ray-Circle intersection for hitscan
- [Source: game-architecture.md#System Ordering] — CoreSet chain in FixedUpdate
- [Source: game-architecture.md#Configuration Management] — *Config naming, RON assets
- [Source: game-architecture.md#Cross-cutting Concerns] — #[deny(clippy::unwrap_used)], graceful degradation
- [Source: game-architecture.md#System Location Mapping] — Weapon System in src/core/weapons.rs
- [Source: game-architecture.md#Procedural Vector Art Pattern] — Two-tier rendering, lyon for detail
- [Source: epics.md#Epic 0] — Arcade Prototype scope, story 0.2 definition
- [Source: 0-1-thrust-and-rotate-ship.md#Dev Agent Record] — Bevy 0.18 gotchas, test patterns

## Dev Agent Record

### Agent Model Used

Claude Opus 4.6

### Debug Log References

- Bevy 0.18 renamed `Event`/`EventWriter`/`EventReader` to `Message`/`MessageWriter`/`MessageReader` (in `bevy::ecs::message`)
- Messages must be registered with `app.add_message::<T>()` (not `add_event`)
- `MessageWriter` uses `.write()` instead of `.send()`
- Laser rendering separated from core firing logic to avoid `Assets<Mesh>` dependency in test harness
- `NeedsLaserVisual` marker component pattern: core spawns entity, rendering adds mesh next frame

### Completion Notes List

- All 6 tasks complete, all 32 tests pass (19 unit + 5 flight integration + 8 laser integration)
- Fire input: Space key and Gamepad South button / Right Trigger > 0.5
- WeaponConfig loaded from `assets/config/weapons.ron` with graceful fallback to defaults
- Laser firing separated into core (fire_laser spawns LaserPulse + Transform) and rendering (render_laser_pulses adds Mesh2d + ColorMaterial)
- Fire rate enforced via FireCooldown component on player entity
- LaserFired message emitted on each pulse for future collision system (Story 0.5)
- Clippy clean, no warnings

### Code Review Fixes (2026-02-26)

- H1: Added `generate_laser_mesh()` to `vector_art.rs` per Task 5.1 spec
- H2: Added `laser_fired_message_emitted_on_fire` integration test (AC #11 coverage)
- H3: Rewrote `fire_rate_limits_pulses` test with meaningful assertions (verifies cooldown blocks, then allows fire after expiry)
- M1: Documented gamepad test limitation (MinimalPlugins constraint)
- M2: Added `#[allow(dead_code)]` to `spawn_player_with_velocity` (used by flight_physics.rs, warning from separate test compilation)
- M3: Cached laser mesh/material in `LaserAssets` resource (init once at startup, reuse per pulse)
- L1: Moved `render_laser_pulses` from `core/weapons.rs` to `rendering/mod.rs` (correct architectural boundary)

### File List

- `src/core/weapons.rs` — NEW: WeaponConfig, LaserPulse, FireCooldown, LaserFired, fire_laser, tick_fire_cooldown, tick_laser_pulses + 4 unit tests
- `src/core/mod.rs` — MODIFIED: Added weapons module, WeaponConfig loading, LaserFired message registration, weapon system scheduling
- `src/core/input.rs` — MODIFIED: Added Space/Gamepad fire mapping in read_input + 2 unit tests
- `src/rendering/mod.rs` — MODIFIED: Added FireCooldown to player spawn, LaserAssets resource, setup_laser_assets startup, render_laser_pulses system
- `src/rendering/vector_art.rs` — MODIFIED: Added generate_laser_mesh() function
- `assets/config/weapons.ron` — NEW: WeaponConfig balance values
- `tests/helpers/mod.rs` — MODIFIED: Added WeaponConfig, weapon systems, FireCooldown to test_app and spawn helpers
- `tests/laser_firing.rs` — NEW: 8 integration tests for laser firing (incl. LaserFired message test)
