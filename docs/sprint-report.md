# Sprint Report — Epic 9: Wormhole Mini-Levels

**Datum:** 2026-03-01
**Epic:** 9 — Wormhole Mini-Levels
**Tests bei Sprint-Start:** 723 (nach Epic 8)
**Tests bei Sprint-Ende:** 756 (+33)

---

## Stories

| Story | Titel | Status | Tests |
|-------|-------|--------|-------|
| 9-5 | Isolated Scene Architecture | ✅ done | +5 (728) |
| 9-1 | Wormhole Visuals | ✅ done | +6 (734) |
| 9-2 | Enter Wormhole | ✅ done | +8 (742) |
| 9-3 | Arena Combat | ✅ done | +9 (751) |
| 9-4 | Arena Rewards | ✅ done | +5 (756) |

**5 / 5 Stories abgeschlossen (100%)**

---

## Architektur-Änderungen

| Datei | Änderung |
|-------|----------|
| `src/game_states.rs` | `PlayingSubState::InWormhole` hinzugefügt |
| `src/core/wormhole.rs` | Neue Datei: `WormholeEntrance`, `ArenaState`, `Wormhole`, `ArenaEnemy`, `ArenaBoundary`, `ClearedWormholes`, alle Arena-Systeme |
| `src/core/mod.rs` | `wormhole` Modul, alle Arena-Systeme registriert |
| `src/world/mod.rs` | `update_chunks .run_if(Flying)`, Wormhole-Spawn beim Chunk-Load |
| `src/social/mod.rs` | Boss-AI Systeme `.run_if(Flying)` — laufen nicht in Arena |
| `src/rendering/mod.rs` | `WormholeAssets`, `attach_wormhole_visual` (Cyan/Grau) |
| `src/shared/events.rs` | `WormholeEntered { coord }`, `WormholeCleared { coord }` Events |
| `src/infrastructure/events.rs` | Neue Events in Default-Mappings + `severity_for()` |
| `src/infrastructure/save/player_save.rs` | `cleared_wormholes: Vec<[i32; 2]>`, Save/Load-Integration |
| `assets/config/event_severity.ron` | `WormholeEntered: Tier1`, `WormholeCleared: Tier1` |

---

## Feature-Überblick

### State-basierte Isolation
`PlayingSubState::InWormhole` trennt Arena-Gameplay vollständig vom Open-World-Betrieb:
- Chunk-Loading pausiert (`.run_if(Flying)`)
- Boss-AI pausiert (`.run_if(Flying)`)
- Arena-Systeme laufen nur in `InWormhole`

### Wormhole-Spawn
- Hash-basiert deterministisch, ca. 1/8 Chunks ab Distanz >= 2
- Pulsierender Cyan-Kreis (Radius 40), Grau wenn gecleared
- Beim Chunk-Unload/-Load: Cleared-Status aus `ClearedWormholes` Resource persistent

### Arena-Mechanik
- 3 Wellen: ScoutDrones → Fighters → HeavyCruisers
- Kreisförmige Arena (Radius 800), Boundary-Enforcement (Position clamp + Velocity reset)
- Alle Arena-Feinde mit `ArenaEnemy` markiert → sauberes Cleanup bei Exit
- Player respawnt bei Tod in Arena an `WormholeEntrance.world_position`

### Rewards & Persistence
- Credits: `calculate_arena_reward(distance)` → 200–1000 Credits
- 3–5 MaterialDrops bei Completion
- `cleared_wormholes: Vec<[i32; 2]>` in PlayerSave (SAVE_VERSION 8)
- Gecleared Wormholes bleiben permanent inaktiv (grau, nicht betretbar)

---

## Carry-Over Action Items

| Item | Priorität | Status |
|------|-----------|--------|
| End-to-End-Test Enemy-Schüsse (Epic 7 Retro) | Medium | Offen |
| Boss-Spawn auf Noise-basiert (Epic 7 Retro) | Low | Backlog |
| Config-Sync DoD: `event_severity.ron` bei neuen Events | High | ✅ Eingehalten |

---

## Tests: 756 — alle grün ✅
