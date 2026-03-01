# Story 2.11: Gravity Well Visual Feedback

Status: ready-for-dev

## Story

As a player,
I want to see a visible boundary marking the edge of the gravity well's safe zone,
so that I understand where the pull begins before I feel it.

## Acceptance Criteria

1. A circular boundary ring is rendered centered on the `GravityWellGenerator` position at radius `safe_radius`
2. The ring is visually distinct — dashed or semi-transparent, warning color (e.g. orange/red with low alpha)
3. The ring is rendered as a thin circle outline (not filled)
4. When the `GravityWellGenerator` is despawned (destroyed), the boundary ring disappears
5. The ring does not interfere with collision or physics — rendering only
6. The ring is always visible regardless of tutorial phase

## Tasks / Subtasks

- [ ] Task 1: Marker component
  - [ ] Add `GravityWellBoundary` marker component in `src/core/tutorial.rs`
  - [ ] Spawned as a child entity of `GravityWellGenerator` OR as a separate entity tracking generator position

- [ ] Task 2: Boundary rendering system
  - [ ] Add `spawn_gravity_well_boundary` in rendering: spawns a circle outline mesh at `safe_radius`
  - [ ] Use `lyon_tessellation` circle stroke (follow existing lyon patterns in `vector_art.rs`)
  - [ ] Semi-transparent orange material (alpha ~0.4)
  - [ ] Thin stroke width (~4–8 px)

- [ ] Task 3: Boundary follows generator
  - [ ] If spawned as separate entity: add system to track generator Transform
  - [ ] If spawned as child entity: handled automatically by Bevy transform hierarchy

- [ ] Task 4: Cleanup on generator despawn
  - [ ] If child entity: despawned automatically with parent
  - [ ] If separate entity: add system to despawn when generator is gone

- [ ] Task 5: Register in RenderingPlugin

- [ ] Task 6: Tests
  - [ ] Test: boundary entity exists when generator exists
  - [ ] Test: boundary entity has correct radius (matches safe_radius from config)

## Dev Notes

### Recommended Approach: Child Entity
Spawn `GravityWellBoundary` as a child of `GravityWellGenerator` in `spawn_tutorial_zone`. Rendering adds mesh to it. When generator despawns, child despawns automatically — no cleanup system needed.

### Lyon Circle Outline
```rust
// Reference: vector_art.rs uses lyon for all mesh generation
// Use stroke builder with circle path at safe_radius
```

### Existing Code to Reuse
- `src/rendering/vector_art.rs` — lyon stroke patterns
- `src/core/tutorial.rs` — `GravityWellGenerator` spawn in `spawn_tutorial_zone`

### File Structure

| File | Action | Purpose |
|------|--------|---------|
| `src/core/tutorial.rs` | MODIFY | Add `GravityWellBoundary` component, spawn as child |
| `src/rendering/mod.rs` | MODIFY | Add boundary visual system |
| `src/rendering/vector_art.rs` | MODIFY | Add circle outline mesh generator |
| `tests/tutorial_zone.rs` | MODIFY | Boundary existence tests |
