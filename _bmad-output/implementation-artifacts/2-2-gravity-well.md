# Story 2.2: Gravity Well

Status: done

<!-- Note: Validation is optional. Run validate-create-story for quality check before dev-story. -->

## Story

As a player,
I want to feel the gravity well pull me back when I fly too far,
so that I understand I need to stay and explore this area first.

## Acceptance Criteria

1. When the player is within `safe_radius` of the gravity well generator, no pull force is applied
2. When the player exceeds `safe_radius`, a linear pull force is applied: `pull_force = max(0, (distance - safe_radius) * pull_strength)`
3. The pull force direction is always toward the gravity well generator position
4. The pull force modifies the player's `Velocity` component, not their position directly
5. The gravity well system runs in `FixedUpdate` for deterministic physics
6. The gravity well system runs after `CoreSet::Physics` (after thrust/drag/velocity) so the player feels the tug as a competing force
7. Pull strength and safe_radius are read from the existing `GravityWellGenerator` component (already spawned by Story 2-1)
8. When the generator entity is despawned (destroyed), the gravity well effect ceases immediately
9. The gravity well only affects entities with the `Player` component
10. At extreme distances, the pull force is strong enough to prevent the player from escaping indefinitely

## Tasks / Subtasks

- [x] Task 1: Gravity well physics system (AC: #1, #2, #3, #4, #5, #6, #7, #8, #9)
  - [x] Add `apply_gravity_well` system in `src/core/tutorial.rs`
  - [x] Query `GravityWellGenerator` + `Transform` to find active generators
  - [x] Query `Player` + `Transform` + `Velocity` for affected entities
  - [x] Compute distance from player to generator
  - [x] If `distance > safe_radius`: compute `pull_force = (distance - safe_radius) * pull_strength`
  - [x] Apply force as `velocity -= direction_away * pull_force * dt` (pull toward generator)
  - [x] Register system in `CoreSet::Physics` or after it, before `CoreSet::Collision`
- [x] Task 2: System registration in CorePlugin (AC: #5, #6)
  - [x] Register `apply_gravity_well` in `FixedUpdate` after `CoreSet::Physics`, before `CoreSet::Collision`
  - [x] Ensure ordering: thrust → drag → velocity → gravity well → collision
- [x] Task 3: Unit tests for force calculation (AC: #1, #2, #3, #4, #10)
  - [x] Test: no force inside safe_radius (velocity unchanged)
  - [x] Test: linear force outside safe_radius (velocity modified toward generator)
  - [x] Test: force increases with distance (farther = stronger pull)
  - [x] Test: force direction is toward generator (not origin)
  - [x] Test: zero force at exact safe_radius boundary
- [x] Task 4: Integration tests (AC: #5, #6, #7, #8, #9)
  - [x] Test: gravity well works with full flight physics pipeline
  - [x] Test: player can fly freely within safe_radius (thrust + no pull)
  - [x] Test: player outside safe_radius gets pulled back over multiple frames
  - [x] Test: no gravity effect when no GravityWellGenerator exists
  - [x] Test: gravity well ceases when generator entity despawned

## Dev Notes

### Architecture Patterns

- **Gravity well is a physics system, not a movement override.** It modifies `Velocity`, letting drag and thrust still function. The player "fights" the pull with thrust — this creates the organic feel described in the GDD.
- **Formula from architecture doc:** `pull_force = max(0, (distance - safe_radius) * pull_strength)` — linear, predictable, learnable.
- **System ordering:** Must run after `apply_velocity` (CoreSet::Physics) so the player's position is updated before computing gravity. Must run before `CoreSet::Collision` to ensure velocity is correct for collision detection.
- **Component-based:** Uses existing `GravityWellGenerator` component from Story 2-1. No new components needed.
- **No rendering in this story.** Visual effects (distortion at field edge, station sparks) are future stories.

### Existing Code to Reuse (DO NOT Reinvent)

- `src/core/tutorial.rs` — `GravityWellGenerator` component (safe_radius, pull_strength, requires_projectile) already defined
- `src/core/flight.rs` — `Player` component, `Velocity` usage pattern, `apply_thrust`/`apply_drag` for reference on velocity modification
- `src/shared/components.rs` — `Velocity(pub Vec2)` component
- `src/core/mod.rs` — `CoreSet` enum for system ordering, existing system registration patterns
- `tests/helpers/mod.rs` — `test_app()` harness with all required resources

### Implementation Guidance

```rust
/// Apply gravity well pull to entities outside the safe radius.
/// pull_force = max(0, (distance - safe_radius) * pull_strength)
/// Force is applied as velocity change toward the generator.
pub fn apply_gravity_well(
    time: Res<Time>,
    generator_query: Query<(&GravityWellGenerator, &Transform)>,
    mut player_query: Query<(&Transform, &mut Velocity), With<Player>>,
) {
    let dt = time.delta_secs();
    for (gen_comp, gen_transform) in generator_query.iter() {
        let gen_pos = gen_transform.translation.truncate();
        for (player_transform, mut velocity) in player_query.iter_mut() {
            let player_pos = player_transform.translation.truncate();
            let diff = gen_pos - player_pos;
            let distance = diff.length();
            if distance > gen_comp.safe_radius && distance > f32::EPSILON {
                let pull_magnitude = (distance - gen_comp.safe_radius) * gen_comp.pull_strength;
                let direction = diff / distance; // normalized toward generator
                velocity.0 += direction * pull_magnitude * dt;
            }
        }
    }
}
```

### File Structure

| File | Action | Purpose |
|------|--------|---------|
| `src/core/tutorial.rs` | MODIFY | Add `apply_gravity_well` system |
| `src/core/mod.rs` | MODIFY | Register `apply_gravity_well` in FixedUpdate ordering |
| `tests/tutorial_zone.rs` | MODIFY | Add gravity well integration tests |

### Testing Requirements

- **Unit tests** in `src/core/tutorial.rs`:
  - Force calculation at various distances (inside, at boundary, outside safe_radius)
  - Direction correctness (pull toward generator, not origin)
  - Force magnitude linearity
- **Integration tests** in `tests/tutorial_zone.rs`:
  - Full pipeline test: spawn player + generator, run multiple frames, verify velocity changes
  - Safe zone freedom: player inside safe_radius has no pull
  - Pullback test: player far outside eventually gets pulled back
  - Missing generator: no crash, no force applied
- **Pattern:** Use `#[deny(clippy::unwrap_used)]` — use `.expect()` in tests
- **Time:** Use `TimeUpdateStrategy::ManualDuration` for deterministic tests

### Project Structure Notes

- No new files needed — extends existing `src/core/tutorial.rs` module
- System registration follows existing pattern in `src/core/mod.rs`
- Integration tests extend existing `tests/tutorial_zone.rs`
- No new config fields needed — `GravityWellGenerator` already has `safe_radius` and `pull_strength`

### References

- [Source: _bmad-output/epics.md#Epic 2 — Story 2]
- [Source: _bmad-output/game-architecture.md#Gravity Well Tutorial Pattern]
- [Source: _bmad-output/game-architecture.md#Physics System — Gravity Well formula]
- [Source: _bmad-output/game-architecture.md#System Execution Order — FixedUpdate]
- [Source: _bmad-output/gdd.md#Gravity Well Boundary]
- [Source: src/core/tutorial.rs — GravityWellGenerator component]
- [Source: src/core/flight.rs — Velocity modification patterns]
- [Source: src/core/mod.rs — CoreSet system ordering]

### Key Bevy 0.18 Notes

- Query with component: `Query<(&GravityWellGenerator, &Transform)>` — no filter needed, component acts as filter
- `time.delta_secs()` for FixedUpdate dt
- System ordering: `.after(CoreSet::Physics).before(CoreSet::Collision)`
- `transform.translation.truncate()` converts Vec3 → Vec2 for 2D physics

## Dev Agent Record

### Agent Model Used

Claude Opus 4.6

### Debug Log References
- No issues encountered — clean implementation

### Completion Notes List
- Task 1: Implemented `apply_gravity_well` system with linear pull formula `pull_force = max(0, (distance - safe_radius) * pull_strength)`. System queries GravityWellGenerator for parameters and modifies Player Velocity toward generator position.
- Task 2: Registered system in CorePlugin FixedUpdate schedule with `.after(CoreSet::Physics).before(CoreSet::Collision)` ordering.
- Task 3: 6 unit tests in tutorial.rs covering: no force inside safe_radius, no force at boundary, force outside safe_radius, force increases with distance, direction toward generator, no effect without generator.
- Task 4: 4 integration tests in tutorial_zone.rs covering: player inside safe_radius no pull, multi-frame pullback, despawned generator, full tutorial zone pipeline.

### File List
- `src/core/tutorial.rs` — MODIFIED: Added `apply_gravity_well` system and 6 unit tests
- `src/core/mod.rs` — MODIFIED: Imported and registered `apply_gravity_well` in FixedUpdate
- `tests/tutorial_zone.rs` — MODIFIED: Added 4 gravity well integration tests

## Change Log
- 2026-02-28: Implemented Story 2.2 Gravity Well — linear pull physics system with 10 new tests. 354 total tests passing.
