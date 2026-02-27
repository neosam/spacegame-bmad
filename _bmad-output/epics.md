# Void Drifter - Development Epics

## Epic Overview

| # | Epic Name | Category | Dependencies | Est. Stories |
|---|-----------|----------|--------------|-------------|
| 0 | Arcade Prototype | Must | None | 5-7 |
| 1 | Open World Foundation | Must | Epic 0 | 10-12 |
| 2 | Tutorial Zone | Must | Epic 1 | 6-8 |
| 3 | Stations & Economy | Must | Epic 1 | 6-8 |
| 4 | Combat Depth | Must | Epic 1 | 12-15 |
| 5 | Progression & Upgrades | Should | Epic 3 | 5-7 |
| 6a | Companion Core | Should | Epic 3, 4 | 5-7 |
| 6b | Companion Personality | Should | Epic 6a | 4-6 |
| 7 | Boss Encounters | Should | Epic 4, 5 | 4-6 |
| 8 | Logbook UI | Should | Epic 6a | 4-6 |
| 9 | Wormhole Mini-Levels | Could | Epic 4, 5 | 5-7 |
| 10 | Art & Audio Polish | Could | Core epics stable | 6-8 |
| 11 | Platform & Release | Could | All MVP epics | 5-7 |

---

## Epic 0: Arcade Prototype

### Goal

Prove that flying and shooting feels good. This is the foundation for everything — if this doesn't feel right, nothing else matters.

### Scope

**Includes:**
- Physics-based flight (thrust, rotate, drift, soft speed cap)
- Laser weapon (hitscan pulse, energy-free)
- Spread weapon (projectile, energy cost)
- Weapon switching
- Basic asteroid spawning (destructible)
- Basic enemy spawning (Scout Drone)
- Screen shake, impact particles, thruster particles (basic visual juice)
- Player death and instant respawn
- 60fps validation on Tier 1 hardware

**Excludes:**
- Chunk system, procedural generation
- Stations, economy, upgrades
- Companions, factions
- Save system
- Tutorial zone
- UI beyond basic HUD (health, energy)

### Dependencies

None — this is the starting point.

### Deliverable

A playable arcade loop: fly, shoot asteroids and drones, die, respawn, repeat. The question this answers: "Does the core feel good?"

### Stories

1. As a player, I can thrust and rotate my ship with physics-based flight so that movement feels responsive and satisfying
2. As a player, I can fire a laser (hitscan pulse) in my facing direction so that I have a reliable baseline weapon
3. As a player, I can fire spread projectiles that consume energy so that I have a tactical weapon choice
4. As a player, I can switch between weapons instantly so that combat has tactical variety
5. As a player, I can destroy asteroids and Scout Drones so that the arcade loop has targets
6. As a player, I see visual feedback (screen shake, particles, flashes) on impacts so that combat feels satisfying
7. As a player, I respawn instantly after death so that the failure loop is fast and non-punishing

---

## Epic 1: Open World Foundation

### Goal

Transform the arcade prototype into an explorable infinite world with persistent state.

### Scope

**Includes:**
- Chunk-based procedural world generation (seed-deterministic)
- Biome types: Deep Space, Asteroid Fields, Wreck Fields
- Noise-based biome distribution
- Minimap with unidentified blips
- World map (player-revealed)
- Chunk loading/unloading around player
- Event-Observer pattern (infrastructure — all systems emit events from here forward)
- Basic delta-save system (player position, chunk state, inventory)
- WASM build validation (runs alongside native from Sprint 1)

**Excludes:**
- Stations (Epic 3)
- Enemy variety beyond Scout Drone (Epic 4)
- Faction territories (Epic 4)
- Tutorial Zone (Epic 2)
- Companions (Epic 6a)
- Upgrades (Epic 5)

### Dependencies

Epic 0 (flight and combat must work)

### Deliverable

An infinite procedural world the player can fly through endlessly, with different biomes, a minimap, and a world map. Progress is saved. Events are emitted for future logbook use.

### Stories

1. As a player, I can fly in any direction and the world generates seamlessly around me so that the universe feels infinite
2. As a player, I encounter different biome types (deep space, asteroid fields, wreck fields) so that the world has visual and tactical variety
3. As a player, I see unidentified blips on my minimap so that I always have a "what's that?" to fly toward
4. As a player, I can open a world map showing areas I've explored so that I have a sense of progress and orientation
5. As a developer, the chunk system loads and unloads chunks based on player position so that memory stays bounded
6. As a developer, biome distribution is determined by noise layers so that the world is seed-deterministic
7. As a developer, all game systems emit events via Bevy's event system so that future systems (logbook, telemetry) can subscribe without code changes
8. As a player, my position and world state are saved when I quit so that I can continue where I left off
9. As a player, the save is delta-based (seed + changes) so that save files are small
10. As a developer, the WASM build runs alongside native from this epic forward so that web compatibility is never an afterthought
11. As a developer, soft speed cap prevents the player from outrunning chunk generation so that the world always feels complete

---

## Epic 2: Tutorial Zone

### Goal

Create a contained starting experience that teaches core mechanics through play, without the player realizing it's a tutorial.

### Scope

**Includes:**
- Tutorial zone as a special chunk type with constraint-based generation
- Gravity well boundary (defective tractor beam, linear pull formula)
- Ability unlock sequence: Fly → Laser (at wreck) → Spread (at station)
- Generator mini-boss fight (requires Spread to destroy)
- Generator destruction feedback cascade (shockwave, field dissolution, Tier 1 logbook event)
- Station transformation (defective → functional home station)
- Constraint validation (guaranteed playable layouts)
- 100-seed automated test

**Excludes:**
- Companion encounter (companions come after tutorial in open world)
- Crafting system
- Economy beyond receiving Spread as gift
- Mini-levels

### Dependencies

Epic 1 (chunk system, event-observer, basic save)

### Deliverable

A new player spawns in a contained area, learns to fly, finds a laser, receives spread at a station, destroys the generator, and emerges into the open world with a home station. Blind playtesters complete it in under 10 minutes.

### Stories

1. As a player, I spawn in a contained area with a visibly defective station so that I have an immediate point of interest
2. As a player, I feel the gravity well pull me back when I fly too far so that I understand I need to stay and explore this area first
3. As a player, I find a laser at a nearby wreck that auto-docks on approach so that I gain my first weapon without UI complexity
4. As a player, enemies appear after I have the laser so that I learn combat with a weapon available
5. As a player, I dock at the station and receive the Spread weapon so that I learn weapon switching and energy management
6. As a player, I can attack and destroy the generator using Spread so that I experience my first "boss" moment and earn freedom
7. As a player, I witness an epic destruction cascade when the generator explodes so that the moment feels like a breakthrough
8. As a developer, the tutorial zone generates valid layouts for any seed with constraint validation so that no player gets stuck

---

## Epic 3: Stations & Economy

### Goal

Give the player reasons to return to stations and a resource loop that drives exploration.

### Scope

**Includes:**
- Station docking mechanic
- Station UI (shop, inventory)
- Credits system (earn from enemies/discoveries, spend at stations)
- Material drops (Common, Uncommon, Rare tiers)
- Buy-only items at stations
- Material drop scaling with `distance_from_spawn`
- Station price scaling (reward_value > station_price scaling)
- Partial credit loss on death
- Multiple station types with different inventories
- Save system extended with economy data

**Excludes:**
- Crafting recipes (Epic 5)
- Upgrade tiers (Epic 5)
- Companion recruitment at stations (Epic 6a)

### Dependencies

Epic 1 (open world, save system)

### Deliverable

Players dock at stations, buy items, collect resources from enemies and asteroids, and manage a credit balance. Death costs money. Different stations sell different things.

### Stories

1. As a player, I can dock at stations by approaching and pressing interact so that I have safe harbors in the world
2. As a player, I see a station shop UI where I can buy items with credits so that I can improve my ship
3. As a player, I earn credits from destroying enemies and discovering new areas so that exploration is economically rewarded
4. As a player, I collect material drops (Common, Uncommon, Rare) from asteroids and enemies so that I accumulate crafting resources
5. As a player, I lose a portion of my credits on death so that death has meaningful but non-devastating consequences
6. As a player, I find different station types with different inventories so that I have reasons to visit multiple stations
7. As a developer, drop values and station prices scale with distance so that the economy stays balanced across the world

---

## Epic 4: Combat Depth

### Goal

Transform basic combat into a rich tactical experience with enemy variety, faction identity, and neutral life.

### Scope

**Includes:**
- All 5 enemy types: Scout Drone, Fighter, Heavy Cruiser, Sniper, Swarm
- Faction-specific AI behavior (Pirates, Aliens, Military, Rogue Drones)
- Faction territory noise layer
- Enemy spawn system (faction patrols + rare random encounters)
- Enemy telegraphing (facing direction, projectile visibility)
- Damage feedback (red blink taken, white blink dealt)
- Neutral entities: Trader Ship (MVP neutral type)
- Difficulty scaling with `distance_from_spawn` (enemy count, type distribution, AI sophistication)
- Respawn delay timer (tunable per spawn point)

**Excludes:**
- Boss encounters (Epic 7)
- Companion combat coordination (Epic 6a)
- Damage types beyond flat damage (post-MVP)

### Dependencies

Epic 1 (open world, chunk system)

### Deliverable

The world feels alive with diverse enemies, distinct factions, and neutral traders. Combat varies by location and faction. Near spawn is safe, far from spawn is dangerous.

### Stories

1. As a player, I fight Scout Drones that are fast and erratic so that I learn basic combat
2. As a player, I fight Fighters that aggressively pursue me so that I face a standard threat
3. As a player, I fight Heavy Cruisers that are slow but powerful so that I need strategy or upgraded weapons
4. As a player, I fight Snipers that keep distance and use precise lasers so that I'm forced to keep moving
5. As a player, I fight Swarms of 3-5 coordinated enemies so that I face overwhelming numbers
6. As a player, I notice different factions behave differently (pirates aggressive, military defensive, aliens varied) so that the world has political identity
7. As a developer, faction territories are determined by a noise layer so that borders emerge organically from the seed
8. As a player, I see enemies telegraph their attacks through facing direction so that I can learn to read and avoid threats
9. As a player, I see damage feedback (red blink when hit, white blink when I hit) so that combat is readable
10. As a player, I encounter Trader Ships flying between stations so that the world feels populated beyond enemies
11. As a player, I notice more and tougher enemies the further I fly from spawn so that distance naturally increases challenge
12. As a developer, enemies respawn on a tunable delay timer so that areas repopulate without feeling infinite

---

## Epic 5: Progression & Upgrades

### Goal

Give the player visible, meaningful ship improvement that rewards exploration.

### Scope

**Includes:**
- 5-tier upgrade system for 8 ship systems (Thrust, Max Speed, Rotation, Energy Capacity, Energy Regen, Scanner Range, Hull Strength, Cargo Capacity)
- Weapon upgrade system (damage, fire rate, energy efficiency per weapon type)
- Crafting system (recipes + materials)
- Recipe discovery through exploration and station purchase
- Craft-only vs buy-only item distinction
- Visual ship changes with upgrades
- Acceptance criterion: no upgrade costs more than 2 outer-loop cycles (~20 min)

**Excludes:**
- Boss-specific loot (Epic 7)
- Companion gear preferences (Epic 6b)

### Dependencies

Epic 3 (economy and materials must exist)

### Deliverable

Players craft and buy upgrades that visibly improve their ship. The progression feels rewarding without grind walls.

### Stories

1. As a player, I can craft upgrades from materials at a station so that exploration resources turn into ship improvements
2. As a player, I see 5 tiers of upgrades per ship system so that I have a clear progression path
3. As a player, I can upgrade my weapons independently so that I specialize my combat style
4. As a player, I discover new recipes through exploration and station purchase so that crafting rewards curiosity
5. As a player, my ship visually changes as I upgrade so that my progression is visible
6. As a player, I notice some items are craft-only and some are buy-only so that I have reasons for both activities

---

## Epic 6a: Companion Core

### Goal

Introduce companions as gameplay-relevant wingmen the player cares about.

### Scope

**Includes:**
- Companion recruitment at stations and through events
- Companion follow-AI (follows player ship)
- Simple wingman commands (Attack, Defend, Retreat)
- Companion faction-specific visual silhouettes
- Companion survival on player death (retreat to station)
- Companion data in save system

**Excludes:**
- Opinion system (Epic 6b)
- Bark system (Epic 6b)
- Advanced companion AI behaviors (Epic 6b)

### Dependencies

Epic 3 (stations for recruitment), Epic 4 (combat for wingman relevance)

### Deliverable

Players recruit companions at stations who follow them and fight alongside them. Companions can be commanded in combat and survive player death.

### Stories

1. As a player, I can recruit a companion at a station so that I gain a wingman
2. As a player, my companion follows my ship so that I'm not alone in the void
3. As a player, I can issue Attack/Defend/Retreat commands so that I have tactical control over my companion
4. As a player, my companion has a distinct visual silhouette based on their faction origin so that they feel unique
5. As a player, my companions survive when I die and retreat to the station so that death doesn't destroy my crew
6. As a player, my companion roster is saved so that my crew persists between sessions

---

## Epic 6b: Companion Personality

### Goal

Transform companions from gameplay tools into characters the player cares about emotionally.

### Scope

**Includes:**
- Opinion system: HashMap<(EntityId, EntityId), i32> — companions judge player and each other
- Bark system: contextual one-liners based on situation and personality
- Opinion changes from player actions (protecting raises, reckless lowers)
- Companion-to-companion opinions and barks
- Extended companion AI (personality-influenced combat behavior)

**Excludes:**
- Dialogue trees (explicitly not in scope — barks only)
- Voice acting

### Dependencies

Epic 6a (companion core must exist)

### Deliverable

Companions express personality through barks, have opinions about the player and each other, and react emotionally to events. The player feels responsible for their crew.

### Stories

1. As a player, my companions react to situations with contextual barks so that they feel alive
2. As a player, I notice companions have opinions about me that change based on my actions so that my choices matter
3. As a player, I notice companions have opinions about each other so that crew dynamics emerge
4. As a player, companions behave differently in combat based on their personality so that each feels unique

---

## Epic 7: Boss Encounters

### Goal

Add climactic combat highlights that reward courage and mark progression milestones.

### Scope

**Includes:**
- Boss spawn via noise layer (`boss_chance = noise(chunk_x, chunk_y, seed + BOSS_OFFSET) * distance_factor`)
- 4 faction boss types (Pirate Flagship, Alien Mothership, Military Cruiser, Drone Swarm Core)
- Boss telegraphing (minimap warning, companion bark reactions)
- Boss loot scaling with distance
- Boss difficulty scaling with distance
- Boss flee cooldown (chunk-based timer before re-encounter)
- Boss encounters as Tier 1 logbook events

**Excludes:**
- Boss-specific arenas or phases (post-MVP complexity)

### Dependencies

Epic 4 (enemy/faction system), Epic 5 (upgrade system for balanced loot)

### Deliverable

Players encounter rare, powerful bosses that are faction-themed, properly telegraphed, and rewarding to defeat. Bosses are always optional.

### Stories

1. As a player, I encounter rare boss enemies further from spawn so that exploration has climactic moments
2. As a player, bosses are telegraphed on my minimap and my companions react nervously so that I can prepare or flee
3. As a player, each faction has a unique boss type so that faction identity extends to their strongest units
4. As a player, defeating a boss gives guaranteed valuable loot scaling with distance so that the risk is rewarded
5. As a player, I can flee from a boss without permanent consequence so that bosses never block progression

---

## Epic 8: Logbook UI

### Goal

Surface the events the game has been recording since Epic 1 in a readable, shareable format.

### Scope

**Includes:**
- Logbook UI screen accessible from HUD
- Event display with timestamps, categories, severity filtering
- Soft milestone chapter headings (first companion, first new biome, etc.)
- Logbook in save system

**Excludes:**
- Narrative text generation from events (post-MVP)
- Export/share functionality (post-MVP)

### Dependencies

Epic 6a (companion events make the logbook meaningful)

### Deliverable

Players can open a logbook that shows their journey — battles, discoveries, companion events, boss encounters — organized by milestones.

### Stories

1. As a player, I can open a logbook from the HUD so that I can review my journey
2. As a player, I see events organized by severity (Tier 1 always, Tier 2 notable) so that the logbook isn't cluttered
3. As a player, I see soft milestone chapter headings so that my journey has narrative structure
4. As a player, my logbook persists in my save file so that my story is never lost

---

## Epic 9: Wormhole Mini-Levels

### Goal

Add focused, finite challenges as contrast to the open world sandbox.

### Scope

**Includes:**
- Wormhole spawn in open world (frequency scales with distance)
- Wormhole visual identity (most prominent element in world)
- Combat Arena mini-level type (MVP — only type)
- Isolated scene architecture (separate from main world)
- Mini-level difficulty scaling with wormhole distance
- Crash recovery (respawn at wormhole entrance)
- Cleared flag in save system

**Excludes:**
- Exploration Maze, Loot Vault, Mixed Challenge types (post-MVP)

### Dependencies

Epic 4 (combat system for arena), Epic 5 (upgrades for balanced rewards)

### Deliverable

Players find wormholes in the open world, enter combat arenas, survive enemy waves, and earn rewards. Completed arenas are marked on the map.

### Stories

1. As a player, I see visually prominent wormholes in the world so that I always notice them
2. As a player, I can enter a wormhole to start a combat arena so that I have focused challenges
3. As a player, I fight enemy waves in an enclosed arena so that combat is intense and concentrated
4. As a player, I earn guaranteed rewards for completing an arena so that the risk is worth it
5. As a developer, mini-levels are isolated scenes that don't interfere with main world state so that the architecture is clean

---

## Epic 10: Art & Audio Polish

### Goal

Elevate the game's aesthetic from functional to polished and atmospheric.

### Scope

**Includes:**
- Detailed procedural ship meshes (player ship via lyon)
- Full visual juice suite (all particle effects, screen shake, drift trails)
- Visual juice settings (Low/Medium/High)
- Background gradients and star field layers
- Curated color palette system (5-6 schemes, seed-selected)
- Music integration (MVP: 2 tracks + crossfade)
- Full sound effect suite (~25-35 sounds)
- Ambient audio loops (5 environments)

**Excludes:**
- Interactive music state machine (post-MVP — MVP uses simple crossfade)
- Custom composed soundtrack

### Dependencies

Core epics (0-6) stable — polish on top of working gameplay

### Deliverable

The game looks and sounds like a finished product. Warm, inviting atmosphere. Every action has satisfying feedback.

### Stories

1. As a player, my ship and stations have detailed vector art silhouettes so that the world looks polished
2. As a player, every action produces satisfying visual and audio feedback so that the game feels juicy
3. As a player, I can adjust visual juice intensity so that the game runs smoothly on my hardware
4. As a player, the background is colorful and atmospheric (never black) so that the void feels inviting
5. As a player, music shifts between exploration and combat so that audio matches the gameplay mood
6. As a player, each environment has distinct ambient audio so that biomes have audio identity

---

## Epic 11: Platform & Release

### Goal

Prepare the game for distribution across all target platforms.

### Scope

**Includes:**
- WASM build optimization (target under 50MB, stretch under 30MB)
- WASM feature subset (reduced particles, smaller chunk radius, audio fallback)
- Steam Deck testing and UI scaling
- Full save system (cloud saves via Steam)
- Performance optimization pass
- Steam integration (achievements, cloud saves)
- Build pipeline for all platforms

**Excludes:**
- Mod support
- Localization
- Crash reporting and telemetry (architecture allows, not implemented)

### Dependencies

All MVP epics complete

### Deliverable

The game ships on PC (Win/Linux/Mac), Web (WASM), and Steam Deck with platform-appropriate performance and features.

### Stories

1. As a player, I can play in a web browser with acceptable performance so that the game is accessible without installation
2. As a player, the WASM build is under 50MB so that it loads quickly
3. As a player, I can play on Steam Deck with readable UI so that portable play works
4. As a player, my saves sync via Steam Cloud so that I can play on multiple machines
5. As a player, the game runs at 60fps with 200 entities on screen so that performance is consistently smooth
6. As a developer, I have a build pipeline producing artifacts for all platforms so that releases are automated
