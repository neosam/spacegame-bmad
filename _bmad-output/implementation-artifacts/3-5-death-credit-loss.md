# Story 3.5: Death Credit Loss

Status: done

## Story

As a player,
I want to lose 10% of my credits when I die,
so that death has a meaningful economic consequence and resource management matters.

## Acceptance Criteria

1. When `GameEventKind::PlayerDeath` is received, `Credits.balance` is reduced by `credits.balance / 10` (floor integer division)
2. If `Credits.balance` is 0, no change occurs (result stays 0, no underflow)
3. If `Credits.balance` is 9 or less, the deduction is 0 (floor of 9/10 = 0)
4. `PlayerInventory` (materials) is **not** affected by player death — only credits
5. The system reads `MessageReader<GameEvent>`, is registered in `CorePlugin` after the existing credits chain
6. All existing 502 tests remain green
7. New tests in `tests/death_credit_loss.rs`: 10% deduction, zero balance stays zero, nine credits loses zero, ten credits loses one

## Tasks / Subtasks

- [x] Task 1: Add `on_player_death_deduct_credits` system to `src/core/economy.rs` (AC: #1, #2, #3, #4, #5)
  - [x] Add `pub fn on_player_death_deduct_credits(mut reader: MessageReader<GameEvent>, mut credits: ResMut<Credits>)`
  - [x] For each event: if `GameEventKind::PlayerDeath` → `credits.balance -= credits.balance / 10`
  - [x] Export function from module (add to `pub use` in `src/core/mod.rs`)
  - [x] Register in `CorePlugin::build()` — add to the credits economy chain `.after(CoreSet::Events)`

- [x] Task 2: Integration tests `tests/death_credit_loss.rs` (AC: #6, #7)
  - [x] `player_death_deducts_ten_percent`: credits=100, send PlayerDeath, expect 90
  - [x] `player_death_with_zero_credits_stays_zero`: credits=0, send PlayerDeath, expect 0
  - [x] `player_death_nine_credits_loses_zero`: credits=9, send PlayerDeath, expect 9 (floor 9/10=0)
  - [x] `player_death_ten_credits_loses_one`: credits=10, send PlayerDeath, expect 9

## Dev Notes

### Architecture Rules

- **No B0002 concern**: `on_player_death_deduct_credits` only reads `MessageReader<GameEvent>` and writes `Credits` resource — there is no `MessageWriter<GameEvent>` in the same system. This is safe.
- **Multiple MessageReaders fine**: `award_credits_on_kill`, `award_credits_on_discovery`, `queue_material_drops`, and the new system all read `Messages<GameEvent>` independently — each has its own read cursor. Only read+write in the **same** system causes B0002.
- **No pending buffer needed**: Unlike 3-3 and 3-4, we don't emit a new `GameEvent` from this system, so no `PendingDeathCreditEvents` buffer is required.
- **Core/Rendering separation**: This is purely a Core system. No rendering changes needed.
- **No save schema change**: `SAVE_VERSION` stays at `4`. Credits are already saved in `PlayerSave.credits`.

### System Registration Pattern

Add to existing credits chain in `CorePlugin::build()`:

```rust
// existing:
app.add_systems(
    FixedUpdate,
    (award_credits_on_kill, award_credits_on_discovery, emit_credit_events)
        .chain()
        .after(CoreSet::Events),
);
// new: separate system, also after CoreSet::Events
app.add_systems(
    FixedUpdate,
    on_player_death_deduct_credits.after(CoreSet::Events),
);
```

Or append to the chain — either approach is correct. Separate system preferred (cleaner separation of concern).

### Test App Setup Pattern

Follow the pattern from `tests/earn_credits.rs`:

```rust
fn death_test_app() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_message::<GameEvent>();
    app.init_resource::<Credits>();
    app.init_resource::<EventSeverityConfig>();
    app.add_systems(Update, on_player_death_deduct_credits);
    app.update(); // prime first frame
    app
}
```

Write `PlayerDeath` events via `MessageWriter<GameEvent>` in one-shot systems (same pattern as `write_enemy_destroyed` in earn_credits.rs):

```rust
fn write_player_death(app: &mut App) {
    app.world_mut()
        .run_system_once(|mut w: MessageWriter<GameEvent>, config: Res<EventSeverityConfig>| {
            let kind = GameEventKind::PlayerDeath;
            w.write(GameEvent {
                severity: config.severity_for(&kind),
                kind,
                position: Vec2::ZERO,
                game_time: 0.0,
            });
        })
        .expect("Should run player death writer");
}
```

### Key Files to Touch

- `src/core/economy.rs` — add `on_player_death_deduct_credits` function
- `src/core/mod.rs` — import + register the new system
- `tests/death_credit_loss.rs` — new test file (follow `tests/earn_credits.rs` pattern)
- `tests/helpers/mod.rs` — no changes needed (helpers already initialize Credits etc.)

### Floor Division Arithmetic

Rust `u32` integer division is floor by default:
- `100u32 / 10 = 10` → `100 - 10 = 90` ✓
- `0u32 / 10 = 0` → `0 - 0 = 0` ✓
- `9u32 / 10 = 0` → `9 - 0 = 9` ✓
- `10u32 / 10 = 1` → `10 - 1 = 9` ✓
- `1u32 / 10 = 0` → `1 - 0 = 1` ✓

### References

- Previous story patterns: `src/core/economy.rs` — `award_credits_on_kill`, `award_credits_on_discovery`
- Test setup pattern: `tests/earn_credits.rs`
- System registration: `src/core/mod.rs:292-301`
- Event kinds: `src/shared/events.rs` — `GameEventKind::PlayerDeath`
- Credits resource: `src/core/economy.rs:14-17`

## Dev Agent Record

### Agent Model Used

claude-sonnet-4-6

### Debug Log References

### Completion Notes List

- on_player_death_deduct_credits: `credits.balance -= credits.balance / 10` — u32 floor division, no underflow possible
- No B0002 concern: only MessageReader + ResMut<Credits>, no MessageWriter in same system
- No pending buffer needed: no GameEvent emitted from this system
- 4 new tests: 100→90, 0→0, 9→9, 10→9

### File List

- src/core/economy.rs
- src/core/mod.rs
- tests/death_credit_loss.rs
