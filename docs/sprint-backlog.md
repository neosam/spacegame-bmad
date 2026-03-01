# Sprint Backlog — Epic 8: Logbook UI

**Sprint:** Epic 8
**Datum:** 2026-03-01
**Dependencies:** Epic 6a (Companion Core — CompanionRecruited Events)
**Epic Status:** done

---

## Stories

| Story ID | Titel | Status | Abhängigkeiten |
|----------|-------|:------:|----------------|
| 8-1 | Logbook UI Screen | done | keine |
| 8-2 | Severity Filtering | done | 8-1 |
| 8-3 | Milestone Chapter Headings | done | 8-1 |
| 8-4 | Logbook Save Persistence | done | 8-1 |

---

## Story Beschreibungen

### 8-1: Logbook UI Screen

**Ziel:** Der Spieler kann mit `L` ein Logbuch-Overlay öffnen, das die aufgezeichneten Game Events
chronologisch anzeigt.

**Infrastruktur bereits vorhanden:**
- `Logbook` Resource in `src/infrastructure/logbook.rs` — enthält alle Events
- `LogbookEntry { kind, severity, game_time, position }` — vollständig befüllt
- `record_game_events` System befüllt Logbook automatisch aus allen GameEvents

**Neue Resource (`src/rendering/mod.rs` oder neues `logbook_ui.rs`):**
```rust
#[derive(Resource, Default)]
pub struct LogbookUiOpen(pub bool);
```

**Neuer Marker:**
```rust
#[derive(Component)]
pub struct LogbookUiRoot;
```

**Systeme:**
- `toggle_logbook_ui`: L-Taste öffnet/schließt
- `spawn_logbook_ui`: Baut UI-Panel auf (zentriertes Modal, 80% Breite, 70% Höhe)
- `despawn_logbook_ui`: Entfernt UI beim Schließen
- `update_logbook_ui`: Aktualisiert Einträge wenn Logbuch offen

**UI-Layout:**
```
╔════════════════════════════════╗
║ LOGBUCH              [L] Close ║
║ ─────────────────────────────  ║
║ [0.0s] ★★★ Tutorial Complete   ║
║ [12.3s] ★★  Station docked     ║
║ [45.1s] ★   Enemy destroyed    ║
╚════════════════════════════════╝
```
- Zeigt die letzten 20 Einträge (Tier1 + Tier2 als Standard)
- Chronologisch aufsteigend, neueste unten
- Farbkodierung: Tier1 = weiß, Tier2 = gelb, Tier3 = grau
- `GlobalZIndex(100)` — über allem anderen

**Acceptance Criteria:**
- L öffnet/schließt das Logbuch-Overlay
- Einträge werden angezeigt (kind als lesbarer Text, game_time, severity-Farbe)
- UI ist über allen anderen Overlays (WorldMap, StationUI)
- `LogbookUiRoot`-Entity wird beim Schließen despawnt

---

### 8-2: Severity Filtering

**Abhängigkeit:** 8-1

**Ziel:** Der Spieler kann den Filter-Level umschalten um mehr oder weniger Events zu sehen.

**Neue Resource:**
```rust
#[derive(Resource, Default, PartialEq, Clone, Copy, Debug)]
pub enum LogbookFilter {
    #[default]
    ImportantOnly,  // Tier1 + Tier2
    All,            // Tier1 + Tier2 + Tier3
}
```

**Steuerung:** Tab-Taste während Logbuch offen → Filter wechseln
**Anzeige:** Filterstatus in Header: "LOGBUCH [Important]" / "LOGBUCH [All]"

**Acceptance Criteria:**
- `LogbookFilter::ImportantOnly` zeigt nur Tier1 + Tier2
- `LogbookFilter::All` zeigt zusätzlich Tier3 (WeaponFired etc.)
- Tab-Taste togglet zwischen den zwei Modi
- Header zeigt aktuellen Filter an

---

### 8-3: Milestone Chapter Headings

**Abhängigkeit:** 8-1

**Ziel:** Wichtige Meilensteine erscheinen als Kapitelüberschriften im Logbuch, um dem Spieler
narrative Struktur zu geben.

**Meilensteine:**
| Trigger | Kapiteltext |
|---------|-------------|
| `TutorialComplete` | "— Tutorial abgeschlossen —" |
| `BossDestroyed` (erste) | "— Erster Boss besiegt —" |
| `CompanionRecruited` (erste) | "— Erster Begleiter rekrutiert —" |
| `StationDocked` (erste) | "— Erste Station angedockt —" |

**Neue Resource:**
```rust
#[derive(Resource, Default, Debug)]
pub struct LogbookMilestones {
    pub tutorial_complete: bool,
    pub first_boss_destroyed: bool,
    pub first_companion_recruited: bool,
    pub first_station_docked: bool,
}
```

**System `update_logbook_milestones`:** Liest GameEvents via MessageReader und setzt Milestone-Flags.

**UI-Integration:** Beim Rendern des Logbuchs werden Kapitelüberschriften als visuelle Trennlinie vor
dem ersten Entry nach einem Milestone-Event eingefügt.

**Acceptance Criteria:**
- `LogbookMilestones` wird durch Events korrekt befüllt
- Kapitelüberschriften erscheinen korrekt im Logbuch-Panel
- Milestones werden nur einmal markiert (nicht dupliziert)

---

### 8-4: Logbook Save Persistence

**Abhängigkeit:** 8-1

**Ziel:** Das Logbuch wird beim Speichern persistiert und beim Laden wiederhergestellt.

**Ansatz:** Nur Tier1 + Tier2 Events speichern (max. 100) — Tier3 ist zu laut.

**PlayerSave-Erweiterung (`src/infrastructure/save/player_save.rs`):**
```rust
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct LogbookEntrySave {
    pub kind_name: String,
    pub severity: u8,       // 1, 2, or 3
    pub game_time: f64,
    pub pos_x: f32,
    pub pos_y: f32,
}
```

```rust
// In PlayerSave:
#[serde(default)]
pub logbook_entries: Vec<LogbookEntrySave>,
```

**SAVE_VERSION:** 6 → 7. `check_version()` in schema.rs anpassen.

**`PlayerSave::from_world()`:** Logbuch aus `Logbook`-Resource auslesen.
**`PlayerSave::apply_to_world()`:** Logbuch-Einträge wiederherstellen.

**Acceptance Criteria:**
- `SAVE_VERSION = 7`
- `check_version()` akzeptiert v6 für Migration
- Logbuch-Einträge werden in RON serialisiert und deserialisiert
- Nach Laden: Logbuch enthält die gespeicherten Einträge
- Unit test: Roundtrip-Test für LogbookEntrySave

---

## Technischer Kontext

| Bereich | Datei |
|---------|-------|
| Logbook Resource | `src/infrastructure/logbook.rs` |
| Event Recording | `src/infrastructure/events.rs` |
| Save System | `src/infrastructure/save/player_save.rs`, `schema.rs` |
| UI Patterns (Station Shop, WorldMap) | `src/rendering/mod.rs`, `world_map.rs` |
| Input: KeyCode | `src/core/input.rs` |
| GameEventKind, EventSeverity | `src/shared/events.rs` |

## Architektur-Regeln

- **Core/Rendering-Trennung:** Logbuch-Daten bleiben in `src/infrastructure/`. UI-Code in `src/rendering/`.
- **Logbook Resource** wird von Infrastructure befüllt — Rendering liest nur.
- **Milestones:** `LogbookMilestones` Resource wird von einem Rendering-System aktualisiert (da es UI-relevant ist).
- **Kein Logbuch-Code in Core** — Infrastructure ist zuständig.
