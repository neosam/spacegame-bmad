# Story 1.10: WASM Build Validation

Status: done

## Story

As a developer,
I want the WASM build to run alongside native from this epic forward,
so that web compatibility is never an afterthought.

## Acceptance Criteria

1. **WASM compilation** — `cargo build --target wasm32-unknown-unknown --no-default-features --features wasm` exits 0 with no errors or warnings about unavailable features.
2. **Platform-split Cargo.toml** — `x11` and `wayland` bevy features are moved into the `native` feature gate; `webgpu` (or `webgl2` as fallback) is added to the `wasm` feature; base bevy features contain only cross-platform items (`2d`, `bevy_ui`).
3. **WASM entropy fix** — `getrandom` with `js` feature added as a target-specific dependency for `wasm32-unknown-unknown` so `rand` compiles and runs on WASM.
4. **Save system WASM guard** — `save_game` and `load_game` are guarded with `#[cfg(not(target_arch = "wasm32"))]` so they no-op silently on WASM. No `std::fs` call is reached at runtime in a WASM build. A `warn!()` is emitted once on startup when running on WASM to inform the developer saves are disabled.
5. **Trunk index.html** — A `index.html` exists in the project root with standard Bevy/trunk WASM boilerplate (canvas element, wasm-bindgen loader). No external CDN required.
6. **`trunk build` succeeds** — Running `trunk build` exits 0 and produces a `dist/` directory containing `.wasm`, `.js`, and `index.html` files.
7. **Build size under budget** — The produced `.wasm` file is under 50 MB. Size is logged to the terminal after trunk build via a `du -sh dist/*.wasm` line in a build script or noted in the story output.
8. **All 307 existing tests pass** — `cargo test` on the native target exits 0. Zero regressions.

## Tasks / Subtasks

- [x] Task 1: Fix Cargo.toml for WASM compilation (AC: #1, #2, #3)
  - [x] 1.1 Move `x11` and `wayland` from `bevy` base features into the `native` feature list
  - [x] 1.2 Add `bevy/webgpu` to the `wasm` feature list (fallback: `bevy/webgl2` if webgpu unsupported)
  - [x] 1.3 Add `[target.'cfg(target_arch = "wasm32")'.dependencies] getrandom = { version = "0.3", features = ["wasm_js"] }` to resolve `rand` entropy on WASM (note: rand 0.9 uses getrandom 0.3, not 0.2)
  - [x] 1.4 Verify `bevy_kira_audio 0.25.0` compiles on WASM (check if `wasm-support` feature needed; add if so)
  - [x] 1.5 Run `cargo build --target wasm32-unknown-unknown --no-default-features --features wasm` and confirm 0 errors

- [x] Task 2: Guard save system for WASM (AC: #4)
  - [x] 2.1 In `src/infrastructure/save/mod.rs`, wrap `save_game` function body with `#[cfg(not(target_arch = "wasm32"))]` — or add an early return with a `warn!("Save not available on WASM")` when `cfg(target_arch = "wasm32")`
  - [x] 2.2 Apply same guard to `load_game` function body
  - [x] 2.3 The `SavePlugin::build` should still register the systems (they just no-op); OR skip registering them on WASM — pick the simpler approach
  - [x] 2.4 Add a WASM-startup warning: a one-shot `Startup` system that runs only on `#[cfg(target_arch = "wasm32")]` and emits `warn!("Save system disabled on WASM — no persistent storage configured")`
  - [x] 2.5 Confirm `std::fs` module is NOT called in any WASM code path (search for `std::fs` and verify all uses are behind `cfg(not(target_arch = "wasm32"))`)

- [x] Task 3: Create trunk build files (AC: #5, #6)
  - [x] 3.1 Create `index.html` in project root (see Implementation Guidance below)
  - [x] 3.2 Optional: Create `Trunk.toml` for build configuration (wasm-opt, dist directory)
  - [x] 3.3 Run `trunk build` and confirm dist/ produced with .wasm, .js, index.html
  - [ ] 3.4 Run `trunk serve` locally and confirm browser opens without console errors (black window with ship is acceptable; full input may require interaction)

- [x] Task 4: Build size check (AC: #7)
  - [x] 4.1 After `trunk build`, run `du -sh dist/*.wasm` and record size
  - [x] 4.2 If size > 50 MB: enable LTO in `[profile.release]` and/or add `wasm-opt = "-Oz"` to Trunk.toml, rebuild
  - [x] 4.3 Document final size in the Completion Notes

- [x] Task 5: Tests and regression (AC: #8)
  - [x] 5.1 Run `cargo test` on native — confirm all 308 tests pass (307 + 1 new)
  - [x] 5.2 Add unit test `wasm_save_guard_native_save_still_works` that verifies save_game/load_game still work normally on native (regression guard for the WASM guard changes)

## Dev Notes

### Architecture Patterns & Constraints

- **Feature gates** — Architecture mandates `native = ["bevy/multi_threaded"]` and `wasm = []` (currently empty). This story fills in the WASM feature list and fixes the misplaced `x11`/`wayland` features. [Source: architecture.md — Feature Gates & Cargo Configuration]
- **WASM platform tier** — Single-threaded, ~2-4 GB memory, Web Audio restrictions, WebGL2/WebGPU rendering. **No `multi_threaded` feature on WASM.** [Source: architecture.md — Platform Constraints]
- **Save on WASM** — Architecture specifies LocalStorage/IndexedDB for WASM saves (5-10 MB limit). **This story only guards the filesystem calls — full LocalStorage backend is out of scope** (deferred to Epic 11 cloud saves or a follow-up story). [Source: architecture.md — Platform Storage]
- **Trunk as bundler** — Architecture selected `trunk` (not wasm-pack) as the WASM bundler. [Source: architecture.md — WASM Build Pipeline]
- **Size budget** — 50 MB hard limit, 30 MB stretch goal. [Source: architecture.md — WASM Build Pipeline]
- **No unwrap()** — `#[deny(clippy::unwrap_used)]` enforced crate-wide. Any new code must use `.expect()`.
- **DevPlugin on WASM** — `bevy-inspector-egui` is behind the `dev` feature which is NOT in `native` or `wasm` defaults. DevPlugin must handle `#[cfg(feature = "dev")]` guards (likely already does). Verify it compiles without `dev` feature.
- **`x11`/`wayland` in bevy** — Currently in the base bevy `features = [...]` array in Cargo.toml. These are Linux display server backends and fail on WASM. Must move to `native` feature.

### Current State (What's Already Done)

| Item | Status |
|------|--------|
| `[features] native = ["bevy/multi_threaded"]` | ✅ Exists |
| `[features] wasm = []` | ✅ Exists but empty |
| `bevy` base features include `x11`, `wayland` | ❌ Must move to `native` |
| `bevy/webgpu` or `bevy/webgl2` in `wasm` | ❌ Missing |
| `getrandom/js` for rand on wasm32 | ❌ Missing |
| `index.html` | ❌ Missing |
| `Trunk.toml` | ❌ Missing |
| Save system WASM guard | ❌ Missing |

### Key Files to Touch

| File | Action |
|------|--------|
| `Cargo.toml` | MODIFY — move x11/wayland to native, add webgpu to wasm, add getrandom/js dep |
| `index.html` | CREATE — trunk WASM entry point |
| `Trunk.toml` | CREATE (optional) — trunk configuration |
| `src/infrastructure/save/mod.rs` | MODIFY — add WASM guards to save_game, load_game |

### Implementation Guidance

**Cargo.toml target diff:**
```toml
[features]
default = ["native"]
native = ["bevy/multi_threaded", "bevy/x11", "bevy/wayland"]  # was: x11/wayland in base deps
wasm = ["bevy/webgpu"]  # or webgl2 if webgpu has issues
dev = ["bevy-inspector-egui", "bevy_egui"]

[dependencies]
bevy = { version = "0.18.0", default-features = false, features = [
    "2d",
    "bevy_ui",
    # x11, wayland moved to native feature above
] }

# WASM entropy for rand
[target.'cfg(target_arch = "wasm32")'.dependencies]
getrandom = { version = "0.2", features = ["js"] }
```

**index.html (minimal trunk boilerplate):**
```html
<!DOCTYPE html>
<html>
  <head>
    <meta charset="utf-8"/>
    <title>Void Drifter</title>
    <style>
      body { margin: 0; background: #000; }
      canvas { display: block; }
    </style>
  </head>
  <body>
    <!-- Trunk injects the WASM loader script here -->
  </body>
</html>
```

**Save WASM guard pattern (save_game):**
```rust
pub fn save_game(/* ... existing params ... */) {
    #[cfg(target_arch = "wasm32")]
    {
        // Saves not available on WASM — LocalStorage backend not yet implemented
        return;
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        // ... existing save_game body ...
    }
}
```

**Alternative cleaner approach — early return:**
```rust
pub fn save_game(/* ... */) {
    if !action_state.save {
        return;
    }
    #[cfg(target_arch = "wasm32")]
    {
        warn!("save_game: filesystem saves not available on WASM");
        return;
    }
    // rest of function unchanged...
}
```

**WASM startup warning (one-shot system):**
```rust
// In SavePlugin::build, add:
#[cfg(target_arch = "wasm32")]
app.add_systems(Startup, warn_saves_disabled_on_wasm);

#[cfg(target_arch = "wasm32")]
fn warn_saves_disabled_on_wasm() {
    warn!("Save system is disabled on WASM — filesystem not available. Progress will not be persisted.");
}
```

**Trunk.toml (optional, recommended):**
```toml
[build]
dist = "dist"

[build.wasm_opt]
level = "z"  # optimize for size
```

**Verify WASM build command:**
```bash
rustup target add wasm32-unknown-unknown
cargo install trunk  # if not installed
cargo build --target wasm32-unknown-unknown --no-default-features --features wasm
trunk build  # full bundle
du -sh dist/*.wasm
```

### Previous Story Intelligence (1-9: Delta-Save)

- **`std::fs` usage** — save/load use `std::fs::read_to_string`, `std::fs::write`, `std::fs::create_dir_all`. All three must be guarded. [Source: src/infrastructure/save/mod.rs]
- **`#[allow(clippy::too_many_arguments)]`** — `save_game` has this attribute; preserve it.
- **`Path::new().join()` pattern** — Already uses this (not `format!()`). Keep as is.
- **No unwrap in save code** — Uses `.expect()` and graceful `match`. Maintain this.
- **`WorldDeltas` registered in SavePlugin** — Stays registered on WASM too; only the filesystem I/O is guarded.
- **307 tests pass** — Must remain green after this story.

### Git Context

Recent work: 1.7 events → 1.8 save → 1.9 delta-save → 1.10 WASM build. Feature work is complete for Epic 1 core mechanics. This story is purely infrastructure (build system, platform compatibility). No new gameplay logic.

### References

- [Source: architecture.md — Feature Gates & Cargo Configuration (lines 250-265)]
- [Source: architecture.md — WASM Build Pipeline (lines 267-275)]
- [Source: architecture.md — Platform Constraints (lines 145-152)]
- [Source: architecture.md — Platform Storage (lines 325-330)]
- [Source: architecture.md — WASM-Specific Generation Constraints (lines 372-373)]
- [Source: Cargo.toml — current feature definitions]
- [Source: src/infrastructure/save/mod.rs — save_game, load_game, std::fs calls]
- [Source: epics.md — Epic 1 Story 10: "WASM build runs alongside native from this epic forward"]

## Dev Agent Record

### Agent Model Used

Claude Opus 4.6

### Debug Log References

- getrandom version mismatch: Story specified getrandom 0.2 with `js` feature, but rand 0.9 → rand_core 0.9 → getrandom 0.3. Fixed by using `getrandom = { version = "0.3", features = ["wasm_js"] }` instead.
- Dev build WASM size was 77 MB (over 50 MB budget). Release build with LTO + opt-level "z" + strip reduced to 22 MB.
- WASM cfg guards caused unused variable/import warnings. Resolved with `#[cfg_attr(target_arch = "wasm32", allow(unused_variables, unused_mut))]` and conditional imports.

### Completion Notes List

- Cargo.toml restructured: x11/wayland moved from bevy base features to `native` feature gate; `bevy/webgpu` added to `wasm` feature; `getrandom 0.3` with `wasm_js` added as target-specific dependency for wasm32.
- Save system WASM guarded: `save_game` and `load_game` function bodies wrapped in `#[cfg(not/target_arch = "wasm32")]` blocks. Systems still registered on WASM (no-op). Startup warning system added for WASM.
- All `std::fs` calls in save module confirmed behind `cfg(not(target_arch = "wasm32"))` guards. Imports also conditionally compiled.
- index.html and Trunk.toml created for trunk WASM bundler.
- Release build with LTO: 22 MB WASM (under 50 MB budget, under 30 MB stretch goal).
- Release profile added: `lto = true`, `opt-level = "z"`, `strip = true`.
- bevy_kira_audio 0.25.0 compiles on WASM without additional features.
- 308 tests pass (307 existing + 1 new regression guard test).
- Task 3.4 (trunk serve browser test) skipped — requires browser/GUI environment not available in CLI session.

### File List

- `Cargo.toml` — MODIFIED: feature gates restructured, getrandom wasm dep added, release profile added
- `Cargo.lock` — MODIFIED: updated lockfile for getrandom dependency
- `src/infrastructure/save/mod.rs` — MODIFIED: WASM cfg guards on save_game/load_game, conditional imports, warn_saves_disabled_on_wasm startup system
- `index.html` — CREATED: trunk WASM entry point
- `Trunk.toml` — CREATED: trunk build configuration with wasm-opt -Oz
- `tests/save_system.rs` — MODIFIED: added wasm_save_guard_native_save_still_works regression test
- `.gitignore` — MODIFIED: added dist/ to prevent build artifacts in VCS

### Change Log

- 2026-02-28: Story 1.10 WASM Build Validation implemented. Platform-split Cargo.toml, save system WASM guards, trunk build files, release size optimization (22 MB). 308 tests pass.
- 2026-02-28: Code review fixes — (1) dist/ untracked from VCS, added to .gitignore; (2) [profile.release] opt-level changed from "z" to 2 to preserve native performance; (3) Trunk.toml wasm-opt -Oz configured for WASM size optimization; (4) Cargo.lock added to File List.
