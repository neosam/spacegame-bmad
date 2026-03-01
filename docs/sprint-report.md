# Sprint Report — Epic 11: Platform & Release (Iteration 1)

**Datum:** 2026-03-01
**Epic:** 11 — Platform & Release
**Tests bei Sprint-Start:** 788 (nach Epic 10)
**Tests bei Sprint-Ende:** 799 (+11)

---

## Stories

| Story | Titel | Status | Tests |
|-------|-------|--------|-------|
| 11-1 | WASM Feature Subset | done | +1 (789) |
| 11-2 | Performance Optimierung | done | +6 (795) |
| 11-3 | Steam Deck UI Scaling | done | +4 (799) |
| 11-4 | Build Pipeline | done | +0 (799) |

**4 / 4 Stories abgeschlossen (100%)**
**Parallelisierung:** Alle 4 Stories parallel implementiert (keine Abhängigkeiten untereinander).

---

## Architektur-Änderungen

| Datei | Änderung |
|-------|----------|
| `assets/config/wasm_spawning.ron` | Neu: WASM-spezifische Spawn-Limits (chunk_radius: 2, max_asteroids: 4, max_enemies: 2) |
| `src/core/spawning.rs` | `WasmSpawningConfig` Resource mit `load()` — lädt wasm_spawning.ron unter WASM |
| `src/core/mod.rs` | `WasmSpawningConfig::load()` beim Plugin-Start registriert |
| `src/rendering/effects.rs` | `MAX_THRUSTER_PARTICLES`: WASM=5, Native=50 (cfg-guards) |
| `src/rendering/mod.rs` | `NebulaConfig.enabled=false` unter WASM; `DisplayConfig` Resource + Startup-Loader; alle HUD-Texte skaliert |
| `assets/config/display.ron` | Neu: ui_scale=1.0, hud_text_size=18.0, minimap_size=120.0 |
| `src/core/collision.rs` | `aabb_prefilter()` Funktion + AABB-Vorfilter in allen 4 Kollisionssystemen |
| `tests/performance.rs` | Neu: AABB-Prefilter-Tests, 200-Entity-Kollisionstest, Particle-Cap-Test |
| `Makefile` | Neu: build-native, build-wasm, build-linux, build-windows, test, clean |
| `.github/workflows/ci.yml` | Neu: CI-Pipeline (test + build-wasm) |
| `dist/index.html` | Neu: WASM-Host-HTML |

---

## Technische Highlights

### Story 11-1: WASM Feature Subset
- `WasmSpawningConfig` Resource als separates WASM-Konfigurationsobjekt
- `cfg(target_arch = "wasm32")` Guards in `effects.rs` (Particle-Cap) und `rendering/mod.rs` (Nebula)
- `NebulaConfig.enabled = false` unter WASM für weniger GPU-Last im Browser
- `wasm_spawning.ron` mit reduzierten Spawn-Werten für Browser-Performance

### Story 11-2: Performance Optimierung
- `aabb_prefilter(pos_a, radius_a, pos_b, radius_b)` — günstige Distanzprüfung vor teurer Kollisionsberechnung
- AABB-Slop von 50.0 für konservative Fehlertoleranz (keine False Negatives)
- Vorfilter in: `check_laser_collisions`, `check_projectile_collisions`, `check_enemy_projectile_collisions`, `check_contact_collisions`
- Particle-Cap erhöht auf 50 (statt 20) für bessere visuelle Qualität bei vertretbarer GPU-Last
- `tests/performance.rs`: 6 neue Tests verifizieren AABB-Korrektheit und 200-Entity-Verhalten

### Story 11-3: Steam Deck UI Scaling
- `DisplayConfig { ui_scale, hud_text_size, minimap_size }` als konfigurierbares Resource
- `STEAM_DECK=1` env-var setzt `ui_scale=1.4` automatisch
- Alle HUD-Elemente (Credits, Vitals, Bark, Coords, Save-Indicator, Boss-Retreat) verwenden `display.effective_font_size()` bzw. `font_size * display.ui_scale`
- Graceful fallback zu `DisplayConfig::default()` wenn `display.ron` fehlt
- 4 neue Tests: Default-Werte, Deserialisierung, STEAM_DECK-Override, effective_font_size

### Story 11-4: Build Pipeline
- `Makefile` mit Targets: `build-native`, `build-wasm`, `build-linux`, `build-windows`, `build-all`, `test`, `clean`
- `.github/workflows/ci.yml`: GitHub Actions CI mit `cargo test` und `cargo build --target wasm32-unknown-unknown`
- `dist/index.html`: HTML-Host für WASM-Bundle

---

## Qualitätssicherung

- Alle 799 Tests grün
- `tutorial_happy_path_full_flow` unverändert bestanden
- Core/Rendering-Trennung eingehalten: `WasmSpawningConfig` in `core/`, `DisplayConfig` in `rendering/`
- Kein `unwrap()` in Tests
- Alle WASM-spezifischer Code hinter `#[cfg(target_arch = "wasm32")]`
