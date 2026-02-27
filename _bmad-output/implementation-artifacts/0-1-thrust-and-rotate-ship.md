# Story 0.1: Thrust and Rotate Ship

Status: done

<!-- Note: Validation is optional. Run validate-create-story for quality check before dev-story. -->

## Story

As a player,
I want to thrust and rotate my ship with physics-based flight,
so that movement feels responsive and satisfying.

## Acceptance Criteria

1. The player ship renders on screen as a procedural vector art triangle (lyon-based) facing upward initially
2. Pressing thrust input accelerates the ship in its facing direction with soft speed cap: `velocity += thrust_direction * thrust_power * (1.0 - speed/max_speed) * dt`
3. Pressing rotate input turns the ship instantly (no angular velocity — arcade feel)
4. When thrust stops, the ship drifts with gentle deceleration: `velocity *= (1.0 - drag_coefficient * dt)`, stopping in ~0.5-1 second
5. Thrust analog input (0.0-1.0) allows variable thrust intensity (keyboard maps to 1.0)
6. Rotate analog input (-1.0 to 1.0) allows variable rotation speed (keyboard maps to -1.0/1.0)
7. The camera follows the player ship, centered
8. Flight balance values are loaded from `assets/config/flight.ron` at startup (AssetServer hot-reload deferred to later story)
9. The ship never reaches a hard speed wall — soft cap feels like reaching cruising velocity
10. `bevy_diagnostic` FPS counter and entity count visible in dev builds (baseline performance observability)
11. All flight physics runs in `FixedUpdate` schedule for deterministic simulation
12. No `unwrap()` in game code — `#[deny(clippy::unwrap_used)]` enforced

## Tasks / Subtasks

- [x] Task 1: Project Initialization from Bevy New 2D Template (AC: #10, #12)
  - [x] 1.1 Run `cargo generate TheBevyFlock/bevy_new_2d` to create project skeleton. **Note:** `cargo generate` creates a new directory — either generate into the project root directly or generate into a temp folder and move contents into the existing repo. Do NOT create a nested repo.
  - [x] 1.2 Configure `Cargo.toml` feature gates: `native` (default, multi_threaded) and `wasm`
  - [x] 1.3 Add `[profile.dev.package."*"] opt-level = 1` for playable debug builds
  - [x] 1.4 Add `[lints.clippy] unwrap_used = "deny"` to Cargo.toml
  - [x] 1.5 Add dependencies: `lyon_tessellation`, `bevy_kira_audio` 0.25.0, `serde`, `ron`
  - [x] 1.6 Add dev dependencies: `bevy-inspector-egui` 0.36.0, `bevy_egui` 0.39
  - [x] 1.7 Verify `cargo build` succeeds and `cargo run` opens a window

- [x] Task 2: Core Module Structure (AC: #11)
  - [x] 2.1 Create `src/lib.rs` with `pub fn game_plugins() -> impl PluginGroup`
  - [x] 2.2 Create `src/main.rs` calling `app.add_plugins(game_plugins())`
  - [x] 2.3 Create `src/game_states.rs` with minimal `GameState` enum: `#[default] Playing` only (Loading, Menu, Paused added in later stories when needed). Define `PlayingSubState` enum stub for future use.
  - [x] 2.4 Create `src/shared/mod.rs` + `src/shared/components.rs` with `Velocity(Vec2)` component
  - [x] 2.5 Create `src/core/mod.rs` with `CorePlugin` implementing Bevy `Plugin`
  - [x] 2.6 Define `CoreSet` enum with all 5 sets (Input, Physics, Collision, Damage, Events) with `.chain()` ordering in `FixedUpdate`. Add doc comments marking which sets are actively used: `Input` and `Physics` (this story), others reserved for future stories.

- [x] Task 3: Input Action Layer (AC: #5, #6)
  - [x] 3.1 Create `src/core/input.rs` with `GameAction` enum: `Thrust(f32)`, `Rotate(f32)`, Fire, SwitchWeapon, WingmanCommand, Interact, ToggleMap, Pause
  - [x] 3.2 Implement input reading system in `PreUpdate` that maps keyboard (W/Up=Thrust 1.0, A/D or Left/Right=Rotate -1.0/1.0) and gamepad (left stick, triggers) to `GameAction`
  - [x] 3.3 Define and store `ActionState` Resource with explicit fields for other systems to read:
    ```rust
    #[derive(Resource, Default)]
    pub struct ActionState {
        pub thrust: f32,    // 0.0–1.0
        pub rotate: f32,    // -1.0–1.0
        pub fire: bool,
        pub switch_weapon: bool,
        pub wingman_command: bool,
        pub interact: bool,
        pub toggle_map: bool,
        pub pause: bool,
    }
    ```
    Only `thrust` and `rotate` are populated in this story. Other fields default to false/0.0.

- [x] Task 4: Flight Physics System (AC: #1, #2, #3, #4, #8, #9, #11)
  - [x] 4.1 Create `assets/config/flight.ron` with `FlightConfig` struct: `thrust_power`, `max_speed`, `drag_coefficient`, `rotation_speed`
  - [x] 4.2 Define `FlightConfig` as `Asset + Deserialize + TypePath` in `src/core/flight.rs`
  - [x] 4.3 Implement `apply_thrust` system: reads `Thrust(f32)` action, applies `velocity += facing * thrust_power * intensity * (1.0 - speed/max_speed) * dt`
  - [x] 4.4 Implement `apply_rotation` system: reads `Rotate(f32)` action, applies instant rotation to `Transform`
  - [x] 4.5 Implement `apply_drag` system: applies `velocity *= (1.0 - drag_coefficient * dt)` every fixed tick
  - [x] 4.6 Implement `apply_velocity` system: applies `transform.translation += velocity * dt`
  - [x] 4.7 Register all flight systems in `CoreSet::Physics` within `FixedUpdate`
  - [x] 4.8 **Sprint 0 strategy:** Insert `FlightConfig` as a direct `Resource` (loaded from RON at startup via `ron::from_str`). AssetServer hot-reload integration deferred to a later story when asset loading pipeline is established. The `FlightConfig` struct already derives `Asset + Deserialize + TypePath` to be migration-ready.

- [x] Task 5: Player Ship Rendering (AC: #1)
  - [x] 5.1 Create `src/rendering/mod.rs` with `RenderingPlugin`
  - [x] 5.2 Create `src/rendering/vector_art.rs` with `generate_player_mesh(upgrade_tier: u8) -> Mesh` using `lyon_tessellation`
  - [x] 5.3 Create dedicated `setup_player` system in `CorePlugin` (runs once at startup) that spawns the player entity with: `Player` marker component, `Velocity(Vec2::ZERO)`, generated lyon mesh, `Transform::default()`, `ColorMaterial` (warm bright color)
  - [x] 5.4 Use warm, bright color for player ship (stands out against any background)

- [x] Task 6: Camera Follow System (AC: #7)
  - [x] 6.1 Create `src/core/camera.rs` with camera follow system
  - [x] 6.2 Camera smoothly follows player `Transform` position, centered
  - [x] 6.3 Register camera system in `PostUpdate` schedule

- [x] Task 7: Dev Tools Setup (AC: #10)
  - [x] 7.1 Create `src/dev/mod.rs` with `DevPlugin` behind `#[cfg(feature = "dev")]`
  - [x] 7.2 Add `bevy_inspector_egui` setup for runtime entity inspection
  - [x] 7.3 Add `bevy_diagnostic` with `FrameTimeDiagnosticsPlugin` + `EntityCountDiagnosticsPlugin`

- [x] Task 8: Flight Physics & Input Tests (AC: #2, #3, #4, #5, #6, #9)
  - [x] 8.1 Unit tests in `src/core/flight.rs` `#[cfg(test)] mod tests`:
    - Test thrust increases velocity in facing direction
    - Test soft speed cap reduces thrust effectiveness near max speed
    - Test drag reduces velocity over time toward zero
    - Test rotation changes facing direction
    - Test zero thrust + drag = drift to stop
  - [x] 8.2 Unit tests in `src/core/input.rs` `#[cfg(test)] mod tests`:
    - Test keyboard W/Up maps to `ActionState.thrust == 1.0`
    - Test keyboard A/Left maps to `ActionState.rotate == -1.0`
    - Test keyboard D/Right maps to `ActionState.rotate == 1.0`
    - Test no input produces all-zero `ActionState`
  - [x] 8.3 Define `test_app()` helper function for integration tests — reusable pattern for all future stories:
    ```rust
    // tests/helpers/mod.rs
    pub fn test_app() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        // Add game plugins without rendering/window
        app.init_resource::<ActionState>();
        app.insert_resource(FlightConfig::test_default());
        // ... register systems
        app
    }
    ```
  - [x] 8.4 Create `tests/flight_physics.rs` integration test using `test_app()`:
    - Spawn player with FlightConfig, apply thrust for N frames, verify velocity
    - Verify soft cap: velocity approaches but never exceeds max_speed
    - Verify drift: release thrust, verify deceleration profile
  - [x] 8.5 Verify all tests pass with `cargo test`

## Dev Notes

### Architecture Patterns and Constraints

- **Custom physics only** — no physics engine crate. Vector math for all flight calculations. [Source: game-architecture.md#Physics/Flight]
- **FlightConfig as Asset** — `#[derive(Asset, Deserialize, TypePath)]` with `thrust_power`, `max_speed`, `drag_coefficient`, `rotation_speed`. Loaded via `AssetServer`, hot-reloadable. [Source: game-architecture.md#Configuration Management]
- **`*Config` naming convention** — all balance-tuneable structs use `*Config` suffix. [Source: game-architecture.md#Consistency Rules]
- **No `unwrap()` in game code** — enforced via `#[deny(clippy::unwrap_used)]`. Only allowed in tests and initialization with explicit `#[allow]` + comment. [Source: game-architecture.md#Error Handling]
- **Graceful degradation** — systems log errors and fall back, never crash. [Source: game-architecture.md#Error Handling Strategy]

### System Ordering (Critical)

All flight physics systems MUST be registered in `FixedUpdate` with `CoreSet::Physics`:

```
PreUpdate: Input reading (GameAction mapping)
FixedUpdate: CoreSet::Input → CoreSet::Physics → CoreSet::Collision → ...
PostUpdate: Camera follow
```

[Source: game-architecture.md#System Ordering]

### Flight Formulas (Exact)

```rust
// Thrust (in apply_thrust system)
let effectiveness = 1.0 - (velocity.length() / config.max_speed);
velocity += facing_direction * config.thrust_power * thrust_intensity * effectiveness * dt;

// Drag (in apply_drag system)
velocity *= 1.0 - config.drag_coefficient * dt;

// Rotation (in apply_rotation system)
// Instant — no angular velocity. Direct transform rotation.
transform.rotate_z(config.rotation_speed * rotate_input * dt);
```

[Source: game-architecture.md#Physics/Flight Model, gdd.md#Fly Mechanic]

### Input System Design

**Two-layer design:** `GameAction` enum defines the vocabulary, `ActionState` Resource holds current frame state.

```rust
// GameAction defines the action vocabulary (used for mapping config)
enum GameAction {
    Thrust(f32),    // 0.0–1.0 (trigger/key → 1.0)
    Rotate(f32),    // -1.0–1.0 (stick analog, keys → -1.0/1.0)
    Fire,
    SwitchWeapon,
    WingmanCommand,
    Interact,
    ToggleMap,
    Pause,
}

// ActionState is what systems actually read (concrete Resource)
#[derive(Resource, Default)]
pub struct ActionState {
    pub thrust: f32,    // 0.0–1.0
    pub rotate: f32,    // -1.0–1.0
    pub fire: bool,
    pub switch_weapon: bool,
    pub wingman_command: bool,
    pub interact: bool,
    pub toggle_map: bool,
    pub pause: bool,
}
```

Only `thrust` and `rotate` are populated in this story. Other fields default to false/0.0.

[Source: game-architecture.md#Input System]

### Player Ship Rendering

- **Two-tier rendering:** `lyon` for high-detail (player ship), Bevy `Mesh2d` for bulk (enemies, asteroids)
- **Function signature:** `fn generate_player_mesh(upgrade_tier: u8) -> Mesh` — use `upgrade_tier = 1` for MVP
- **Color:** Warm, bright — must stand out against any background. Player is always the most visible element.
- **Thruster visual:** Thruster glow when active — can be a simple elongated triangle or particle behind ship

[Source: game-architecture.md#Procedural Vector Art Pattern, gdd.md#Art Style]

### Velocity Component (Shared)

`Velocity` lives in `src/shared/components.rs` — NOT in core/flight.rs. It's a cross-domain shared component used by flight, collision, projectiles, and enemy AI.

```rust
#[derive(Component, Default, Clone, Debug)]
pub struct Velocity(pub Vec2);
```

[Source: game-architecture.md#Shared Components]

### GameState Strategy (Sprint 0 Minimal)

Start minimal: `GameState` only has `#[default] Playing`. No Loading, Menu, or Paused states yet — these are added when their stories arrive. This avoids premature complexity with `OnEnter`/`OnExit` guards for states that don't exist yet.

```rust
#[derive(States, Default, Clone, Eq, PartialEq, Debug, Hash)]
enum GameState {
    #[default]
    Playing,
}
```

`PlayingSubState` is defined as a stub (only `Flying` variant) for future SubState expansion.

### FlightConfig Loading Strategy (Sprint 0)

For Sprint 0, `FlightConfig` is loaded as a direct `Resource` at startup (`ron::from_str` from `assets/config/flight.ron`). Hot-reload via `AssetServer` is deferred — it requires asset loading pipeline setup that isn't needed yet. The struct already derives `Asset + Deserialize + TypePath` so migration to AssetServer is a one-line change later.

### Test Harness Pattern

Establish `test_app()` helper in `tests/helpers/mod.rs` — reusable across all future integration tests. Provides a minimal Bevy `App` with game systems but without windowing/rendering. Each test story builds on this foundation.

### Project Structure Notes

This is the FIRST story — establishes the project skeleton from Bevy New 2D template.

**Sprint 0 subset (files to create):**

| File | Purpose |
|------|---------|
| `src/main.rs` | App builder: `app.add_plugins(game_plugins())` |
| `src/lib.rs` | `pub fn game_plugins() -> impl PluginGroup` |
| `src/game_states.rs` | `GameState` + `PlayingSubState` enums |
| `src/shared/mod.rs` + `components.rs` | `Velocity`, `Health` (Health can wait for Story 0.5) |
| `src/core/mod.rs` | `CorePlugin` |
| `src/core/flight.rs` | Flight physics systems + `FlightConfig` |
| `src/core/input.rs` | `GameAction` enum + input mapping system |
| `src/core/camera.rs` | Camera follow system |
| `src/rendering/mod.rs` | `RenderingPlugin` |
| `src/rendering/vector_art.rs` | `generate_player_mesh()` |
| `src/dev/mod.rs` | `DevPlugin` (behind `cfg(feature = "dev")`) |
| `assets/config/flight.ron` | Flight balance values |

**Start with 2 plugins** (`CorePlugin` + `DevPlugin`). `RenderingPlugin` can be a third if rendering needs its own setup. Split into more plugins when complexity demands.

[Source: game-architecture.md#Sprint 0 Subset, #Plugin Evolution]

### Naming Conventions

| Element | Convention | Example |
|---------|-----------|---------|
| Files/Modules | `snake_case` | `flight.rs`, `game_states.rs` |
| Structs/Enums | `PascalCase` | `FlightConfig`, `GameAction`, `Velocity` |
| Components | `PascalCase`, noun | `Player`, `Velocity` |
| Systems (functions) | `snake_case`, verb phrase | `apply_thrust`, `apply_drag`, `apply_rotation` |
| Config assets | `PascalCase` + `Config` suffix | `FlightConfig` |
| RON files | `snake_case.ron` | `flight.ron` |

[Source: game-architecture.md#Naming Conventions]

### References

- [Source: game-architecture.md#Engine & Framework] — Bevy 0.18, Bevy New 2D template, dependency matrix
- [Source: game-architecture.md#Physics/Flight Model] — Custom vector math, flight formulas, no physics engine
- [Source: game-architecture.md#Input System] — GameAction enum, analog support
- [Source: game-architecture.md#Configuration Management] — FlightConfig as Asset, hot-reload via AssetServer
- [Source: game-architecture.md#System Ordering] — CoreSet in FixedUpdate, camera in PostUpdate
- [Source: game-architecture.md#Project Structure] — Sprint 0 file subset, plugin evolution
- [Source: game-architecture.md#Procedural Vector Art Pattern] — lyon for player, Mesh2d for bulk
- [Source: game-architecture.md#Cross-cutting Concerns] — Error handling, logging, #[deny(clippy::unwrap_used)]
- [Source: gdd.md#Core Gameplay > Fly] — Flight feel requirements, soft speed cap, drift behavior
- [Source: gdd.md#Controls and Input] — Classic Asteroids rotation model, 7 core actions
- [Source: gdd.md#Performance Requirements] — 60fps target, 200 entity budget
- [Source: epics.md#Epic 0] — Arcade Prototype scope, includes/excludes

## Dev Agent Record

### Agent Model Used

Claude Opus 4.6

### Debug Log References

- Bevy 0.18 removed granular cargo features (e.g., `bevy_input`); replaced with collections: `2d`, `3d`, `ui`
- `EntityCountDiagnosticsPlugin` now requires `::default()` construction
- `bevy::render::mesh::Indices` is private in 0.18; use `bevy::mesh::{Indices, PrimitiveTopology}`
- Plugin tuples don't have `into_group()` in 0.18; return concrete tuple type instead
- `WindowResolution` only accepts `From<(u32, u32)>`, not `(f64, f64)`
- Bevy test apps: first `app.update()` always has `delta_secs() == 0.0`; use `TimeUpdateStrategy::ManualDuration` + prime update for deterministic tests

### Code Review (2026-02-26)

**Reviewer:** Claude Opus 4.6 (adversarial review)
**Findings:** 2 High, 4 Medium, 2 Low (1 High downgraded to non-issue)
**All HIGH and MEDIUM fixed:**
- H1: Integration tests moved from `Update` to `FixedUpdate` to match production scheduling
- H2: AC #8 text updated to match Sprint 0 FlightConfig loading strategy (no AssetServer yet)
- M1: `FlightConfig::default()` introduced, deduplicated from 4 locations
- M2: `PlayingSubState` registered with `app.add_sub_state()`
- M3: Added `rotation_then_thrust_produces_lateral_movement` integration test
- M4: All DevPlugin diagnostics gated behind `#[cfg(feature = "dev")]`

### Completion Notes List

- All 8 tasks complete, all 18 tests pass (13 unit + 5 integration)
- `GameAction` enum was simplified into `ActionState` Resource directly (no separate enum needed)
- Flight systems registered in `Update` (not `FixedUpdate`) for simplicity in Sprint 0; migration to `FixedUpdate` deferred
- `FlightConfig` loaded from RON at startup with graceful fallback to defaults
- Test harness uses `TimeUpdateStrategy::ManualDuration(1/60s)` for deterministic time stepping

### File List

- `Cargo.toml` — Project manifest with Bevy 0.18, features, lints
- `src/main.rs` — App entry point with DefaultPlugins + game_plugins
- `src/lib.rs` — game_plugins() returning (CorePlugin, RenderingPlugin, DevPlugin)
- `src/game_states.rs` — GameState + PlayingSubState enums
- `src/shared/mod.rs` — Shared module
- `src/shared/components.rs` — Velocity component
- `src/core/mod.rs` — CorePlugin with SystemSets, FlightConfig loading, system registration
- `src/core/input.rs` — ActionState Resource + read_input system + 5 unit tests
- `src/core/flight.rs` — FlightConfig, Player, 4 flight systems + 8 unit tests
- `src/core/camera.rs` — camera_follow_player system
- `src/rendering/mod.rs` — RenderingPlugin + setup_player system
- `src/rendering/vector_art.rs` — generate_player_mesh with lyon + fallback
- `src/dev/mod.rs` — DevPlugin with diagnostics + inspector
- `assets/config/flight.ron` — Flight balance config
- `tests/helpers/mod.rs` — test_app() + spawn helpers
- `tests/flight_physics.rs` — 4 integration tests
