# Story 1.4: World Map

Status: done

<!-- Note: Validation is optional. Run validate-create-story for quality check before dev-story. -->

## Story

As a player,
I want to open a world map showing areas I've explored,
so that I have a sense of progress and orientation.

## Acceptance Criteria

1. **Map Toggle** — Pressing `M` opens/closes the world map as a fullscreen semi-transparent overlay. Toggle is instant (<1 frame). The game continues running underneath.
2. **Explored Chunks Displayed** — All chunks the player has ever entered appear as colored tiles on the map. Unexplored area is dark/empty.
3. **Biome Coloring** — Tiles are colored by biome type: dark blue for Deep Space, gray for Asteroid Field, amber/brown for Wreck Field.
4. **Player Position Marker** — A bright white marker shows the player's current position on the map. The map is centered on the player's current chunk.
5. **Exploration Tracking** — An `ExploredChunks` resource (`HashMap<ChunkCoord, BiomeType>`) persistently tracks all visited chunks during the session. Once a chunk is explored, it stays explored. (Save/load integration deferred to Story 1.8.)
6. **Map Scale** — Each chunk is rendered as a tile of configurable pixel size (default 12px). Tiles outside the visible map area are culled (not spawned).
7. **Config-Driven** — `WorldMapConfig` loaded from `assets/config/world_map.ron` via `from_ron()` + `Default` fallback pattern. Configurable: tile_size, colors, player_marker_size, map_width, map_height, background_opacity.
8. **Performance** — Map open/close is instant. No performance impact when map is closed. Rendering 200+ explored chunks is smooth. Tiles spawned incrementally (only new chunks added, not full rebuild every frame).
9. **No Gameplay Impact** — The map is purely visual/informational. It does not affect physics, collision, or entity behavior.
10. **Overflow Clipping** — Tiles that fall outside the map container area are clipped (Bevy UI `Overflow::clip()`), preventing visual overflow.

## Tasks / Subtasks

- [x] Task 1: Add ExploredChunks tracking to world module (AC: #5)
  - [x] 1.1 Define `ExploredChunks` resource (`HashMap<ChunkCoord, BiomeType>`) in `src/world/mod.rs`
  - [x] 1.2 Initialize `ExploredChunks` as empty in `WorldPlugin` startup
  - [x] 1.3 In `update_chunks`: when inserting a chunk into `ActiveChunks`, also insert into `ExploredChunks` (if not already present)
  - [x] 1.4 Ensure `ExploredChunks` is `pub` and exported from the world module

- [x] Task 2: Define WorldMapConfig and create RON file (AC: #7)
  - [x] 2.1 Create `src/rendering/world_map.rs` with `WorldMapConfig` struct
  - [x] 2.2 Implement `from_ron()` + `Default` pattern (identical to MinimapConfig)
  - [x] 2.3 Create `assets/config/world_map.ron` with tuned defaults
  - [x] 2.4 Load `WorldMapConfig` in `RenderingPlugin` startup

- [x] Task 3: Implement pure helper functions (AC: #3, #4, #6)
  - [x] 3.1 `chunk_to_map_position(chunk, player_chunk, map_center, tile_size) -> Vec2`
  - [x] 3.2 `biome_map_color(biome, config) -> Color`
  - [x] 3.3 `is_tile_visible(chunk, player_chunk, map_width, map_height, tile_size) -> bool`

- [x] Task 4: Implement map toggle system (AC: #1)
  - [x] 4.1 Define `WorldMapOpen` resource (`bool`, default `false`)
  - [x] 4.2 Define `WorldMapRoot` marker component
  - [x] 4.3 Create `toggle_world_map` system: reads `ButtonInput<KeyCode>` for `KeyCode::KeyM`
  - [x] 4.4 On toggle ON → call `open_world_map()` to spawn UI hierarchy
  - [x] 4.5 On toggle OFF → despawn `WorldMapRoot` entity recursively

- [x] Task 5: Implement map rendering — open/close lifecycle (AC: #1, #2, #3, #4, #6, #10)
  - [x] 5.1 `open_world_map()`: Spawn `WorldMapRoot` with fullscreen overlay Node
  - [x] 5.2 Child: MapContainer Node with `Overflow::clip()`
  - [x] 5.3 Spawn tile child Nodes for explored chunks
  - [x] 5.4 Spawn `WorldMapPlayerMarker` Node at map center
  - [x] 5.5 Use `WorldMapTile` and `WorldMapPlayerMarker` marker components
  - [x] 5.6 Track rendered chunks in `WorldMapState`

- [x] Task 6: Implement live map updates while open (AC: #2, #4, #8)
  - [x] 6.1 `update_world_map` system: incremental tile spawning for newly explored chunks
  - [x] 6.2 Efficient: only iterate new chunks, not full rebuild

- [x] Task 7: Register systems in RenderingPlugin (AC: all)
  - [x] 7.1 Add `pub mod world_map` to `src/rendering/mod.rs`
  - [x] 7.2 Add `WorldMapOpen`, `WorldMapState` resource init
  - [x] 7.3 Register `toggle_world_map` in `Update`
  - [x] 7.4 Register `update_world_map` in `Update`

- [x] Task 8: Unit tests — 10 tests (AC: all)
  - [x] 8.1 `WorldMapConfig` RON parsing
  - [x] 8.2 `WorldMapConfig` default
  - [x] 8.3 `WorldMapConfig` fallback
  - [x] 8.4 `chunk_to_map_position`: player position → center
  - [x] 8.5 `chunk_to_map_position`: east of player
  - [x] 8.6 `chunk_to_map_position`: north of player (Y-flip)
  - [x] 8.7 `biome_map_color`: correct colors
  - [x] 8.8 `is_tile_visible`: within range → true
  - [x] 8.9 `is_tile_visible`: far away → false
  - [x] 8.10 `is_tile_visible`: large offset culled

- [x] Task 9: Integration tests — 6 tests (AC: all)
  - [x] 9.1 World map overlay spawns when M key pressed
  - [x] 9.2 World map overlay despawns when M key pressed again
  - [x] 9.3 ExploredChunks populated when update_chunks runs
  - [x] 9.4 Tile count matches explored chunks when map is opened
  - [x] 9.5 Player marker entity exists when map is open
  - [x] 9.6 World map tiles reposition when player moves

## Dev Notes

### Architecture Patterns & Constraints

- **Core/Rendering separation MUST be maintained:** `ExploredChunks` is game/world state → lives in `src/world/mod.rs`. World map rendering systems live in `src/rendering/world_map.rs`. The rendering module reads `ExploredChunks` and `ActiveChunks` but never writes to them.
- **Bevy UI for in-game surfaces:** Architecture decision #1 mandates Bevy UI (native) for all in-game UI. Do NOT use `bevy_egui` for the world map.
- **Config pattern:** `WorldMapConfig` follows the exact same pattern as `MinimapConfig`, `WorldConfig`, `FlightConfig`, `WeaponConfig`, `BiomeConfig`: `from_ron()` method, `Default` impl, `warn!` on parse error.
- **No unwrap():** `#[deny(clippy::unwrap_used)]` is enforced project-wide. Use `.expect("msg")` in tests.
- **System scheduling:** World map rendering runs in `Update` (frame-dependent visual updates), NOT `FixedUpdate`. The toggle system also runs in `Update`.
- **Input handling:** The toggle system reads `ButtonInput<KeyCode>` directly (not through `ActionState`). This avoids modifying the core input module for a rendering concern. Uses `just_pressed(KeyCode::KeyM)` for single-fire toggle.

### Implementation Strategy

- **Spawn/Despawn lifecycle:** When map opens, spawn entire UI hierarchy. When map closes, despawn `WorldMapRoot` recursively. This is simpler and cleaner than hiding/showing persistent UI nodes. The map is opened infrequently (player decision), so spawn cost is acceptable.
- **Incremental tile management:** While the map is open, only spawn tiles for newly explored chunks (tracked via `WorldMapState.rendered_chunks`). No full rebuild each frame. On close, `rendered_chunks` is cleared. On next open, all explored chunks are spawned fresh.
- **Coordinate mapping:**
  ```
  map_center = Vec2(map_width / 2.0, map_height / 2.0)
  player_chunk = world_to_chunk(player_pos, chunk_size)
  dx = (chunk.x - player_chunk.x) as f32
  dy = (chunk.y - player_chunk.y) as f32
  tile_left = map_center.x + dx * tile_size - tile_size / 2.0
  tile_top  = map_center.y - dy * tile_size - tile_size / 2.0   // Y-flip: world Y-up → UI Y-down
  ```
- **Overflow clipping:** MapContainer uses `Overflow::clip()` so tiles that extend beyond the map area are hidden. This avoids manual culling for edge tiles.
- **Z-index:** WorldMapRoot should render above the minimap and game world. Use a high `GlobalZIndex` or spawn order to ensure it's on top.

### Bevy 0.18 UI Notes (from Story 1.3 learnings)

- **`BorderRadius`** is a field on `Node`, NOT a separate Component
- **RON arrays:** `[f32; 4]` serializes as tuples `(1.0, 1.0, 1.0, 1.0)`, not `[1.0, ...]`
- **Node** is the base UI component with all style properties (width, height, position_type, left, top, margin, overflow, etc.)
- **BackgroundColor** for coloring UI nodes
- **`position_type: PositionType::Absolute`** for manual positioning within parent
- **`Overflow::clip()`** on the map container to clip overflowing tiles
- **Children despawn:** When despawning a parent entity with `commands.entity(e).despawn()`, Bevy auto-despawns all children (use `despawn_recursive` if needed, but default despawn in 0.18 handles hierarchy)

### UI Hierarchy

```
WorldMapRoot (Node: 100% × 100%, Absolute, z_index high, BackgroundColor semi-transparent)
  └── MapContainer (Node: map_width × map_height px, centered via margin:Auto, Overflow::clip())
        ├── WorldMapTile (Node: tile_size × tile_size, Absolute, left/top computed, BackgroundColor by biome)
        ├── WorldMapTile ...
        ├── WorldMapTile ...
        └── WorldMapPlayerMarker (Node: player_marker_size, Absolute, centered, bright white)
```

### Code Patterns (MUST REUSE from Story 1.3)

```rust
// Config loading — identical to MinimapConfig
impl WorldMapConfig {
    pub fn from_ron(data: &str) -> Self {
        ron::from_str(data).unwrap_or_else(|e| {
            warn!("Failed to parse world_map.ron: {e}, using defaults");
            Self::default()
        })
    }
}

// Pure coordinate conversion — same structure as world_to_minimap()
pub fn chunk_to_map_position(
    chunk: ChunkCoord,
    player_chunk: ChunkCoord,
    map_center: Vec2,
    tile_size: f32,
) -> Vec2 {
    let dx = (chunk.x - player_chunk.x) as f32;
    let dy = (chunk.y - player_chunk.y) as f32;
    Vec2::new(
        map_center.x + dx * tile_size - tile_size / 2.0,
        map_center.y - dy * tile_size - tile_size / 2.0, // Y-flip
    )
}

// Color lookup — same structure as blip_color()
pub fn biome_map_color(biome: BiomeType, config: &WorldMapConfig) -> Color {
    let arr = match biome {
        BiomeType::DeepSpace => config.color_deep_space,
        BiomeType::AsteroidField => config.color_asteroid_field,
        BiomeType::WreckField => config.color_wreck_field,
    };
    Color::srgba(arr[0], arr[1], arr[2], arr[3])
}
```

### ExploredChunks Design

```rust
// In src/world/mod.rs
#[derive(Resource, Default)]
pub struct ExploredChunks {
    pub chunks: HashMap<ChunkCoord, BiomeType>,
}
```

- Inserted into in `update_chunks()` alongside `ActiveChunks` inserts
- Never cleared during a session (permanent discovery)
- Read-only from rendering module
- Save/load integration deferred to Story 1.8 (delta-save)

### Previous Story Intelligence (from Stories 1.1, 1.2, 1.3)

**Learnings to apply:**
1. **Config loading is reliable** — `from_ron()` + `Default` pattern used in all 6 existing configs. Reuse exactly.
2. **Test harness** — `tests/helpers/mod.rs` must add `WorldMapConfig`, `WorldMapOpen`, `WorldMapState`, and `ExploredChunks` resources.
3. **ActiveChunks is `HashMap<ChunkCoord, BiomeType>`** — ExploredChunks follows the same structure.
4. **Y-axis flip is critical** — Bevy UI Y-axis is top-down, world Y-axis is bottom-up. Verified working in minimap coordinate conversion. Apply same flip in `chunk_to_map_position()`.
5. **Marker components for cleanup** — `MinimapRoot` pattern works well. Use `WorldMapRoot` for the same purpose.
6. **Rendering plugin Startup** — all config loading happens at Startup. WorldMapConfig loaded here too.
7. **`HashMap` import** — use `use std::collections::HashMap;` (was a code review finding in 1.3).
8. **`HashSet`** — `use std::collections::HashSet;` for `WorldMapState.rendered_chunks`.

**Code review patterns to watch for (from 1.3 findings):**
- Tile leak: map closed but tiles not despawned (ensure recursive despawn)
- Missing Y-flip in coordinate conversion
- Division by zero if tile_size is 0 in config
- Node accumulation if map is opened/closed rapidly without proper cleanup
- Overflow not set causing tiles to render outside map boundary
- WorldMapState not cleared on close causing stale state on next open

### Project Structure Notes

- **Files to create:**
  - `src/rendering/world_map.rs` — WorldMapConfig, WorldMapOpen, WorldMapState, marker components, pure functions, toggle/render systems, unit tests
  - `assets/config/world_map.ron` — World map configuration
  - `tests/world_map.rs` — Integration tests for world map
- **Files to modify:**
  - `src/world/mod.rs` — Add `ExploredChunks` resource, init in WorldPlugin, update in `update_chunks`
  - `src/rendering/mod.rs` — Add `pub mod world_map`, imports, WorldMapConfig loading, system registration
  - `tests/helpers/mod.rs` — Add `WorldMapConfig`, `WorldMapOpen`, `WorldMapState`, `ExploredChunks` to test harness
- **Files NOT to touch:**
  - `src/core/` — No core changes needed (input handled directly in rendering)
  - `src/shared/` — No shared component changes
  - `src/rendering/minimap.rs` — Minimap stays independent
  - `assets/config/minimap.ron` — Minimap config unchanged
  - `assets/config/world.ron` — World config unchanged
  - `assets/config/biome.ron` — Biome config unchanged

### Key Libraries

- **`bevy` 0.18.0** — `Node`, `BackgroundColor`, `PositionType`, `Val`, `UiRect`, `Overflow`, `ButtonInput<KeyCode>`, `KeyCode`, `Query`, `With`, `Entity`, `Commands`, `Res`, `ResMut`, `Transform`, `GlobalZIndex`
- **`ron`** + **`serde`** (already in Cargo.toml) — Config deserialization for WorldMapConfig
- **`std::collections::HashMap`** and **`HashSet`** — For ExploredChunks and WorldMapState
- **NO new dependencies needed**

### References

- [Source: _bmad-output/gdd.md — "World map: Player-revealed — only shows areas already explored"]
- [Source: _bmad-output/gdd.md — "Stations, death markers, and discovered points of interest are marked permanently"]
- [Source: _bmad-output/gdd.md — "Toggle map — Switch between minimap and world map"]
- [Source: _bmad-output/gdd.md — "Minimap scanning: Passive — always shows nearby points of interest as unidentified blips"]
- [Source: _bmad-output/game-architecture.md — "UI Framework" decision: Bevy UI native for in-game]
- [Source: _bmad-output/game-architecture.md — "System Ordering" — rendering in Update]
- [Source: _bmad-output/game-architecture.md — "Rendering Separation" — game logic never touches rendering directly]
- [Source: _bmad-output/game-architecture.md — "Project Structure" — src/rendering/world_map.rs, src/ui/map_ui.rs]
- [Source: _bmad-output/game-architecture.md — "Configuration Management" — RON *Config pattern]
- [Source: _bmad-output/epics.md — Epic 1 Story 4: "As a player, I can open a world map showing areas I've explored"]
- [Source: _bmad-output/implementation-artifacts/1-3-minimap-blips.md — MinimapConfig, pure functions, Bevy UI patterns, Y-flip handling]
- [Source: _bmad-output/implementation-artifacts/1-2-biome-types.md — BiomeType, ActiveChunks as HashMap]
- [Source: _bmad-output/implementation-artifacts/1-1-seamless-world-generation.md — Chunk system, ChunkCoord, world_to_chunk]

### Git Intelligence

- **Last commits:**
  - `fix(1.3): code review — blip color test, MinimapState cleanup on root removal. 2 new integration tests.`
  - `feat(1.3): minimap blips — scanner range, blip types, Bevy UI minimap rendering, MinimapConfig. 12 unit + 5 integration tests.`
  - `feat(1.2): biome types — deep space, asteroid fields, wreck fields, deterministic biome assignment. 18 new tests.`
- **Commit convention:** `feat(1.4): world map — explored chunk tracking, toggle overlay, biome-colored tiles, WorldMapConfig. X new tests (Y unit + Z integration).`
- **VCS:** Using `jj` (Jujutsu). Use `jj new` after code review marks story done.
- **Total tests before this story:** 202 (from 1.3 completion notes: 117 unit + 82 integration... actually code review notes say 199 total)

## Dev Agent Record

### Agent Model Used

Claude Opus 4.6

### Debug Log References

- Clippy `too_many_arguments`: Added `#[allow(clippy::too_many_arguments)]` to `toggle_world_map` (9 params) and `update_chunks` (8 params) — standard for Bevy systems with many ECS queries.
- Integration test `player_marker_exists_when_map_is_open`: Required extra `app.update()` cycles after despawn for deferred command propagation through Bevy UI hierarchy.
- `ButtonInput<KeyCode>` missing in test harness: Added `app.init_resource::<ButtonInput<KeyCode>>()` to `test_app()`.
- 4 existing `world_generation` tests failed: They build custom `App` without `ExploredChunks` — added `app.init_resource::<ExploredChunks>()` to each.

### Completion Notes List

- 10 unit tests + 6 integration tests = 16 new tests
- Total project tests: 137 unit + 83 integration = 220 (all passing)
- Clippy clean with `-D warnings`
- All 10 acceptance criteria satisfied

### File List

- `src/rendering/world_map.rs` — **NEW** — WorldMapConfig, pure functions, toggle/render systems, 10 unit tests
- `assets/config/world_map.ron` — **NEW** — World map configuration (tile_size, colors, dimensions)
- `tests/world_map.rs` — **NEW** — 5 integration tests for world map overlay
- `src/world/mod.rs` — **MODIFIED** — Added ExploredChunks resource, init in WorldPlugin, populate in update_chunks, clippy allow
- `src/rendering/mod.rs` — **MODIFIED** — Added world_map module, imports, WorldMapConfig loading, resource init, system registration
- `tests/helpers/mod.rs` — **MODIFIED** — Added WorldMapConfig, WorldMapOpen, WorldMapState, ExploredChunks, ButtonInput<KeyCode> to test harness
- `src/world/chunk.rs` — **MODIFIED** — Added `Default` derive to `ChunkCoord` (needed for `WorldMapState::default()`)
- `tests/world_generation.rs` — **MODIFIED** — Added ExploredChunks init to 4 custom App tests
