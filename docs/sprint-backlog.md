# Sprint Backlog — Bugfix Sprint BF-1: Respawn & Tutorial Zone

**Sprint:** BF-1
**Typ:** Bugfix Sprint
**Datum:** 2026-02-28
**Dependencies:** keine (pre-existing bugs seit Epic 1)

---

## Stories

| Story ID | Titel | Status | Abhängigkeiten |
|----------|-------|:------:|----------------|
| BF-1 | Respawn an letzter Station | todo | keine |
| BF-2 | Tutorial Zone Schutz | todo | keine |

---

## Story Beschreibungen

### BF-1: Respawn an letzter Station

**Problem:** Bei Tod wird der Spieler zu `Vec3::ZERO` teleportiert (`handle_player_death` in `src/core/collision.rs`). In der offenen Welt ist das ein komplett anderer Ort als wo der Spieler zuletzt war.

**Fix:**
- Neue Resource `LastDockedStation { position: Vec2 }` in `src/core/station.rs`
- In `handle_dock_player` (oder `handle_undock_player`): Position der Station speichern wenn gedockt wird
- In `handle_player_death`: `transform.translation = last_station.position.extend(0.0)` statt `Vec3::ZERO`
- Fallback: `Vec3::ZERO` wenn noch nie gedockt (Tutorial-Start)

**Acceptance Criteria:**
- Nach dem Tod spawnt der Spieler an der letzten genutzten Station
- Erster Tod (noch nie gedockt) → Respawn weiterhin bei `Vec3::ZERO`
- Unit test: `handle_player_death` nutzt `LastDockedStation` wenn vorhanden

---

### BF-2: Tutorial Zone Schutz

**Problem:** `update_chunks` unloadet alle Chunks außerhalb des `load_radius`. Wenn der Spieler die Tutorial Zone verlässt, werden ihre Chunks entladen. Bei Rückkehr (z.B. nach Respawn) generiert das System die Chunks **prozedural** — Asteroid-Biom statt Tutorial Zone.

**Fix:**
- `TutorialZoneChunks { coords: HashSet<ChunkCoord> }` Resource in `src/world/mod.rs`
- Beim Startup: Tutorial-Zone-Chunks in die Resource eintragen (Chunks rund um (0,0) innerhalb des Tutorial-Radius)
- In `update_chunks`: Tutorial-Zone-Chunks vom Unloading ausschließen (nie in `to_unload`)
- In `generate_chunk` / Chunk-Loading: Tutorial-Zone-Chunks nicht prozedural neu befüllen wenn schon aktiv

**Acceptance Criteria:**
- Nach Verlassen und Rückkehr zur Tutorial Zone sieht sie gleich aus wie beim ersten Besuch
- Prozedurale Chunks außerhalb der Tutorial Zone unverändert
- Unit test: Tutorial-Zone-Chunks werden nicht in `to_unload` aufgenommen

---

## Technischer Kontext

- `handle_player_death` → `src/core/collision.rs:276`
- `handle_dock_player` → `src/core/station.rs`
- `update_chunks` → `src/world/mod.rs:286`
- Tutorial Zone Spawn → `src/core/tutorial.rs` + `src/rendering/mod.rs`
