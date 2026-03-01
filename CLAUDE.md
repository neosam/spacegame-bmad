# Void Drifter — Projektregeln

## Definition of Done (nach jeder Story)

**Eine Story ist erst dann fertig, wenn das Spiel spielbar bleibt.**

### Pflichtprüfungen nach jeder Implementierung

1. **Alle Tests grün** — `cargo test` ohne Fehler
2. **Happy-Path-Test besteht** — `tutorial_happy_path_full_flow` läuft durch (belegt dass der Tutorial-Spielfluss intakt ist)
3. **Sichtbarkeit** — Jede neue Entity, mit der der Spieler interagieren soll, hat ein Mesh (kein unsichtbares Gameplay-Objekt)
4. **Spielfluss** — Nach der Story kann der Spieler immer noch: fliegen, schießen, und mit dem neuen Feature interagieren
5. **Playtest** — Neue Gameplay-Mechaniken manuell spielen und prüfen ob sie sich korrekt anfühlen (Balance, Feedback, Sichtbarkeit im Spiel)

### Architektur-Regeln

- **Core/Rendering-Trennung:** Core spawnt Entities mit Marker-Komponenten. Rendering fügt Mesh2d + MeshMaterial2d hinzu. **Niemals Rendering-Code in `src/core/`.**
- **Kein doppelter Player-Spawn:** `spawn_tutorial_zone` darf keine Player-Entity spawnen. Nur `setup_player` in Rendering spawnt den Player.
- **Waffen-Gating:** Spread nur wenn `SpreadUnlocked` auf dem Player. Generator nur durch Spread-Projektile zerstörbar (nicht Laser).

### Testing

- Kein `unwrap()` in Tests — immer `.expect("beschreibung")`
- Jede neue spielerrelevante Mechanik braucht einen Integrationstest
- State-Machine-Änderungen: `tutorial_happy_path_full_flow` updaten wenn neue Phasen hinzukommen

## Technisches Setup

- **Sprache:** Rust, Bevy 0.18.0, lyon_tessellation, bevy_kira_audio
- **VCS:** jj (Jujutsu) — `jj describe -m "..."` dann `jj new`
- **Commit-Format:** `feat(X.Y): kurze beschreibung. N tests.`
- **Tests ausführen:** `cargo test`
