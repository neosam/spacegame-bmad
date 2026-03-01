# Story 1.11: Soft Speed Cap

Status: done

## Story

As a developer,
I want a soft speed cap that prevents the player from outrunning chunk generation,
so that the world always feels complete and the player never flies into unloaded space.

## Acceptance Criteria

1. **Speed-cap config** — `FlightConfig` gains a `speed_cap_fraction: f32` field (default `0.85`) loaded from `assets/config/flight.ron`. This represents the fraction of `max_speed` at which the speed cap fully engages. The speed cap formula is: `capped_max = chunk_generation_speed * speed_cap_fraction` where `chunk_generation_speed = chunk_size * max_chunks_per_frame * fixed_timestep_hz / load_radius`. If `capped_max < max_speed`, a startup `warn!()` logs the effective cap.
2. **Velocity clamping system** — A new system `clamp_speed` runs in `FixedUpdate` after `CoreSet::Physics` and before `CoreSet::Collision`. It clamps the player's velocity magnitude to `min(max_speed, capped_max)` using smooth deceleration (lerp toward cap over multiple frames), not a hard cut. The deceleration factor is configurable via `speed_cap_decel: f32` (default `0.92`) in `FlightConfig`.
3. **Startup validation** — On plugin build, if `max_speed` exceeds the computed `chunk_generation_speed * speed_cap_fraction`, a `warn!()` is emitted: `"FlightConfig: max_speed ({}) exceeds chunk generation capacity ({}). Speed will be soft-capped at {}."`. No panic — the cap silently limits speed.
4. **Existing thrust soft cap preserved** — The existing `apply_thrust` formula `(1.0 - speed/max_speed)` is unchanged. The new `clamp_speed` system acts as an additional safety net, clamping only when speed exceeds the computed cap.
5. **Config-driven** — All speed cap parameters are in `flight.ron` and fall back to defaults if missing (serde `#[serde(default)]`). The chunk-related values (`chunk_size`, `max_chunks_per_frame`) are read from `WorldConfig` at runtime.
6. **All 308 existing tests pass** — `cargo test` exits 0. Zero regressions.
7. **New tests** — At least 3 new unit tests: (a) `clamp_speed` limits velocity when above cap, (b) `clamp_speed` does nothing when below cap, (c) startup warning emitted when max_speed exceeds chunk generation speed.

## Tasks / Subtasks

- [x] Task 1: Add speed cap config fields to FlightConfig (AC: #1, #5)
  - [x] 1.1 Add `speed_cap_fraction: f32` (default `0.85`) with `#[serde(default)]` to `FlightConfig`
  - [x] 1.2 Add `speed_cap_decel: f32` (default `0.92`) with `#[serde(default)]` to `FlightConfig`
  - [x] 1.3 Update `FlightConfig::default()` with new fields
  - [x] 1.4 Verify existing `flight.ron` still parses (serde defaults handle missing fields)

- [x] Task 2: Implement clamp_speed system (AC: #2, #4)
  - [x] 2.1 Create `pub fn clamp_speed(config: Res<FlightConfig>, world_config: Res<WorldConfig>, time: Res<Time<Fixed>>, mut query: Query<&mut Velocity, With<Player>>)` in `src/core/flight.rs`
  - [x] 2.2 Compute `chunk_gen_speed = world_config.chunk_size * world_config.max_chunks_per_frame as f32 * (1.0 / time.timestep().as_secs_f32()) / world_config.load_radius as f32`
  - [x] 2.3 Compute `effective_cap = (chunk_gen_speed * config.speed_cap_fraction).min(config.max_speed)`
  - [x] 2.4 If player speed > effective_cap: `velocity.0 = velocity.0.normalize_or_zero() * velocity.0.length().lerp(effective_cap, 1.0 - config.speed_cap_decel)`
  - [x] 2.5 Register system in `CorePlugin::build` in `FixedUpdate` after `CoreSet::Physics` and before `CoreSet::Collision`

- [x] Task 3: Startup validation warning (AC: #3)
  - [x] 3.1 In `CorePlugin::build`, after loading both `FlightConfig` and `WorldConfig`, compute `chunk_gen_speed` and compare with `max_speed`
  - [x] 3.2 If `max_speed > chunk_gen_speed * speed_cap_fraction`, emit `warn!()` with effective cap value
  - [x] 3.3 Note: `WorldConfig` is loaded in `WorldPlugin`. Since `CorePlugin` runs first, either read `WorldConfig` directly in CorePlugin::build (if accessible) or add the validation as a `Startup` system that runs when both resources exist

- [x] Task 4: Write tests (AC: #6, #7)
  - [x] 4.1 Unit test: `clamp_speed_limits_velocity_above_cap` — set velocity above computed cap, run `clamp_speed`, verify velocity decreased toward cap
  - [x] 4.2 Unit test: `clamp_speed_does_nothing_below_cap` — set velocity below cap, run `clamp_speed`, verify velocity unchanged
  - [x] 4.3 Unit test: `speed_cap_config_defaults_when_missing_from_ron` — parse RON without new fields, verify defaults applied
  - [x] 4.4 Unit test: `clamp_speed_smooth_deceleration` — verify velocity isn't hard-cut but lerps toward cap over multiple frames
  - [x] 4.5 Verify all 308 existing tests still pass

## Dev Notes

### Architecture Patterns & Constraints

- **Flight Formula:** `velocity += facing * thrust_power * (1.0 - speed/max_speed) * dt` — the existing soft cap in `apply_thrust` asymptotically approaches `max_speed` but technically never hard-limits it. External forces or accumulated floating-point drift could push speed above `max_speed`. The new `clamp_speed` system is a safety net that catches these cases.
- **System Ordering:** `FixedUpdate` ordering is `Input → Physics → Collision → Damage → Events` via `CoreSet`. The new `clamp_speed` runs between Physics and Collision. [Source: architecture.md — System Ordering]
- **Config pattern:** All balance values use `*Config` suffix, `#[derive(Resource, Asset, Deserialize, TypePath)]`, loaded from RON with graceful fallback. New fields MUST use `#[serde(default)]` for backward compatibility. [Source: architecture.md — Configuration Management]
- **No unwrap:** `#[deny(clippy::unwrap_used)]` enforced crate-wide. Use `.expect()` in tests only.
- **Graceful degradation:** The speed cap should never crash or panic. If `WorldConfig` is unavailable or values are invalid, fall back to using `max_speed` as-is (no clamping).

### Current State (What's Already Done)

| Item | Status |
|------|--------|
| `FlightConfig` with `max_speed: 800.0` | ✅ Exists in `src/core/flight.rs` |
| `WorldConfig` with `chunk_size: 1000.0`, `load_radius: 2`, `max_chunks_per_frame: 4` | ✅ Exists in `src/world/mod.rs` |
| Soft speed cap via `(1.0 - speed/max_speed)` in `apply_thrust` | ✅ Works but only limits thrust effectiveness |
| `CoreSet` ordering in `FixedUpdate` | ✅ Defined: Input → Physics → Collision → Damage → Events |
| `clamp_speed` system | ❌ Missing |
| Speed cap config fields | ❌ Missing |
| Startup validation | ❌ Missing |

### Key Math

With current defaults:
- `chunk_size = 1000.0`, `max_chunks_per_frame = 4`, `fixed_hz = 60`, `load_radius = 2`
- `chunk_gen_speed = 1000.0 * 4 * 60 / 2 = 120,000 px/s`
- This is far above `max_speed = 800.0`, so the speed cap will NOT engage with current values
- The cap is a safety net for when configs are changed (e.g. larger chunks, smaller load radius, higher max_speed)
- The cap becomes relevant when: `max_speed > chunk_size * max_chunks_per_frame * 60 / load_radius * 0.85`

### Key Files to Touch

| File | Action |
|------|--------|
| `src/core/flight.rs` | MODIFY — add speed cap fields to FlightConfig, add clamp_speed system |
| `src/core/mod.rs` | MODIFY — register clamp_speed system, add startup validation |
| `assets/config/flight.ron` | NO CHANGE — serde defaults handle missing fields |

### Previous Story Intelligence (1.10: WASM Build)

- **Code review fixed:** `dist/` untracked from VCS, `opt-level` changed from "z" to 2 for native perf, Trunk.toml wasm-opt configured
- **cfg guards pattern:** `#[cfg(target_arch = "wasm32")]` blocks used for platform-specific code. Not needed for this story.
- **308 tests pass** — current baseline
- **getrandom 0.3 with wasm_js** — added as target-specific dependency, no impact on this story

### Git Context

Recent commits: 1.8 save → 1.9 delta-save → 1.10 WASM build. This story completes Epic 1 (Open World Foundation) by ensuring the flight model is safely bounded relative to chunk generation capacity.

### References

- [Source: architecture.md — Physics/Flight Model: `velocity += facing * thrust_power * (1.0 - speed/max_speed) * dt`]
- [Source: architecture.md — System Ordering: Input → Physics → Collision → Damage → Events]
- [Source: architecture.md — Configuration Management: RON assets with *Config suffix]
- [Source: architecture.md — Error Handling: Graceful Degradation]
- [Source: src/core/flight.rs — FlightConfig, apply_thrust, apply_drag]
- [Source: src/world/mod.rs — WorldConfig, update_chunks, max_chunks_per_frame]
- [Source: assets/config/flight.ron — current values: thrust_power 450, max_speed 800]
- [Source: assets/config/world.ron — current values: chunk_size 1000, load_radius 2, max_chunks_per_frame 4]

## Dev Agent Record

### Agent Model Used

Claude Opus 4.6

### Debug Log References

### Completion Notes List

- Task 1: Added `speed_cap_fraction` (default 0.85) and `speed_cap_decel` (default 0.92) to `FlightConfig` with `#[serde(default)]` for backward compatibility. Existing RON files parse correctly without the new fields.
- Task 2: Implemented `clamp_speed` system in `src/core/flight.rs`. Computes `effective_cap = min(max_speed, chunk_gen_speed * speed_cap_fraction)` and applies smooth lerp deceleration when speed exceeds cap. Registered in `FixedUpdate` after `CoreSet::Physics`, before `CoreSet::Collision`.
- Task 3: Implemented `validate_speed_cap` as a `Startup` system (runs when both `FlightConfig` and `WorldConfig` resources exist). Emits `warn!()` with exact message format from AC#3 when `max_speed` exceeds chunk generation capacity.
- Task 4: Added 7 new tests (315 total, up from 308): `clamp_speed_limits_velocity_above_cap`, `clamp_speed_does_nothing_below_cap`, `clamp_speed_smooth_deceleration`, `speed_cap_config_defaults_when_missing_from_ron`, `flight_config_default_includes_speed_cap_fields`, `validate_speed_cap_warns_when_max_speed_exceeds_chunk_gen`, `validate_speed_cap_no_warn_when_below`. All 315 tests pass, clippy clean.

### Change Log

- 2026-02-28: Implemented soft speed cap (Story 1.11) — added speed cap config fields, clamp_speed system, startup validation, 7 new tests
- 2026-02-28: Code review fixes — extracted `compute_chunk_gen_speed` helper (DRY), replaced hardcoded magic numbers in tests with computed values, added `speed_cap_decel` boundary tests (decel=0 hard cut, decel=1 no-op), added `compute_chunk_gen_speed` zero-timestep test, made warning condition directly testable. Clamped `speed_cap_decel` to [0,1] in `clamp_speed`. 319 tests total, all pass, clippy clean.

### File List

- `src/core/flight.rs` — MODIFIED: Added `speed_cap_fraction`, `speed_cap_decel` to FlightConfig; added `compute_chunk_gen_speed` helper, `validate_speed_cap` and `clamp_speed` systems; 11 new tests
- `src/core/mod.rs` — MODIFIED: Registered `clamp_speed` (FixedUpdate) and `validate_speed_cap` (Startup) systems
- `_bmad-output/implementation-artifacts/sprint-status.yaml` — MODIFIED: Updated 1-11-soft-speed-cap status
- `_bmad-output/implementation-artifacts/1-11-soft-speed-cap.md` — MODIFIED: Updated tasks, status, dev agent record
