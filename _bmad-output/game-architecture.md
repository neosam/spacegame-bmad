---
title: 'Game Architecture'
project: 'spacegame-bmad'
date: '2026-02-25'
author: 'Simon'
version: '1.0'
stepsCompleted: [1, 2, 3, 4, 5, 6, 7, 8, 9]
status: 'complete'
engine: 'Bevy 0.18'
platform: 'PC (Linux/Win/Mac), WASM, Steam Deck'

# Source Documents
gdd: '_bmad-output/gdd.md'
epics: '_bmad-output/epics.md'
brief: '_bmad-output/game-brief.md'
---

# Game Architecture

## Document Status

**Status: COMPLETE** — All 9 steps finished. Ready for implementation.

---

## Executive Summary

**Void Drifter** architecture is designed for Bevy 0.18 (Rust) targeting PC, WASM, and Steam Deck. Solo developer project with all visuals code-generated — zero external art assets.

**Key Architectural Decisions:**

- **Custom physics** (vector math) over bevy_rapier — full control over arcade flight feel
- **Delta-save on seed basis** — seed reproduces base world, only deviations persisted as RON
- **Data-driven AI** — FSM transitions as RON configs, pure functions for testability
- **Hybrid UI** — Bevy UI native in-game, `bevy_egui` for dev tools only
- **Graceful degradation** — systems log errors and fall back to safe defaults, never crash

**Project Structure:** Domain-driven organization with 7 target plugins (starting as 2 in Sprint 0), 5 novel patterns, 4 standard patterns, and 12 consistency rules.

**Novel Patterns:** Delta-Save, Gravity Well Tutorial, Procedural Vector Art, Event-Severity Logbook, Wormhole Scene Transition.

**Ready for:** Sprint Planning and Epic implementation.

---

## Development Environment

### Prerequisites

| Tool | Version | Purpose |
|------|---------|---------|
| Rust | Stable (MSRV per Bevy 0.18) | Language |
| `cargo` | Bundled with Rust | Build system |
| `trunk` | Latest | WASM build bundler |
| `wasm32-unknown-unknown` target | Via `rustup target add` | WASM compilation |

### AI Tooling (MCP Servers)

The following MCP servers were accepted during architecture for AI-assisted development:

| MCP Server | Purpose |
|------------|---------|
| Bevy Debugger MCP | Direct Bevy scene inspection from AI assistant |
| Context7 | Up-to-date Bevy API documentation context |

### Setup Commands

```bash
# Install Rust (if not present)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Add WASM target
rustup target add wasm32-unknown-unknown

# Install trunk for WASM builds
cargo install trunk

# Create project from Bevy New 2D template
cargo generate TheBevyFlock/bevy_new_2d

# Verify native build
cargo run

# Verify WASM build
trunk serve
```

### First Steps

1. Initialize project from Bevy New 2D template
2. Replace sprite pipeline with `lyon`-based vector rendering
3. Configure feature gates (`native` / `wasm`) in `Cargo.toml`
4. Add `[profile.dev.package."*"] opt-level = 1`
5. Set up `bevy_inspector_egui` as dev dependency
6. Configure MCP servers (Bevy Debugger + Context7) per AI tooling instructions
7. Implement flight physics proof-of-concept (Sprint 0 goal)

---

## Project Context

### Game Overview

**Void Drifter** — 2D Arcade Space Shooter with open-world sandbox in an infinite, seed-generated universe. Built with Bevy (Rust) by a solo developer. All visuals code-generated, no external art assets.

### Technical Scope

**Platform:** PC (Linux/Win/Mac), WASM, Steam Deck
**Genre:** Arcade Space Shooter / Open-World Sandbox
**Project Level:** High complexity — 4 novel concepts requiring custom patterns

### Architecture Domains

| Domain | Systems | Overall Complexity |
|--------|---------|-------------------|
| **Core Engine** | Flight Physics, Weapon System (Hitscan + Projectile), Camera System | Medium |
| **World** | Chunk Generation, Seed-based Procedural Generation, Noise Layers (Biome + Faction + Boss), Biome Types | High |
| **Gameplay** | Stations & Economy, Upgrade System (5 Tiers), Tutorial Zone (Gravity Well), Wormhole Mini-Levels, Boss Encounters | Medium-High |
| **Social** | Companion Core (Follow + Commands), Companion Personality (Opinions + Barks), Enemy AI (5 Types × 4 Factions), Neutral Entities | High |
| **Infrastructure** | Delta-Save System, Event-Observer/Logbook, Audio (MVP: 2 Tracks + Crossfade), UI/HUD Framework, Procedural Vector Art Pipeline (`lyon` + `bevy_hanabi`), Minimap/World Map | High |

### Entity Budget

Design target: 200 simultaneously active entities on screen.

| Category | Budget | Notes |
|----------|--------|-------|
| Player | 1 | Always present |
| Companions | 3-5 | Follow player |
| Enemies | 15-25 | Faction patrols + random encounters |
| Projectiles | 40-60 | Player + enemy + companion fire |
| Asteroids | 60-80 | Bulk environment objects |
| Pickups/Loot | 10-20 | Dropped resources |
| Neutral Entities | 3-5 | Traders, civilians |
| **Total** | **~200** | Hard cap for performance budgeting |

### Technical Requirements

- 60fps with 200 entities on Tier 1 reference hardware
- Memory under 500MB after 2 hours of continuous exploration
- WASM build under 50MB (stretch: under 30MB)
- Warm start under 5 seconds, seamless chunk generation
- Delta-save files under 1MB for 10+ hours of play

### Platform Constraints

| Tier | Platform | Constraints |
|------|----------|-------------|
| Tier 1 | Linux Native | Primary dev/test — no special constraints |
| Tier 2 | Windows, macOS | Full feature parity |
| Tier 3 | WASM/WebGL2 | Single-threaded, ~2-4GB memory, Web Audio restrictions, feature subset |
| Stretch | Steam Deck | 720p UI scaling, controller-only |

### Networking

**Singleplayer-first.** No multiplayer preparation in architecture. If multiplayer is ever added, it will require refactoring — and that's acceptable. Half-hearted multiplayer preparation costs hidden complexity across every system.

### Complexity Drivers

**High Complexity:**
1. **Chunk-based infinite world** — Load/unload, seed determinism, delta-saves per chunk
2. **Multi-layered noise generation** — Biomes + factions + bosses as overlapping noise layers
3. **Companion Personality System** — Opinions + barks + personality-influenced AI (Epic 6b)
4. **Enemy AI with faction behavior** — 5 types × 4 faction behaviors
5. **Delta-Save System** — Highest risk system: every other system writes to it, must work from Epic 1

**Novel Concepts (no standard patterns):**
1. **Gravity Well Tutorial** — Physics-based boundary with constraint-validated procedural generation
2. **Delta-Save on Seed Basis** — Seed reproduces base world, only deviations saved
3. **Procedural Vector Art** — All visuals generated at runtime via code (no sprite import)
4. **Event-Severity Logbook** — Automatic storytelling through event-observer with tier filtering

### Technical Risks

| Risk | Severity | Notes |
|------|----------|-------|
| Bevy breaking changes | Medium | Pin version during sprints, upgrade between epics |
| WASM single-thread chunk generation | Medium | Smaller chunk radius, per-frame generation budget |
| `bevy_hanabi` version compatibility | Medium | Fallback: custom particle system |
| Tutorial zone constraint validation | Medium | 100-seed automated test |
| **System ordering / race conditions** | High | Bevy ECS event processing order is critical |
| **Delta-Save corruption** | High | Single bug affects all persistence. Schema versioning from day one |

### Key Architectural Decision: UI Framework

The architecture must decide early: **Bevy UI native vs `bevy_egui` vs other community crate.** This affects every epic with a UI component.

---

## Engine & Framework

### Engine Selection

| Property | Value |
|----------|-------|
| **Engine** | Bevy |
| **Version** | 0.18 (Released January 13, 2026) |
| **Language** | Rust |
| **Renderer** | wgpu (WebGPU/WebGL2) |
| **Architecture** | Entity Component System (ECS) |
| **Starter Template** | Bevy New 2D (TheBevyFlock) |

### Engine-Provided Architecture

Bevy 0.18 provides these architectural foundations out of the box:

| System | Bevy Provides | Custom Needed |
|--------|--------------|---------------|
| **ECS** | Full ECS with `World`, `Query`, `Commands`, `SystemParam` | Domain-specific components and systems |
| **Scheduling** | `Schedule`, `SystemSet`, run conditions, ordering | Game-specific system ordering |
| **State Machine** | `States` trait, `OnEnter`/`OnExit`/`OnTransition` | Game states (Menu, Playing, Paused, etc.) |
| **Events** | `Event`, `EventReader`, `EventWriter` | Game-specific event types |
| **Assets** | `AssetServer`, `Handle<T>`, hot reload | Custom asset types for procedural content |
| **Input** | `ButtonInput<KeyCode>`, `ButtonInput<GamepadButton>`, axis | Action mapping layer |
| **Rendering** | 2D Camera, Sprite, Mesh2d, Material2d | Vector art pipeline via `lyon` |
| **Audio** | `AudioPlayer`, `PlaybackSettings` | Music state machine (post-MVP) |
| **Windowing** | Multi-platform window management | None |
| **Transform** | `Transform`, `GlobalTransform`, hierarchy | None |

### Dependency Matrix

**MVP Runtime Dependencies:**

| Crate | Purpose | Epic |
|-------|---------|------|
| `bevy` 0.18 | Game engine (with custom feature flags) | All |
| `lyon_tessellation` | Vector path tessellation for procedural art | Epic 0+ |
| `noise` | Seed-based procedural generation (noise layers) | Epic 2+ |
| `bevy_kira_audio` 0.25.0 | Audio with crossfade, tween effects, Web Audio | Epic 0+ |
| `serde` + `ron` | Serialization for delta-save system | Epic 1+ |

**Post-MVP Runtime Dependencies (added when needed):**

| Crate | Purpose | Epic |
|-------|---------|------|
| `bevy_hanabi` | GPU particle effects | Epic 5+ |
| `tracing_wasm` | WASM console logging via `tracing` | WASM builds |

**Fallback:** If `bevy_hanabi` is incompatible with Bevy 0.18, implement a custom particle system using Bevy's `Mesh2d` with instancing.

**Dev-Only Dependencies:**

| Crate | Purpose |
|-------|---------|
| `bevy-inspector-egui` 0.36.0 | Runtime entity/component inspection — critical for debugging procedural generation |
| `bevy_egui` 0.39 | Egui integration for dev tools UI (behind `#[cfg(feature = "dev")]`) |
| `proptest` | Property-based testing for delta-save roundtrip verification |

### Cargo Configuration

**Feature Gates — two profiles for platform targeting:**

```toml
[features]
default = ["native"]
native = ["bevy/multi_threaded", "bevy/bevy_audio"]
wasm = ["bevy/bevy_audio"]  # No multi_threaded on WASM
```

**Dev build optimization (required for playable debug builds):**

```toml
[profile.dev.package."*"]
opt-level = 1
```

### WASM Build Pipeline

| Component | Tool | Notes |
|-----------|------|-------|
| Build target | `wasm32-unknown-unknown` | Standard Rust WASM target |
| Bundler | `trunk` or `wasm-pack` | Decision deferred to Sprint 0 |
| Size budget | 50MB (stretch: 30MB) | Monitor with `wasm-opt` and `twiggy` |
| Audio | Web Audio API | Bevy handles via `bevy_audio` feature |

### Sprint 0 Setup Tasks

1. Initialize project from Bevy New 2D template
2. Replace sprite pipeline with `lyon`-based vector rendering
3. Configure feature gates (`native` / `wasm`)
4. Add `[profile.dev.package."*"] opt-level = 1`
5. Verify WASM build compiles and runs
6. Add `bevy_inspector_egui` as dev dependency

---

## Architectural Decisions

### Decision Summary

| # | Category | Decision | Version | Rationale |
|---|----------|----------|---------|-----------|
| 1 | UI Framework | Hybrid: Bevy UI (in-game) + `bevy_egui` (dev tools) | Bevy UI nativ / `bevy_egui` 0.39 | One UI system in-game, egui only for debug/inspector |
| 2 | Save/Persistence | RON files with schema versioning, split into player + world | `serde` + `ron` | Readability critical for highest-risk system, <1MB no perf factor |
| 3 | AI System | FSM (MVP) + Utility AI (Companion Personality only) | Custom | FSM manageable for 5 enemy types × 4 factions, Utility AI only where emergence desired |
| 4 | Physics/Flight | Custom vector math | Custom | Arcade physics needs no physics engine, full control over game feel |
| 5 | World Streaming | Hybrid: Grid-Chunks + Player-Radius-Loading | Custom | Grid for seed determinism + delta-save per chunk, radius for seamless loading |
| 6 | Audio | `bevy_kira_audio` | 0.25.0 | Crossfade between Exploration/Combat is MVP requirement |
| 7 | Input | Custom Action Layer | Custom | ~8 actions, enum + mapping struct suffices |
| 8 | Noise Library | `noise` | crates.io latest | Established, good docs, speed not a bottleneck |
| 9 | Error Handling | Graceful Degradation | — | Systems log errors and fall back to safe defaults instead of crashing |

### UI Framework

**Approach:** Hybrid — Bevy UI native for all in-game surfaces, `bevy_egui` exclusively for dev tools.

- **In-Game (Bevy UI):** HUD, menus, logbook, map, crafting, station interface
- **Dev-Only (`bevy_egui`):** Entity inspector, noise-layer visualizer, debug overlays
- Compiled behind `#[cfg(feature = "dev")]` — never included in release builds

### Data Persistence

**Save System:** Delta-Save with RON serialization, split file architecture.

**File Structure:**

| File | Contents | Growth Pattern |
|------|----------|---------------|
| `player.ron` | Player state, ship config, companions, economy, tutorial flags | Fixed size (~small) |
| `world.ron` | Chunk deltas, station states, cleared mini-levels, logbook events | Grows with playtime |

- **Schema Versioning:** Version header in every save file from day one
- **Migration:** Versioned migration functions for schema changes
- **Golden Save Fixtures:** Test fixtures `test_save_v{N}.ron` per schema version in repo; migration tests run automatically
- **Platform Storage:**

| Platform | Storage | Notes |
|----------|---------|-------|
| Native (Linux/Win/Mac) | Filesystem (platform user data dir) | Standard save location per OS |
| WASM | LocalStorage or IndexedDB | LocalStorage limit 5-10MB (browser-dependent), sufficient for <1MB saves |

- **Rationale:** Split enables loading `player.ron` for save-slot preview without deserializing the entire world. Readability over performance — at <1MB, RON parsing is negligible.

### AI System

**Approach:** Two-tier — FSM for combat AI, Utility AI for companion personality.

**Enemy AI (MVP):** Finite State Machines per enemy type.

- States: `Idle → Patrol → Chase → Attack → Flee`
- Faction modifiers: FSM parameters vary per faction (aggression, flee threshold, reinforcement calls)
- **Data-Driven:** FSM transitions defined as RON files per enemy type — balance tuning without recompilation
- **Pure Functions:** FSM logic as `fn next_state(current: State, context: &AiContext) -> State` — testable without ECS world setup

**Companion Core (Epic 6a):** FSM via wingman commands (Follow → Attack → Defend → Retreat)

**Companion Personality (Epic 6b):** Utility AI — opinion scores + context influence bark selection and behavior priorities.

### Physics / Flight Model

**Approach:** Custom vector math, no physics engine.

| Interaction | Method |
|-------------|--------|
| Flight (thrust, drag, rotation) | Vector math with soft speed cap |
| Hitscan Laser | Ray-Circle intersection |
| Projectiles | Circle-Circle collision |
| Environment (asteroids, stations) | Circle-Circle collision |
| Gravity Well | Directional force: `pull_force = max(0, (distance - safe_radius) * pull_strength)` |

- **Flight Formula:** `velocity += thrust_direction * thrust_power * (1.0 - speed/max_speed) * dt`
- **Drag:** `velocity *= (1.0 - drag_coefficient * dt)` for gentle drift deceleration
- **Rotation:** Instant (arcade feel), no angular velocity

### World Streaming

**Approach:** Grid-chunks with player-centered load radius.

- **Chunk Grid:** Fixed size, deterministic position from seed
- **Load Radius:** Chunks within radius around player are loaded, outside unloaded
- **Delta-Save:** Per chunk — only modified chunks are persisted
- **WASM:** Smaller load radius, per-frame generation budget
- **Chunk Size:** To be determined in Sprint 0 via playtesting. Constraint: chunk must be generatable in <16ms on WASM (single-threaded budget).

### Audio Architecture

**Approach:** `bevy_kira_audio` 0.25.0

- **MVP:** 2 audio channels (Music + SFX), crossfade between Exploration/Combat
- **Trigger:** `enemies_in_range > 0` switches to combat track
- **WASM:** Web Audio API (supported by Kira)
- **Autoplay Policy:** `AudioState::WaitingForInteraction` state — audio system gracefully handles browser blocking AudioContext until first user interaction
- **Post-MVP:** State machine for 7 music states

### Input System

**Approach:** Custom Action Layer with analog support.

```rust
enum GameAction {
    Thrust(f32),          // 0.0–1.0 (trigger/key → 1.0)
    Rotate(f32),          // -1.0–1.0 (stick analog, keys → -1.0/1.0)
    Fire,
    SwitchWeapon,
    WingmanCommand,
    Interact,
    ToggleMap,
    Pause,
}
```

- **Mapping:** `HashMap<GameAction, Vec<InputBinding>>` with keyboard + gamepad support
- **Rebinding:** Post-MVP (settings menu)

### Noise Library

**Library:** `noise` (crates.io)

- **Usage:** Biome layer, faction layer, boss layer as overlapping noise maps
- **Seed Pattern:** `noise(chunk_x, chunk_y, seed + LAYER_OFFSET)` per layer

### Error Handling Strategy

**Approach:** Graceful Degradation — systems log errors and fall back to safe defaults instead of crashing.

| System | Failure | Fallback |
|--------|---------|----------|
| Save (load) | Corrupt `world.ron` | Discard corrupt chunk deltas, regenerate from seed. Log warning. Never lose `player.ron`. |
| Save (write) | Write failure | Retry once, then warn player. Keep last known good save. |
| Chunk Generation | Generation fails | Skip chunk, mark as empty, log error. Player sees empty space instead of crash. |
| Audio | No AudioContext (WASM) | Silent mode until user interaction. No crash. |
| `bevy_hanabi` | Plugin incompatible | Fallback to `Mesh2d` particle system. |

### Architecture Decision Records

**ADR-001: Singleplayer-First**
No multiplayer preparation in architecture. Refactoring acceptable if multiplayer ever added. Half-hearted multiplayer preparation costs hidden complexity across every system.

**ADR-002: RON over Binary for Saves**
Readability beats performance for the highest-risk system. At <1MB delta-saves, serialization speed is negligible. Debugging save corruption requires human-readable files.

**ADR-003: Custom Physics over bevy_rapier**
Arcade flight model is simple vector math. A full physics engine adds dependency risk, WASM compatibility concerns, and overhead for 200 entities — all for features we don't need (joints, friction, rigid body simulation).

**ADR-004: Data-Driven AI**
FSM transitions stored as RON files, not hardcoded Rust. Enables balance iteration without recompilation. Pure transition functions enable unit testing without ECS setup.

---

## Cross-cutting Concerns

These patterns apply to ALL systems and must be followed by every implementation.

### Error Handling

**Strategy:** Graceful Degradation with Rust's `Result<T, E>`.

**Compiler Enforcement:**

```toml
# Cargo.toml / clippy.toml
[lints.clippy]
unwrap_used = "deny"
```

No `unwrap()` in game code — only in tests and initialization. Exceptions via `#[allow(clippy::unwrap_used)]` with comment explaining why.

**Error Levels:**

| Level | Behavior | Example |
|-------|----------|---------|
| **Fatal** | Log + clean shutdown | Bevy plugin init failed |
| **Recoverable** | Log + fallback + keep playing | Corrupt chunk delta → regenerate from seed |
| **Warning** | Log + ignore | AudioContext not yet available (WASM) |

**Pattern:**

```rust
fn load_chunk_delta(path: &Path) -> Result<ChunkDelta, SaveError> {
    match ron::from_str(&std::fs::read_to_string(path)?) {
        Ok(delta) => Ok(delta),
        Err(e) => {
            warn!("Corrupt chunk delta at {:?}: {}. Regenerating from seed.", path, e);
            Err(SaveError::CorruptDelta(path.to_owned()))
        }
    }
}
```

### Logging

**Framework:** Bevy's integrated `tracing`. WASM via `tracing_wasm`.

**Log Levels:**

| Level | Usage | Example |
|-------|-------|---------|
| `error!` | System cannot fulfill its task | Save write failed |
| `warn!` | Unexpected but handled state | Chunk delta corrupt, regenerated |
| `info!` | Normal milestones | "Chunk (3,7) generated", "Game saved" |
| `debug!` | Development diagnostics | FSM transitions, entity spawn details |
| `trace!` | Extremely verbose, activate selectively | Per-frame physics values |

**Rules:**
- No logging in hot paths (physics loop, collision check) — only behind `trace!`
- Structured logging with Bevy's `tracing` spans for system attribution
- WASM: Console output via `tracing_wasm`

### Configuration Management

**Three tiers with `*Config` naming convention:**

| Tier | Format | Hot Reloadable? | Location | Example |
|------|--------|----------------|----------|---------|
| **Constants** | Rust `const` | No | Source code | `MAX_ENTITIES: usize = 200` |
| **Balance Values** | RON assets (`*Config`) | Yes (via `AssetServer`) | `assets/config/` | `WeaponConfig`, `FlightConfig`, `AiConfig`, `SpawnConfig`, `TutorialConfig` |
| **Player Settings** | RON file | Yes | Platform user data dir / WASM LocalStorage | Audio volume, key bindings, visual preferences |
| **Platform Config** | Feature gates + `cfg` | No | `Cargo.toml` / source code | WASM chunk radius, thread count |

**Naming Convention:** All balance-tuneable structs use the `*Config` suffix and implement `Asset + Deserialize + TypePath`. This signals to every developer: "This struct is tuneable via Inspector at runtime."

```rust
#[derive(Asset, Deserialize, TypePath)]
struct FlightConfig {
    thrust_power: f32,
    max_speed: f32,
    drag_coefficient: f32,
    rotation_speed: f32,
}
```

### Event System

**Pattern:** Bevy's native typed Event system.

**Two event categories:**

| Category | Purpose | Lifetime | Example |
|----------|---------|----------|---------|
| **System Events** | Technical inter-system communication | Single frame | `ChunkLoaded`, `EntitySpawned`, `SaveRequested` |
| **Game Events** | Gameplay-relevant, flow into Event-Observer/Logbook | Persisted | `EnemyDestroyed`, `CompanionRecruited`, `BossEncountered` |

**Game Events carry severity for the Logbook:**

```rust
#[derive(Event)]
struct GameEvent {
    kind: GameEventKind,
    severity: EventSeverity,  // Tier1 (critical), Tier2 (notable), Tier3 (minor)
    position: Vec2,
    game_time: f64,           // From Time<Virtual> — pauses when game pauses
}
```

**Rules:**
- All gameplay-relevant state changes MUST emit a `GameEvent`
- Event-Observer (Epic 1) consumes all `GameEvent`s and writes to logbook
- System Events are ephemeral (one frame), Game Events are persisted

**Test Helper Pattern:**

```rust
#[cfg(test)]
fn assert_event_emitted<T: Event>(world: &World) {
    let events = world.resource::<Events<T>>();
    assert!(!events.is_empty(), "Expected event {} to be emitted", std::any::type_name::<T>());
}
```

### Object Pooling

**Pattern:** Entity recycling for high-frequency spawn/despawn entities.

**Pooled entity types:**

| Type | Reason | Pool Size |
|------|--------|-----------|
| Projectiles | 40-60 active, constant spawn/despawn at 60fps | Pre-allocate 80 |
| Pickups/Loot | 10-20 active, spawn on enemy death | Pre-allocate 30 |

**Mechanism:**
- Despawn = deactivate (`Visibility::Hidden` + remove from collision) — entity stays in ECS
- Spawn = reactivate (reset components, `Visibility::Inherited`)
- Pool manager tracks available/active entities per type
- Avoids per-frame ECS archetype changes from real spawn/despawn

**Not pooled:** Player, companions, enemies, asteroids, stations — these have longer lifecycles and lower churn.

### System Ordering

**Bevy Schedule Assignment (mandatory for all systems):**

| Schedule | Systems | Rationale |
|----------|---------|-----------|
| `PreUpdate` | Input reading, action mapping | Capture input before anything processes it |
| `FixedUpdate` | Flight physics, collision detection, AI FSM, gravity well, object pool cleanup | Deterministic, framerate-independent simulation |
| `Update` | Event processing, audio crossfade, UI updates, rendering, chunk load/unload triggers | Frame-dependent, visual updates |
| `PostUpdate` | Camera follow, chunk load/unload execution | After all transforms settled |

**System Ordering within `FixedUpdate` (critical — prevents race conditions):**

```
Input → Physics (thrust/drag/rotation) → Collision Detection → Damage Application →
  Death/Despawn → Event Emission → FSM Update → Gravity Well
```

**Enforcement:** Define `SystemSet` enums per domain. Systems declare their set; ordering is defined once in plugin registration.

```rust
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
enum CoreSet {
    Input,
    Physics,
    Collision,
    Damage,
    Events,
}

// In plugin registration:
app.configure_sets(FixedUpdate, (
    CoreSet::Input,
    CoreSet::Physics,
    CoreSet::Collision,
    CoreSet::Damage,
    CoreSet::Events,
).chain());
```

### Debug & Development Tools

**All behind `#[cfg(feature = "dev")]` — zero cost in release builds.**

| Tool | Purpose | Activation |
|------|---------|------------|
| `bevy-inspector-egui` | Entity/component inspection, balance tuning | Automatic in dev build |
| `bevy_diagnostic` | FPS counter, entity count, memory | `FrameTimeDiagnosticsPlugin` + `EntityCountDiagnosticsPlugin` |
| Noise Visualizer | Overlay noise layers on world map | Configurable keybind |
| Collision Debugger | Circle/ray gizmos for all colliders | Configurable keybind |
| AI State Overlay | FSM state displayed above enemy entities | Configurable keybind |
| Chunk Grid Overlay | Chunk boundaries and load status | Configurable keybind |

**Debug keybinds** defined in `assets/config/debug_config.ron` — not hardcoded. Enables alternative bindings for controller-only testing (Steam Deck).

```rust
#[derive(Asset, Deserialize, TypePath)]
struct DebugConfig {
    noise_visualizer: KeyCode,    // default: F1
    collision_debugger: KeyCode,  // default: F2
    ai_state_overlay: KeyCode,    // default: F3
    chunk_grid_overlay: KeyCode,  // default: F4
}
```

---

## Project Structure

### Organization Pattern

**Pattern:** Domain-Driven — each architecture domain maps to a Bevy Plugin module.

**Evolution Strategy:** Start with 2 plugins in Sprint 0 (`GamePlugin` + `DevPlugin`). Refactor into domain sub-plugins as complexity grows. The target structure below documents the final architecture; files are created on-demand as epics require them.

### Target Directory Structure

```
void_drifter/
├── Cargo.toml
├── assets/
│   ├── config/                        # Balance RON files (*Config assets)
│   │   ├── flight.ron
│   │   ├── weapons.ron
│   │   ├── ai/
│   │   │   ├── scout_drone.ron
│   │   │   ├── fighter.ron
│   │   │   ├── heavy_cruiser.ron
│   │   │   ├── sniper.ron
│   │   │   └── swarm.ron
│   │   ├── spawn.ron
│   │   ├── tutorial.ron
│   │   └── debug_config.ron           # Debug keybinds (dev only)
│   ├── audio/
│   │   ├── music/
│   │   │   ├── exploration.ogg
│   │   │   └── combat.ogg
│   │   └── sfx/
│   │       └── *.ogg
│   └── fonts/
├── src/
│   ├── main.rs                        # App builder: app.add_plugins(game_plugins())
│   ├── lib.rs                         # pub fn game_plugins() -> PluginGroup for testability
│   ├── game_states.rs                 # GameState + PlayingSubState (SubStates)
│   ├── shared/                        # Shared components used across domains
│   │   ├── mod.rs
│   │   ├── components.rs              # Health, Velocity, Faction, etc.
│   │   └── events.rs                  # GameEvent, EventSeverity, GameEventKind
│   ├── core/                          # Core Engine Domain
│   │   ├── mod.rs                     # CorePlugin
│   │   ├── flight.rs                  # Flight physics (thrust, drag, rotation)
│   │   ├── weapons.rs                 # Weapon system (laser hitscan, spread projectile)
│   │   ├── collision.rs               # Pure math: circle-circle, ray-circle intersection
│   │   ├── camera.rs                  # 2D camera follow
│   │   └── input.rs                   # Custom action layer (GameAction enum)
│   ├── world/                         # World Domain
│   │   ├── mod.rs                     # WorldPlugin
│   │   ├── chunk.rs                   # Chunk grid, load/unload logic
│   │   ├── generation.rs              # Seed-based procedural generation
│   │   ├── noise_layers.rs            # Biome, faction, boss noise overlays
│   │   ├── biomes.rs                  # Biome type definitions
│   │   ├── tutorial_zone.rs           # Gravity well, ability unlocks, generator
│   │   └── wormhole.rs               # Mini-level loading, scene isolation
│   ├── gameplay/                      # Gameplay Domain
│   │   ├── mod.rs                     # GameplayPlugin
│   │   ├── stations.rs               # Docking, trading, crafting
│   │   ├── economy.rs                # Credits, materials, tiers
│   │   ├── upgrades.rs               # 5-tier upgrade system
│   │   ├── boss.rs                   # Boss encounters
│   │   └── pickups.rs                # Loot drops, resource collection
│   ├── social/                        # Social Domain
│   │   ├── mod.rs                     # SocialPlugin
│   │   ├── companion_core.rs          # Follow, commands (Attack/Defend/Retreat)
│   │   ├── companion_personality.rs   # Utility AI, opinions, barks
│   │   ├── enemy_ai.rs               # FSM per type, faction modifiers
│   │   ├── neutral.rs                # Traders, civilians, explorers
│   │   └── faction.rs                # Faction affiliation, reputation
│   ├── infrastructure/                # Infrastructure Domain
│   │   ├── mod.rs                     # InfrastructurePlugin
│   │   ├── save/
│   │   │   ├── mod.rs                 # SavePlugin
│   │   │   ├── player_save.rs         # player.ron serialization
│   │   │   ├── world_save.rs          # world.ron serialization
│   │   │   ├── schema.rs              # Version header, migration registry
│   │   │   └── migration.rs           # v1→v2, v2→v3, etc.
│   │   ├── events.rs                  # Event-Observer system, logbook writer
│   │   ├── logbook.rs                 # Logbook storage, tier filtering, query
│   │   ├── audio.rs                   # bevy_kira_audio setup, crossfade logic
│   │   └── pool.rs                    # Object pooling (projectiles, pickups)
│   ├── rendering/                     # Procedural Art Pipeline
│   │   ├── mod.rs                     # RenderingPlugin
│   │   ├── vector_art.rs              # lyon tessellation for player/stations/bosses
│   │   ├── mesh_art.rs               # Bevy Mesh2d for enemies/asteroids/projectiles
│   │   ├── effects.rs                # Particles (bevy_hanabi or fallback)
│   │   └── minimap.rs                # Minimap/world map rendering
│   ├── ui/                            # UI (Bevy UI native)
│   │   ├── mod.rs                     # UiPlugin
│   │   ├── hud.rs                     # Health, energy, minimap blips
│   │   ├── menus.rs                   # Main menu, pause, settings
│   │   ├── logbook_ui.rs              # Logbook interface
│   │   ├── station_ui.rs              # Trading, crafting screens
│   │   └── map_ui.rs                  # World map overlay
│   └── dev/                           # Dev tools (behind cfg(feature = "dev"))
│       ├── mod.rs                     # DevPlugin
│       ├── inspector.rs               # bevy_inspector_egui setup
│       ├── noise_viz.rs               # Noise layer visualizer
│       ├── collision_debug.rs         # Collision gizmos
│       ├── ai_debug.rs               # FSM state overlay
│       └── chunk_debug.rs            # Chunk grid overlay
├── tests/                             # Integration tests only
│   ├── save_migration.rs             # Golden save fixture tests
│   ├── tutorial_zone.rs              # 100-seed validation
│   ├── fixtures/
│   │   └── saves/                     # Golden save files per schema version
│   │       ├── test_save_v1.ron
│   │       └── ...
│   ├── delta_roundtrip.rs             # Property-based delta-save roundtrip (proptest) — CRITICAL
│   └── integration/
│       ├── combat_flow.rs             # Weapon → Collision → Damage → Event chain
│       ├── chunk_lifecycle.rs         # Generate → Load → Modify → Save → Reload
│       └── audio_transitions.rs       # State change → Crossfade trigger
```

### Sprint 0 Subset

Only these files exist at project start. Everything else is created when its epic begins.

| File | Purpose |
|------|---------|
| `src/main.rs` | App builder |
| `src/lib.rs` | `pub fn game_plugins()` |
| `src/game_states.rs` | `GameState` enum |
| `src/shared/mod.rs` + `components.rs` | `Velocity`, `Health` |
| `src/core/mod.rs` | `CorePlugin` (or `GamePlugin` initially) |
| `src/core/flight.rs` | Flight physics |
| `src/core/input.rs` | Action layer |
| `src/rendering/mod.rs` | `RenderingPlugin` |
| `src/rendering/vector_art.rs` | Lyon pipeline for player ship |
| `src/dev/mod.rs` | `DevPlugin` |
| `assets/config/flight.ron` | Flight balance values |
| `tests/flight_physics.rs` | Flight model unit tests |

**Note:** Save infrastructure (`save/mod.rs`, `schema.rs`) is created in Epic 1 Story 1, not Sprint 0. Sprint 0 focuses on flight + rendering proof-of-concept.

### System Location Mapping

| System | Location | Plugin |
|--------|----------|--------|
| Flight Physics | `src/core/flight.rs` | `CorePlugin` |
| Weapon System | `src/core/weapons.rs` | `CorePlugin` |
| Collision (pure math) | `src/core/collision.rs` | `CorePlugin` |
| Camera | `src/core/camera.rs` | `CorePlugin` |
| Input Action Layer | `src/core/input.rs` | `CorePlugin` |
| Chunk Management | `src/world/chunk.rs` | `WorldPlugin` |
| Procedural Generation | `src/world/generation.rs` | `WorldPlugin` |
| Noise Layers | `src/world/noise_layers.rs` | `WorldPlugin` |
| Tutorial Zone | `src/world/tutorial_zone.rs` | `WorldPlugin` |
| Wormhole Mini-Levels | `src/world/wormhole.rs` | `WorldPlugin` |
| Stations & Economy | `src/gameplay/stations.rs` + `economy.rs` | `GameplayPlugin` |
| Upgrades | `src/gameplay/upgrades.rs` | `GameplayPlugin` |
| Boss Encounters | `src/gameplay/boss.rs` | `GameplayPlugin` |
| Companion Core | `src/social/companion_core.rs` | `SocialPlugin` |
| Companion Personality | `src/social/companion_personality.rs` | `SocialPlugin` |
| Enemy AI (FSM) | `src/social/enemy_ai.rs` | `SocialPlugin` |
| Delta-Save System | `src/infrastructure/save/` | `InfrastructurePlugin` |
| Event-Observer/Logbook | `src/infrastructure/events.rs` + `logbook.rs` | `InfrastructurePlugin` |
| Audio/Crossfade | `src/infrastructure/audio.rs` | `InfrastructurePlugin` |
| Object Pool | `src/infrastructure/pool.rs` | `InfrastructurePlugin` |
| Vector Art (lyon) | `src/rendering/vector_art.rs` | `RenderingPlugin` |
| Mesh Art | `src/rendering/mesh_art.rs` | `RenderingPlugin` |
| HUD/Menus | `src/ui/` | `UiPlugin` |
| Dev Tools | `src/dev/` | `DevPlugin` (dev feature only) |

### Naming Conventions

| Element | Convention | Example |
|---------|-----------|---------|
| Files/Modules | `snake_case` | `enemy_ai.rs`, `tutorial_zone.rs` |
| Structs/Enums | `PascalCase` | `ChunkCoord`, `GameAction`, `EventSeverity` |
| Components | `PascalCase`, noun | `Player`, `Velocity`, `Health`, `FactionAffiliation` |
| Resources | `PascalCase`, noun | `WorldSeed`, `ObjectPool`, `Logbook` |
| Systems (functions) | `snake_case`, verb phrase | `apply_thrust`, `check_collisions`, `update_fsm` |
| Events | `PascalCase`, past tense | `EnemyDestroyed`, `ChunkLoaded`, `CompanionRecruited` |
| Config Assets | `PascalCase` + `Config` suffix | `WeaponConfig`, `FlightConfig`, `AiConfig` |
| Constants | `UPPER_SNAKE_CASE` | `MAX_ENTITIES`, `FACTION_NOISE_OFFSET` |
| RON files | `snake_case.ron` | `flight.ron`, `scout_drone.ron` |

### Architectural Boundaries

1. **Plugin Isolation:** Each domain plugin registers its own systems, components, and events. No direct cross-plugin `Query` of domain-specific components — communicate via Events or shared components from `src/shared/`.
2. **Shared Components:** Components used across multiple domains (`Health`, `Velocity`, `Faction`, etc.) live in `src/shared/components.rs`. Domain-specific components (e.g., `FsmState`) stay in their domain module.
3. **Collision Separation:** `src/core/collision.rs` contains pure math functions only (ray-circle, circle-circle). Collision *response systems* (who collides with whom and what happens) live in their respective domain modules.
4. **Rendering Separation:** Game logic never touches rendering directly. Components hold data, rendering systems read components.
5. **Save Boundary:** Only `InfrastructurePlugin` writes save files. Other plugins emit events; the save system observes them.
6. **Dev Feature Gate:** Everything in `src/dev/` behind `#[cfg(feature = "dev")]`. Zero cost in release.
7. **Config Files On-Demand:** Only create `assets/config/*.ron` files when the epic that needs them begins. No speculative empty configs.
8. **Plugin Evolution:** Start with `GamePlugin` + `DevPlugin` in Sprint 0. Split into domain sub-plugins when a module exceeds ~500 lines or when parallel development requires isolation.

### Test Strategy

| Test Type | Location | When |
|-----------|----------|------|
| **Unit Tests** | Inline `#[cfg(test)] mod tests` in each `.rs` file | Pure logic: math, FSM transitions, serialization |
| **Integration Tests** | `tests/` directory | Multi-system flows: combat chain, save lifecycle, audio transitions |
| **Golden Fixtures** | `tests/fixtures/saves/` | Save migration regression tests |
| **Seed Validation** | `tests/tutorial_zone.rs` | 100-seed completability check |
| **Property-Based** | `tests/delta_roundtrip.rs` | Delta-save roundtrip verification via `proptest` |

**Epic Dependencies:** For epic execution order and dependencies, see `epics.md`. The architecture assumes epics are implemented in order; later epics build on earlier foundations.

---

## Implementation Patterns

These patterns ensure consistent implementation across all AI agents.

### Novel Patterns

#### 1. Delta-Save Pattern

**Purpose:** Seed reproduces base world; only deviations are persisted.

**Seeded Entity Identity:**

```rust
/// Deterministic identity for seed-generated entities.
/// Same seed + same chunk = same generation order = same index.
#[derive(Serialize, Deserialize, Clone, Eq, PartialEq, Hash)]
struct SeededEntityId {
    chunk: ChunkCoord,
    index: u32,  // Index in deterministic generation order
}
```

**Delta Structure:**

```rust
#[derive(Serialize, Deserialize)]
struct ChunkDelta {
    coord: ChunkCoord,
    destroyed_entities: Vec<SeededEntityId>,
    added_entities: Vec<EntitySnapshot>,
    modified_states: Vec<(SeededEntityId, StateChange)>,
}
```

**Data Flow:**

```
World Seed → Generate Base Chunk → Apply Deltas (if any) → Active Chunk
                                         ↑
Player Action → Modify Chunk → Diff vs Base → Store Delta
```

**Rules:**
- Every system that persistently modifies the world MUST emit a `WorldModified` event. No direct delta writes.
- `SeededEntityId = (ChunkCoord, u32)` — chunk position + deterministic generation index.
- Diff algorithm is the single most critical function: must be property-tested.

**Property-Based Test (Critical):**

```rust
use proptest::prelude::*;

proptest! {
    #[test]
    fn delta_roundtrip(seed: u64, modifications: Vec<Modification>) {
        let base = generate_chunk(seed);
        let modified = apply_modifications(base.clone(), &modifications);
        let delta = diff(&base, &modified);
        let restored = apply_delta(base, delta);
        assert_eq!(modified, restored);
    }
}
```

#### 2. Gravity Well Tutorial Pattern

**Purpose:** Procedurally generated tutorial zone with physics-based boundary, ability unlock sequence, and destructive finale.

**State Machine:**

```rust
#[derive(States, Clone, Eq, PartialEq, Debug, Hash)]
enum TutorialPhase {
    Flying,          // Initial: only thrust + rotate
    Shooting,        // Laser pickup found
    SpreadUnlocked,  // Spread pickup found (last ability)
    Complete,        // Generator destroyed, field deactivated
}
```

**Components:**

```rust
#[derive(Component)]
struct GravityWellGenerator {
    safe_radius: f32,
    pull_strength: f32,
    requires_projectile: bool,  // Only Spread can destroy
    health: f32,
}

#[derive(Resource)]
struct TutorialState {
    phase: TutorialPhase,
    abilities_unlocked: Vec<AbilityType>,
}
```

**Constraint Validation (100-seed test):**

```rust
struct TutorialConstraints {
    min_safe_area: f32,           // Player must have room to practice
    all_pickups_reachable: bool,  // No pickup outside safe_radius
    max_completion_steps: u32,    // No seed may require >N steps
    generator_destructible: bool, // Spread must be able to reach generator
}

fn validate_tutorial_seed(seed: u64) -> Result<(), Vec<ConstraintViolation>> {
    let zone = generate_tutorial_zone(seed);
    let constraints = TutorialConstraints::default();
    constraints.validate(&zone)
}
```

#### 3. Procedural Vector Art Pattern

**Purpose:** All visuals generated at runtime. Two render tiers by complexity.

| Tier | Renderer | Entities | Budget |
|------|----------|----------|--------|
| **High-Detail** | `lyon` → `Mesh2d` | Player, Stations, Bosses | Few, always visible |
| **Bulk** | Bevy `Mesh2d` direct | Enemies, Asteroids, Projectiles | Many, performance-critical |

**Specific generation functions (no generic trait):**

```rust
fn generate_player_mesh(upgrade_tier: u8) -> Mesh { ... }
fn generate_enemy_mesh(enemy_type: EnemyType, faction: Faction, seed: u64) -> Mesh { ... }
fn generate_asteroid_mesh(seed: u64, size: AsteroidSize) -> Mesh { ... }
fn generate_station_mesh(station_type: StationType, seed: u64) -> Mesh { ... }
fn generate_boss_mesh(boss_type: BossType, seed: u64) -> Mesh { ... }
```

**Seed Determinism:** Same seed + entity type = same visual. Appearance must not change between sessions.

**Color Palette:** 5-6 curated palettes per biome. Seed selects palette and varies within it.

#### 4. Event-Severity Logbook Pattern

**Purpose:** Automatic storytelling through event-observer with tier filtering.

**Data Flow:**

```
Any System → emit GameEvent → Event-Observer → Severity Lookup → Logbook
                                                     ↑
                                          assets/config/event_severity.ron
```

**Severity Mapping as Config (not hardcoded):**

```rust
// assets/config/event_severity.ron
EventSeverityConfig({
    EnemyDestroyed: Tier3,
    BossDefeated: Tier1,
    CompanionRecruited: Tier1,
    CompanionLost: Tier1,
    StationDiscovered: Tier2,
    UpgradeInstalled: Tier2,
    WormholeCleared: Tier2,
    ResourceCollected: Tier3,
    FactionReputationChanged: Tier2,
    ZoneFirstEntered: Tier1,
})
```

**Logbook Resource:**

```rust
#[derive(Resource)]
struct Logbook {
    entries: Vec<LogbookEntry>,
    max_entries: usize,  // Ring buffer for memory
}

#[derive(Serialize, Deserialize)]
struct LogbookEntry {
    kind: GameEventKind,
    severity: EventSeverity,
    game_time: f64,
    position: Vec2,
    description: String,  // Generated from event kind + context
}
```

**UI default:** Player sees Tier 1+2. Tier 3 available via filter toggle.

#### 5. Wormhole Scene Transition Pattern

**Purpose:** Isolated mini-levels accessed via wormholes with safe state management.

**Approach:** State-gating via `PlayingSubState::InWormhole` (same Bevy `World`, not separate).

**Transition Flow:**

```
Player enters Wormhole →
  1. Emit SaveRequested (auto-save main world state)
  2. Store WormholeReturnPoint { position, chunk }
  3. Despawn/hide main world entities (outside visible range)
  4. Generate mini-level from seed + distance_factor
  5. Transition to PlayingSubState::InWormhole
  6. On complete: Award rewards, transition back to Flying
  7. On crash/exit: Respawn at wormhole entrance in main world
```

**Components:**

```rust
#[derive(Resource)]
struct WormholeReturnPoint {
    position: Vec2,
    chunk: ChunkCoord,
    entrance_seed: u64,
}

#[derive(Component)]
struct WormholeEntity;  // Marker for mini-level entities (despawn on exit)
```

**Camera:** Resets to default follow-mode on wormhole exit. Boss-encounter camera zoom-out is post-MVP.

**Crash Recovery:** If game crashes during wormhole, the auto-save from step 1 is the recovery point. Player reappears at wormhole entrance. No mini-level progress lost (there was none to save).

### Standard Patterns

#### Communication: Bevy Events

Systems communicate via typed events. No direct Query across plugin boundaries.

```rust
// System A emits
fn destroy_enemy(mut events: EventWriter<EnemyDestroyed>, ...) {
    events.send(EnemyDestroyed { entity, position, faction });
}

// System B reacts
fn on_enemy_destroyed(mut events: EventReader<EnemyDestroyed>, ...) {
    for event in events.read() {
        // Spawn loot, update score, log event...
    }
}
```

#### Entity Creation: Commands + Pool

```rust
// Non-pooled (enemies, asteroids): Commands::spawn
fn spawn_enemy(mut commands: Commands, configs: Res<Assets<AiConfig>>, ...) {
    commands.spawn((
        Enemy { enemy_type: EnemyType::Fighter },
        Health(config.fighter_health),
        Velocity::ZERO,
        FactionAffiliation(faction),
    ));
}

// Pooled (projectiles, pickups): Pool with soft/hard limits
fn fire_weapon(mut pool: ResMut<ObjectPool>, ...) {
    match pool.activate(PoolType::Projectile) {
        PoolResult::Activated(entity) => { /* reset components */ }
        PoolResult::ForceRecycled(entity) => { warn!("Pool soft limit exceeded"); /* reset */ }
        PoolResult::Exhausted => { warn!("Pool hard limit reached, projectile dropped"); }
    }
}
```

**Pool Limits:**

| Type | Soft Limit | Hard Limit | On Exhaust |
|------|-----------|-----------|------------|
| Projectiles | 80 | 120 | Drop (weapon doesn't fire) |
| Pickups | 30 | 50 | Oldest pickup despawns |

#### State: Bevy States + Data-Driven FSM

```rust
// Game-level: Bevy States
#[derive(States, Default, Clone, Eq, PartialEq, Debug, Hash)]
enum GameState { #[default] Loading, Menu, Playing, Paused }

#[derive(SubStates, Clone, Eq, PartialEq, Debug, Hash)]
#[source(GameState = GameState::Playing)]
enum PlayingSubState { Flying, Docked, InWormhole, InTutorial }

// Entity-level AI: Data-driven FSM with action output
struct FsmTransition {
    new_state: AiState,
    actions: Vec<AiAction>,  // Fire, CallReinforcements, Flee, etc.
}

fn next_state(current: AiState, context: &AiContext, config: &AiConfig) -> FsmTransition {
    // Pure function — no ECS dependency, fully unit-testable
}
```

#### Data Access: AssetServer for Config, Resources for Runtime

```rust
// Balance data: AssetServer, hot-reloadable
fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    let flight_config: Handle<FlightConfig> = asset_server.load("config/flight.ron");
    commands.insert_resource(FlightConfigHandle(flight_config));
}

// Runtime state: Bevy Resources
#[derive(Resource)]
struct WorldSeed(u64);
```

### Consistency Rules

| Pattern | Convention | Enforcement |
|---------|-----------|-------------|
| No `unwrap()` in game code | `Result<T, E>` everywhere | `#[deny(clippy::unwrap_used)]` |
| Config structs | `*Config` suffix + `Asset` derive | Code review |
| System functions | Verb phrase, `snake_case` | `apply_thrust`, `update_fsm` |
| Events | `PascalCase`, past tense | `EnemyDestroyed`, `ChunkLoaded` |
| Components | `PascalCase`, noun | `Health`, `Velocity`, `Player` |
| Cross-plugin communication | Events only, no direct Query | Architectural boundary rule |
| Pooled entities | Via `ObjectPool` resource | `Projectile`, `Pickup` types |
| Balance tuning | RON asset + `AssetServer` | Never hardcode tuneable values |
| Game state changes | Via `GameEvent` emission | Event-Observer catches all |
| Severity mapping | `event_severity.ron` config | Never hardcode severity in emitter |
| FSM transitions | Return `FsmTransition` (state + actions) | Pure function, no ECS dependency |
| Seeded identity | `SeededEntityId = (ChunkCoord, u32)` | Deterministic generation order |
| System scheduling | Physics/Collision in `FixedUpdate`, Rendering in `Update` | Schedule assignment rule |

---

## Architecture Validation

### Validation Summary

| Check | Result | Notes |
|-------|--------|-------|
| Decision Compatibility | ✅ PASS | All 9 decisions compatible, no conflicts |
| GDD Coverage | ✅ PASS | 21/21 systems covered |
| Pattern Completeness | ✅ PASS | 9/9 scenarios with concrete examples |
| Epic Mapping | ✅ PASS | 13/13 epics mapped to architecture |
| Document Completeness | ✅ PASS | All sections present, no placeholders |

### Coverage Report

- **Systems Covered:** 21/21
- **Patterns Defined:** 5 novel + 4 standard
- **Decisions Made:** 9
- **Consistency Rules:** 12

### Issues Resolved During Validation

| Issue | Resolution |
|-------|-----------|
| System ordering undefined (HIGH RISK) | Added SystemSet ordering + Bevy Schedule mapping as cross-cutting concern |
| WASM save storage unspecified | Added platform-specific storage table (Filesystem vs LocalStorage/IndexedDB) |
| Delta roundtrip test location missing | Added `tests/delta_roundtrip.rs` to test structure |
| Sprint 0 vs Epic 1 save boundary unclear | Clarified: Save infrastructure created in Epic 1 Story 1, not Sprint 0 |
| Epic dependencies not referenced | Added reference to `epics.md` for dependency graph |
| Camera behavior during transitions undefined | Added camera reset note to Wormhole pattern |
| Bevy Schedule assignment undocumented | Added FixedUpdate/Update/PostUpdate assignment table |

### Validation Date

2026-02-25
