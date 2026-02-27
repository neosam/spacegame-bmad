# Story 1.3: Minimap Blips

Status: done

<!-- Note: Validation is optional. Run validate-create-story for quality check before dev-story. -->

## Story

As a player,
I want to see unidentified blips on my minimap,
so that I always have a "what's that?" to fly toward.

## Acceptance Criteria

1. **Minimap Rendered** — A circular minimap is displayed in a fixed HUD corner (top-right). It shows the player's ship at center and nearby entities as colored blips. The minimap is always visible during gameplay.
2. **Blip Types** — Entities within scanner range appear as blips with distinct colors per type:
   - **Asteroids:** Dim gray dots
   - **Scout Drones:** Red dots
   - **Stations:** (future — not spawned yet, but BlipType variant exists)
   - **Player:** White dot at center (always present)
3. **Scanner Range** — A configurable `scanner_range` (world units) in `MinimapConfig` determines which entities appear as blips. Only entities within this range of the player are shown. Default: ~2× chunk_size.
4. **Unidentified Blips** — All blips appear as generic "unidentified" dots (same shape, type-specific color only). No labels or detailed information. The player must fly toward them to learn what they are. (Identification system is deferred to a future story.)
5. **Minimap Scale** — The minimap maps world coordinates to minimap-local coordinates using a configurable scale factor. Blips move smoothly as the player moves.
6. **Blip Visibility Updates** — Blips appear/disappear in real-time as entities enter/leave scanner range or are spawned/despawned by the chunk system.
7. **Performance** — Minimap rendering adds <1ms per frame. Entity query is efficient (spatial proximity check, not full world scan).
8. **Config-Driven** — All minimap parameters (scanner_range, minimap_radius, blip_size, colors) are tunable via `assets/config/minimap.ron` using the established `from_ron()` + `Default` fallback pattern.
9. **No Gameplay Impact** — The minimap is purely visual. It does not affect collision, physics, or entity behavior.

## Tasks / Subtasks

- [x] Task 1: Define MinimapConfig and BlipType (AC: #2, #8)
  - [x] 1.1 Create `BlipType` enum (`Asteroid`, `ScoutDrone`, `Station`, `Player`) in `src/rendering/minimap.rs`
  - [x] 1.2 Create `MinimapConfig` resource with fields: `scanner_range`, `minimap_radius`, `blip_size`, per-type blip colors, `minimap_offset` (screen position)
  - [x] 1.3 Create `assets/config/minimap.ron` with default tuning values
  - [x] 1.4 Load `MinimapConfig` via `from_ron()` + `Default` fallback in `RenderingPlugin` startup

- [x] Task 2: Create minimap background and player indicator (AC: #1, #5)
  - [x] 2.1 Spawn minimap background entity: a circular dark semi-transparent shape at fixed screen position using Bevy UI
  - [x] 2.2 Spawn player center dot (white, fixed at minimap center)
  - [x] 2.3 Add minimap border ring for visual clarity
  - [x] 2.4 Rendering approach: Bevy UI nodes with absolute positioning (architecture mandates Bevy UI for in-game surfaces)

- [x] Task 3: Implement blip spawning and positioning (AC: #2, #3, #5, #6)
  - [x] 3.1 Create `update_minimap_blips` system that runs in `Update` schedule
  - [x] 3.2 Query all entities with `Transform` + (`Asteroid` OR `ScoutDrone`) within scanner_range of player
  - [x] 3.3 Convert world-space offset (entity pos - player pos) to minimap-space coordinates using scale factor
  - [x] 3.4 Clamp blips to minimap radius (entities at edge of range shown at minimap edge)
  - [x] 3.5 Spawn/despawn blip UI nodes as entities enter/leave range
  - [x] 3.6 Color blips by entity type: gray for Asteroid, red for ScoutDrone

- [x] Task 4: Blip lifecycle management (AC: #6, #7)
  - [x] 4.1 Track which entities currently have visible blips (MinimapState HashMap<Entity, Entity>)
  - [x] 4.2 Remove blips when source entity is despawned (chunk unload, destruction)
  - [x] 4.3 Remove blips when source entity moves out of scanner range
  - [x] 4.4 Efficient proximity check: use squared distance comparison (avoid sqrt)

- [x] Task 5: Unit tests (AC: all) — 11 tests
  - [x] 5.1 `MinimapConfig` RON parsing and Default validation (3 tests)
  - [x] 5.2 World-to-minimap coordinate conversion: correct scaling, Y-flip, and center (3 tests)
  - [x] 5.3 Blip clamping: entity beyond minimap radius clamped to edge (1 test)
  - [x] 5.4 Scanner range filtering: entity inside/outside/boundary (3 tests)
  - [x] 5.5 BlipType color mapping returns expected colors (1 test)

- [x] Task 6: Integration tests (AC: all) — 5 tests
  - [x] 6.1 Minimap blips appear for nearby asteroids within scanner range
  - [x] 6.2 Blips disappear when entity despawned
  - [x] 6.3 Blips update position when player moves
  - [x] 6.4 No blips for entities outside scanner range
  - [x] 6.5 Both asteroid and drone types show blips (multiple entity types)

## Dev Notes

### Architecture Patterns & Constraints

- **Core/Rendering separation MUST be maintained:** The minimap is purely a rendering/UI concern. It reads `Transform`, `Asteroid`, `ScoutDrone` components but does NOT modify them. No new components on game entities — the minimap queries existing components.
- **Bevy UI for in-game surfaces:** Architecture decision #1 mandates Bevy UI (native) for all in-game UI. Do NOT use `bevy_egui` for the minimap — that's dev tools only.
- **No new shared components needed:** The minimap reads existing `Asteroid`, `ScoutDrone`, `Player` marker components and `Transform`. No changes to `src/shared/` or `src/core/`.
- **Config pattern:** `MinimapConfig` follows the exact same pattern as `WorldConfig`, `FlightConfig`, `WeaponConfig`, `BiomeConfig`: `from_ron()` method, `Default` impl, `warn!` on parse error.
- **No unwrap():** `#[deny(clippy::unwrap_used)]` is enforced project-wide. Use `.expect("msg")` in tests.
- **System scheduling:** Minimap rendering runs in `Update` (frame-dependent visual updates), NOT `FixedUpdate`.
- **Entity budget unchanged:** Minimap blip entities are UI nodes, not collidable game entities. They do not count toward the 200 entity budget.

### Implementation Strategy

- **Bevy UI Approach:** Use Bevy's `Node` component system with absolute positioning. The minimap is a `Node` container positioned in the top-right corner. Blips are child `Node` elements with `BackgroundColor` set to their type's color. This avoids a second camera and keeps things simple.
- **Coordinate Mapping:** `minimap_pos = (entity_world_pos - player_world_pos) / scanner_range * minimap_radius`. Clamp magnitude to `minimap_radius` for entities at edge of range.
- **Blip Tracking:** Use a `MinimapBlips` resource with `HashMap<Entity, Entity>` mapping game entity → minimap blip UI entity. When game entity despawns, detect via `RemovedComponents<Asteroid>` or similar, and despawn the blip.
- **Performance:** Distance check uses squared distances (no sqrt). Only query entities with `Collider` component (which all asteroids and drones have). Expected entity count: ≤200 entities in the budget, so a full query is fast.

### Bevy 0.18 UI Notes

- **Bevy UI** uses `Node` component with `Style` for layout (width, height, position_type, left, top, etc.)
- **Circular minimap:** Use `BorderRadius::all(Val::Percent(50.0))` on the container node for circular shape
- **Blip positioning:** Use `Style { position_type: PositionType::Absolute, left: Val::Px(...), top: Val::Px(...) }` for each blip within the minimap container
- **Semi-transparent background:** `BackgroundColor(Color::srgba(0.0, 0.0, 0.1, 0.6))` for dark blue translucent
- **Z-ordering:** Minimap container should be on a high z-index UI layer so it renders above game world

### Code Patterns from Story 1.2 (MUST REUSE)

```rust
// Config loading pattern (same as BiomeConfig, WorldConfig, etc.)
impl MinimapConfig {
    pub fn from_ron(data: &str) -> Self {
        ron::from_str(data).unwrap_or_else(|e| {
            warn!("Failed to parse minimap.ron: {e}, using defaults");
            Self::default()
        })
    }
}

// Entity querying pattern — read-only access to game entities
fn update_minimap_blips(
    player_query: Query<&Transform, With<Player>>,
    entity_query: Query<(Entity, &Transform), Or<(With<Asteroid>, With<ScoutDrone>)>>,
    config: Res<MinimapConfig>,
    // ... minimap blip management
) { ... }
```

### Previous Story Intelligence (from Stories 1.1 and 1.2)

**Learnings to apply:**
1. **Config loading is reliable** — `from_ron()` + `Default` pattern used in all 5 existing configs. Reuse exactly.
2. **Test harness needs updating** — `tests/helpers/mod.rs` must add `MinimapConfig` resource.
3. **ActiveChunks is HashMap<ChunkCoord, BiomeType>** — can be used to understand what's loaded near the player.
4. **Entity budget enforcement** — minimap blip UI entities must NOT be counted in the entity budget (they have no `Collider` component, so the existing budget query won't include them).
5. **Chunk unload despawns entities** — blip tracking must handle `RemovedComponents` or orphan cleanup when chunk entities are despawned.
6. **Rendering plugin Startup** — all asset caching (LaserAssets, AsteroidAssets, etc.) happens at Startup. MinimapConfig can also be loaded at Startup.

**Code review patterns to watch for:**
- Blip positioning off-by-one when player is at chunk boundary
- Blip leak: game entity despawned but minimap blip entity not cleaned up
- Division by zero if scanner_range is 0
- UI node accumulation: blips spawned but never despawned causing memory/entity growth
- Minimap coordinate flip: Bevy UI Y-axis is top-down, world Y-axis is bottom-up

### Project Structure Notes

- **Files to create:**
  - `src/rendering/minimap.rs` — BlipType, MinimapConfig, minimap systems
  - `assets/config/minimap.ron` — Minimap configuration
- **Files to modify:**
  - `src/rendering/mod.rs` — Add `pub mod minimap`, register minimap systems, load MinimapConfig
  - `tests/helpers/mod.rs` — Add `MinimapConfig` to test harness
  - `tests/world_generation.rs` — Add minimap-specific integration tests (or new test file)
- **Files NOT to touch:**
  - `src/core/` — No core changes needed
  - `src/world/` — No world changes needed (minimap reads, never writes)
  - `src/shared/` — No shared component changes
  - `assets/config/world.ron` — World config unchanged
  - `assets/config/biome.ron` — Biome config unchanged

### Key Libraries

- **`bevy` 0.18.0** — `Node`, `Style`, `BackgroundColor`, `BorderRadius`, `PositionType`, `Val`, `Query`, `With`, `Or`, `Entity`, `Commands`, `Res`, `Transform`
- **`ron`** + **`serde`** (already in Cargo.toml) — Config deserialization for MinimapConfig
- **NO new dependencies needed**

### References

- [Source: _bmad-output/game-architecture.md — "UI Framework" decision: Bevy UI native for in-game]
- [Source: _bmad-output/game-architecture.md — "System Ordering" — rendering in Update]
- [Source: _bmad-output/game-architecture.md — "Rendering Separation" — game logic never touches rendering directly]
- [Source: _bmad-output/game-architecture.md — "Project Structure" — src/rendering/minimap.rs, src/ui/hud.rs]
- [Source: _bmad-output/game-architecture.md — "Entity Budget" — 200 entities target]
- [Source: _bmad-output/gdd.md — "Discover" loop: minimap blips as primary engagement driver]
- [Source: _bmad-output/gdd.md — "Minimap scanning: Passive — always shows nearby points of interest as unidentified blips"]
- [Source: _bmad-output/gdd.md — "Minimap blips pulse gently. New discoveries get a brief highlight."]
- [Source: _bmad-output/epics.md — Epic 1 Story 3: "As a player, I see unidentified blips on my minimap"]
- [Source: _bmad-output/implementation-artifacts/1-2-biome-types.md — BiomeType, ActiveChunks as HashMap]
- [Source: _bmad-output/implementation-artifacts/1-1-seamless-world-generation.md — Chunk system, entity budget, ChunkEntity]

### Git Intelligence

- **Last commit:** `feat(1.2): biome types — deep space, asteroid fields, wreck fields, deterministic biome assignment, per-biome entity density, BiomeConfig. 18 new tests (13 unit + 5 integration).`
- **Commit convention:** `feat(1.3): minimap blips — scanner range, blip types, Bevy UI minimap rendering, MinimapConfig. X new tests (Y unit + Z integration).`
- **VCS:** Using `jj` (Jujutsu). Use `jj new` after code review marks story done.
- **Total tests before this story:** 180 (105 unit + 75 integration)

## Dev Agent Record

### Agent Model Used

Claude Opus 4.6

### Debug Log References

- Bevy 0.18: `BorderRadius` is a field on `Node`, NOT a separate Component
- RON: `[f32; 4]` arrays serialize as tuples `(1.0, 1.0, 1.0, 1.0)`, not `[1.0, ...]`

### Completion Notes List

- 11 unit tests + 5 integration tests = 16 new tests (total project: 122 unit + 80 integration = 202)
- Pure functions (`world_to_minimap`, `is_in_scanner_range`, `blip_color`) kept testable outside ECS
- Y-axis flip handled in coordinate conversion (world Y-up → UI Y-down)
- MinimapState tracks world entity → blip entity mapping for lifecycle management
- `bevy_ui` feature added to Cargo.toml for Node/BackgroundColor support

### Change Log

- 2026-02-26: Code review fixes — entity_query now uses `Or<(With<Asteroid>, With<ScoutDrone>)>` filter with `Has<Asteroid>` (was unfiltered iterating all entities), `color_station` field added to MinimapConfig (was hardcoded green), `HashMap` import added (was inline `std::collections::HashMap`). 202 total tests, clippy clean.

### File List

- `src/rendering/minimap.rs` — NEW: BlipType, MinimapConfig, pure functions, marker components, MinimapState, setup_minimap, update_minimap_blips, 12 unit tests
- `src/rendering/mod.rs` — MODIFIED: Added `pub mod minimap`, minimap imports, MinimapConfig loading, system registration
- `assets/config/minimap.ron` — NEW: Minimap configuration (scanner_range, colors, sizes, color_station)
- `tests/minimap_blips.rs` — NEW: 5 integration tests for minimap blip lifecycle
- `tests/helpers/mod.rs` — MODIFIED: Added MinimapConfig and MinimapState to test harness
- `Cargo.toml` — MODIFIED: Added `bevy_ui` feature to bevy dependencies
