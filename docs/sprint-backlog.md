# Sprint Backlog — Epic 9: Wormhole Mini-Levels

**Sprint:** Epic 9
**Datum:** 2026-03-01
**Dependencies:** Epic 4 (Combat), Epic 5 (Upgrades/Rewards)
**Epic Status:** in-progress

---

## Stories

| Story ID | Titel | Status | Abhängigkeiten |
|----------|-------|:------:|----------------|
| 9-5 | Isolated Scene Architecture | done | keine |
| 9-1 | Wormhole Visuals | done | 9-5 |
| 9-2 | Enter Wormhole | todo | 9-1, 9-5 |
| 9-3 | Arena Combat | todo | 9-2 |
| 9-4 | Arena Rewards | todo | 9-3 |

---

## Story Beschreibungen

### 9-5: Isolated Scene Architecture

**Ziel:** Als Entwickler existieren Mini-Levels als isolierte Szenen, die den Hauptwelt-Zustand nicht beeinflussen.

**Ansatz:** `PlayingSubState::InWormhole` in `src/game_states.rs` ergänzen. Während des InWormhole-Zustands sind alle Open-World-Systeme (Chunk-Loading, Enemy-Spawning) inaktiv; nur Arena-Systeme laufen. Der Spieler kehrt nach Abschluss an die Eingangsposition zurück.

**Änderungen:**

`src/game_states.rs`:
```rust
#[derive(SubStates, Default, Clone, Eq, PartialEq, Debug, Hash)]
#[source(GameState = GameState::Playing)]
pub enum PlayingSubState {
    #[default]
    Flying,
    InWormhole,
}
```

`src/core/wormhole.rs` (neue Datei):
```rust
/// Stores the world position where the player entered the wormhole.
/// Used for crash recovery: player returns here on death or exit.
#[derive(Resource, Debug)]
pub struct WormholeEntrance {
    pub world_position: Vec2,
    pub wormhole_entity: Entity,
}

/// Tracks whether the current wormhole arena has been cleared.
#[derive(Resource, Default, Debug)]
pub struct ArenaState {
    pub wave: u32,
    pub total_waves: u32,
    pub enemies_remaining: u32,
    pub cleared: bool,
}
```

**Open-World-Systeme erhalten `run_if(in_state(PlayingSubState::Flying))` Bedingung:**
- `update_chunks` in `src/world/mod.rs`
- `spawn_entity_budget` / `drift_entities` in `src/core/spawning.rs`
- `update_boss_ai` in `src/social/enemy_ai.rs` (Boss nicht im Arena-Kontext)

**Save-Schema (`src/infrastructure/save/player_save.rs`):**
```rust
#[serde(default)]
pub cleared_wormholes: Vec<[i32; 2]>,  // ChunkCoord als [x, y] Array
```
SAVE_VERSION: 7 → 8

**Neue Datei `src/core/wormhole.rs` exportieren via `src/core/mod.rs`:**
```rust
pub mod wormhole;
```

**Acceptance Criteria:**
- `PlayingSubState::InWormhole` existiert und ist von `Flying` erreichbar
- `WormholeEntrance` Resource wird beim State-Eintritt gesetzt, beim Verlassen entfernt
- `ArenaState` Resource initialisiert sich beim Eintritt
- `update_chunks` läuft nur in `Flying`-State (kein Chunk-Loading im Arena)
- SAVE_VERSION = 8, akzeptiert v7
- Unit-Tests: State-Transitions, ArenaState-Initialisierung

---

### 9-1: Wormhole Visuals

**Abhängigkeit:** 9-5 (InWormhole State muss existieren)

**Ziel:** Als Spieler sehe ich visuell prominente Wormholes in der Welt, sodass ich sie immer bemerke.

**Neue Komponenten (`src/core/wormhole.rs`):**
```rust
/// Marker für Wormhole-Entity in der Hauptwelt.
#[derive(Component, Debug, Clone)]
pub struct Wormhole {
    pub coord: ChunkCoord,  // Chunk-Koordinate des Wormholes
    pub cleared: bool,      // Wurde diese Arena bereits abgeschlossen?
}

/// Marker: Rendering-Plugin soll Wormhole-Mesh anhängen.
#[derive(Component)]
pub struct NeedsWormholeVisual;
```

**Spawn-Logik (`src/world/mod.rs` oder `src/core/wormhole.rs`):**
```rust
/// Wormhole spawnt ab Chunk-Distanz >= 2 vom Ursprung.
/// Wahrscheinlichkeit: 1 pro 8 Chunks (deterministisch via Chunk-Seed).
/// Jeder Chunk spawnt max. 1 Wormhole.
fn should_spawn_wormhole(coord: ChunkCoord, seed: u64) -> bool {
    let hash = (coord.x as u64).wrapping_mul(2654435761)
        ^ (coord.y as u64).wrapping_mul(2246822519)
        ^ seed;
    let dist = ((coord.x * coord.x + coord.y * coord.y) as f32).sqrt();
    dist >= 2.0 && hash % 8 == 0
}
```

**Wormhole-Visual (`src/rendering/mod.rs`):**
- Pulsierender Kreis (ähnlich GravityWell-Visual)
- Radius: 40.0 Einheiten
- Farbe: Cyan/Türkis (`Color::srgb(0.0, 0.8, 1.0)`) mit Alpha-Pulsieren
- `WormholeAssets` Resource (gecachtes Mesh + Material)
- System `attach_wormhole_visual`: Liest `NeedsWormholeVisual`, fügt Mesh2d + MeshMaterial2d hinzu
- Gecleared: Farbe wechselt zu Grau (`Color::srgba(0.3, 0.3, 0.3, 0.5)`)

**Spawn beim Chunk-Load:**
In `src/world/mod.rs`, Funktion `load_chunk()`:
- Nach dem Spawn von Entities (Asteroids, Enemies): Prüfe `should_spawn_wormhole(coord, seed)`
- Wenn ja: Spawn Wormhole-Entity mit `Transform`, `Collider { radius: 40.0 }`, `Wormhole { coord, cleared: false }`, `NeedsWormholeVisual`

**Persistence:** Beim `apply_to_world()` des Save-Systems werden `cleared_wormholes` geladen und entsprechende Wormhole-Entities direkt mit `cleared: true` gespawnt (Visual grau).

**Acceptance Criteria:**
- Wormholes erscheinen ab Chunk-Distanz 2 in der Welt (ca. 1 von 8 Chunks)
- Visuell prominent: pulsierender Cyan-Kreis, Radius 40
- Gecleared = grau (nicht erneut betretbar)
- Unit-Test: `should_spawn_wormhole` mit verschiedenen Koordinaten

---

### 9-2: Enter Wormhole

**Abhängigkeit:** 9-1 (Wormhole-Entity muss existieren), 9-5 (State-Transition)

**Ziel:** Als Spieler kann ich ein Wormhole betreten um eine Combat-Arena zu starten.

**Neue Systeme (`src/core/wormhole.rs`):**

```rust
/// System: Player nähert sich Wormhole → HUD-Prompt anzeigen.
/// Player innerhalb von 60 Einheiten → "E: Wormhole betreten"
pub fn check_wormhole_proximity(
    player_query: Query<&Transform, With<Player>>,
    wormhole_query: Query<(Entity, &Transform, &Wormhole)>,
    mut writer: MessageWriter<GameEvent>,
    mut next_sub_state: ResMut<NextState<PlayingSubState>>,
    mut commands: Commands,
    action_state: Res<ActionState>,
    mut entrance: Option<ResMut<WormholeEntrance>>,
    time: Res<Time>,
    severity_config: Res<EventSeverityConfig>,
)
```

**Interaction-Flow:**
1. Player kommt in Radius 60 → HUD zeigt "E: Wormhole betreten"
2. Player drückt E (interact) → State-Transition: `Flying → InWormhole`
3. `WormholeEntrance { world_position: player_pos, wormhole_entity }` wird als Resource inseriert
4. `ArenaState { wave: 0, total_waves: 3, enemies_remaining: 0, cleared: false }` wird inseriert
5. `GameEventKind::WormholeEntered { coord: wormhole.coord }` Event emittiert (Tier1)
6. Player wird zur Arena-Position teleportiert: `Vec2::ZERO` (Arena ist bei Weltmittelpunkt intern)

**Exit-Flow (Abbruch oder Tod):**
- Bei `PlayerDeath` während `InWormhole`: Respawn an `WormholeEntrance.world_position`
- Exit-Taste (aktuell: kein Abbruch außer Tod — Arena muss abgeschlossen oder Tod akzeptiert werden)

**`GameEventKind` Erweiterung (`src/shared/events.rs`):**
```rust
WormholeEntered { coord: ChunkCoord },
WormholeCleared { coord: ChunkCoord },
```
Severity: beide Tier1.
`event_severity.ron` aktualisieren: `"WormholeEntered": Tier1`, `"WormholeCleared": Tier1`.

**Neue `PlayingSubState::InWormhole` run-conditions:**
- `check_wormhole_proximity` läuft nur in `Flying`
- Arena-Systeme laufen nur in `InWormhole`

**Acceptance Criteria:**
- E-Taste nahe Wormhole triggert State-Transition zu InWormhole
- `WormholeEntrance` wird korrekt gesetzt
- Tod in der Arena respawnt an Eingangsposition
- WormholeEntered-Event wird emittiert
- Unit-Tests: Proximity-Check, State-Transition

---

### 9-3: Arena Combat

**Abhängigkeit:** 9-2 (Enter Wormhole — InWormhole-State muss erreichbar sein)

**Ziel:** Als Spieler kämpfe ich gegen Feindwellen in einer abgeschlossenen Arena, sodass der Kampf intensiv und konzentriert ist.

**Arena-Konzept:**
- Kreisförmige Arena, Radius: 800 Einheiten
- Arena-Wand: Unsichtbare Kollisionsgrenze (Spieler kann Arena nicht verlassen)
- 3 Wellen: Welle 1 (leicht), Welle 2 (mittel), Welle 3 (schwer)
- Schwierigkeit skaliert mit Wormhole-Distanz (analog zu `enemy_stats_for_distance()`)

**Neue Komponenten (`src/core/wormhole.rs`):**
```rust
/// Marker für Arena-Feinde (werden beim State-Exit despawnt).
#[derive(Component)]
pub struct ArenaEnemy;

/// Arena-Boundary: unsichtbare kreisförmige Wand.
#[derive(Component)]
pub struct ArenaBoundary;
```

**Neue Systeme (`src/core/wormhole.rs`):**

```rust
/// Spawnt Arena-Entities beim Eintritt in InWormhole.
/// Läuft OnEnter(PlayingSubState::InWormhole).
pub fn setup_arena(
    mut commands: Commands,
    mut arena_state: ResMut<ArenaState>,
    entrance: Res<WormholeEntrance>,
    wormhole_query: Query<&Wormhole>,
    spawning_config: Res<SpawningConfig>,
)

/// Spawnt nächste Welle wenn alle Feinde besiegt.
/// Läuft in FixedUpdate wenn InWormhole.
pub fn spawn_arena_wave(
    mut arena_state: ResMut<ArenaState>,
    arena_enemies: Query<&ArenaEnemy>,
    mut commands: Commands,
    entrance: Res<WormholeEntrance>,
    wormhole_query: Query<&Wormhole>,
    spawning_config: Res<SpawningConfig>,
)

/// Verhindert dass Spieler die Arena verlässt.
/// Läuft in FixedUpdate wenn InWormhole.
pub fn enforce_arena_boundary(
    mut player_query: Query<(&mut Transform, &mut Velocity), With<Player>>,
)

/// Räumt Arena-Entities auf beim Exit.
/// Läuft OnExit(PlayingSubState::InWormhole).
pub fn cleanup_arena(
    arena_enemies: Query<Entity, With<ArenaEnemy>>,
    boundaries: Query<Entity, With<ArenaBoundary>>,
    mut commands: Commands,
    mut arena_state: ResMut<ArenaState>,
)
```

**Wellen-Zusammensetzung:**
| Welle | Enemies | Typ |
|-------|---------|-----|
| 1 | 3 | ScoutDrone |
| 2 | 3 ScoutDrone + 2 Fighter | Mix |
| 3 | 2 Fighter + 1 HeavyCruiser (oder Boss wenn Distanz >= 5) | Schwer |

**Arena-Boundary-Enforcement:**
- Spieler-Position wird geclampt wenn Distanz > 800
- Velocity wird auf null gesetzt wenn außerhalb

**Acceptance Criteria:**
- Arena spawnt 3 Wellen sequentiell
- Feinde sind `ArenaEnemy`-markiert und werden beim Exit despawnt
- Spieler kann Arena nicht verlassen (clamp bei Radius 800)
- Schwierigkeit skaliert mit Wormhole-Distanz
- `ArenaState.wave` inkrementiert korrekt
- Unit-Tests: Wellen-Spawn-Logik, Boundary-Enforcement, Cleanup

---

### 9-4: Arena Rewards

**Abhängigkeit:** 9-3 (Arena Combat — ArenaState.cleared muss gesetzt werden)

**Ziel:** Als Spieler verdiene ich garantierte Belohnungen für das Abschließen einer Arena, sodass das Risiko sich lohnt.

**Completion-Flow:**
1. Alle 3 Wellen geleert → `arena_state.cleared = true`
2. `WormholeCleared { coord }` Event emittiert
3. Belohnungen gespawnt:
   - **Credits:** +200 × Distanz-Multiplikator (mind. 200, max 1000)
   - **Materials:** 3-5 zufällige MaterialDrops nahe Player-Position
4. `Wormhole.cleared = true` auf der Wormhole-Entity gesetzt
5. HUD-Nachricht: "ARENA CLEARED! [E] Wormhole verlassen"
6. E-Taste → State-Transition zurück zu `Flying`
7. Player teleportiert zu `WormholeEntrance.world_position`
8. `WormholeEntrance` Resource entfernt
9. `ArenaState` Resource entfernt

**Save-Persistence:**
- `cleared_wormholes: Vec<[i32; 2]>` in `PlayerSave` — beim Speichern mit aktuellen `Wormhole.cleared`-Entities befüllt
- Beim Laden: Wormholes die in `cleared_wormholes` sind, spawnen mit `cleared: true` (Visual grau)

**Minimap-Integration:**
- Gecleared Wormholes erscheinen als graue Blips auf der Minimap

**Neue Systeme (`src/core/wormhole.rs`):**
```rust
/// Prüft ob alle Wellen geleert → triggert Completion.
pub fn check_arena_completion(
    arena_state: Res<ArenaState>,
    arena_enemies: Query<&ArenaEnemy>,
    mut game_events: MessageWriter<GameEvent>,
    entrance: Option<Res<WormholeEntrance>>,
    mut wormhole_query: Query<&mut Wormhole>,
    severity_config: Res<EventSeverityConfig>,
    time: Res<Time>,
)

/// Player drückt E wenn Arena cleared → Exit.
pub fn handle_arena_exit(
    action_state: Res<ActionState>,
    arena_state: Res<ArenaState>,
    entrance: Option<Res<WormholeEntrance>>,
    mut player_query: Query<&mut Transform, With<Player>>,
    mut next_sub_state: ResMut<NextState<PlayingSubState>>,
)

/// Spawnt Rewards wenn WormholeCleared-Event eintrifft.
pub fn spawn_arena_rewards(
    mut events: MessageReader<GameEvent>,
    mut credits: ResMut<Credits>,
    mut pending_drops: ResMut<PendingDropSpawns>,
    player_query: Query<&Transform, With<Player>>,
    entrance: Option<Res<WormholeEntrance>>,
    wormhole_query: Query<&Wormhole>,
)
```

**Acceptance Criteria:**
- Nach 3 Wellen: HUD zeigt "ARENA CLEARED!"
- Credits werden korrekt vergeben (Basis 200 × Distanz-Multiplikator)
- 3-5 MaterialDrops erscheinen nahe dem Spieler
- `Wormhole.cleared = true` nach Abschluss
- E-Taste teleportiert zurück zur Eingangsposition
- Gecleared Wormhole wird grau (Visual-Update)
- Save: `cleared_wormholes` wird korrekt serialisiert/deserialisiert
- Unit-Tests: Reward-Berechnung, Completion-Detection, Save-Roundtrip

---

## Technischer Kontext

| Bereich | Datei |
|---------|-------|
| GameState/SubState | `src/game_states.rs` |
| Wormhole-Kernlogik | `src/core/wormhole.rs` (neu) |
| Core Plugin-Registrierung | `src/core/mod.rs` |
| Welt-Chunk-Loading | `src/world/mod.rs` |
| Spawn-Systeme | `src/core/spawning.rs` |
| Economy (Credits, Drops) | `src/core/economy.rs` |
| Save System | `src/infrastructure/save/player_save.rs`, `schema.rs` |
| Rendering/Visual | `src/rendering/mod.rs` |
| Events | `src/shared/events.rs` |
| event_severity.ron | `assets/config/event_severity.ron` |

## Architektur-Regeln

- **Core/Rendering-Trennung:** Wormhole-Logik in `src/core/wormhole.rs`. Visuals in `src/rendering/mod.rs` via `NeedsWormholeVisual` Marker.
- **State-basierte Isolation:** Open-World-Systeme via `.run_if(in_state(PlayingSubState::Flying))` deaktiviert.
- **Kein direkter State-Zugriff in Core:** Core schreibt Events, Rendering reagiert.
- **Config-Sync:** Neue `GameEventKind`-Varianten immer sofort in `event_severity.ron` eintragen.
- **Arena-Enemies markieren:** `ArenaEnemy`-Komponente sichert sauberes Cleanup beim State-Exit.
