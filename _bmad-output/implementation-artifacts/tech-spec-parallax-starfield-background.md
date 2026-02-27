---
title: 'Parallax Starfield Background'
slug: 'parallax-starfield-background'
created: '2026-02-26'
status: 'ready-for-dev'
stepsCompleted: [1, 2, 3, 4]
tech_stack: ['bevy 0.18', 'Mesh2d', 'ColorMaterial', 'Camera2d']
files_to_modify: ['src/rendering/background.rs (CREATE)', 'src/rendering/mod.rs (MODIFY)']
code_patterns: ['asset caching at startup', 'grid-cell hash for deterministic positions', 'entity pool with reposition', 'parallax offset from camera position']
test_patterns: ['deterministic unit tests for hash/math', 'MinimalPlugins test app']
---

# Tech-Spec: Parallax Starfield Background

**Created:** 2026-02-26

## Overview

### Problem Statement

The player cannot perceive ship movement because the camera follows the player centered with nothing in the background. Without visual reference points, flying feels static — the ship appears to rotate in place rather than drift through space.

### Solution

Add 3 layers of procedurally generated stars at different parallax speeds. Stars are generated deterministically from world-space grid cell coordinates using a hash function. A fixed pool of star entities per layer is repositioned each frame based on camera position, creating an infinite starfield in all directions. Closer layers move faster relative to the camera, providing immediate motion feedback.

### Scope

**In Scope:**
- 3 star layers with parallax factors 0.02, 0.1, 0.35
- Procedural star positions from grid cell coordinate hashing (deterministic, reproducible)
- Stars rendered as small circles (Mesh2d + ColorMaterial), varying size/brightness per layer
- Fixed entity pool per layer, repositioned each frame (no runtime spawn/despawn)
- Black background (default Bevy clear color)
- Performance: no measurable impact on 60fps target

**Out of Scope:**
- Color gradients or nebula layers (Epic 10)
- Curated color palettes (Epic 10)
- Twinkling or animation effects
- Chunk-based persistence or save integration
- Star variety within a single layer

## Context for Development

### Codebase Patterns

- **Asset caching:** All visual assets (LaserAssets, ProjectileAssets, DestructionAssets, ImpactFlashAssets) are initialized at Startup and stored as Resources with `commands.insert_resource()`. One shared `Handle<Mesh>` + `Handle<ColorMaterial>` per visual type. Stars follow this pattern with one mesh+material per layer.
- **Rendering separation:** Core game logic never touches rendering. Background is purely rendering-domain. Lives in `src/rendering/background.rs`.
- **Camera system:** `camera_follow_player` in PostUpdate sets `camera.translation = player.translation`. Screen shake adds offset afterward via `apply_screen_shake.after(camera_follow_player)`. Star repositioning reads camera position in Update (before PostUpdate camera changes, so it uses previous frame's camera pos — acceptable for background).
- **Mesh2d pattern:** Entities use `Mesh2d(handle) + MeshMaterial2d(handle) + Transform` tuple for 2D rendering. Handles are cloned from cached resources.
- **System registration:** All rendering systems registered in `RenderingPlugin::build()`. Startup for asset init, Update for per-frame logic.
- **`#![deny(clippy::unwrap_used)]`** enforced crate-wide. Use `.expect()` in tests.

### Files to Reference

| File | Purpose |
| ---- | ------- |
| `src/rendering/mod.rs` | RenderingPlugin — system registration, asset caching pattern, Startup/Update/PostUpdate |
| `src/rendering/effects.rs` | ImpactFlashAssets — asset caching resource pattern reference |
| `src/core/camera.rs` | `camera_follow_player` — camera query pattern `Query<&Transform, With<Camera2d>>` |
| `src/rendering/vector_art.rs` | Procedural mesh generation reference (`Circle::new(radius)`) |

### Technical Decisions

1. **Entity pool, not spawn/despawn:** Fixed pool of star entities spawned at Startup, repositioned each frame. Zero allocation at runtime. Pool size = max visible stars per layer with margin.
2. **Grid-cell hashing for deterministic positions:** World space divided into cells per layer. Hash `(cell_x, cell_y, layer_index)` → pseudo-random star offsets within cell. Same camera position always shows same stars.
3. **Parallax via position offset:** `star_world_pos = star_layer_pos + camera_pos * (1.0 - parallax_factor)`. No separate cameras, no render layers. Simple Transform manipulation.
4. **Stars at negative Z-depth:** Stars rendered behind gameplay entities via `Transform.translation.z = -10.0 - layer_index`. Bevy's 2D renderer respects Z-ordering.
5. **3 layers** with configuration:
   - Layer 0 (far): parallax 0.02, radius 0.8, brightness 0.15, cell_size 600, 3 stars/cell
   - Layer 1 (mid): parallax 0.1, radius 1.2, brightness 0.25, cell_size 400, 2 stars/cell
   - Layer 2 (near): parallax 0.35, radius 1.8, brightness 0.45, cell_size 300, 2 stars/cell
6. **Viewport assumption:** Default Camera2d with no zoom → viewport ~1280x720 world units (standard HD). Padding of 1 extra cell on each side for seamless scrolling.

## Implementation Plan

### Tasks

- [ ] Task 1: Create `src/rendering/background.rs` — module structure and types
  - File: `src/rendering/background.rs` (CREATE)
  - Action: Define the following types:
    - `StarLayerConfig` struct: `parallax_factor: f32`, `star_radius: f32`, `brightness: f32`, `cell_size: f32`, `stars_per_cell: u32`, `z_depth: f32`
    - `StarfieldConfig` resource: `layers: Vec<StarLayerConfig>` with `Default` impl containing 3 layer configs
    - `Star` component: `layer_index: usize` (identifies which layer a star entity belongs to)
    - `StarAssets` resource: `meshes: Vec<Handle<Mesh>>`, `materials: Vec<Handle<ColorMaterial>>` (one per layer)
  - Notes: All types `pub`. StarfieldConfig implements Default with the 3 layer values from Technical Decisions.

- [ ] Task 2: Implement `cell_hash` and `stars_in_cell` helper functions
  - File: `src/rendering/background.rs`
  - Action: Create two pure functions:
    - `pub fn cell_hash(cx: i32, cy: i32, layer: usize, star_index: u32) -> u64` — deterministic hash from cell coordinates. Use wrapping multiply with large primes (e.g., 2654435761, 40503, 12289) for distribution.
    - `pub fn stars_in_cell(cx: i32, cy: i32, layer: usize, config: &StarLayerConfig) -> Vec<Vec2>` — returns star positions in world space for a given cell. Each star offset = `hash → normalize to 0..1 → scale by cell_size → add cell origin`.
  - Notes: These are the only testable pure functions. Keep them public for unit testing.

- [ ] Task 3: Implement `setup_starfield` startup system
  - File: `src/rendering/background.rs`
  - Action: Create startup system that:
    1. Reads `StarfieldConfig` resource
    2. For each layer, creates `Handle<Mesh>` (Circle with layer radius) and `Handle<ColorMaterial>` (white with layer brightness as alpha: `Color::srgba(1.0, 1.0, 1.0, brightness)`)
    3. Stores handles in `StarAssets` resource
    4. Spawns star entity pool: for each layer, spawn `pool_size` entities with `Star { layer_index }`, `Mesh2d(handle)`, `MeshMaterial2d(handle)`, `Transform::from_xyz(0.0, 0.0, z_depth)` with `Visibility::Hidden`
    5. Pool size per layer: `(ceil(1280/cell_size) + 3) * (ceil(720/cell_size) + 3) * stars_per_cell` — enough to cover viewport + margin
  - Notes: Stars start hidden. The update system will position and show visible ones each frame.

- [ ] Task 4: Implement `update_starfield` Update system
  - File: `src/rendering/background.rs`
  - Action: Create Update system that each frame:
    1. Query camera position: `Query<&Transform, With<Camera2d>>`
    2. For each layer in `StarfieldConfig`:
       a. Calculate effective camera pos: `cam_pos * parallax_factor`
       b. Calculate visible cell range: `floor((effective_cam - viewport_half) / cell_size)` to `ceil((effective_cam + viewport_half) / cell_size)`
       c. Generate star positions for all visible cells via `stars_in_cell()`
       d. Transform to world space: `star_world_pos = star_layer_pos + cam_pos * (1.0 - parallax_factor)`
       e. Assign positions to pooled star entities (query `Query<(&Star, &mut Transform, &mut Visibility)>`), set `Visibility::Visible`
       f. Hide remaining unused pool entities: set `Visibility::Hidden`
  - Notes: The viewport half-size is hardcoded to 640x360 (half of 1280x720). This is acceptable for the prototype — proper viewport calculation from camera projection can be added later.

- [ ] Task 5: Register module and systems in RenderingPlugin
  - File: `src/rendering/mod.rs` (MODIFY)
  - Action:
    1. Add `pub mod background;` declaration
    2. Import `setup_starfield` and `update_starfield` from background module
    3. Add `app.init_resource::<StarfieldConfig>()` in plugin build
    4. Add `setup_starfield` to Startup systems tuple
    5. Add `update_starfield` to Update systems tuple
  - Notes: No ordering constraints needed — background doesn't interact with gameplay systems.

- [ ] Task 6: Unit tests for hash and position generation
  - File: `src/rendering/background.rs` (in `#[cfg(test)] mod tests`)
  - Action: Write unit tests:
    1. `cell_hash_is_deterministic` — same inputs produce same output
    2. `cell_hash_varies_with_inputs` — different cell coords produce different hashes
    3. `cell_hash_varies_with_layer` — same cell, different layer → different hash
    4. `stars_in_cell_positions_within_bounds` — all generated positions fall within the cell's world-space bounds
    5. `stars_in_cell_count_matches_config` — returns exactly `stars_per_cell` positions
  - Notes: Use `.expect()` not `.unwrap()`. Pure function tests, no App needed.

### Acceptance Criteria

- [ ] AC 1: Given a player flying in any direction, when the camera moves, then background stars at different layers move at visibly different speeds — far stars barely move, near stars move noticeably.
- [ ] AC 2: Given a specific camera position, when revisiting that exact position later, then the same star pattern is displayed (deterministic generation from position hash).
- [ ] AC 3: Given 3 star layers rendered simultaneously, then far stars appear smaller and dimmer (radius 0.8, brightness 0.15) while near stars appear larger and brighter (radius 1.8, brightness 0.45).
- [ ] AC 4: Given the starfield system running alongside 200+ gameplay entities, when measuring frame rate, then performance remains at 60fps on Tier 1 hardware.
- [ ] AC 5: Given any camera position (including after flying 10000+ units from origin), when looking at the background, then stars are visible and provide motion reference without gaps or visual artifacts.
- [ ] AC 6: Given no `unwrap()` in game code, when running `cargo clippy`, then no warnings from `deny(clippy::unwrap_used)`.
- [ ] AC 7: All existing 79 tests continue to pass (no regression).

## Additional Context

### Dependencies

None — purely additive rendering feature with no core logic changes. No new crate dependencies required.

### Testing Strategy

**Unit tests** (5 tests in `background.rs`):
- Hash determinism and distribution
- Star position bounds and count

**Manual testing:**
- Fly the ship in all directions — verify parallax depth effect
- Fly far from origin (10000+ units) — verify no visual artifacts
- Verify stars don't distract from gameplay elements

**No integration tests** — background doesn't interact with core gameplay systems. Visual correctness is verified by playtesting.

### Notes

- GDD specifies background should be "atmospheric, never competing with gameplay" in contrast hierarchy (layer 5 of 5)
- Stars are white with varying alpha (0.15-0.45) — dim enough not to distract from player (golden), enemies, or projectiles (cyan/orange)
- Epic 10 will replace/enhance this with color gradients, nebulae, and curated palettes selected by world seed
- Viewport size hardcoded to 1280x720 for prototype — will need camera projection query when zoom is added
- Entity pool approach means zero GC pressure and consistent frame times
