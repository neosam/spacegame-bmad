# Sprint Report — Epic 5: Progression & Upgrades

**Datum:** 2026-02-28
**Epic:** 5 — Progression & Upgrades
**Status:** ✅ Abgeschlossen

---

## Zusammenfassung

Epic 5 "Progression & Upgrades" wurde vollständig implementiert. Alle 6 Stories wurden kohärent als ein Feature-Set umgesetzt. Die Testsuite wuchs von **605 Tests** (Ende Epic 4) auf **622 Tests** (+17 neue Tests).

---

## Stories

| Story | Titel | Status |
|-------|-------|:------:|
| 5-1 | Craft Upgrades | ✅ done |
| 5-2 | Upgrade Tiers | ✅ done |
| 5-3 | Weapon Upgrades | ✅ done |
| 5-4 | Recipe Discovery | ✅ done |
| 5-5 | Visual Ship Changes | ✅ done |
| 5-6 | Craft-Buy Distinction | ✅ done |

---

## Implementierte Features

### Upgrade-System (`src/core/upgrades.rs`)
- `ShipSystem` enum (8 Varianten): Thrust, MaxSpeed, Rotation, EnergyCapacity, EnergyRegen, ScannerRange, HullStrength, CargoCapacity
- `WeaponSystem` enum (5 Varianten): LaserDamage, LaserFireRate, SpreadDamage, SpreadFireRate, EnergyEfficiency
- `AcquisitionMethod { CraftOnly, BuyOnly, CraftOrBuy }` — Story 5-6
- `InstalledUpgrades { ship: HashMap<ShipSystem, u8>, weapon: HashMap<WeaponSystem, u8> }`
- `DiscoveredRecipes` — startet mit 13 Tier-1-Rezepten (8 Schiff + 5 Waffe)
- `CraftingRequest` Resource — entkoppelter Crafting-Trigger
- `BaseStats` Resource — speichert unkomprimierte Basiswerte für korrekte Recompute
- `compute_upgrade_multiplier(tier) -> f32` — Pure Function (tier 0=1.0, tier 5=1.5)
- `can_craft()` — Pure Function, prüft Materialien + Credits

### Systems
- `init_base_stats` (Startup) — initialisiert BaseStats aus FlightConfig + WeaponConfig
- `process_crafting_request` — verarbeitet CraftRequest, zieht Materialien/Credits ab
- `apply_upgrade_effects` — skaliert FlightConfig + WeaponConfig aus BaseStats × Multiplikator
- `emit_craft_events` — B0002-safe Event-Emission (PendingCraftEvents Buffer)
- `mark_player_needs_upgrade_visual` — setzt `NeedsShipUpgradeVisual` bei Änderung
- `discover_recipe_for_chunk` — Tier-2/3 Rezepte durch Chunk-Entdeckung

### Visuelle Schiffsveränderungen
- `NeedsShipUpgradeVisual` Marker-Komponente (src/shared/components.rs)
- `update_ship_upgrade_visual` in Rendering — Hull-Tier → Schiffsfarbe:
  - Tier 0: Gold | Tier 1-2: Blau | Tier 3-4: Hellgold | Tier 5: Silber

### Save/Load
- SAVE_VERSION 4 → 5
- 13 neue `#[serde(default)]` Felder in PlayerSave (je 1 pro System)
- `InstalledUpgrades` wird in from_world/apply_to_world korrekt serialisiert

### Events
- `GameEventKind::UpgradeCrafted { system_name: String, tier: u8 }`
- `EventSeverityConfig`: 14 → 15 Mappings (UpgradeCrafted → Tier2)

---

## Neue Tests (`tests/craft_upgrades.rs`)

17 neue Tests (10 Integration + 7 Unit):
1. `can_craft_with_sufficient_materials` — Pure Function
2. `can_craft_fails_insufficient_scrap` — Pure Function
3. `can_craft_fails_insufficient_credits` — Pure Function
4. `upgrade_multiplier_scales_correctly` — Pure Function
5. `craft_deducts_materials` — App Test
6. `craft_increments_tier` — App Test
7. `craft_requires_docked` — App Test (Crafting nur wenn gedockt)
8. `upgrade_save_load_roundtrip` — Serialisierung
9. `can_craft_buy_only_recipe` — AcquisitionMethod
10. `discover_recipe_adds_to_list` — DiscoveredRecipes

---

## Architektur-Muster (bestätigt)

- **Pure Functions**: `can_craft()`, `compute_upgrade_multiplier()` — vollständig testbar ohne App
- **BaseStats Pattern**: Basiswerte gespeichert, dann Recompute bei jedem Upgrade
- **B0002 Buffer**: `PendingCraftEvents` für Event-Emission
- **Core/Rendering-Trennung**: `NeedsShipUpgradeVisual` Marker — Rendering reagiert
- **`is_changed()` Guard**: `apply_upgrade_effects` läuft nur wenn `InstalledUpgrades` geändert

---

## Commit

```
feat(5-1..5-6): Epic 5 Progression & Upgrades — craft system, 5-tier upgrades, weapon upgrades, recipe discovery, visual ship changes, craft/buy distinction. 622 tests.
```

---

## Nächster Sprint

**Epic 6a: Companion Core** (Dependencies: Epic 3 ✅, Epic 4 ✅)
- 6a-1 Recruit Companion
- 6a-2 Companion Follow
- 6a-3 Wingman Commands
- 6a-4 Companion Visuals
- 6a-5 Companion Survival
- 6a-6 Companion Save

---

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
