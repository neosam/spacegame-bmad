# Sprint Report — BF-1: Respawn & Tutorial Zone

**Sprint:** BF-1 (Bugfix Sprint)
**Datum:** 2026-02-28
**Stories:** 2 / 2 abgeschlossen
**Tests:** 696 (alle grün), +5 neue Tests

---

## Abgeschlossene Stories

### BF-1: Respawn an letzter Station ✅
- `LastDockedStation { position: Vec2 }` Resource in `src/core/station.rs`
- `update_docking` speichert Station-Position bei Dock
- `handle_player_death` nutzt `Option<Res<LastDockedStation>>` — Fallback `Vec3::ZERO`
- 2 neue Tests

### BF-2: Tutorial Zone Schutz ✅
- `TutorialZoneChunks { coords: HashSet<ChunkCoord> }` Resource in `src/world/mod.rs`
- `init_tutorial_zone_chunks` registriert 9 Chunks (3×3 rund um Ursprung) beim Startup
- `update_chunks` schließt Tutorial-Zone-Chunks vom Unloading aus
- 3 neue Tests

---

## Ergebnis

| Metrik | Wert |
|--------|------|
| Tests gesamt | 696 |
| Neue Tests | +5 |
| Regressions | 0 |
| Bugs gefixt | 2 |
