# Story 3.4: Material Drops

Status: done

## Story

As a player,
I want to collect material drops from destroyed enemies,
so that I accumulate resources for future upgrades.

## Acceptance Criteria

1. A `MaterialType` enum exists in `src/core/economy.rs` with variants `CommonScrap`, `RareAlloy`, `EnergyCore`
2. When an Asteroid is destroyed, an 80% chance spawns a `CommonScrap` drop at the destruction position
3. When a Scout Drone is destroyed: 60% → `CommonScrap`, 30% → `RareAlloy`, 10% → `EnergyCore` (cumulative: 0–0.6 scrap, 0.6–0.9 alloy, 0.9–1.0 core)
4. A `MaterialDrop` entity has `MaterialDrop` marker, `MaterialType` component, `Transform`, `Collider { radius: 15.0 }`, and `NeedsMaterialDropVisual` marker for rendering
5. `PlayerInventory` resource (`HashMap<MaterialType, u32>`) exists in `src/core/economy.rs`, initialized to empty
6. When the player's `Transform` is within 15.0 + player_collider_radius of a `MaterialDrop`, the drop is despawned and its `MaterialType` is added (+1) to `PlayerInventory`
7. Each pickup emits `GameEventKind::MaterialCollected { material: MaterialType }` (Tier3 severity)
8. `PlayerSave` gains three `#[serde(default)]` fields: `inventory_common_scrap: u32`, `inventory_rare_alloy: u32`, `inventory_energy_core: u32`; `SAVE_VERSION` bumped to `4`; `check_version` accepts 1, 2, 3, and 4
9. `MaterialDropAssets` resource in Rendering holds pre-created mesh/material handles for all three drop types; drops render as small diamonds (8px radius) — grey (CommonScrap), blue (RareAlloy), yellow (EnergyCore)
10. All existing 485 tests remain green
11. New tests in `tests/material_drops.rs`: decide_material_drop_asteroid, decide_material_drop_drone_full_range, pickup_adds_to_inventory, no_double_pickup, inventory_save_load_roundtrip

## Tasks / Subtasks

- [x] Task 1: Extend `src/core/economy.rs` — MaterialType, PlayerInventory, drop systems (AC: #1, #2, #3, #4, #5, #6, #7)
  - [x] Add `MaterialType` enum: `CommonScrap`, `RareAlloy`, `EnergyCore` — derive `Component, Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize`
  - [x] Add `MaterialDrop` marker component: `#[derive(Component, Debug)]`
  - [x] Add `NeedsMaterialDropVisual` marker component: `#[derive(Component, Debug)]`
  - [x] Add `PlayerInventory` resource: `HashMap<MaterialType, u32>` + `Default` (empty map)
  - [x] Add `PendingDropSpawns` buffer resource: `Vec<(MaterialType, Vec2)>`
  - [x] Add pure function `decide_material_drop(entity_type: &str, roll: f32) -> Option<MaterialType>`
  - [x] Add system `queue_material_drops`: reads `MessageReader<GameEvent>`, EnemyDestroyed → calls `decide_material_drop(entity_type, rand::random::<f32>())`, pushes to `PendingDropSpawns`
  - [x] Add system `spawn_material_drops`: drains `PendingDropSpawns`, `Commands::spawn` each drop with `(MaterialDrop, material_type, Transform::from_translation(pos.extend(0.0)), Collider { radius: 15.0 }, NeedsMaterialDropVisual)`
  - [x] Add system `collect_material_drops`: query player `(Transform, Collider)` + query drops `(Entity, Transform, MaterialType, Collider)`; if `distance < player_collider.radius + drop_collider.radius` → despawn drop, add to `PlayerInventory`, push to `PendingPickupEvents`
  - [x] Add `PendingPickupEvents` buffer resource: `Vec<(MaterialType, Vec2, f64)>` — same B0002 pattern as 3-3
  - [x] Add system `emit_pickup_events`: drains `PendingPickupEvents`, emits `GameEventKind::MaterialCollected { material }` via `MessageWriter<GameEvent>`
  - [x] Register all new resources + systems in `CorePlugin::build()`

- [x] Task 2: Add `GameEventKind::MaterialCollected` (AC: #7)
  - [x] In `src/shared/events.rs`, add `MaterialCollected { material: MaterialType }` to `GameEventKind` — requires import of `MaterialType`
  - [x] In `src/infrastructure/events.rs`, add `"MaterialCollected"` → `Tier3` to mappings + `known_keys`; update test count 13 → 14

- [x] Task 3: Save/load integration (AC: #8)
  - [x] Bump `SAVE_VERSION` from 3 → 4 in `src/infrastructure/save/schema.rs`
  - [x] Add `4` to `check_version` allow-list (accepts 1 | 2 | 3 | 4); update tests
  - [x] Add three `#[serde(default)]` fields to `PlayerSave`: `inventory_common_scrap: u32`, `inventory_rare_alloy: u32`, `inventory_energy_core: u32`
  - [x] Update `PlayerSave::from_components` initializer: set all three to 0
  - [x] Update `PlayerSave::from_world`: read `PlayerInventory` resource and set the three fields
  - [x] Update `PlayerSave::apply_to_world`: restore `PlayerInventory` from the three fields
  - [x] Update `save_game` in `src/infrastructure/save/mod.rs`: accept `Res<PlayerInventory>`, set `ps.inventory_*` fields
  - [x] Update `load_game`: accept `ResMut<PlayerInventory>`, restore from loaded fields
  - [x] Fix all `PlayerSave` struct literals in `tests/save_system.rs` (add three `0` fields)
  - [x] Fix `world_save.rs` corrupt_deltas test: update hardcoded schema_version 3 → 4

- [x] Task 4: Rendering — `MaterialDropAssets` + visual attach system (AC: #9)
  - [x] In `src/rendering/vector_art.rs`: add `generate_material_drop_mesh(radius: f32) -> Mesh` — rhombus (4 vertices, lyon polygon)
  - [x] In `src/rendering/mod.rs`: add `MaterialDropAssets` resource with 3 mesh handles + 3 material handles
  - [x] Add `setup_material_drop_assets` Startup system: build meshes via `generate_material_drop_mesh(8.0)`, insert materials (grey #808080, blue #4488FF, yellow #FFD700), store in `MaterialDropAssets`
  - [x] Add `attach_material_drop_visual` Update system: query `(Entity, &MaterialType, With<NeedsMaterialDropVisual>)` — insert `Mesh2d` + `MeshMaterial2d` handles from `MaterialDropAssets`, remove `NeedsMaterialDropVisual` marker
  - [x] Register both systems in `RenderingPlugin::build()`
  - [x] Import `MaterialDrop, MaterialType, NeedsMaterialDropVisual, PlayerInventory` in `src/rendering/mod.rs`

- [x] Task 5: Integration tests `tests/material_drops.rs` (AC: #10, #11)
  - [x] `decide_material_drop_asteroid`: roll < 0.8 → Some(CommonScrap); roll >= 0.8 → None
  - [x] `decide_material_drop_drone_full_range`: roll < 0.6 → CommonScrap; 0.6–0.9 → RareAlloy; 0.9–1.0 → EnergyCore
  - [x] `pickup_adds_to_inventory`: spawn player + MaterialDrop at same position, run collect system, verify inventory
  - [x] `no_double_pickup`: after first pickup frame, drop entity is gone; second update doesn't crash or double-count
  - [x] `inventory_save_load_roundtrip`: build PlayerSave with inventory fields set, serialize/deserialize, verify all three counts survive

## Dev Notes

### Architecture Rules

- **Core/Rendering separation STRICTLY enforced**: `MaterialType`, `MaterialDrop`, `NeedsMaterialDropVisual`, `PlayerInventory`, drop spawn systems → `src/core/economy.rs` only. Mesh generation → `src/rendering/`. Never Rendering in Core.
- **B0002 pattern from Story 3-3**: `collect_material_drops` MUST NOT also call `MessageWriter<GameEvent>`. Use `PendingPickupEvents` buffer (same pattern as `PendingCreditEvents`). The `emit_pickup_events` system drains it.
- **Multiple MessageReaders are fine**: Both `award_credits_on_kill` and `queue_material_drops` can read `Messages<GameEvent>` independently — each has its own read cursor. Only read+write in the SAME system causes B0002.
- **`despawn` in collect system**: Use `commands.entity(drop_entity).despawn()` — standard pattern.
- **No `unwrap()` in tests** — always `.expect("description")`.

### Key Pure Function: `decide_material_drop`

```rust
/// Pure function for testability — takes a pre-rolled f32 in [0.0, 1.0).
/// Returns Some(MaterialType) if a drop occurs, None otherwise.
pub fn decide_material_drop(entity_type: &str, roll: f32) -> Option<MaterialType> {
    match entity_type {
        "asteroid" => {
            if roll < 0.8 { Some(MaterialType::CommonScrap) } else { None }
        }
        "drone" => {
            if roll < 0.6 {
                Some(MaterialType::CommonScrap)
            } else if roll < 0.9 {
                Some(MaterialType::RareAlloy)
            } else {
                Some(MaterialType::EnergyCore)
            }
        }
        _ => None, // unknown entity types drop nothing
    }
}
```

### B0002-safe collect + emit pattern

```rust
/// SYSTEM 1: Detects pickups, updates inventory, buffers for event emission.
/// Does NOT write GameEvent directly — avoids B0002 with reader systems.
pub fn collect_material_drops(
    mut commands: Commands,
    player_query: Query<(&Transform, &Collider), With<Player>>,
    drop_query: Query<(Entity, &Transform, &MaterialType, &Collider), With<MaterialDrop>>,
    mut inventory: ResMut<PlayerInventory>,
    mut pending: ResMut<PendingPickupEvents>,
    time: Res<Time>,
) {
    let Ok((player_transform, player_collider)) = player_query.get_single() else { return; };
    let player_pos = player_transform.translation.truncate();

    for (drop_entity, drop_transform, material_type, drop_collider) in drop_query.iter() {
        let drop_pos = drop_transform.translation.truncate();
        let pickup_radius = player_collider.radius + drop_collider.radius;
        if player_pos.distance(drop_pos) <= pickup_radius {
            *inventory.items.entry(*material_type).or_insert(0) += 1;
            commands.entity(drop_entity).despawn();
            pending.events.push((*material_type, drop_pos, time.elapsed_secs_f64()));
        }
    }
}

/// SYSTEM 2: Drains pending pickups and emits GameEvents (no reader here).
pub fn emit_pickup_events(
    mut pending: ResMut<PendingPickupEvents>,
    mut game_events: MessageWriter<GameEvent>,
    severity_config: Res<EventSeverityConfig>,
) {
    for (material, position, game_time) in pending.events.drain(..) {
        let kind = GameEventKind::MaterialCollected { material };
        game_events.write(GameEvent {
            severity: severity_config.severity_for(&kind),
            kind,
            position,
            game_time,
        });
    }
}
```

### System Registration in CorePlugin

```rust
// Material drop systems: after CoreSet::Events (same ordering as credits systems)
app.init_resource::<PlayerInventory>();
app.init_resource::<PendingDropSpawns>();
app.init_resource::<PendingPickupEvents>();
app.add_systems(
    FixedUpdate,
    (queue_material_drops, spawn_material_drops, collect_material_drops, emit_pickup_events)
        .chain()
        .after(CoreSet::Events),
);
```

### Save Schema v3 → v4

`PlayerSave` gains three individual fields (NOT a HashMap — avoids RON enum key serialization complexity):
```rust
#[serde(default)] pub inventory_common_scrap: u32,
#[serde(default)] pub inventory_rare_alloy: u32,
#[serde(default)] pub inventory_energy_core: u32,
```

`from_world` reads from `PlayerInventory`:
```rust
save.inventory_common_scrap = world.get_resource::<PlayerInventory>()
    .and_then(|inv| inv.items.get(&MaterialType::CommonScrap).copied())
    .unwrap_or(0);
// repeat for RareAlloy, EnergyCore
```

`apply_to_world` restores inventory:
```rust
if let Some(mut inv) = world.get_resource_mut::<PlayerInventory>() {
    if self.inventory_common_scrap > 0 {
        inv.items.insert(MaterialType::CommonScrap, self.inventory_common_scrap);
    }
    // repeat for RareAlloy, EnergyCore
}
```

`check_version` updated:
```rust
if header.schema_version != SAVE_VERSION
    && header.schema_version != 1
    && header.schema_version != 2
    && header.schema_version != 3
{
    return Err(SaveError::VersionMismatch { ... });
}
```

### MaterialCollected in events.rs

`GameEventKind::MaterialCollected` needs `MaterialType` — import from `crate::core::economy`:
```rust
// src/shared/events.rs
use crate::core::economy::MaterialType;
// ...
/// Player picked up a material drop.
MaterialCollected { material: MaterialType },
```

**Warning**: This creates a dependency from `shared/events.rs` → `core/economy.rs`. Check if this causes any circular import. The existing pattern has `core/economy.rs` importing `shared/events.rs` (for `GameEvent`). A reverse import would be circular!

**Solution**: Define `MaterialType` in `src/shared/` rather than `src/core/economy.rs`. Add to `src/shared/components.rs` or create `src/shared/materials.rs`. Then both `events.rs` and `economy.rs` import from `shared/`.

**Revised placement**: `MaterialType`, `MaterialDrop`, `NeedsMaterialDropVisual` → `src/shared/components.rs`; `PlayerInventory`, `PendingDropSpawns`, `PendingPickupEvents`, drop systems → `src/core/economy.rs`.

### Rendering: Diamond Mesh

Rhombus with half-width=8, half-height=8 (45° rotated square):
```rust
// In src/rendering/vector_art.rs
pub fn generate_material_drop_mesh(half_size: f32) -> Mesh {
    use lyon_tessellation::*;
    let mut geometry: VertexBuffers<[f32; 2], u32> = VertexBuffers::new();
    let mut builder = geometry.builder();
    // Diamond: top, right, bottom, left
    let path = {
        let mut pb = Path::builder();
        pb.begin(point(0.0, half_size));
        pb.line_to(point(half_size, 0.0));
        pb.line_to(point(0.0, -half_size));
        pb.line_to(point(-half_size, 0.0));
        pb.close();
        pb.build()
    };
    let mut tessellator = FillTessellator::new();
    tessellator.tessellate_path(
        &path,
        &FillOptions::default(),
        &mut BuffersBuilder::new(&mut geometry, |v: FillVertex| v.position().to_array()),
    ).expect("Diamond tessellation succeeded");
    // ... build Mesh from geometry (same pattern as generate_tutorial_generator_mesh)
}
```

Look at `generate_tutorial_generator_mesh` in `src/rendering/vector_art.rs` for the exact mesh construction pattern (VertexBuffers → Mesh with positions + normals + uvs + indices).

### Rendering: MaterialDropAssets

```rust
#[derive(Resource, Default)]
pub struct MaterialDropAssets {
    pub common_scrap_mesh: Handle<Mesh>,
    pub common_scrap_material: Handle<ColorMaterial>,
    pub rare_alloy_mesh: Handle<Mesh>,
    pub rare_alloy_material: Handle<ColorMaterial>,
    pub energy_core_mesh: Handle<Mesh>,
    pub energy_core_material: Handle<ColorMaterial>,
}
```

`setup_material_drop_assets` (Startup):
- All three use the same mesh shape from `generate_material_drop_mesh(8.0)` — different colors only
- CommonScrap: `Color::srgb(0.5, 0.5, 0.5)` grey
- RareAlloy: `Color::srgb(0.27, 0.53, 1.0)` blue
- EnergyCore: `Color::srgb(1.0, 0.84, 0.0)` yellow

`attach_material_drop_visual` (Update):
```rust
pub fn attach_material_drop_visual(
    mut commands: Commands,
    query: Query<(Entity, &MaterialType), With<NeedsMaterialDropVisual>>,
    assets: Res<MaterialDropAssets>,
) {
    for (entity, material_type) in query.iter() {
        let (mesh, mat) = match material_type {
            MaterialType::CommonScrap => (&assets.common_scrap_mesh, &assets.common_scrap_material),
            MaterialType::RareAlloy => (&assets.rare_alloy_mesh, &assets.rare_alloy_material),
            MaterialType::EnergyCore => (&assets.energy_core_mesh, &assets.energy_core_material),
        };
        commands.entity(entity)
            .insert((Mesh2d(mesh.clone()), MeshMaterial2d(mat.clone())))
            .remove::<NeedsMaterialDropVisual>();
    }
}
```

### Test app setup for material_drops tests

```rust
fn material_test_app() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_message::<GameEvent>();
    app.init_resource::<PlayerInventory>();
    app.init_resource::<PendingDropSpawns>();
    app.init_resource::<PendingPickupEvents>();
    app.init_resource::<EventSeverityConfig>();
    app.add_systems(
        Update,
        (collect_material_drops, emit_pickup_events).chain(),
    );
    app.update(); // prime
    app
}
```

For `pickup_adds_to_inventory` and `no_double_pickup`: spawn player with `Player, Transform::from_translation(Vec3::ZERO), Collider { radius: 20.0 }` and a drop at same position with `MaterialDrop, MaterialType::CommonScrap, Transform::from_translation(Vec3::ZERO), Collider { radius: 15.0 }`.

### Files to Modify

| File | Action | Why |
|------|--------|-----|
| `src/shared/components.rs` | MODIFY | Add `MaterialType` enum + `MaterialDrop` + `NeedsMaterialDropVisual` (here to avoid circular import) |
| `src/core/economy.rs` | MODIFY | Add `PlayerInventory`, `PendingDropSpawns`, `PendingPickupEvents`, `decide_material_drop`, drop systems |
| `src/core/mod.rs` | MODIFY | Import new types + systems, init resources, register systems |
| `src/shared/events.rs` | MODIFY | Add `MaterialCollected { material: MaterialType }` variant |
| `src/infrastructure/events.rs` | MODIFY | Add `MaterialCollected` → Tier3, known_keys, update count 13→14 |
| `src/infrastructure/save/schema.rs` | MODIFY | Bump SAVE_VERSION 3→4, accept 1\|2\|3\|4, update tests |
| `src/infrastructure/save/player_save.rs` | MODIFY | Add 3 inventory fields, update from_world/apply_to_world/from_components |
| `src/infrastructure/save/mod.rs` | MODIFY | save_game + load_game accept PlayerInventory |
| `src/infrastructure/save/world_save.rs` | MODIFY | Update corrupt_deltas test: schema_version 3→4 |
| `src/rendering/vector_art.rs` | MODIFY | Add `generate_material_drop_mesh` |
| `src/rendering/mod.rs` | MODIFY | Add `MaterialDropAssets`, `setup_material_drop_assets`, `attach_material_drop_visual` |
| `tests/material_drops.rs` | CREATE | 5 integration tests |
| `tests/helpers/mod.rs` | MODIFY | Add `PlayerInventory`, `PendingDropSpawns`, `PendingPickupEvents` to `test_app()` |
| `tests/save_system.rs` | MODIFY | Add 3 inventory fields (0,0,0) to all PlayerSave struct literals |

### References

- `Collider` component: [src/core/collision.rs:70-73]
- `despawn_destroyed` pattern: [src/core/collision.rs:351-384]
- `EnemyDestroyed` event: [src/shared/events.rs:29-30]
- `rand::random::<f32>()` usage: [src/core/spawning.rs:105]
- B0002 pattern (PendingCreditEvents): [src/core/economy.rs:23-28]
- `NeedsAsteroidVisual` marker: [src/core/spawning.rs:19-21]
- `attach_visual` system pattern: [src/rendering/mod.rs]
- `generate_tutorial_generator_mesh`: [src/rendering/vector_art.rs]
- `PlayerSave` fields: [src/infrastructure/save/player_save.rs:14-27]
- `check_version` pattern: [src/infrastructure/save/schema.rs:45-59]
- Previous story 3-3: [_bmad-output/implementation-artifacts/3-3-earn-credits.md]

## Dev Agent Record

### Agent Model Used

claude-sonnet-4-6

### Debug Log References

### Completion Notes List

- Story 3-4 implemented: MaterialType/MaterialDrop/NeedsMaterialDropVisual in src/shared/components.rs
- PlayerInventory, PendingDropSpawns, PendingPickupEvents, decide_material_drop, 4 systems in src/core/economy.rs
- GameEventKind::MaterialCollected added; EventSeverityConfig updated (14 mappings)
- SAVE_VERSION bumped to 4; PlayerSave extended with 3 inventory fields (#[serde(default)])
- MaterialDropAssets + setup/attach systems in src/rendering/mod.rs; diamond mesh in vector_art.rs
- 8 integration tests in tests/material_drops.rs (plus 7 unit tests in economy.rs)
- 502 total tests, all green (was 485)

### File List

- `src/shared/components.rs` — MODIFIED: Added MaterialType enum, MaterialDrop, NeedsMaterialDropVisual
- `src/core/economy.rs` — MODIFIED: PlayerInventory, PendingDropSpawns, PendingPickupEvents, decide_material_drop, queue_material_drops, spawn_material_drops, collect_material_drops, emit_pickup_events
- `src/core/mod.rs` — MODIFIED: Imported new types/systems, init resources, registered drop systems
- `src/shared/events.rs` — MODIFIED: Added MaterialCollected { material: MaterialType } variant
- `src/infrastructure/events.rs` — MODIFIED: Added MaterialCollected → Tier3, updated known_keys, updated tests (14 mappings)
- `src/infrastructure/save/schema.rs` — MODIFIED: Bumped SAVE_VERSION to 4, check_version accepts 1|2|3|4, updated tests
- `src/infrastructure/save/player_save.rs` — MODIFIED: Added 3 inventory fields, updated from_world/apply_to_world/from_components
- `src/infrastructure/save/mod.rs` — MODIFIED: save_game accepts Res<PlayerInventory>, load_game restores inventory
- `src/infrastructure/save/world_save.rs` — MODIFIED: Updated corrupt_deltas test to schema_version 4
- `src/rendering/vector_art.rs` — MODIFIED: Added generate_material_drop_mesh
- `src/rendering/mod.rs` — MODIFIED: MaterialDropAssets resource, setup_material_drop_assets, attach_material_drop_visual
- `tests/material_drops.rs` — NEW: 8 integration tests
- `tests/earn_credits.rs` — MODIFIED: Added 3 inventory fields to PlayerSave literal
- `tests/helpers/mod.rs` — MODIFIED: Added PlayerInventory, PendingDropSpawns, PendingPickupEvents to test_app()
- `tests/save_system.rs` — MODIFIED: Added 3 inventory fields to 6 PlayerSave literals, added PlayerInventory to manual apps
