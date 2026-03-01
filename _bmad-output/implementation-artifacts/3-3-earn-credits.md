# Story 3.3: Earn Credits

Status: done

## Story

As a player,
I want to earn credits by destroying enemies and discovering new areas,
so that exploration and combat are economically rewarded.

## Acceptance Criteria

1. A `Credits` resource exists in `src/core/economy.rs` with a `u32` balance field, initialized to `0`
2. Destroying an Asteroid awards `+2` credits; destroying a Scout Drone awards `+10` credits
3. First-time discovery of a new chunk (i.e., the first time `ExploredChunks` tracks it) awards `+5` credits
4. Each credit award emits a `GameEventKind::CreditsEarned { amount: u32 }` event (Tier3 severity)
5. Credits survive save/load: `PlayerSave` gains a `credits: u32` field (schema version bumped to `3`); `SAVE_VERSION` in `schema.rs` updated accordingly
6. A HUD text element in the top-left corner shows `"Credits: N"` (white, font_size 18, `GlobalZIndex(10)`) and updates every frame
7. All existing 475 tests remain green
8. New tests in `tests/earn_credits.rs` cover: asteroid kill awards credits, drone kill awards credits, chunk discovery awards credits, no double-award on same chunk, credits persist through save/load roundtrip

## Tasks / Subtasks

- [x] Task 1: Create `src/core/economy.rs` (AC: #1, #2, #3, #4)
  - [x] Define `Credits` resource: `pub struct Credits { pub balance: u32 }` + `Default` impl (balance = 0)
  - [x] System `award_credits_on_kill`: listen to `GameEvent` messages where `kind == GameEventKind::EnemyDestroyed`; award `+2` for `"asteroid"`, `+10` for `"drone"`; buffer in `PendingCreditEvents`
  - [x] System `award_credits_on_discovery`: listen to `GameEvent` messages where `kind == GameEventKind::ChunkLoaded`; award `+5` per new chunk; buffer in `PendingCreditEvents`
  - [x] System `emit_credit_events`: drain `PendingCreditEvents` and emit `GameEventKind::CreditsEarned` (separate system to avoid B0002 conflict)
  - [x] Register `Credits`, `DiscoveredChunks`, `PendingCreditEvents` resources + all systems in `CorePlugin` (after `CoreSet::Events`)

- [x] Task 2: Add `GameEventKind::CreditsEarned` variant (AC: #4)
  - [x] In `src/shared/events.rs`, add `CreditsEarned { amount: u32 }` to `GameEventKind` enum
  - [x] In `src/infrastructure/events.rs` (severity config), map `CreditsEarned` → `Tier3`

- [x] Task 3: Save/load integration (AC: #5)
  - [x] Bump `SAVE_VERSION` in `src/infrastructure/save/schema.rs` from `2` to `3`
  - [x] Add `schema_version: 2` to the accepted list in `check_version` (now accepts 1, 2, and 3)
  - [x] Add `#[serde(default)] pub credits: u32` field to `PlayerSave` in `src/infrastructure/save/player_save.rs`
  - [x] Update `PlayerSave::from_world` to read `Credits` resource and set `save.credits`
  - [x] Update `PlayerSave::apply_to_world` to write `Credits` resource
  - [x] Update `save_game` to accept `Res<Credits>` and set `ps.credits = credits.balance`
  - [x] Update `load_game` to restore `Credits` and `DiscoveredChunks` after loading

- [x] Task 4: HUD Credits display in `src/rendering/mod.rs` (AC: #6)
  - [x] Define `CreditsHudRoot` marker component
  - [x] Define `CreditsHudText` marker component
  - [x] System `spawn_credits_hud`: `Startup` — spawns a top-left `Node` with `Text("Credits: 0")` child
  - [x] System `update_credits_hud`: `Update` — reads `Credits` resource, updates the `Text` component on the HUD entity
  - [x] Register both systems in `RenderingPlugin`

- [x] Task 5: Integration tests `tests/earn_credits.rs` (AC: #7, #8)
  - [x] `asteroid_kill_awards_two_credits`
  - [x] `drone_kill_awards_ten_credits`
  - [x] `chunk_discovery_awards_five_credits`
  - [x] `same_chunk_not_awarded_twice` — verify DiscoveredChunks gating prevents double award
  - [x] `credits_save_load_roundtrip`

## Dev Notes

### Architecture Rules

- `Credits` resource lives in `src/core/economy.rs` — no Rendering code there
- `award_credits_on_kill` and `award_credits_on_discovery` read `MessageReader<GameEvent>` (they consume the messages — this is a **second** reader alongside the existing logbook reader; Bevy messages support multiple readers)
- `CreditsHudRoot` marker + both HUD systems live in `src/rendering/mod.rs`
- No `unwrap()` in tests — always `.expect("description")`

### Bevy Message Reader pattern

```rust
use bevy::ecs::message::MessageReader;
use crate::shared::events::GameEvent;

pub fn award_credits_on_kill(
    mut reader: MessageReader<GameEvent>,
    mut credits: ResMut<Credits>,
    mut game_events: MessageWriter<GameEvent>,
    time: Res<Time>,
    severity_config: Res<EventSeverityConfig>,
) {
    for event in reader.read() {
        if let GameEventKind::EnemyDestroyed { entity_type } = &event.kind {
            let amount = match *entity_type {
                "drone" => 10u32,
                _ => 2u32, // asteroid and unknown
            };
            credits.balance += amount;
            let kind = GameEventKind::CreditsEarned { amount };
            game_events.write(GameEvent {
                severity: severity_config.severity_for(&kind),
                kind,
                position: event.position,
                game_time: time.elapsed_secs_f64(),
            });
        }
    }
}
```

### ChunkLoaded — gating double-award

`ChunkLoaded` fires every time a chunk enters `ActiveChunks`, including when a previously-explored chunk is re-entered. We must check `ExploredChunks` to avoid awarding credits on revisits:

```rust
pub fn award_credits_on_discovery(
    mut reader: MessageReader<GameEvent>,
    explored: Res<ExploredChunks>,
    mut credits: ResMut<Credits>,
    ...
) {
    for event in reader.read() {
        if let GameEventKind::ChunkLoaded { coord, .. } = &event.kind {
            // ExploredChunks is updated by update_chunks BEFORE the event is emitted,
            // so the chunk IS in explored.chunks at this point.
            // We award credits only the FIRST time — but ChunkLoaded fires every time
            // the chunk enters ActiveChunks. We need a separate tracking set.
        }
    }
}
```

**IMPORTANT**: `ChunkLoaded` fires every load, not just first-ever discovery. To gate first-discovery-only, add a `DiscoveredChunks` resource (separate from `ExploredChunks`) that we insert into when crediting:

```rust
#[derive(Resource, Default)]
pub struct DiscoveredChunks {
    pub chunks: std::collections::HashSet<ChunkCoord>,
}

pub fn award_credits_on_discovery(
    mut reader: MessageReader<GameEvent>,
    mut discovered: ResMut<DiscoveredChunks>,
    mut credits: ResMut<Credits>,
    mut game_events: MessageWriter<GameEvent>,
    time: Res<Time>,
    severity_config: Res<EventSeverityConfig>,
) {
    for event in reader.read() {
        if let GameEventKind::ChunkLoaded { coord, .. } = &event.kind {
            if discovered.chunks.insert(*coord) {
                // insert returns true only on first insertion
                credits.balance += 5;
                let kind = GameEventKind::CreditsEarned { amount: 5 };
                game_events.write(GameEvent {
                    severity: severity_config.severity_for(&kind),
                    kind,
                    position: event.position,
                    game_time: time.elapsed_secs_f64(),
                });
            }
        }
    }
}
```

`DiscoveredChunks` does NOT need to be saved (it mirrors `ExploredChunks` in world_save). On load, restore `DiscoveredChunks` from the loaded `ExploredChunks` entries.

### Save Version Bump: Schema v2 → v3

`schema.rs` currently accepts versions `1` and `2`. After bumping `SAVE_VERSION` to `3`:
- `check_version` must also accept `2` (not just `1`)
- Update `check_version`'s explicit allow-list: `header.schema_version == SAVE_VERSION || header.schema_version == 1 || header.schema_version == 2`
- `PlayerSave::from_ron` for a v2 save: `credits` field will be missing → use `#[serde(default)]` on the `credits` field for backward compatibility

```rust
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct PlayerSave {
    pub schema_version: u32,
    // ... existing fields ...
    #[serde(default)]
    pub credits: u32,
}
```

### HUD layout (Bevy 0.18 pattern)

Top-left absolute positioned node:

```rust
commands.spawn((
    CreditsHudRoot,
    Node {
        position_type: PositionType::Absolute,
        top: Val::Px(8.0),
        left: Val::Px(8.0),
        ..default()
    },
    GlobalZIndex(10),
)).with_children(|parent| {
    parent.spawn((
        CreditsHudText,
        Text("Credits: 0".to_string()),
        TextFont { font_size: 18.0, ..default() },
        TextColor(Color::WHITE),
    ));
});
```

Update system:
```rust
pub fn update_credits_hud(
    credits: Res<Credits>,
    mut text_query: Query<&mut Text, With<CreditsHudText>>,
) {
    for mut text in text_query.iter_mut() {
        *text = Text(format!("Credits: {}", credits.balance));
    }
}
```

### EventSeverityConfig — adding CreditsEarned

In `src/infrastructure/events.rs`, the `severity_for` method pattern-matches on `GameEventKind`. Add:
```rust
GameEventKind::CreditsEarned { .. } => EventSeverity::Tier3,
```

### Test app setup for earn_credits tests

```rust
fn credits_test_app() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_message::<GameEvent>();
    app.init_resource::<Credits>();
    app.init_resource::<DiscoveredChunks>();
    app.init_resource::<EventSeverityConfig>();
    app.add_systems(Update, (award_credits_on_kill, award_credits_on_discovery).chain());
    app.update(); // prime
    app
}
```

To trigger `award_credits_on_kill`, write a `GameEvent` with `EnemyDestroyed` via `MessageWriter` before calling `app.update()`.

### Severity config for tests

`EventSeverityConfig` must be initialized. Check how existing tests initialize it:

```rust
app.init_resource::<EventSeverityConfig>();
// OR if it loads from RON:
// Insert manually: app.insert_resource(EventSeverityConfig::default());
```

Inspect `src/infrastructure/events.rs` to confirm `EventSeverityConfig` implements `Default`.

### Files to Modify

| File | Action | Why |
|------|--------|-----|
| `src/core/economy.rs` | CREATE | `Credits`, `DiscoveredChunks`, `award_credits_on_kill`, `award_credits_on_discovery` |
| `src/core/mod.rs` | MODIFY | `pub mod economy;`, import + register systems, `init_resource::<Credits>`, `init_resource::<DiscoveredChunks>` |
| `src/shared/events.rs` | MODIFY | Add `CreditsEarned { amount: u32 }` variant to `GameEventKind` |
| `src/infrastructure/events.rs` | MODIFY | Map `CreditsEarned` → `Tier3` in `severity_for` |
| `src/infrastructure/save/schema.rs` | MODIFY | Bump `SAVE_VERSION` to `3`, accept `1 | 2 | 3` |
| `src/infrastructure/save/player_save.rs` | MODIFY | Add `#[serde(default)] pub credits: u32`, update `from_world` / `apply_to_world` |
| `src/rendering/mod.rs` | MODIFY | `CreditsHudRoot`, `CreditsHudText`, `spawn_credits_hud`, `update_credits_hud` |
| `tests/earn_credits.rs` | CREATE | 5 integration tests |

### References

- `GameEventKind::EnemyDestroyed`: [src/shared/events.rs:30]
- `despawn_destroyed` emits EnemyDestroyed: [src/core/collision.rs:375]
- `GameEventKind::ChunkLoaded`: [src/shared/events.rs:36]
- `ExploredChunks`: [src/world/mod.rs:200-203]
- `PlayerSave`: [src/infrastructure/save/player_save.rs]
- `SAVE_VERSION` / `check_version`: [src/infrastructure/save/schema.rs]
- `StationUiRoot` HUD pattern (Bevy UI): [src/rendering/mod.rs]
- Previous story (3-2): [_bmad-output/implementation-artifacts/3-2-station-shop-ui.md]

## Dev Agent Record

### Agent Model Used

claude-sonnet-4-6

### Debug Log References

- **B0002 conflict**: `MessageReader<GameEvent>` and `MessageWriter<GameEvent>` access the same `Messages<GameEvent>` resource. Solved by introducing `PendingCreditEvents` buffer resource — award systems write to buffer, `emit_credit_events` system reads buffer and writes GameEvents.
- **PlayerSave struct literal**: Added `credits: 0` to `from_components` initializer and to `sample_player_save()` in tests.
- **Schema version**: Bumped SAVE_VERSION from 2 to 3; `check_version` now accepts 1, 2, or 3.

### Completion Notes List

- Story 3-3 implemented: Credits resource, DiscoveredChunks, PendingCreditEvents in src/core/economy.rs
- 3 economy systems: award_credits_on_kill, award_credits_on_discovery, emit_credit_events
- GameEventKind::CreditsEarned added, EventSeverityConfig updated
- SAVE_VERSION bumped to 3, PlayerSave extended with #[serde(default)] credits field
- CreditsHudRoot + CreditsHudText + spawn/update systems in src/rendering/mod.rs
- 5 integration tests in tests/earn_credits.rs, all passing
- 485 total tests, all green (was 475)

### File List

- `src/core/economy.rs` — NEW: Credits, DiscoveredChunks, PendingCreditEvents, award_credits_on_kill, award_credits_on_discovery, emit_credit_events
- `src/core/mod.rs` — Added economy module, imports, resource init, system registration
- `src/shared/events.rs` — Added CreditsEarned { amount: u32 } to GameEventKind
- `src/infrastructure/events.rs` — Added CreditsEarned → Tier3, updated known_keys, updated tests (13 mappings)
- `src/infrastructure/save/schema.rs` — Bumped SAVE_VERSION to 3, check_version accepts 1|2|3, updated tests
- `src/infrastructure/save/player_save.rs` — Added credits field, updated from_world, apply_to_world, from_components, tests
- `src/infrastructure/save/world_save.rs` — Updated corrupt_deltas test to schema_version 3
- `src/infrastructure/save/mod.rs` — save_game accepts Res<Credits>, load_game restores Credits+DiscoveredChunks
- `src/rendering/mod.rs` — Added Credits import, CreditsHudRoot, CreditsHudText, spawn_credits_hud, update_credits_hud
- `tests/earn_credits.rs` — NEW: 5 integration tests
- `tests/helpers/mod.rs` — Added Credits, DiscoveredChunks init to test_app()
- `tests/save_system.rs` — Added credits: 0 to 6 PlayerSave literals, added Credits/DiscoveredChunks to manual app instances
