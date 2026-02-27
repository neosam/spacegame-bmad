# Story 0.5: Destroy Asteroids and Drones

Status: review

<!-- Note: Validation is optional. Run validate-create-story for quality check before dev-story. -->

## Story

As a player,
I want to destroy asteroids and Scout Drones,
so that the arcade loop has targets.

## Acceptance Criteria

1. Laser pulses hit asteroids and Scout Drones within range and deal damage
2. Spread projectiles collide with asteroids and Scout Drones and deal damage on impact
3. Asteroids and Scout Drones have health values and are destroyed when health reaches zero
4. Destroyed entities despawn immediately (no delay, no animation in this story)
5. Collision detection uses circle-circle intersection for projectiles and ray-circle for laser hitscan
6. Damage values from `WeaponConfig` are used (laser_damage for Laser, spread_damage for Spread projectiles)
7. Multiple projectiles can hit the same target in a single frame (no invincibility frames)
8. All existing weapon and flight tests continue to pass (no regression)
9. No `unwrap()` in game code — `#[deny(clippy::unwrap_used)]` enforced

## Tasks / Subtasks

- [x] Task 1: Collision Detection System (AC: #1, #2, #5)
  - [x] 1.1 Create `src/core/collision.rs` module with collision detection functions:
    - `ray_circle_intersection(origin: Vec2, direction: Vec2, range: f32, center: Vec2, radius: f32) -> Option<Vec2>` for laser hitscan
    - `circle_circle_intersection(center1: Vec2, radius1: f32, center2: Vec2, radius2: f32) -> bool` for projectile collisions
  - [x] 1.2 Add `Collider` component: `pub struct Collider { pub radius: f32 }` for asteroids and enemies
  - [x] 1.3 Add `Health` component: `pub struct Health { pub current: f32, pub max: f32 }` for destructible entities
  - [x] 1.4 Create `check_laser_collisions` system that reads `LaserFired` messages and checks ray-circle intersection against all entities with `Collider` and `Health`
  - [x] 1.5 Create `check_projectile_collisions` system that checks circle-circle intersection between `SpreadProjectile` entities and entities with `Collider` and `Health`

- [x] Task 2: Damage Application System (AC: #3, #6, #7)
  - [x] 2.1 Create `apply_damage` system in `CoreSet::Damage` that applies damage from collision results to `Health` components
  - [x] 2.2 Damage values: Use `WeaponConfig.laser_damage` for laser hits, `SpreadProjectile.damage` for projectile hits
  - [x] 2.3 Multiple hits in same frame: Allow multiple projectiles to hit same target (no invincibility)
  - [x] 2.4 Create `despawn_destroyed` system that despawns entities when `Health.current <= 0.0`

- [x] Task 3: Entity Setup (AC: #3, #4)
  - [x] 3.1 Add `Collider` and `Health` components to asteroid entities (spawned in future story, but prepare component structure)
  - [x] 3.2 Add `Collider` and `Health` components to Scout Drone entities (spawned in future story, but prepare component structure)
  - [x] 3.3 Health values: Asteroids have higher health than Scout Drones (configurable via future spawn config)

- [x] Task 4: System Registration (AC: #1, #2, #5)
  - [x] 4.1 Register collision systems in `CorePlugin`:
    - `check_laser_collisions` in `CoreSet::Collision` (after weapon systems, before Damage)
    - `check_projectile_collisions` in `CoreSet::Collision` (after weapon systems, before Damage)
  - [x] 4.2 Register damage systems in `CorePlugin`:
    - `apply_damage` in `CoreSet::Damage` (after Collision)
    - `despawn_destroyed` in `CoreSet::Damage` (after apply_damage)

- [x] Task 5: Tests (AC: #1, #2, #3, #4, #5, #6, #7, #8)
  - [x] 5.1 Unit tests in `src/core/collision.rs`:
    - Ray-circle intersection: ray hits circle
    - Ray-circle intersection: ray misses circle
    - Ray-circle intersection: ray stops before circle (range limit)
    - Circle-circle intersection: circles overlap
    - Circle-circle intersection: circles do not overlap
  - [x] 5.2 Integration tests in `tests/collision_damage.rs`:
    - Laser pulse hits asteroid and deals damage
    - Laser pulse hits Scout Drone and deals damage
    - Spread projectile hits asteroid and deals damage
    - Spread projectile hits Scout Drone and deals damage
    - Entity despawns when health reaches zero
    - Multiple projectiles can hit same target in one frame
    - Laser damage uses `WeaponConfig.laser_damage`
    - Spread damage uses `SpreadProjectile.damage`
  - [x] 5.3 Verify all existing tests still pass (no regression)

## Dev Notes

### Architecture Patterns and Constraints

- **Collision Detection:** Custom circle-circle and ray-circle math (no physics engine). [Source: game-architecture.md#Physics / Flight Model]
- **System Ordering:** Collision runs after weapon systems, Damage runs after Collision. [Source: game-architecture.md#System Ordering]
- **Hitscan Laser:** Ray-circle intersection for instant hit detection. [Source: gdd.md#Hit Detection]
- **Projectile Collision:** Circle-circle collision for physical projectiles. [Source: game-architecture.md#Physics / Flight Model]
- **No Invincibility Frames:** Multiple hits allowed in same frame for satisfying multi-projectile impacts. [Source: AC #7]
- **Immediate Despawn:** No delay or animation — destroyed entities vanish instantly. [Source: AC #4]
- **No `unwrap()`** — enforced via `#[deny(clippy::unwrap_used)]`. [Source: game-architecture.md#Error Handling]

### Existing Infrastructure (from Stories 0.1-0.4)

**Already implemented — DO NOT recreate:**
- `LaserFired` message with `origin`, `direction`, `range` — in `src/core/weapons.rs:133-138`
- `SpreadFired` message with `origin`, `direction`, `count` — in `src/core/weapons.rs:141-146`
- `LaserPulse` component with `origin`, `direction`, `range` — in `src/core/weapons.rs:97-103`
- `SpreadProjectile` component with `origin`, `direction`, `speed`, `damage`, `timer` — in `src/core/weapons.rs:112-119`
- `WeaponConfig` with `laser_damage` and `spread_damage` — in `src/core/weapons.rs:11-40`
- `CoreSet::Collision` and `CoreSet::Damage` system sets — in `src/core/mod.rs:25-28`
- System ordering chain: Input → Physics → Collision → Damage — in `src/core/mod.rs:76-85`
- `Velocity` component in `src/shared/components.rs` for entity positions

**What's missing (implement in this story):**
1. No collision detection functions (ray-circle, circle-circle)
2. No `Collider` component for collision radius
3. No `Health` component for destructible entities
4. No systems that read `LaserFired`/`SpreadFired` messages and check collisions
5. No damage application system
6. No despawn system for destroyed entities
7. No tests for collision and damage

### System Ordering

```
PreUpdate: read_input (sets switch_weapon = true on Tab press)
FixedUpdate:
  CoreSet::Input    → tick_fire_cooldown, regenerate_energy, switch_weapon
  CoreSet::Physics  → apply_thrust, apply_rotation, apply_drag, apply_velocity
  (after Physics)   → fire_weapon, tick_laser_pulses, move_spread_projectiles, tick_spread_projectiles
  CoreSet::Collision → check_laser_collisions, check_projectile_collisions (NEW)
  CoreSet::Damage    → apply_damage, despawn_destroyed (NEW)
```

The collision systems run after projectiles have moved (`move_spread_projectiles`), and damage systems run after collision detection completes.

### Collision Detection Implementation

**Ray-Circle Intersection (Laser Hitscan):**

```rust
/// Checks if a ray intersects a circle, returns hit point if found.
/// Returns None if ray misses or stops before reaching circle.
pub fn ray_circle_intersection(
    origin: Vec2,
    direction: Vec2,
    range: f32,
    center: Vec2,
    radius: f32,
) -> Option<Vec2> {
    // Vector from origin to circle center
    let to_center = center - origin;
    
    // Project to_center onto direction to find closest point on ray
    let projection_length = to_center.dot(direction);
    
    // If projection is negative, circle is behind origin
    if projection_length < 0.0 {
        return None;
    }
    
    // If projection exceeds range, circle is beyond ray range
    if projection_length > range {
        return None;
    }
    
    // Closest point on ray to circle center
    let closest_point = origin + direction * projection_length;
    
    // Distance from closest point to circle center
    let distance_to_center = closest_point.distance(center);
    
    // If distance is less than radius, ray hits circle
    if distance_to_center <= radius {
        Some(closest_point)
    } else {
        None
    }
}
```

**Circle-Circle Intersection (Projectile Collision):**

```rust
/// Checks if two circles intersect.
pub fn circle_circle_intersection(
    center1: Vec2,
    radius1: f32,
    center2: Vec2,
    radius2: f32,
) -> bool {
    let distance = center1.distance(center2);
    distance <= (radius1 + radius2)
}
```

### Damage Application Pattern

```rust
/// Applies damage from collision results to Health components.
pub fn apply_damage(
    mut query: Query<&mut Health>,
    // Collision results stored in a resource or component
) {
    for (entity, mut health) in query.iter_mut() {
        // Apply accumulated damage for this entity
        health.current = (health.current - damage_amount).max(0.0);
    }
}

/// Despawns entities with zero or negative health.
pub fn despawn_destroyed(
    mut commands: Commands,
    query: Query<(Entity, &Health)>,
) {
    for (entity, health) in query.iter() {
        if health.current <= 0.0 {
            commands.entity(entity).despawn();
        }
    }
}
```

### Laser Collision System Pattern

```rust
/// Checks LaserFired messages against all collidable entities.
pub fn check_laser_collisions(
    mut laser_reader: MessageReader<LaserFired>,
    config: Res<WeaponConfig>,
    colliders: Query<(Entity, &Transform, &Collider, &mut Health)>,
) {
    for laser in laser_reader.read() {
        // Find closest hit within range
        let mut closest_hit: Option<(Entity, Vec2)> = None;
        let mut closest_distance = laser.range;
        
        for (entity, transform, collider, mut health) in colliders.iter_mut() {
            let center = Vec2::new(transform.translation.x, transform.translation.y);
            
            if let Some(hit_point) = ray_circle_intersection(
                laser.origin,
                laser.direction,
                laser.range,
                center,
                collider.radius,
            ) {
                let distance = laser.origin.distance(hit_point);
                if distance < closest_distance {
                    closest_distance = distance;
                    closest_hit = Some((entity, hit_point));
                }
            }
        }
        
        // Apply damage to closest hit
        if let Some((entity, _)) = closest_hit {
            // Store damage to apply in Damage set
            // Use a resource or component to pass damage to apply_damage system
        }
    }
}
```

### Projectile Collision System Pattern

```rust
/// Checks SpreadProjectile entities against collidable entities.
pub fn check_projectile_collisions(
    mut commands: Commands,
    projectiles: Query<(Entity, &Transform, &SpreadProjectile)>,
    colliders: Query<(Entity, &Transform, &Collider, &mut Health)>,
) {
    for (proj_entity, proj_transform, projectile) in projectiles.iter() {
        let proj_center = Vec2::new(proj_transform.translation.x, proj_transform.translation.y);
        let proj_radius = 2.0; // Small radius for projectiles
        
        for (target_entity, target_transform, collider, mut health) in colliders.iter_mut() {
            let target_center = Vec2::new(target_transform.translation.x, target_transform.translation.y);
            
            if circle_circle_intersection(proj_center, proj_radius, target_center, collider.radius) {
                // Store damage to apply in Damage set
                // Despawn projectile immediately
                commands.entity(proj_entity).despawn();
                break; // Projectile can only hit one target
            }
        }
    }
}
```

### Bevy 0.18 Gotchas

- **Messages:** `#[derive(Message)]`, `MessageWriter`, `.write()`, `MessageReader`, `.read()` — NOT events
- **System Ordering:** Use `SystemSet` with `.chain()` for explicit ordering
- **Query Mutability:** `&mut Health` requires mutable query, but collision detection can use `&Health` for read-only checks
- **Despawn:** `commands.entity(entity).despawn()` removes entity at end of frame, not immediately
- **First `app.update()` has dt=0** — prime in tests
- **`TimeUpdateStrategy::ManualDuration`** for deterministic tests

### What This Story Does NOT Include

- **No asteroid/enemy spawning** — entities are spawned in future stories, but collision/damage systems must work when they exist
- **No visual feedback on hit** — screen shake, particles, flashes deferred to Story 0.6
- **No death animations** — immediate despawn per AC #4
- **No invincibility frames** — multiple hits allowed per AC #7
- **No health bars/UI** — HUD deferred to future stories
- **No damage numbers** — visual feedback deferred to Story 0.6

### Previous Story Intelligence (Story 0.4)

**Patterns established:**
- `ActiveWeapon` enum with `#[derive(Component, Default, Debug, Clone, Copy, PartialEq, Eq)]`
- `fire_weapon` system branches on `match active_weapon { Laser => ..., Spread => ... }`
- `Energy` initialized from `WeaponConfig.energy_max` in `setup_player`
- `switch_weapon` system in `CoreSet::Input` runs before `fire_weapon`
- Test helper `test_app()` includes all weapon systems + flight systems
- `LaserFired` and `SpreadFired` messages are emitted by `fire_weapon` system

**Code review fixes from 0.4:**
- Unused imports removed
- Clarifying comments added for gamepad button mapping

### Project Structure Notes

| File | Action | Purpose |
|------|--------|---------|
| `src/core/collision.rs` | CREATE | Collision detection functions (ray-circle, circle-circle) |
| `src/core/collision.rs` | CREATE | `Collider` and `Health` components |
| `src/core/collision.rs` | CREATE | `check_laser_collisions` and `check_projectile_collisions` systems |
| `src/core/collision.rs` | CREATE | `apply_damage` and `despawn_destroyed` systems |
| `src/core/mod.rs` | MODIFY | Import collision module and register systems in appropriate sets |
| `tests/collision_damage.rs` | CREATE | Integration tests for collision and damage |

### References

- [Source: epics.md#Epic 0, Story 5] — "As a player, I can destroy asteroids and Scout Drones so that the arcade loop has targets"
- [Source: gdd.md#Hit Detection] — Laser hitscan via raycast, projectiles via physics collision
- [Source: game-architecture.md#Physics / Flight Model] — Custom circle-circle and ray-circle collision math
- [Source: game-architecture.md#System Ordering] — Collision after Physics, Damage after Collision
- [Source: game-architecture.md#Error Handling] — No `unwrap()` in game code
- [Source: 0-4-switch-weapons.md#Dev Notes] — System ordering, message patterns, test helpers

## Dev Agent Record

### Agent Model Used

Claude claude-4.6-opus (via Cursor)

### Debug Log References

No debug issues encountered. All tasks implemented cleanly in a single pass.

### Completion Notes List

- Implemented `ray_circle_intersection` and `circle_circle_intersection` pure math functions in `src/core/collision.rs`
- Created `Collider` component (radius-based circle collider) and `Health` component (current/max health pool)
- Created `DamageQueue` resource to buffer damage between Collision and Damage system sets
- Implemented `check_laser_collisions` system: reads `LaserFired` messages, finds closest ray-circle hit, pushes damage to DamageQueue using `WeaponConfig.laser_damage`
- Implemented `check_projectile_collisions` system: checks circle-circle intersection between `SpreadProjectile` entities and colliders, pushes damage using `SpreadProjectile.damage`, despawns projectile on hit
- Implemented `apply_damage` system: drains DamageQueue and applies accumulated damage to Health components (allows multiple hits per frame — no invincibility frames)
- Implemented `despawn_destroyed` system: despawns entities with `Health.current <= 0.0`
- Registered all systems in CorePlugin: collision systems in `CoreSet::Collision`, damage systems in `CoreSet::Damage` (chained: apply_damage → despawn_destroyed)
- 5 unit tests for collision math (ray hit, ray miss, ray range limit, circles overlap, circles don't overlap)
- 8 integration tests covering all acceptance criteria (laser/spread vs asteroid/drone, despawn, multiple hits, damage values)
- Updated test helper `test_app()` to include collision and damage systems + DamageQueue resource
- Added `spawn_asteroid` and `spawn_drone` test helpers
- Full regression suite: 68 tests passing, 0 failures
- Clippy clean (no new warnings)
- No `unwrap()` in game code

### Change Log

- 2026-02-26: Implemented collision detection and damage system — ray-circle for laser hitscan, circle-circle for projectiles. DamageQueue buffers damage between system sets. 5 unit tests + 8 integration tests added.

### File List

- `src/core/collision.rs` — CREATED: Collision math functions, Collider/Health components, DamageQueue resource, check_laser_collisions, check_projectile_collisions, apply_damage, despawn_destroyed systems, 5 unit tests
- `src/core/mod.rs` — MODIFIED: Added `pub mod collision`, imported collision systems and DamageQueue, registered systems in CoreSet::Collision and CoreSet::Damage
- `tests/collision_damage.rs` — CREATED: 8 integration tests for collision and damage (laser/spread vs asteroid/drone, despawn, multiple hits, damage values)
- `tests/helpers/mod.rs` — MODIFIED: Added collision/damage system imports and registration in test_app(), added spawn_asteroid() and spawn_drone() helpers

