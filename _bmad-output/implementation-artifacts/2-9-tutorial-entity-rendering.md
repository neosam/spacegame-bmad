# Story 2.9: Tutorial Entity Rendering

Status: ready-for-dev

## Story

As a player,
I want to see the tutorial station, wreck, and gravity well generator as visible objects,
so that I know where to go and what to interact with.

## Acceptance Criteria

1. `TutorialStation` is rendered as a recognizable shape (e.g. simple vector polygon or rectangle, distinct color)
2. `TutorialWreck` is rendered as a recognizable shape (e.g. broken/irregular polygon, grey/dark color)
3. `GravityWellGenerator` is rendered as a recognizable shape (e.g. circle or diamond, warning color like orange/red)
4. All three entities are visible when the game starts in the tutorial zone
5. `TutorialStation` visual changes when `defective` transitions from `true` to `false` (after docking: different color or shape)
6. `GravityWellGenerator` visual disappears when the entity is despawned (destruction cascade)
7. `TutorialWreck` visual disappears when the entity is despawned
8. Rendering uses the existing core/rendering separation pattern: core entities are already spawned, rendering adds `Mesh2d` + `MeshMaterial2d` in a Startup or Added system

## Tasks / Subtasks

- [ ] Task 1: Marker components for visual needs
  - [ ] Add `NeedsTutorialStationVisual` marker component (or reuse pattern with `TutorialStation` directly)
  - [ ] Add `NeedsTutorialWreckVisual` marker component
  - [ ] Add `NeedsTutorialGeneratorVisual` marker component (or use existing `GravityWellGenerator` as filter)

- [ ] Task 2: Visual setup system in rendering
  - [ ] Add `setup_tutorial_station_visual` system: spawns Mesh2d + MeshMaterial2d on `TutorialStation` entities Added
  - [ ] Add `setup_tutorial_wreck_visual` system: spawns Mesh2d + MeshMaterial2d on `TutorialWreck` entities Added
  - [ ] Add `setup_tutorial_generator_visual` system: spawns Mesh2d + MeshMaterial2d on `GravityWellGenerator` entities Added

- [ ] Task 3: Defective station color change
  - [ ] Add system `update_tutorial_station_visual` that updates material color when `defective` changes

- [ ] Task 4: Register systems in RenderingPlugin

- [ ] Task 5: Tests
  - [ ] Test: TutorialStation entity has Mesh2d component after setup
  - [ ] Test: TutorialWreck entity has Mesh2d component after setup
  - [ ] Test: GravityWellGenerator entity has Mesh2d component after setup

## Dev Notes

### Architecture Pattern (MUST follow)
- Core already spawns `TutorialStation`, `TutorialWreck`, `GravityWellGenerator` in `spawn_tutorial_zone`
- Rendering plugin adds visual components — use `Added<TutorialStation>` query or Startup system
- Follow the existing `NeedsDroneVisual` → `setup_drone_visual` pattern in `src/rendering/mod.rs`
- DO NOT add rendering to `src/core/tutorial.rs`

### Existing Code to Reuse
- `src/rendering/mod.rs` — `setup_drone_visual`, `generate_drone_mesh` as reference
- `src/rendering/vector_art.rs` — `generate_player_mesh`, `generate_asteroid_mesh` as reference
- `src/core/tutorial.rs` — `TutorialStation`, `TutorialWreck`, `GravityWellGenerator` components

### Suggested Shapes
- TutorialStation: hexagon or rectangle, teal/blue color → grey when defective
- TutorialWreck: irregular quad, dark grey
- GravityWellGenerator: diamond/circle, orange warning color

### File Structure

| File | Action | Purpose |
|------|--------|---------|
| `src/rendering/mod.rs` | MODIFY | Add tutorial visual systems, register in plugin |
| `src/rendering/vector_art.rs` | MODIFY | Add mesh generators for tutorial entities |
| `tests/tutorial_zone.rs` | MODIFY | Add visual component tests |
