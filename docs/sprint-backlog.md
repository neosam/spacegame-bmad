# Sprint Backlog — Epic 7: Boss Encounters

**Sprint:** Epic 7
**Datum:** 2026-02-28
**Dependencies:** Epic 4 (Combat Depth), Epic 5 (Progression & Upgrades)
**Epic Status:** backlog

---

## Stories

| Story ID | Titel | Status | Abhängigkeiten |
|----------|-------|:------:|----------------|
| 7-1 | Boss Encounters (Basis) | backlog | keine |
| 7-2 | Boss Telegraphing | backlog | 7-1 |
| 7-3 | Faction Bosses | backlog | 7-1 |
| 7-4 | Boss Loot | backlog | 7-1 |
| 7-5 | Boss Flee | backlog | 7-1 |

---

## Story Beschreibungen

### 7-1: Boss Encounters (Basis)

**Ziel:** Einführung eines seltenen, mächtigen Boss-Gegnertyps in die offene Welt mit eigenem Mesh, deutlich erhöhten Stats und zugehörigen Game-Events.

**Neue Komponenten (`src/core/spawning.rs`):**
- `BossEnemy` — Marker-Komponente für Boss-Entities
- `NeedsBossVisual` — Marker für RenderingPlugin (größeres, auffälliges Mesh)

**Neue Variante in `SpawnType` (`src/core/spawning.rs`):**
- `SpawnType::Boss` — für Respawn-Timer (Bosse respawnen mit sehr langer Verzögerung, z.B. 120.0s)

**Erweiterung `SpawningConfig` (`src/core/spawning.rs`):**
```rust
pub boss_health: f32,         // default: 500.0
pub boss_radius: f32,         // default: 28.0
pub boss_contact_damage: f32, // default: 40.0 (höher als normaler CONTACT_DAMAGE=20)
pub boss_respawn_delay: f32,  // default: 120.0
```

**Spawn-Logik (`src/world/generation.rs` oder `src/core/spawning.rs`):**
- Boss spawnt selten: 1 Boss pro 5×5-Chunk-Block bei World-Generation
- Distance-scaled: Bosse nur ab Chunk-Distanz ≥ 3 vom Ursprung (außerhalb Tutorial-Zone)
- Spawn-Bundle:
  ```rust
  (BossEnemy, NeedsBossVisual, FactionId::..., AiState::Idle,
   AggroRange(350.0), AttackRange(150.0), FleeThreshold(0.2),
   Collider { radius: 28.0 }, Health { current: 500.0, max: 500.0 },
   Velocity(Vec2::ZERO), EnemyFireCooldown::default(),
   AggroRange, AttackRange, FleeThreshold)
  ```
- Nutzt bestehende `AiState`/`AiContext`/`next_state()`-Logik aus `src/social/enemy_ai.rs`

**AI-System (`src/social/enemy_ai.rs`):**
- Neue Funktion `update_boss_ai(...)` mit `With<BossEnemy>`-Query
- Bewegungsgeschwindigkeit: 60.0 (langsam, schwer)
- Schaden pro Schuss: 25.0, Feuerrate: 1.5s Cooldown
- Nutzt `PendingEnemyShotQueue` wie bestehende AI-Systeme

**Visuals (`src/rendering/mod.rs`):**
- System `attach_boss_visual` reagiert auf `With<NeedsBossVisual>`
- Größeres, markantes Polygon-Mesh (z.B. sechseckige Form, Radius ~28.0)
- Auffällige Farbe (z.B. dunkelrot oder magenta) via `MeshMaterial2d`

**Game Events (`src/shared/events.rs`):**
```rust
GameEventKind::BossSpawned { faction: FactionId },
GameEventKind::BossDestroyed { faction: FactionId, position: Vec2 },
```
- `BossSpawned`: Tier2-Event, wird beim Spawn emittiert
- `BossDestroyed`: Tier1-Event, wird in `despawn_destroyed` emittiert (Boss erkannt via `With<BossEnemy>`)

**`despawn_destroyed` anpassen (`src/core/collision.rs`):**
- Boss-Entity in Query aufnehmen: `Option<&BossEnemy>`
- Bei Boss-Tod: `GameEventKind::BossDestroyed` statt `EnemyDestroyed` emittieren

**Acceptance Criteria:**
- Boss spawnt in der offenen Welt, nicht in der Tutorial-Zone
- Boss hat Health 500.0, Collider radius 28.0
- Boss nutzt AiState FSM (Idle → Chase → Attack)
- Boss hat sichtbares Mesh (NeedsBossVisual führt zu Rendering)
- `GameEventKind::BossSpawned` wird beim Spawn emittiert
- `GameEventKind::BossDestroyed` wird beim Tod emittiert
- Integrationstests: Boss-Entity hat korrekte Komponenten, `update_boss_ai` transitiert States korrekt

---

### 7-2: Boss Telegraphing

**Abhängigkeit:** 7-1 (BossEnemy, update_boss_ai)

**Ziel:** Boss zeigt eine 1.5-sekündige Warnphase bevor er angreift — visuelles Feedback für den Spieler.

**Neue Komponente (`src/core/spawning.rs` oder `src/social/enemy_ai.rs`):**
```rust
/// Aktive Angriffswarnung auf einem Boss-Entity.
/// Wird eingefügt wenn Boss in Attack-Range kommt, entfernt nach Ablauf.
#[derive(Component, Debug, Clone)]
pub struct AttackWarning {
    pub timer: f32, // Startet bei 1.5, zählt runter
}
```

**Neuer Marker (`src/core/spawning.rs`):**
```rust
#[derive(Component)]
pub struct NeedsBossWarningVisual;
```

**System `update_boss_telegraphing` (`src/social/enemy_ai.rs`):**
- Query: `(Entity, &AiState, Option<&mut AttackWarning>), With<BossEnemy>`
- Wenn `AiState::Attack` und keine `AttackWarning` vorhanden: `commands.entity(e).insert(AttackWarning { timer: 1.5 })`
- `AttackWarning`-Timer tick runter: `warning.timer -= dt`
- Bei Ablauf (timer <= 0.0): Komponente entfernen
- Wenn `AiState` nicht mehr `Attack`: Warnung entfernen (Boss hat aufgehört)

**Rendering (`src/rendering/mod.rs`):**
- System `update_boss_warning_visual`: reagiert auf `With<AttackWarning>` und `With<BossEnemy>`
- Modifiziert die Mesh-Farbe: pulsierendes Leuchten (Helligkeit = `(timer * TAU).sin().abs()`)
- Bei Entfernung der `AttackWarning`: Farbe zurücksetzen

**Acceptance Criteria:**
- Wenn Boss in Attack-State: `AttackWarning { timer: 1.5 }` wird eingefügt
- Timer zählt korrekt runter und die Komponente wird nach 1.5s entfernt
- Wenn Boss Attack verlässt, wird Warnung sofort entfernt
- Boss-Entity hat visuelles Pulsieren während AttackWarning aktiv
- Unit tests: `AttackWarning` wird bei Attack-State-Eintritt eingefügt, timer countdown korrekt

---

### 7-3: Faction Bosses

**Abhängigkeit:** 7-1 (BossEnemy, FactionId)

**Ziel:** Jede der vier Fraktionen hat einen einzigartigen Boss mit eigenem Namen, eigenen Stats und charakteristischem Verhalten.

**Neue Komponente (`src/social/enemy_ai.rs` oder `src/core/spawning.rs`):**
```rust
/// Welche Boss-Variante dieser Boss ist.
/// Bestimmt Stats und Verhalten.
#[derive(Component, Debug, Clone, PartialEq, Eq)]
pub enum BossVariant {
    PirateWarlord,   // Pirates   — schnell (100), wenig HP (300), aggressiv
    Admiral,         // Military  — ausgewogen (60), viel HP (500), defensiv
    HiveMind,        // Aliens    — langsam (40), sehr viel HP (700), erratisch
    AlphaDrone,      // RogueDrones — mittel (75), mittel HP (400), aggressiv
}
```

**Boss-Stats pro Variante:**

| Variante | Geschwindigkeit | Health | Collider-Radius | Schaden | Feuerrate |
|----------|----------------|--------|-----------------|---------|-----------|
| PirateWarlord | 100.0 | 300.0 | 22.0 | 18.0 | 0.8s |
| Admiral | 60.0 | 500.0 | 28.0 | 25.0 | 1.5s |
| HiveMind | 40.0 | 700.0 | 35.0 | 30.0 | 2.0s |
| AlphaDrone | 75.0 | 400.0 | 24.0 | 20.0 | 1.0s |

**Spawn-Logik:**
- `BossVariant` wird beim Spawn anhand der `FactionId` an der Chunk-Position gesetzt
- `faction_at_position()` aus `src/social/faction.rs` bestimmt die Fraktion
- `FactionId::Pirates` → `BossVariant::PirateWarlord`, etc.

**`update_boss_ai` anpassen:**
- Stats (Geschwindigkeit, Schaden, Cooldown) aus `BossVariant` auslesen statt feste Werte
- Pure Hilfsfunktion `boss_variant_stats(variant: &BossVariant) -> BossStats` für Testbarkeit

**Acceptance Criteria:**
- `BossVariant`-Enum hat vier Varianten
- `boss_variant_stats()` gibt korrekte Stats pro Variante zurück
- Spawn in Pirates-Territorium → `BossVariant::PirateWarlord`
- Spawn in Military-Territorium → `BossVariant::Admiral`
- Spawn in Aliens-Territorium → `BossVariant::HiveMind`
- Spawn in RogueDrones-Territorium → `BossVariant::AlphaDrone`
- Unit tests: `boss_variant_stats` für alle vier Varianten, Spawn-Mapping korrekt

---

### 7-4: Boss Loot

**Abhängigkeit:** 7-1 (BossDestroyed-Event)

**Ziel:** Bei Boss-Tod spawnen 3–5 Material-Drops und der Spieler erhält einen Bonus von 500 Credits direkt.

**Mechanismus:**
- Liest `GameEventKind::BossDestroyed`-Events via `MessageReader<GameEvent>`
- Oder: Direkte Erkennung in einem neuen System das auf `BossEnemy` + `Health <= 0.0` prüft (analog zu `spawn_respawn_timers`)

**Neues System `spawn_boss_loot` (`src/core/economy.rs`):**
```rust
pub fn spawn_boss_loot(
    mut reader: MessageReader<GameEvent>,
    mut pending: ResMut<PendingDropSpawns>,
    mut player_query: Query<&mut crate::core::economy::Credits, With<Player>>,
)
```
- Pro `BossDestroyed`-Event: 3–5 Material-Drops via `PendingDropSpawns` (nutzt bestehendes System `spawn_material_drops`)
- Material-Typ: zufällig aus verfügbaren `MaterialType`-Varianten
- Drop-Positionen: Boss-Position + zufälliger Offset (±30.0 Einheiten)
- Bonus-Credits: 500 direkt auf `Credits`-Komponente des Players

**`GameEventKind::BossDestroyed` erweitern (aus 7-1):**
```rust
GameEventKind::BossDestroyed { faction: FactionId, position: Vec2 },
```
- `position` wird von `spawn_boss_loot` als Drop-Ursprung genutzt

**Acceptance Criteria:**
- Bei Boss-Tod: 3–5 `MaterialDrop`-Entities spawnen nahe der Boss-Position
- Bei Boss-Tod: Spieler erhält +500 Credits
- Bestehende Material-Drop-Mechanik (`PendingDropSpawns` → `spawn_material_drops`) wird genutzt
- Kein neuer Drop-Spawn-Code dupliziert
- Unit tests: `spawn_boss_loot` queued korrekte Anzahl Drops, Credits werden erhöht

---

### 7-5: Boss Flee

**Abhängigkeit:** 7-1 (BossEnemy, update_boss_ai, AiState::Flee)

**Ziel:** Unter 20% HP wechselt der Boss in `AiState::Flee` mit erhöhter Fluchtgeschwindigkeit und ein kurzer "BOSS RETREATING"-Text erscheint.

**Flee-Logik:**
- `FleeThreshold(0.2)` ist bereits am Boss-Entity (aus Story 7-1)
- `next_state()` in `src/social/enemy_ai.rs` erkennt automatisch `health_ratio < flee_threshold` → `AiState::Flee`
- Keine Änderung an der FSM-Logik nötig

**Flee-Geschwindigkeit (`update_boss_ai`):**
- Im `AiState::Flee`-Branch: Boost-Faktor 2.0× (z.B. bei Admiral: `60.0 * 2.0 = 120.0`)
- Höher als normaler Flee-Multiplikator (1.5×) anderer Enemies

**Neue Komponente (`src/social/enemy_ai.rs`):**
```rust
/// Markiert dass dieser Boss gerade flieht — für einmaliges visuelles Signal.
/// Wird beim ersten Eintritt in AiState::Flee eingefügt, verhindert doppeltes Signal.
#[derive(Component, Debug, Clone)]
pub struct BossFleeSignaled;
```

**System `update_boss_flee_bark` (`src/social/enemy_ai.rs`):**
- Query: `(Entity, &AiState), (With<BossEnemy>, Without<BossFleeSignaled>)`
- Wenn `AiState::Flee`: `commands.entity(e).insert(BossFleeSignaled)`
- Emittiert ein neues Event oder Resource-Eintrag: `BossRetreatBark { position: Vec2, timer: 3.0 }`

**Visuelles Signal (`src/rendering/mod.rs` oder HUD):**
- `BossRetreatBark`-Resource (oder Event): enthält Position und Anzeigedauer (3.0s)
- Rendering-System zeigt "BOSS RETREATING"-Text für 3.0s via `Text2d` oder HUD-Overlay
- Text verschwindet nach Timer-Ablauf (Komponente oder Timer-Entity)

**Acceptance Criteria:**
- Wenn Boss-Health unter 20%: Boss wechselt zu `AiState::Flee`
- Flee-Geschwindigkeit ist 2× die normale Boss-Geschwindigkeit
- "BOSS RETREATING"-Text erscheint einmalig beim ersten Flee-Eintritt
- Text verschwindet nach 3.0s
- `BossFleeSignaled` verhindert mehrfache Text-Einblendung
- Unit tests: Flee bei < 20% HP, `BossFleeSignaled` wird genau einmal eingefügt, Flee-Geschwindigkeit korrekt

---

## Technischer Kontext

| Bereich | Datei |
|---------|-------|
| Enemy AI FSM, `next_state()`, `AiContext` | `src/social/enemy_ai.rs` |
| Spawn-Marker, `SpawningConfig`, `SpawnType` | `src/core/spawning.rs` |
| `Health`, `Collider`, `DamageQueue`, `despawn_destroyed` | `src/core/collision.rs` |
| `FactionId`, `faction_at_position()`, `AggroRange` etc. | `src/social/faction.rs` |
| `GameEventKind`, `GameEvent` | `src/shared/events.rs` |
| `MaterialDrop`, `PendingDropSpawns`, `spawn_material_drops` | `src/core/economy.rs` |
| Boss-Mesh, Warning-Visual, Flee-Bark | `src/rendering/mod.rs` |
| Welt-Generation, Chunk-basierter Spawn | `src/world/generation.rs` |

## Architektur-Regeln (Erinnerung)

- **Core/Rendering-Trennung:** `BossEnemy`, `AttackWarning`, `BossVariant` in `src/core/` oder `src/social/`. Mesh-Code nur in `src/rendering/`.
- **Marker-Pattern:** `NeedsBossVisual`, `NeedsBossWarningVisual` — Core spawnt mit Marker, Rendering reagiert.
- **Pure Functions:** `boss_variant_stats()`, `next_state()` — keine ECS-Queries, vollständig unit-testbar.
- **Kein neuer Spawn im Rendering:** Boss wird ausschließlich durch Core/World-Generation gespawnt.
