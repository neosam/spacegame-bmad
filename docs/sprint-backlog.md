# Sprint Backlog — Epic 11: Platform & Release (Iteration 1)

**Sprint:** Epic 11
**Datum:** 2026-03-01
**Dependencies:** Epic 0–10 abgeschlossen
**Epic Status:** in-progress

---

## Stories

| Story ID | Titel | Status | Abhängigkeiten |
|----------|-------|:------:|----------------|
| 11-1 | WASM Feature Subset | todo | keine |
| 11-2 | Performance Optimierung | todo | keine |
| 11-3 | Steam Deck UI Scaling | todo | keine |
| 11-4 | Build Pipeline | todo | keine |

**Parallelisierung:** 11-1, 11-2, 11-3, 11-4 können parallel starten.

**Ausgeklammert:** Steam Cloud Saves (11-4 aus epics.md) und Steam SDK-Integration benötigen externe Registrierung und sind kein automatisierbarer Code-Task.

---

## Story Beschreibungen

### 11-1: WASM Feature Subset

**Ziel:** Als Spieler kann ich das Spiel im Browser spielen mit angepasster Performance.

**Bestehendes:** WASM-Guards (`#[cfg(target_arch = "wasm32")]`) bereits in `src/infrastructure/save/mod.rs` und `src/rendering/`. Build läuft mit `cargo build --target wasm32-unknown-unknown`.

**Implementierung:**

`assets/config/spawning.ron`:
- Für WASM: kleinerer `chunk_radius` (2 statt 4), weniger max. gleichzeitige Entities

`src/rendering/effects.rs` + `src/rendering/background.rs`:
- Particle-Anzahl unter WASM reduzieren: `cfg(target_arch = "wasm32")` → max 5 statt 20 Thruster-Particles, keine Nebula-Meshes
- `NebulaConfig.enabled = false` unter WASM

`assets/config/wasm_spawning.ron` (neue Datei):
```ron
(
    chunk_radius: 2,
    max_asteroids_per_chunk: 4,
    max_enemies_per_chunk: 2,
    trader_interval_secs: 60.0,
)
```

`src/world/mod.rs` oder `src/core/spawning.rs`:
- Beim Startup unter WASM `wasm_spawning.ron` statt `spawning.ron` laden (cfg-guard)

**Acceptance Criteria:**
- `cargo build --target wasm32-unknown-unknown` kompiliert ohne Fehler
- WASM-Build hat reduzierten Chunk-Radius (2) und weniger Particles
- Kein Crash im Browser (no-std-safe)
- Alle Tests grün

---

### 11-2: Performance Optimierung

**Ziel:** Als Spieler läuft das Spiel flüssig mit 60fps auch wenn viele Entities vorhanden sind.

**Profiling-Targets:**
- Collision-Detection: `check_laser_collisions`, `check_projectile_collisions` — aktuell O(n*m), Optimierung via frühe Distanz-Prüfung
- Chunk-Loading: Unnötige Despawn/Respawn-Zyklen reduzieren
- Rendering: Mesh-Reuse wo möglich

**Implementierung:**

`src/core/collision.rs`:
- Spatial pruning: vor der eigentlichen Kollisionsberechnung AABB-Check (Bounding Box) einbauen
  - Wenn `|pos_a - pos_b| > max_radius_a + max_radius_b + slop` → skip
  - Slop = 50.0 (konservativ)
- `check_contact_collisions`: gleiche Optimierung

`src/world/mod.rs`:
- Chunk-Loaded-Guard: Wenn Chunk bereits geladen und keine WorldDelta → skip regenerate

`src/rendering/effects.rs`:
- Particle-Cap: global max 50 ThrusterTrail-Entities, älteste despawnen wenn Limit erreicht

**Benchmark-Test** (`tests/performance.rs` — neue Datei):
```rust
// Spawne 200 Entities, messe wie viele FixedUpdate-Ticks in 1s passen
// Erwartung: >= 3600 Ticks (60fps * 60s)
```

**Acceptance Criteria:**
- AABB-Vorfilter in allen Kollisionssystemen
- Particle-Cap bei 50
- `tests/performance.rs` kompiliert und besteht
- Alle bestehenden Tests grün

---

### 11-3: Steam Deck UI Scaling

**Ziel:** Als Spieler auf Steam Deck sind alle UI-Elemente lesbar (Mindestgröße 24px Text).

**Bestehendes:** HUD in `src/rendering/hud.rs` — Text und Indikatoren bereits vorhanden. Minimap in `src/rendering/minimap.rs`.

**Neue Config** (`assets/config/display.ron`):
```ron
(
    ui_scale: 1.0,
    hud_text_size: 18.0,
    minimap_size: 120.0,
)
```

**Neue Resource** (`src/rendering/hud.rs`):
```rust
#[derive(Resource, Debug, Clone)]
pub struct DisplayConfig {
    pub ui_scale: f32,
    pub hud_text_size: f32,
    pub minimap_size: f32,
}
impl Default for DisplayConfig { ... } // hud_text_size: 18.0, minimap_size: 120.0
```

**Änderungen:**
- `src/rendering/hud.rs`: Alle `TextFont { font_size: X }` → `TextFont { font_size: config.hud_text_size * config.ui_scale }`
- `src/rendering/minimap.rs`: Minimap-Radius aus `config.minimap_size`
- Laden aus `display.ron` beim Plugin-Start (analog zu anderen RON-Configs)

**Steam Deck Preset** — wenn `STEAM_DECK=1` env-var gesetzt → `ui_scale: 1.4` als Default (größere Elemente):
```rust
let ui_scale = std::env::var("STEAM_DECK").map(|_| 1.4).unwrap_or(1.0);
```

**Acceptance Criteria:**
- `display.ron` existiert und wird geladen
- `STEAM_DECK=1 cargo run` → größere UI
- Alle HUD-Texte nutzen `DisplayConfig`
- Alle Tests grün

---

### 11-4: Build Pipeline

**Ziel:** Als Entwickler kann ich mit einem Befehl Release-Builds für alle Zielplattformen erstellen.

**Neue Datei `Makefile`:**
```makefile
.PHONY: build-native build-wasm build-all test clean

build-native:
	cargo build --release

build-wasm:
	cargo build --release --target wasm32-unknown-unknown
	wasm-bindgen --out-dir dist/wasm --web target/wasm32-unknown-unknown/release/spacegame_bmad.wasm || true

build-linux:
	cargo build --release --target x86_64-unknown-linux-gnu

build-windows:
	cargo build --release --target x86_64-pc-windows-gnu || echo "Cross-compile requires mingw"

build-all: build-native build-wasm

test:
	cargo test

clean:
	cargo clean
	rm -rf dist/
```

**Neue Datei `.github/workflows/ci.yml`:**
```yaml
name: CI
on: [push, pull_request]
jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - run: cargo test
  build-wasm:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
        with:
          targets: wasm32-unknown-unknown
      - run: cargo build --target wasm32-unknown-unknown
```

**Neue Datei `dist/index.html`** (WASM-Host):
```html
<!DOCTYPE html>
<html>
<head><title>Void Drifter</title></head>
<body>
<canvas id="bevy-canvas"></canvas>
<script type="module">
  import init from './wasm/spacegame_bmad.js';
  init();
</script>
</body>
</html>
```

**Acceptance Criteria:**
- `make build-native` läuft durch
- `make build-wasm` läuft durch (benötigt wasm32 target installiert)
- `.github/workflows/ci.yml` existiert und ist valide YAML
- `dist/index.html` existiert
- Alle Tests grün (`make test`)

---

## Technischer Kontext

| Bereich | Datei |
|---------|-------|
| WASM Config | `assets/config/wasm_spawning.ron` (neu) |
| Performance | `src/core/collision.rs` |
| Display Config | `assets/config/display.ron` (neu), `src/rendering/hud.rs` |
| Build Pipeline | `Makefile`, `.github/workflows/ci.yml` |
| WASM Host | `dist/index.html` |

## Architektur-Regeln

- **Keine Breaking Changes:** Alle bestehenden Tests müssen weiter grün bleiben
- **cfg-Guards:** WASM-spezifischer Code immer hinter `#[cfg(target_arch = "wasm32")]`
- **Graceful Degradation:** Neue Configs immer mit sinnvollen Defaults
