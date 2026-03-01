# Sprint Backlog — Epic 10: Art & Audio Polish (Iteration 1)

**Sprint:** Epic 10
**Datum:** 2026-03-01
**Dependencies:** Core Epics stable (0–9 done)
**Epic Status:** in-progress
**Wichtig:** Epic 10 ist ein iterativer Polish-Epic. Simon's Playtest-Feedback am Sprintende ist das primäre Acceptance Criteria. Es werden mehrere Iterationen erwartet.

---

## Stories

| Story ID | Titel | Status | Abhängigkeiten |
|----------|-------|:------:|----------------|
| 10-4 | Atmospheric Background | todo | keine |
| 10-2 | Visual & Audio Juice | todo | keine |
| 10-3 | Juice Settings | todo | 10-2 |
| 10-1 | Vector Art (Ship-Varianten) | todo | keine |
| 10-5 | Music Crossfade | todo | keine |
| 10-6 | Ambient Audio | todo | 10-5 |

**Parallelisierung:** 10-4, 10-2, 10-1 und 10-5 können parallel starten.

---

## Story Beschreibungen

### 10-4: Atmospheric Background

**Ziel:** Als Spieler sehe ich eine lebendigere Weltraumkulisse die Tiefe und Atmosphäre vermittelt.

**Bestehendes:** `src/rendering/background.rs` — 3-Layer-Parallax-Starfield ist bereits implementiert.

**Erweiterungen:**

`src/rendering/background.rs`:
- **Nebula-Effekt:** Großflächige, halbtransparente Farbwolken (Mesh2d aus Kreisen/Dreiecken) als 4. Hintergrundlayer (z: -13.0)
  - Farbe: biom-abhängig — AsteroidField: orange-rot, DeepSpace: blau-lila, sonstige: grün-cyan
  - Radius: 800–2000 Einheiten, Alpha: 0.03–0.08
  - Deterministisch via Chunk-Seed platziert
- **Distant Stars (sehr langsame Parallax):** Zusätzliche Tiefenschicht mit parallax_factor 0.005, sehr kleine Sterne (radius 0.3), sehr dim (brightness 0.08)

`src/rendering/world_map.rs` oder Minimap (optional):
- Biom-Farben auf Minimap stärker differenzieren wenn bisher einheitlich

**Neue Resource:**
```rust
#[derive(Resource)]
pub struct NebulaConfig {
    pub enabled: bool,
    pub base_alpha: f32,
}
```

**Acceptance Criteria:**
- Nebula-Wolken erscheinen im Hintergrund (hinter Stars)
- Farbe wechselt subtil je nach Biom
- Kein Performance-Einbruch (Nebula-Meshes werden einmalig pro Chunk gespawnt, nicht per Frame neu generiert)
- **Simon Playtest: approved**

---

### 10-2: Visual & Audio Juice

**Ziel:** Als Spieler fühlt sich Kampf befriedigender an durch verbessertes visuelles Feedback.

**Bestehendes:** `src/rendering/effects.rs` — `ScreenShake`, `trigger_screen_shake`, `flash_on_damage`, `spawn_destruction_particles` existieren bereits.

**Erweiterungen:**

**Thruster-Effekt** (`src/rendering/effects.rs`):
```rust
#[derive(Component)]
pub struct ThrusterTrail {
    pub timer: f32,  // Lebt 0.15s
    pub alpha: f32,
}
```
- System `spawn_thruster_particles`: Wenn Player Thrust > 0 → spawne kleine Circles (radius 3–5) hinter dem Schiff, weiß→orange, alpha 0.6→0.0 über 0.15s
- System `fade_thruster_trails`: Dekrementiert alpha, despawnt wenn 0

**Laser-Impact-Flash** (verbesserter Hit-Effekt):
- Bestehende `flash_on_damage` erweitern: zusätzlich kurzer weißer Blitz (Circle radius 8, 1 Frame sichtbar) an Trefferposition
- Für Enemies: roten Blitz statt weißem

**Weapon-Switch-Visual**:
- Kleiner HUD-Indikator beim Waffen-Wechsel (bereits `WeaponSwitched` Event vorhanden)
- Kurzes Aufleuchten des aktiven Waffen-Icons für 0.5s

**Acceptance Criteria:**
- Thruster-Trail sichtbar beim Fliegen
- Laser-Hits haben visuelles Feedback auf Feinden
- Kein Performance-Einbruch (Particles sind kurzlebig, max 20 gleichzeitig)
- **Simon Playtest: approved**

---

### 10-3: Juice Settings

**Ziel:** Als Spieler kann ich visuelle Effekte nach Geschmack anpassen.

**Abhängigkeit:** 10-2 (Juice-Effekte müssen existieren)

**Neue Config** (`assets/config/juice.ron`):
```ron
(
    screen_shake_enabled: true,
    screen_shake_multiplier: 1.0,
    thruster_trail_enabled: true,
    destruction_particles_enabled: true,
    laser_impact_flash_enabled: true,
)
```

**Neue Resource** (`src/rendering/effects.rs`):
```rust
#[derive(Resource)]
pub struct JuiceSettings {
    pub screen_shake_enabled: bool,
    pub screen_shake_multiplier: f32,
    pub thruster_trail_enabled: bool,
    pub destruction_particles_enabled: bool,
    pub laser_impact_flash_enabled: bool,
}
```

- Aus `juice.ron` geladen beim Start (analog zu anderen RON-Configs)
- Alle Juice-Systeme prüfen ihre jeweilige Setting-Flag
- `ScreenShake.max_offset` multipliziert mit `screen_shake_multiplier`

**Acceptance Criteria:**
- `juice.ron` existiert und wird geladen
- `screen_shake_enabled: false` deaktiviert Kamera-Shake vollständig
- Fallback auf Defaults wenn Datei fehlt
- **Simon Playtest: approved**

---

### 10-1: Vector Art (Ship-Varianten)

**Ziel:** Als Spieler sehe ich visuell differenzierte Schiffe für verschiedene Fraktionen und Upgrade-Tiers.

**Bestehendes:** `src/rendering/vector_art.rs` — `generate_player_mesh(upgrade_tier)` existiert, generiert ein Basis-Schiff. Upgrade-Tier wird übergeben aber noch nicht ausgewertet.

**Erweiterungen:**

`src/rendering/vector_art.rs`:
```rust
/// Tier 1–2: Basis-Silhouette (aktuell)
/// Tier 3–4: Breitere Flügel, zusätzliche Einbuchtungen
/// Tier 5: Maximale Komplexität — Doppelflügel, markante Silhouette
pub fn generate_player_mesh(upgrade_tier: u8) -> Mesh { ... }
```

**Faction-spezifische Enemy-Meshes** (aktuell alle gleich):
- `generate_scout_drone_mesh()` — kleines, schnelles Dreieck
- `generate_fighter_mesh()` — mittlerer Diamant mit Flügeln
- `generate_heavy_cruiser_mesh()` — großes abgerundetes Rechteck
- `generate_boss_mesh(variant: BossVariant)` — bereits als Sechseck, Verbesserung möglich

Neue Funktion pro Ship-Typ, registriert in `NeedsDroneVisual`, `NeedsFighterVisual`, `NeedsHeavyCruiserVisual` Systemen.

**Acceptance Criteria:**
- Spieler-Schiff ändert visuell seine Form bei Tier 3 und Tier 5
- ScoutDrone, Fighter, HeavyCruiser haben distinct unterschiedliche Silhouetten
- Alle Meshes via lyon_tessellation (kein Sprite-Import)
- **Simon Playtest: approved**

---

### 10-5: Music Crossfade

**Ziel:** Als Spieler höre ich passende Hintergrundmusik die sich subtil je nach Situation ändert.

**Audio-Integration** (bevy_kira_audio 0.25.0 — bereits in Cargo.toml):

`src/infrastructure/audio.rs` (neue Datei):
```rust
use bevy_kira_audio::prelude::*;

#[derive(Resource)]
pub struct MusicTrack {
    pub current: MusicState,
    pub handle: Option<Handle<AudioInstance>>,
}

#[derive(PartialEq, Clone)]
pub enum MusicState {
    Exploration,
    Combat,
    Arena,
    Docked,
}
```

**Systeme:**
- `detect_music_state`: Prüft GameState — InWormhole → Arena, nahe Feinde → Combat, Docked → Docked, sonst Exploration
- `crossfade_music`: Wenn `MusicState` ändert → aktuellen Track ausblenden (0.5s), neuen einblenden

**Assets** (`assets/audio/`):
- Platzhalter-Audiodateien: Für diesen Sprint können kurze Loops als `.ogg` Dateien verwendet werden, oder das System wird mit Dummy-Handles implementiert und die Story als "Audio-Infrastruktur ready" markiert wenn keine echten Assets vorhanden sind.

**`src/infrastructure/mod.rs`**:
```rust
pub mod audio;
```

**Acceptance Criteria:**
- `bevy_kira_audio` Plugin ist registriert und compiliert
- `MusicState` wechselt korrekt basierend auf Spielzustand
- Crossfade-Logik existiert (auch ohne echte Audio-Assets)
- **Simon Playtest: approved** (kann auch "kein Absturz" bedeuten wenn keine Assets)

---

### 10-6: Ambient Audio

**Ziel:** Als Spieler höre ich atmosphärische Soundeffekte die die Immersion erhöhen.

**Abhängigkeit:** 10-5 (bevy_kira_audio muss registriert sein)

`src/infrastructure/audio.rs` erweitern:

**Spieler-Schuss-Sound:**
- `WeaponFired` Event → SFX abspielen
- Laser und Spread-Schuss haben verschiedene Sounds

**Explosion-Sound:**
- `EnemyDestroyed` Event → kurzer Explosion-SFX
- `PlayerDeath` Event → dramatischerer Sound

**UI-Sounds:**
- `StationDocked` Event → Dock-Klang
- `WormholeEntered` Event → mystischer Übergangs-Sound

**Asset-Handling:**
- `AudioAssets` Resource mit gecachten Handles (analog zu `LaserAssets`)
- Fallback: wenn `.ogg` nicht existiert → `warn!` und skip (kein Crash)

**Acceptance Criteria:**
- Waffen-Abschuss hat Soundfeedback
- Explosionen haben Sound
- Graceful fallback wenn Audio-Dateien fehlen (kein Crash)
- **Simon Playtest: approved**

---

## Technischer Kontext

| Bereich | Datei |
|---------|-------|
| Hintergrund/Nebula | `src/rendering/background.rs` |
| Juice-Effekte | `src/rendering/effects.rs` |
| Juice-Config | `assets/config/juice.ron` (neu) |
| Vector Art | `src/rendering/vector_art.rs` |
| Audio-Infrastruktur | `src/infrastructure/audio.rs` (neu) |
| Events für Audio-Trigger | `src/shared/events.rs` |
| bevy_kira_audio | bereits in `Cargo.toml` (0.25.0) |

## Architektur-Regeln

- **Core/Rendering-Trennung:** Audio-Infrastruktur in `src/infrastructure/audio.rs`, Audio-Trigger via Events aus Core.
- **Graceful Degradation:** Alle Audio-Systeme müssen ohne echte Asset-Dateien kompilieren und laufen (warn! + skip).
- **Config-Sync:** Neue `GameEventKind`-Varianten die Audio triggern → sofort in `event_severity.ron`.
- **Simon Playtest ist DoD** für alle Epic 10 Stories.
