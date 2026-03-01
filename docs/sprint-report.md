# Sprint Report — Epic 2: Tutorial Zone

**Datum:** 2026-02-28
**Epic:** 2 — Tutorial Zone
**Status:** ✅ Abgeschlossen

---

## Zusammenfassung

Epic 2 "Tutorial Zone" wurde vollständig implementiert. Alle 8 Stories wurden in einem Sprint abgearbeitet. Die Testsuite wuchs von **354 Tests** (Ende Epic 1) auf **430 Tests**.

---

## Stories

| Story | Titel | Tests hinzugefügt | Gesamt | Status |
|-------|-------|:-----------------:|:------:|:------:|
| 2.1 | Tutorial Spawn | +10 | 354 | ✅ done |
| 2.2 | Gravity Well | +12 | 354→? | ✅ done |
| 2.3 | Laser at Wreck | +12 | 366 | ✅ done |
| 2.4 | Enemies After Laser | +10 | 376 | ✅ done |
| 2.5 | Station Spread Weapon | +11 | 387 | ✅ done |
| 2.6 | Destroy Generator | +10 | 397 | ✅ done |
| 2.7 | Destruction Cascade | +12 | 409 | ✅ done |
| 2.8 | Constraint Validation | +21 | 430 | ✅ done |

---

## Implementierte Features

### TutorialPhase State Machine (8 Phasen)
```
Inactive → Exploring → Shooting → SpreadUnlocked → Complete → StationVisited → GeneratorDestroyed → TutorialComplete
```

### Neue Systeme
| System | Phase | Aufgabe |
|--------|-------|---------|
| `spawn_tutorial_zone` | Startup/Inactive | Spawnt alle Tutorial-Entities |
| `apply_gravity_well` | FixedUpdate/Physics | Lineare Pull-Kraft außerhalb safe_radius |
| `advance_phase_on_wreck_shot` | Damage | Shooting → SpreadUnlocked |
| `spawn_tutorial_enemies` | OnEnter(SpreadUnlocked) | Wave aus Scout-Dronen spawnen |
| `check_tutorial_wave_complete` | Damage | SpreadUnlocked → Complete |
| `dock_at_station` | Events | Complete → StationVisited, SpreadUnlocked-Komponente |
| `check_generator_destroyed` | Events | StationVisited → GeneratorDestroyed |
| `start_destruction_cascade` | OnEnter(GeneratorDestroyed) | CascadeTimer setzen |
| `tick_cascade_timer` | Events | GeneratorDestroyed → TutorialComplete |
| `validate_tutorial_config` | Startup | 9 Constraint-Checks, nur warn! |

### Neue Komponenten
- `TutorialWreck` — Marker für das Tutorial-Wrack
- `WreckShotState { has_been_shot }` — Treffertracking
- `TutorialEnemy` — Marker für Tutorial-Welle
- `TutorialStation` — Marker für Tutorial-Station
- `SpreadUnlocked` — Freigeschaltete Spread-Waffe
- `GravityWellGenerator` erhält jetzt `Health + Collider`

### Neue Ressourcen
- `TutorialEnemyWave { remaining }` — Wellenstatus
- `CascadeTimer { remaining }` — Countdown vor TutorialComplete

### Config-Felder (assets/config/tutorial.ron)
- `wreck_offset_min/max`, `dock_radius`, `tutorial_enemy_count`, `tutorial_enemy_spawn_radius`, `cascade_delay_secs`

---

## Architektur-Muster (bestätigt)

- **Phase Guards:** Alle Systeme prüfen die aktuelle `TutorialPhase` vor Aktionen
- **Idempotenz:** Alle Übergangssysteme sind idempotent durch Flags (`has_been_shot`, Ressourcen-Existenz)
- **Cascade via Timer:** `CascadeTimer`-Ressource für zeitverzögerte Aktionen
- **Velocity-basierte Physik:** Gravity Well modifiziert `Velocity`, nicht `Transform`
- **Warn-only Validation:** `validate_tutorial_config` wie `validate_speed_cap` — kein panic

---

## Commit-Historie

```
feat(2.8): constraint validation — 430 tests
feat(2.7): destruction cascade — 409 tests
feat(2.6): destroy generator — 397 tests
feat(2.5): station spread weapon — 387 tests
feat(2.4): enemies after laser — 376 tests
feat(2.3): laser at wreck — 366 tests
feat(2.2): gravity well — 354 tests
feat(2.1): tutorial spawn — 344 tests
```

---

## Nächster Sprint

**Epic 3: Stations & Economy** (Dependencies: Epic 1 ✅)
- 3.1 Station Docking
- 3.2 Station Shop UI
- 3.3 Earn Credits
- 3.4 Material Drops
- 3.5 Death Credit Loss
- 3.6 Station Types
- 3.7 Economy Scaling
