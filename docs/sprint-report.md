# Sprint Report — Epic 4: Combat Depth

**Datum:** 2026-02-28
**Epic:** 4 — Combat Depth
**Status:** ✅ Abgeschlossen

---

## Zusammenfassung

Epic 4 "Combat Depth" wurde vollständig implementiert. Alle 12 Stories wurden in einem Sprint abgearbeitet. Die Testsuite wuchs von **537 Tests** (Ende Epic 3 + epic4-prep) auf **601 Tests**.

---

## Stories

| Story | Titel | Tests gesamt | Commit | Status |
|-------|-------|:------------:|--------|:------:|
| 4-1 | Scout Drone AI | 547 | `fb449c8a` | ✅ done |
| 4-7+4-9 | Faction Territories + Damage Feedback | 552 | `e8b96409` | ✅ done |
| 4-2+4-3+4-4 | Fighter / Heavy Cruiser / Sniper | 567 | `ba7e4d70` | ✅ done |
| 4-5 | Swarms | 573 | `5b4b7cf3` | ✅ done |
| 4-6 | Faction Behaviors | 581 | `50c1206f` | ✅ done |
| 4-8 | Attack Telegraphing | 584 | `f92ab102` | ✅ done |
| 4-10 | Trader Ships | 589 | `d20a15e3` | ✅ done |
| 4-11 | Distance Difficulty Scaling | 596 | `fd8c5644` | ✅ done |
| 4-12 | Enemy Respawn | 601 | `cf1a2249` | ✅ done |

---

## Implementierte Features

### Enemy-Typen
| Typ | Marker | Besonderheit |
|-----|--------|--------------|
| Scout Drone | `ScoutDrone` | AI-FSM + ErraticOffset (Zufallsbewegung) |
| Fighter | `Fighter` | Aggressiv, große AggroRange(400), FleeThreshold(0.1) |
| Heavy Cruiser | `HeavyCruiser` | Langsam, HP(200), großer Collider |
| Sniper | `Sniper` | `PreferredRange { min, max }`, hält Distanz |
| Swarm | `Swarm / SwarmLeader / SwarmFollower` | 3-5 koordiniert, Leader + Follower AI |
| Trader Ship | `TraderShip` | `TraderRoute`, `FactionId::Neutral`, fliegt zwischen Punkten |

### AI-Systeme (src/social/enemy_ai.rs)
- `update_scout_drone_ai` — FSM + erratischer Offset
- `update_fighter_ai` — Aggressives Verfolgen
- `update_heavy_cruiser_ai` — Langsam aber mächtig
- `update_sniper_ai` — Distanzhalten via PreferredRange
- `update_swarm_ai` — Leader-Follower-Koordination
- `update_enemy_facing` — Transform-Rotation folgt Blickrichtung
- `update_trader_ships` — Lineare Route mit Richtungsumkehr

### Fraktions-System (src/social/faction.rs)
- `faction_at_position(x, y, seed) -> FactionId` — Noise-basierte Territorien
- `FactionBehaviorProfile { aggro_multiplier, flee_multiplier, preferred_attack_style }`
- `AttackStyle`: `Aggressive`, `Defensive`, `Erratic`
- `apply_faction_modifiers()` — Pure Function, wendet Profil auf AI-Parameter an

### Damage Feedback (src/shared/components.rs)
- `DamageFlash { timer: f32, color: FlashColor }` Komponente
- `FlashColor` Enum: `Red` (Spieler getroffen), `White` (Enemy getroffen)

### Distance Difficulty Scaling (src/world/mod.rs)
- `enemy_stats_for_distance(distance, base_health, base_damage) -> (f32, f32)`
- `WorldConfig` Felder: `difficulty_health_scale_per_100u`, `difficulty_count_scale_per_100u`
- ScoutDrone HP skaliert mit Distanz beim Spawn

### Enemy Respawn (src/core/spawning.rs)
- `SpawnType::Swarm` neue Variante
- `swarm_respawn_delay` in SpawningConfig
- SwarmFollower von Respawn ausgeschlossen
- SwarmLeader-Respawn triggert ganzen neuen Schwarm

---

## Architektur-Muster (bestätigt)

- **Pure FSM**: `next_state(&AiState, &AiContext) -> AiState` — vollständig testbar ohne App
- **Pure Functions für Skalierung**: `enemy_stats_for_distance`, `apply_faction_modifiers` — keine Rand-Calls direkt in Systemen
- **Core/Rendering-Trennung**: `NeedsFighterVisual`, `NeedsHeavyCruiserVisual`, `NeedsSniperVisual`, `NeedsTraderVisual` — Rendering hängt Mesh an
- **B0002 Buffer Pattern**: `PendingEnemyShotQueue` für Enemy-Schuss-Events

---

## Commit-Historie

```
feat(4-12): Enemy Respawn — SpawnType::Swarm, swarm_respawn_delay config. 601 tests.
feat(4-11): Distance Difficulty Scaling — enemy_stats_for_distance pure fn. 596 tests.
feat(4-10): Trader Ships — TraderShip marker, TraderRoute component. 589 tests.
feat(4-8): Attack Telegraphing — FacingDirection/TurnRate, update_enemy_facing. 584 tests.
feat(4-6): Faction Behaviors — FactionBehaviorProfile, AttackStyle enum. 581 tests.
feat(4-5): Swarm — Swarm/SwarmLeader/SwarmFollower, spawn_swarm, update_swarm_ai. 573 tests.
feat(4-2+4-3+4-4): Fighter/HeavyCruiser/Sniper — markers, AI systems, respawn timers. 567 tests.
feat(4-7+4-9): faction_at_position noise-based territories + DamageFlash. 552 tests.
feat(4-1): Scout Drone AI — AiState FSM, ErraticOffset, EnemyFireCooldown. 547 tests.
feat(epic4-prep): SocialPlugin skeleton — FactionId, PatrolRadius, AggroRange, AiState FSM. 537 tests.
```

---

## Nächster Sprint

**Epic 5: Progression & Upgrades** (Dependencies: Epic 3 ✅)
- 5.1 Craft Upgrades
- 5.2 Upgrade Tiers
- 5.3 Weapon Upgrades
- 5.4 Recipe Discovery
- 5.5 Visual Ship Changes
- 5.6 Craft-Buy Distinction
