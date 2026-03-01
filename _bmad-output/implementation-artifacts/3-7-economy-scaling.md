# Story 3.7: Economy Scaling

Status: done

## Story

As a player,
I want to earn more credits and find better material drops the further I explore,
so that venturing into distant regions feels rewarding and meaningful.

## Acceptance Criteria

1. Pure function `distance_tier(coord: ChunkCoord) -> u32` computes `min(max(coord.x.unsigned_abs(), coord.y.unsigned_abs()) / 5, 5)` (Chebyshev distance / 5, capped at 5)
2. Pure function `scale_credits(base: u32, tier: u32) -> u32` returns `base * (10 + tier) / 10` (integer floor, equivalent to `base * (1.0 + tier * 0.1)` floored)
3. `award_credits_on_kill` uses the enemy's event position to compute the chunk coord and tier, then calls `scale_credits(base, tier)` — asteroid base=2, drone base=10
4. `decide_material_drop(entity_type, roll, tier)` gains a `tier: u32` parameter:
   - Asteroid: unchanged (80% CommonScrap regardless of tier)
   - Drone tier < 3: 60% Scrap, 30% Alloy, 10% Core (unchanged)
   - Drone tier 3–4: 40% Scrap, 45% Alloy, 15% Core
   - Drone tier 5+: 20% Scrap, 50% Alloy, 30% Core
5. `queue_material_drops` passes the computed tier from event position to `decide_material_drop`
6. `award_credits_on_kill` and `queue_material_drops` receive `Res<WorldConfig>` parameter for `chunk_size`
7. All existing tests for `decide_material_drop` updated to pass `tier: 0` as third argument
8. New tests in `tests/economy_scaling.rs`: `distance_tier_*`, `scale_credits_*`, `scaled_kill_awards_*`, `tiered_drone_drops_*`

## Tasks / Subtasks

- [ ] Task 1: Add pure functions `distance_tier` and `scale_credits` to `src/core/economy.rs` (AC: #1, #2)
  - [ ] Add `pub fn distance_tier(coord: ChunkCoord) -> u32` — `min(max(coord.x.unsigned_abs(), coord.y.unsigned_abs()) / 5, 5)`
  - [ ] Add `pub fn scale_credits(base: u32, tier: u32) -> u32` — `base * (10 + tier) / 10`
  - [ ] Add import: `use crate::world::{WorldConfig, chunk::world_to_chunk};` in economy.rs

- [ ] Task 2: Update `award_credits_on_kill` to use distance scaling (AC: #3, #6)
  - [ ] Add `config: Res<WorldConfig>` parameter to `award_credits_on_kill`
  - [ ] Inside loop: compute `coord = world_to_chunk(event.position, config.chunk_size)`, `tier = distance_tier(coord)`
  - [ ] Replace hardcoded `2u32` / `10u32` with `scale_credits(2, tier)` / `scale_credits(10, tier)`
  - [ ] Update export in `src/core/mod.rs` if needed (parameter change is transparent)

- [ ] Task 3: Update `decide_material_drop` with tier parameter (AC: #4)
  - [ ] Change signature: `pub fn decide_material_drop(entity_type: &str, roll: f32, tier: u32) -> Option<MaterialType>`
  - [ ] Asteroid branch: unchanged (80% CommonScrap, tier ignored)
  - [ ] Drone branch: add tier check — `if tier < 3 { old table } else if tier < 5 { mid table } else { high table }`
  - [ ] Mid table (tier 3–4): roll < 0.40 → Scrap, roll < 0.85 → Alloy, else Core
  - [ ] High table (tier 5+): roll < 0.20 → Scrap, roll < 0.70 → Alloy, else Core

- [ ] Task 4: Update `queue_material_drops` to pass tier (AC: #5, #6)
  - [ ] Add `config: Res<WorldConfig>` parameter to `queue_material_drops`
  - [ ] Compute `coord = world_to_chunk(event.position, config.chunk_size)`, `tier = distance_tier(coord)`
  - [ ] Pass `tier` to `decide_material_drop(entity_type, roll, tier)`

- [ ] Task 5: Fix `tests/material_drops.rs` — add `tier: 0` to all `decide_material_drop` calls (AC: #7)
  - [ ] Update `decide_material_drop_asteroid_low_roll_gives_scrap`: add `, 0` to both calls
  - [ ] Update `decide_material_drop_asteroid_high_roll_gives_none`: add `, 0` to both calls
  - [ ] Update `decide_material_drop_drone_full_range`: add `, 0` to all calls
  - [ ] Update `decide_material_drop_unknown_entity_gives_none`: add `, 0`

- [ ] Task 6: New tests `tests/economy_scaling.rs` (AC: #8)
  - [ ] `distance_tier_origin` — coord (0,0) → tier 0
  - [ ] `distance_tier_at_5` — coord (5,0) → tier 1; coord (25,0) → tier 5
  - [ ] `distance_tier_capped_at_5` — coord (100,100) → tier 5
  - [ ] `distance_tier_chebyshev` — coord (3,7) → max(3,7)=7 → tier 1; coord (-12,4) → max(12,4)=12 → tier 2
  - [ ] `scale_credits_tier_0` — scale_credits(2, 0) == 2; scale_credits(10, 0) == 10
  - [ ] `scale_credits_tier_5` — scale_credits(2, 5) == 3; scale_credits(10, 5) == 15
  - [ ] `scale_credits_tier_3` — scale_credits(10, 3) == 13
  - [ ] `decide_material_drop_drone_tier3_improved` — roll 0.3 tier 3 → Alloy (was Scrap at tier 0)
  - [ ] `decide_material_drop_drone_tier5_improved` — roll 0.1 tier 5 → Scrap, roll 0.4 tier 5 → Alloy

## Dev Notes

### CRITICAL: Files to Touch

- `src/core/economy.rs` — distance_tier, scale_credits, update award_credits_on_kill, update decide_material_drop, update queue_material_drops
- `src/core/mod.rs` — update import if signature changed (parameter addition to systems is transparent)
- `tests/material_drops.rs` — add `, 0` tier argument to decide_material_drop calls
- `tests/economy_scaling.rs` — new test file

### Import Pattern in economy.rs

Add to existing imports:
```rust
use crate::world::{WorldConfig, chunk::world_to_chunk};
```

`WorldConfig` is in `src/world/mod.rs` (exported as `crate::world::WorldConfig`).
`world_to_chunk` is `pub fn` in `src/world/chunk.rs` — accessible as `crate::world::chunk::world_to_chunk`.

### distance_tier Implementation

```rust
pub fn distance_tier(coord: ChunkCoord) -> u32 {
    let chebyshev = coord.x.unsigned_abs().max(coord.y.unsigned_abs());
    (chebyshev / 5).min(5)
}
```

Chebyshev examples:
- (0,0) → max(0,0)=0 → 0/5=0 → tier 0
- (4,4) → max(4,4)=4 → 4/5=0 → tier 0
- (5,0) → max(5,0)=5 → 5/5=1 → tier 1
- (25,0) → max(25,0)=25 → 25/5=5 → tier 5
- (100,100) → max(100,100)=100 → 100/5=20 → min(20,5)=5 → tier 5

### scale_credits Implementation

```rust
pub fn scale_credits(base: u32, tier: u32) -> u32 {
    base * (10 + tier) / 10
}
```

Examples (integer floor division):
- scale_credits(2, 0) = 2 * 10 / 10 = 2 ✓
- scale_credits(2, 5) = 2 * 15 / 10 = 30/10 = 3 ✓
- scale_credits(10, 0) = 10 * 10 / 10 = 10 ✓
- scale_credits(10, 5) = 10 * 15 / 10 = 150/10 = 15 ✓
- scale_credits(10, 3) = 10 * 13 / 10 = 130/10 = 13 ✓
- scale_credits(2, 3) = 2 * 13 / 10 = 26/10 = 2 (floor)

### Updated decide_material_drop Signature

```rust
pub fn decide_material_drop(entity_type: &str, roll: f32, tier: u32) -> Option<MaterialType> {
    match entity_type {
        "asteroid" => {
            if roll < 0.8 { Some(MaterialType::CommonScrap) } else { None }
        }
        "drone" => {
            if tier >= 5 {
                // High tier: 20% Scrap, 50% Alloy, 30% Core
                if roll < 0.20 { Some(MaterialType::CommonScrap) }
                else if roll < 0.70 { Some(MaterialType::RareAlloy) }
                else { Some(MaterialType::EnergyCore) }
            } else if tier >= 3 {
                // Mid tier: 40% Scrap, 45% Alloy, 15% Core
                if roll < 0.40 { Some(MaterialType::CommonScrap) }
                else if roll < 0.85 { Some(MaterialType::RareAlloy) }
                else { Some(MaterialType::EnergyCore) }
            } else {
                // Base tier: 60% Scrap, 30% Alloy, 10% Core
                if roll < 0.6 { Some(MaterialType::CommonScrap) }
                else if roll < 0.9 { Some(MaterialType::RareAlloy) }
                else { Some(MaterialType::EnergyCore) }
            }
        }
        _ => None,
    }
}
```

### Updated award_credits_on_kill

```rust
pub fn award_credits_on_kill(
    mut reader: MessageReader<GameEvent>,
    mut credits: ResMut<Credits>,
    mut pending: ResMut<PendingCreditEvents>,
    config: Res<WorldConfig>,
) {
    for event in reader.read() {
        if let GameEventKind::EnemyDestroyed { entity_type } = &event.kind {
            let coord = world_to_chunk(event.position, config.chunk_size);
            let tier = distance_tier(coord);
            let base = match *entity_type { "drone" => 10u32, _ => 2u32 };
            let amount = scale_credits(base, tier);
            credits.balance += amount;
            pending.events.push((amount, event.position, event.game_time));
        }
    }
}
```

### Updated queue_material_drops

```rust
pub fn queue_material_drops(
    mut reader: MessageReader<GameEvent>,
    mut pending: ResMut<PendingDropSpawns>,
    config: Res<WorldConfig>,
) {
    for event in reader.read() {
        if let GameEventKind::EnemyDestroyed { entity_type } = &event.kind {
            let coord = world_to_chunk(event.position, config.chunk_size);
            let tier = distance_tier(coord);
            let roll = rand::random::<f32>();
            if let Some(material) = decide_material_drop(entity_type, roll, tier) {
                pending.drops.push((material, event.position));
            }
        }
    }
}
```

### Test App Setup (economy_scaling.rs)

Pure function tests need no app setup — just call the functions directly.

For integration tests of the award_credits_on_kill changes, WorldConfig needs to be in the test app:
```rust
fn scaling_test_app() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_message::<GameEvent>();
    app.init_resource::<Credits>();
    app.init_resource::<DiscoveredChunks>();
    app.init_resource::<PendingCreditEvents>();
    app.init_resource::<EventSeverityConfig>();
    app.insert_resource(WorldConfig::default()); // chunk_size=1000
    app.add_systems(Update, (award_credits_on_kill, emit_credit_events).chain());
    app.update(); // prime
    app
}
```

### Impact on earn_credits.rs Tests

The `award_credits_on_kill` integration tests in `tests/earn_credits.rs` write events with `position: Vec2::ZERO`. This means:
- `world_to_chunk(Vec2::ZERO, 1000.0)` = `ChunkCoord { x: 0, y: 0 }`
- `distance_tier(ChunkCoord { x: 0, y: 0 })` = tier 0
- `scale_credits(2, 0)` = 2, `scale_credits(10, 0)` = 10

So existing earn_credits.rs tests that use position Vec2::ZERO should still pass unchanged (tier 0 gives same amounts). Verify after implementation.

### No mod.rs Changes Needed for award_credits_on_kill

Adding `Res<WorldConfig>` to `award_credits_on_kill` system is a transparent change — Bevy injects it automatically. No change needed in `src/core/mod.rs`.

**BUT**: `WorldConfig` must be initialized as a resource before `award_credits_on_kill` runs. In `CorePlugin::build()`, `WorldConfig` is loaded from RON. In test apps, use `app.insert_resource(WorldConfig::default())`.

The existing test in `tests/earn_credits.rs` uses `credits_test_app()` which does NOT initialize `WorldConfig`. This will panic at test time after the change. **FIX**: Add `app.insert_resource(WorldConfig::default())` to `credits_test_app()` in earn_credits.rs.

### References

- `ChunkCoord` type: `src/world/chunk.rs:8`
- `world_to_chunk` function: `src/world/chunk.rs:14`
- `WorldConfig` resource: `src/world/mod.rs:28`
- `award_credits_on_kill` current: `src/core/economy.rs:79`
- `decide_material_drop` current: `src/core/economy.rs:54`
- `queue_material_drops` current: `src/core/economy.rs:147` (approx)
- earn_credits.rs credits_test_app: `tests/earn_credits.rs:18`

## Dev Agent Record

### Agent Model Used

claude-sonnet-4-6

### Debug Log References

### Completion Notes List

- Added distance_tier(coord) → Chebyshev/5 capped at 5
- Added scale_credits(base, tier) → base * (10 + tier) / 10
- Updated decide_material_drop signature to add tier: u32 parameter with tiered drone tables
- Updated award_credits_on_kill to use WorldConfig, compute tier from event.position, scale credits
- Updated queue_material_drops to use WorldConfig, compute tier, pass to decide_material_drop
- Fixed all decide_material_drop calls in economy.rs unit tests (added tier: 0)
- Fixed all decide_material_drop calls in tests/material_drops.rs (added tier: 0)
- Added WorldConfig::default() to credits_test_app() in tests/earn_credits.rs
- Created tests/economy_scaling.rs with 11 new tests
- 527 tests total (was 514, +1 negative coordinate test from code review)

### File List

- src/core/economy.rs
- tests/material_drops.rs
- tests/earn_credits.rs
- tests/economy_scaling.rs
