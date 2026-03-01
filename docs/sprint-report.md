# Sprint Report — Epic 8: Logbook UI

**Sprint:** Epic 8
**Datum:** 2026-03-01
**Status:** ✅ Abgeschlossen
**Tests:** 703 → 717 (+14 neue Tests)

---

## Zusammenfassung

Epic 7 führt Boss-Gegner in die offene Welt ein. Alle 5 Stories wurden implementiert und alle Acceptance Criteria erfüllt.

---

## Stories

### 7-1: Boss Encounters (Basis) ✅
**Commit:** `feat(7-1): Boss Encounters — BossEnemy, AI, Visuals, Events. 703 tests.`

- `BossEnemy` + `NeedsBossVisual` Marker-Komponenten in `src/core/spawning.rs`
- `SpawnType::Boss` mit 120s Respawn-Verzögerung
- Boss spawnt ab Chunk-Distanz ≥ 3 vom Ursprung (außerhalb Tutorial-Zone)
- `update_boss_ai` mit eigenem FSM-Branch, langsam (60 km/h), hoher Schaden (25)
- Sechseckiges dunkelrotes Mesh via `NeedsBossVisual`
- `GameEventKind::BossSpawned` und `GameEventKind::BossDestroyed` Events
- **7 Integrationstests** — Spawn-Position, Health, Collider, AI-Transitionen, Events

### 7-2: Boss Telegraphing ✅
**Commit:** `feat(7-2,7-5): Boss Telegraphing + Boss Flee. 717 tests.`

- `AttackWarning { timer: f32 }` Komponente in `src/social/enemy_ai.rs`
- `update_boss_telegraphing` System: Insert bei Attack-Eintritt, Timer-Countdown, Remove bei Ablauf/State-Wechsel
- Visuelles Pulsieren: Boss-Farbe wechselt von Dunkelrot zu Orange während AttackWarning aktiv
- **3 Tests**: Warning-Insert, Timer-Start bei 1.5s, Removal bei State-Wechsel

### 7-3: Faction Bosses ✅
**Commit:** `feat(7-3,7-4): Faction Bosses + Boss Loot. 711 tests.`

- `BossVariant` Enum (PirateWarlord, Admiral, HiveMind, AlphaDrone) in `src/social/enemy_ai.rs`
- `boss_variant_stats()` pure function mit allen 4 Varianten
- `update_boss_ai` nutzt Variant-Stats statt Konstanten
- Faction → Variant Mapping beim Spawn in `src/world/mod.rs`
- **4 Tests**: `boss_variant_stats` für alle 4 Varianten

| Variante | Speed | HP | Radius | Schaden | Feuerrate |
|----------|-------|-----|--------|---------|-----------|
| PirateWarlord | 100 | 300 | 22 | 18 | 0.8s |
| Admiral | 60 | 500 | 28 | 25 | 1.5s |
| HiveMind | 40 | 700 | 35 | 30 | 2.0s |
| AlphaDrone | 75 | 400 | 24 | 20 | 1.0s |

### 7-4: Boss Loot ✅
**Commit:** `feat(7-3,7-4): Faction Bosses + Boss Loot. 711 tests.`

- `spawn_boss_loot` System in `src/core/economy.rs`
- Liest `GameEventKind::BossDestroyed` via `MessageReader<GameEvent>`
- 3–5 zufällige `MaterialDrop`-Entities nahe Boss-Position (±30 Einheiten)
- +500 Credits direkt auf `Credits`-Resource
- Nutzt bestehende `PendingDropSpawns` → `spawn_material_drops` Pipeline
- **4 Integrationstests**: Drop-Anzahl (3–5), Credits (+500), Drop-Nähe, Kein Drop ohne Event

### 7-5: Boss Flee ✅
**Commit:** `feat(7-2,7-5): Boss Telegraphing + Boss Flee. 717 tests.`

- `FleeThreshold(0.2)` auf Boss-Entities gesetzt (war 0.0)
- `update_boss_ai` Flee-Branch: `away * stats.speed * 2.0` (2× Geschwindigkeit)
- `BossFleeSignaled` Marker verhindert Mehrfach-Einblendung
- `BossRetreatBark { timer: f32 }` Resource
- `update_boss_flee_bark` System: Setzt BossFleeSignaled + timer=3.0
- `tick_boss_retreat_bark` System zählt Timer herunter
- "BOSS RETREATING" HUD-Text erscheint für 3s beim ersten Flee-Eintritt
- **3 Tests**: BossFleeSignaled einmalig, BossRetreatBark timer=3.0, Flee-Speed = 2× Variant-Speed

---

## Architektur-Änderungen

| Datei | Änderung |
|-------|---------|
| `src/core/spawning.rs` | `BossEnemy`, `NeedsBossVisual`, `SpawnType::Boss`, `SpawningConfig` Boss-Felder |
| `src/social/enemy_ai.rs` | `BossVariant`, `BossStats`, `boss_variant_stats()`, `AttackWarning`, `BossFleeSignaled`, `BossRetreatBark`, 5 neue Systeme |
| `src/core/economy.rs` | `spawn_boss_loot` |
| `src/world/mod.rs` | Boss-Spawn-Logik mit Variant-Mapping, FleeThreshold 0.0→0.2 |
| `src/rendering/mod.rs` | `update_boss_warning_visual`, `spawn_boss_retreat_hud`, `update_boss_retreat_hud` |
| `src/social/mod.rs` | Registrierung aller neuen Systeme + Ressourcen |
| `src/core/mod.rs` | Registrierung `spawn_boss_loot` |

---

## Test-Übersicht

| Story | Neue Tests | Typ |
|-------|-----------|-----|
| 7-1 | 7 | Integration (boss_encounters.rs) |
| 7-2 | 3 | Unit (enemy_ai.rs::tests) |
| 7-3 | 4 | Unit (enemy_ai.rs::tests) |
| 7-4 | 4 | Integration (boss_loot.rs) |
| 7-5 | 3 | Unit (enemy_ai.rs::tests) |
| **Gesamt** | **+14** | |

**Gesamt-Testanzahl:** 703 → **717** ✅

---

## Risks / Offene Punkte

- Boss-Warning-Visual nutzt geteiltes Material (`BossAssets`). Bei mehreren gleichzeitig warnenden Bossen pulsieren alle gleichzeitig — akzeptabel für Single-Player-Szenario.
- `BossFleeSignaled` bleibt permanent auf der Entity — wird beim Boss-Despawn automatisch bereinigt.
