---
stepsCompleted: [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14]
inputDocuments:
  - '_bmad-output/game-brief.md'
  - '_bmad-output/brainstorming-session-2026-02-25.md'
documentCounts:
  briefs: 1
  research: 0
  brainstorming: 1
  projectDocs: 0
workflowType: 'gdd'
lastStep: 14
project_name: 'spacegame-bmad'
user_name: 'Simon'
date: '2026-02-25'
game_type: 'shooter'
game_name: 'Void Drifter'
---

# Void Drifter - Game Design Document

**Author:** Simon
**Game Type:** Arcade Space Shooter
**Target Platform(s):** PC (Windows, Linux, macOS), Web (WASM), Steam Deck (stretch)

---

## Executive Summary

### Game Name

**Void Drifter** — Asteroids meets crew RPG in an infinite universe that writes your story.

### Core Concept

Void Drifter is a 2D arcade space game set in an infinite, seed-generated universe. The player pilots a ship with physics-based flight — rotate, thrust, drift — through an open world full of discoveries, dangers, and companions. Starting as a lone pilot in the void, they explore procedurally generated biomes, discover stations, recruit wingmen, and navigate faction conflicts.

Every seed creates a unique universe. Share your seed, and friends explore the same world — but their story will be completely different. The journey from lone pilot to crew captain is the emotional arc at the heart of every playthrough. Designed for solo development, every system prioritizes procedural generation and emergence over hand-crafted content.

### Target Audience

Explorer-Collectors (16+) who live for discovering new places, building emotional bonds with companions, and creating unique stories through gameplay. Players who want tight arcade controls with open-world depth.

### Unique Selling Points (USPs)

1. **Automatic Event Logbook** — The game captures every meaningful moment (battles, discoveries, crew events) and preserves them as a readable story. No other game in this space writes and keeps your narrative.
2. **Companion Progression System** — Recruit, bond with, lose, and rebuild your crew through gameplay. NPCs express personality through contextual barks and opinion scores — they judge you and each other.
3. **Infinite Persistent Universe** — Asteroids-style arcade flight in a seed-generated world that remembers your actions through delta-based saves. Every session builds on the last.
4. **Emergent Crew Journey** — The progression from solitude to family emerges naturally from gameplay. No scripted cutscenes — the emotional arc is yours.

### Game Type

**Arcade Space Shooter** — open-world exploration, resource-driven progression, emergent sandbox gameplay.

*Framework: Shooter template adapted for 2D arcade space combat in an open-world context.*

---

## Target Platform(s)

### Primary Platform

**PC (Windows, Linux, macOS)** — Native builds via Bevy/Rust. Keyboard, mouse, and controller support.

**Platform Tier Hierarchy:**

| Tier | Platform | Role |
|------|----------|------|
| **Tier 1** | Linux Native | Primary development and testing platform |
| **Tier 2** | Windows, macOS Native | Full feature parity with Tier 1 |
| **Tier 3** | Web/WASM | Feature-subset allowed (e.g. smaller chunk radius, reduced particle density). If a feature blocks on WASM, WASM gets an adapted version — native platforms are never limited by WASM constraints. |
| **Stretch** | Steam Deck | Native Linux + controller, UI scaling for 7" screen |

### Platform Considerations

- 60fps non-negotiable on all tiers
- WASM build tested alongside native from sprint 1 — no "port later" approach
- WASM constraints (single-threaded, limited memory) are handled through adaptive settings, not feature cuts on native
- Steam features (achievements, cloud saves, workshop) planned for full launch

### Control Scheme

**Classic Asteroids Model** — the ship rotates to aim. No independent aim direction. This keeps controls consistent across keyboard and controller, simplifies enemy AI balancing, and preserves the authentic Asteroids feel.

**Player Actions:**

| Action | Description |
|--------|-------------|
| Thrust forward | Accelerate in facing direction |
| Rotate left/right | Turn the ship |
| Fire primary weapon | Shoot current weapon |
| Switch weapon | Cycle through available weapons |
| Issue wingman command | Direct companion behavior |
| Interact / Dock | Engage with stations and objects |
| Toggle map | Switch between minimap and world map |

**Controller Layout (reference for button budget):**
- Left Stick: Rotate + thrust
- Right Trigger: Fire
- Bumpers: Weapon switch
- Face buttons: Wingman command, interact
- D-Pad: Quick menu navigation

**Accessibility:** Key/button remapping planned as post-MVP feature. Bevy's input system supports rebinding — architecture should not preclude it.

**Design Note:** Twin-stick aiming may be evaluated as a post-MVP option, but Classic Asteroids rotation is the core control identity.

### Distribution

Distribution strategy (itch.io → Steam Early Access → Steam Full Launch) is detailed in the Game Brief. Storefronts are not platforms — distribution planning is handled separately from platform targeting.

---

## Target Audience

### Demographics

- **Age Range:** 16+ — mature themes (faction conflict, companion loss) but no explicit content
- **Gender:** Broadly appealing — no gender-specific design assumptions
- **Player Type:** Explorer-Collectors (Bartle's Taxonomy) — driven by discovery and attachment

### Gaming Experience

**Accessible entry, scalable depth** — new players learn through gameplay, experienced players find mastery through advanced mechanics (weapon switching, crew management, faction strategy). No genre expertise required. The core loop (fly, shoot, discover) is self-explanatory.

**Onboarding:** No tutorial screens. The spawn area is the tutorial — low enemy density, a visible station nearby, and a companion encounter within the first 5 minutes teach all core mechanics through play. Accessible entry: one button to thrust, one to shoot, one to rotate. The first 5 minutes teach everything needed to play. Scalable depth: weapon switching, crew management, and faction strategy reward experienced players who invest time.

### Session Length

**Design Target: 30-45 minutes** — optimized for sessions that deliver at least 3 complete outer-loop cycles (Discover → Engage → Reward → Decide).

**Pacing implications:**
- One Discover → Engage → Reward cycle every ~10 minutes
- Shorter sessions (15 min) still deliver at least one meaningful discovery
- Longer sessions (1-2 hours) enable deep exploration runs into unknown territory

**Station Density (Design Constraint):** Stations should feel "just far enough to build anticipation, close enough to reach before boredom." Concrete density values are calibrated through playtesting in Sprint 0 — the GDD defines the intent, not the number. Station density is a tunable world-generation parameter.

**Save anywhere without consequences** — respects the player's time regardless of session length.

### Player Motivations

- **Discovery** — "What's behind that blip on the minimap?" The pull to explore drives engagement
- **Collection** — Companions, weapons, upgrades, explored map regions — visible progress
- **Emotional Bonds** — Companions with opinions create attachment that brings players back
- **Personal Story** — The logbook preserves a unique narrative worth sharing and re-reading
- **Mastery** — From struggling near spawn to cruising with a fleet of wingmen
- **Sharing** — Seeds and logbook entries create natural shareable content. Explorer-Collectors document their journeys — the game gives them the tools to do it

---

## Goals and Context

### Project Goals

1. **Personal Enjoyment** — Build a game that is genuinely fun to play — for the developer first, for others second. If the creator doesn't enjoy playing it, no one else will either. This is the north star for every design decision.

2. **Technical Achievement** — Ship a complete, polished game built with Bevy and Rust. Prove that the Bevy ecosystem is ready for a full indie release. 60fps target on any hardware capable of running a modern browser with WebGL2.

3. **Player Reach** — Create a game that is widely played and shared. Seeds and logbook entries become organic marketing — players share their unique stories without being asked.

4. **Creative Evolution** — What started as a BMAD workflow demo and OpenSpec experiment evolved into a genuine game concept through brainstorming. Honor that creative momentum by building something worth playing.

### Background and Rationale

Void Drifter emerged organically from a technical experiment. The developer had already built a procedurally generated Asteroids prototype using OpenSpec, proving the core flight and generation mechanics. When that prototype was run through the BMAD brainstorming workflow, the ideas that surfaced — companions with opinions, an automatic event logbook, faction dynamics, seed-sharing — transformed a tech demo into a genuine game vision.

The timing is right: Bevy is maturing as a game engine, the Rust gamedev community is growing but starved for polished releases, and the procedural storytelling niche (proven by RimWorld, Dwarf Fortress) has an unfilled gap — no game captures and presents the player's emergent story. Void Drifter fills that gap with a warm, inviting universe that says "come explore" instead of "survive if you can."

The developer brings direct experience with Bevy/Rust and procedural generation — not theoretical knowledge, but shipped prototype code. The technical risk is lower than it appears because the foundation already exists.

---

## Core Gameplay

### Game Pillars

1. **Arcade Action** — If it doesn't feel good to fly and shoot, nothing else matters. Physics-based flight with inertia, responsive controls, satisfying weapon feedback. 60fps is non-negotiable. Every system must pass the question: "Does this make the moment-to-moment gameplay worse?"

2. **Exploration** — The infinite procedural universe is the stage. Every direction promises discovery — new biomes, stations, wormholes, and secrets. The minimap's unknown blips are the primary driver of player engagement. The world must always have something worth flying toward.

3. **Crew Bond** — Companions are not gameplay buffs — they are characters the player cares about. Wingmen with personalities, opinions about each other, and memories of shared experiences. Losing one hurts. Saving one feels heroic. The bark system makes them feel alive without dialogue trees.

4. **Emergent Stories** — The game writes itself. Faction shifts, crew conflicts, narrow escapes, unexpected alliances — all captured in an automatic logbook. The logbook doesn't create stories — it preserves the stories that the other three pillars generate.

**Pillar Prioritization:** When pillars conflict, prioritize in this order:
1. Arcade Action (the foundation — everything is built on this)
2. Exploration (the pull that drives the player forward)
3. Crew Bond (the emotional investment that keeps players coming back)
4. Emergent Stories (the record of what the other pillars created)

### Core Gameplay Loop

**Inner Loop (seconds):** The arcade heartbeat — always active, always satisfying.

```
Fly → Encounter → React (shoot, evade, scan, dock) → Survive → Fly
```

The player is always piloting. Movement never stops. Encounters interrupt flight, but the player chooses how to respond — not every encounter demands combat. Sometimes the best move is to evade a patrol, scan an anomaly, or dock at a station. The transition between flying and reacting is seamless, not a mode switch.

**Outer Loop (minutes):** The exploration cycle — drives session engagement.

```
Discover (spot blip on minimap)
  → Approach (fly toward it, build anticipation)
    → Engage (fight, explore, or dock)
      → Reward (loot, companion, lore, upgrade)
        → Absorb (read companion bark, check loot, review logbook — a pacing beat for reflection)
          → Decide (push further into unknown or return to station)
            → Discover (next blip appears)
```

**Pacing Rule:** After every Reward, there must be space for Absorb — no immediate next encounter. The calm makes the action hit harder.

**Loop Timing:**
- Inner Loop: 5-30 seconds per encounter
- Outer Loop: ~10 minutes per complete cycle
- Design target: 3 complete outer loops per 30-45 minute session

**Loop Variation:** Each iteration feels different because:
- Procedural generation ensures no two encounters are identical
- Difficulty and reward scale with `distance_from_spawn` as a continuous gradient — there are no discrete zones or level boundaries. The player feels the world getting harder organically, never through a loading screen or zone transition
- Companion barks react to context ("This place gives me the creeps" vs "Ooh, rare minerals!")
- Faction territory changes what "React" means (trade vs fight vs flee vs negotiate)
- The player's growing crew and upgrades change their tactical options

### Win/Loss Conditions

#### Victory Conditions

**No win state.** Void Drifter is a sandbox — the journey is the destination. There is no final boss, no credits roll, no "you win" screen.

**Ongoing success is measured by:**
- Map exploration coverage — how much of the universe have you seen?
- Crew size and bond strength — how many companions, how deep the relationships?
- Ship progression — upgrades, weapons, scanner range
- Logbook richness — how many stories has your journey created?
- Seed mastery — knowledge of your unique universe's layout and secrets

**Soft Milestones** emerge naturally and are marked in the logbook as chapter headings:
- First station discovered
- First companion recruited
- First new biome entered
- First faction contact
- First wormhole entered
- First companion lost and restored
- First faction alliance formed
- First dangerous biome survived

These are not imposed objectives — they are celebrations of what the player already did.

#### Failure Conditions

**No permadeath. No game over.** Death is a setback, not a punishment.

**On death:**
- Instant respawn at last visited station (automatic transport, no manual flight back)
- Lose a portion of carried currency (Terraria model — only money, not inventory or upgrades)
- Companions survive (they retreat to the station independently)
- Death marker placed on world map (skull icon at death location — provides orientation and a mini-goal to return)
- World state is preserved — nothing is undone

**Design intent:** Death is annoying enough to avoid (lost money, lost position) but never devastating enough to quit. The death marker turns "I died" into "I know where I need to go back to" — creating a new mini-objective from failure.

#### Failure Recovery

The cost of death is **money and position, not progress.** The player never loses:
- Ship upgrades or weapons
- Companion relationships or crew members
- Explored map data
- Logbook entries
- Resources in station storage

This encourages exploration risk-taking: "What's behind that dangerous-looking blip?" has a low failure cost, keeping the Exploration pillar alive even when the player is under-equipped.

---

## Game Mechanics

### Primary Mechanics

#### 1. Fly (Pillar: Arcade Action)

**The core verb — always active, always satisfying.**

- **Frequency:** Constant — the player is always flying
- **Skill tested:** Spatial awareness, momentum management, positioning
- **Feel:** Light and reactive. Immediate response to input. Inertia exists but doesn't fight the player — it enhances the feeling of speed and drift without creating frustration. The ship should feel like an extension of the player's intent, not a physics simulation to wrestle with.
- **Mechanics:**
  - **Thrust:** Accelerate in facing direction. Soft speed cap — thrust effectiveness decreases as speed increases (`thrust_effectiveness = 1.0 - (current_speed / max_speed)`). The player never hits a hard wall, but reaches a natural maximum. This also prevents outrunning chunk generation.
  - **Rotate:** Turn the ship. Rotation is instant-feeling — minimal angular inertia
  - **Drift:** When thrust stops, the ship maintains momentum. Skilled players use drift for strafing maneuvers and quick repositioning. Drift deceleration is gentle (~0.5-1 second to stop), never frustrating
- **Progression:** Thrust power, max speed, rotation speed, and drift control improve through ship upgrades
- **Interactions:** Fly enables every other mechanic — you fly to discover, fly to aim, fly to reach companions, fly to collect resources

#### 2. Shoot (Pillar: Arcade Action)

**Reactive combat with weapon variety.**

- **Frequency:** Situational — during encounters, but not every encounter requires combat
- **Skill tested:** Aiming through rotation, weapon selection, energy management, timing
- **Feel:** Punchy and satisfying. Every shot has visual and audio feedback (bloom flash, screen shake on impact, particle burst). Weapons feel distinct — laser is precise, spread is chaotic, each has personality.
- **Mechanics:**
  - **Fire:** Shoot current weapon in facing direction (Classic Asteroids — ship aims, not cursor)
  - **Switch weapon:** Cycle between equipped weapons. Switching is instant — no animation delay
  - **Energy system:** A single energy bar powers all weapons. Energy regenerates over time. Different weapons consume different amounts:
    - Laser: Low energy cost — the reliable baseline, always available
    - Spread: Higher energy cost — tactical choice, not spammable
    - Rockets (post-MVP): High energy cost — powerful but draining
    - Mines (post-MVP): Medium energy cost — area denial investment
  - The player always has a tactical decision: "Can I afford Spread right now, or should I conserve energy?"
- **Progression:** New weapon types found as loot or bought at stations. Energy capacity and regeneration rate upgradeable through crafting.
- **Interactions:** Shoot + Fly = combat positioning. Shoot + Lead = coordinated attacks with companions. Shoot + Craft = energy regeneration upgrades expand combat options.

#### 3. Discover (Pillar: Exploration)

**The pull that drives the player forward.**

- **Frequency:** Constant passive (minimap always showing blips), active when approaching unknowns
- **Skill tested:** Navigation, risk assessment, curiosity
- **Feel:** Anticipatory. The minimap shows unidentified blips — approaching them reveals what they are. The reveal moment should feel rewarding every time.
- **Mechanics:**
  - **Minimap scanning:** Passive — always shows nearby points of interest as unidentified blips
  - **World map:** Player-revealed — only shows areas already explored. Stations, death markers, and discovered points of interest are marked permanently
  - **Scanner upgrades:** Increase detection range and provide more detail about distant blips before approaching
- **Progression:** Scanner range and detail increase through upgrades. More biomes and station types unlock naturally through distance from spawn.
- **Interactions:** Discover feeds the outer loop — every discovery leads to Engage (fight, explore, or dock). Discover + Lead = companions comment on discoveries via barks.

#### 4. Lead (Pillar: Crew Bond)

**Build your crew, command your wingmen.**

- **Frequency:** Situational — during combat (commands), at stations (recruitment), organically (barks)
- **Skill tested:** Tactical decision-making, emotional investment
- **Feel:** Protective. The player should feel responsible for their companions — not as a burden, but as motivation.
- **Mechanics:**
  - **Recruit:** Meet potential companions at stations or through events. Recruitment is a choice, not automatic
  - **Command:** Simple wingman orders during combat — Attack target, Defend me, Retreat. One button, context-sensitive
  - **Barks:** Companions react to context automatically — no player input needed. Barks express personality and opinions
  - **Opinion system:** Companions track opinion scores for the player and for each other. Actions affect opinions (protecting a companion raises their opinion, reckless behavior lowers it). Architecture: key-value store mapping entity pairs to opinion values — scales to N companions without refactoring
- **Progression:** More companion slots through progression. Deeper relationships unlock new bark lines and combat effectiveness.
- **Interactions:** Lead + Shoot = coordinated combat. Lead + Discover = companions provide context about discoveries. Lead + death = companions retreat safely, preserving the bond.

#### 5. Craft (Pillar: Exploration + Arcade Action)

**Visible progression through ship improvement.**

- **Frequency:** At stations between exploration runs
- **Skill tested:** Resource management, build planning
- **Feel:** Satisfying investment. Spending hard-earned resources on an upgrade that visibly improves the ship. The ship tells its own story through its upgrades.
- **Mechanics:**
  - **Collect:** Resources drop from destroyed asteroids, enemies, and discoverable wrecks. Auto-pickup within ~2x ship radius — close enough to feel tactile, far enough to not require pixel-perfect flying
  - **Craft (recipes):** Combine specific materials to create weapons, upgrades, and ship components. Recipes discovered through exploration or bought at stations
  - **Buy:** Some items only available for purchase at specific station types — not everything can be crafted. Creates reason to visit specialized stations
  - **Upgrade:** Install crafted or bought items into ship systems. Visible changes — the ship looks different as it improves
- **Progression:** Better recipes require rarer materials found further from spawn. Specialized stations sell unique items not available elsewhere.
- **Interactions:** Craft + Fly = better ship feels different to fly (higher max speed, tighter rotation). Craft + Shoot = new weapons and energy upgrades expand combat options. Craft + Discover = exploration rewards feed back into crafting.

#### 6. Log (Passive Mechanic — Pillar: Emergent Stories)

**The game remembers what you did.**

- **Frequency:** Automatic — runs silently in the background, always listening
- **Skill tested:** None — this is a passive system the player reads, not operates
- **Feel:** Reflective. Opening the logbook after a session and reading back what happened creates pride, nostalgia, and the urge to share.
- **Mechanics:**
  - **Event capture:** Every mechanic emits events via Bevy's event system. The logbook subscribes to all relevant events and filters by severity:
    - Tier 1 (always logged): Companion recruited/lost, first station visit, faction shifts, biome first entry, death
    - Tier 2 (logged when notable): Narrow escapes, rare loot found, companion barks about significant events
    - Tier 3 (never logged): Routine combat, standard docking, normal resource pickup
  - **Soft milestones:** Marked as chapter headings in the logbook (first companion, first new biome, first faction contact, etc.)
  - **MVP format:** Simple timestamped event list with categories
  - **Post-MVP:** Narrative text generation that turns events into readable prose
- **Architecture note:** Event-observer pattern must exist from Sprint 1. Events not captured are lost forever — there is no retroactive logging.
- **Interactions:** Every mechanic feeds the logbook. The logbook feeds the Sharing motivation — seeds and stories become shareable content.

### Mechanic Interactions

| | Fly | Shoot | Discover | Lead | Craft | Log |
|---|---|---|---|---|---|---|
| **Fly** | — | Aim through positioning | Reach new areas | Companions follow | Collect in flight | New area events |
| **Shoot** | Combat positioning | — | Clear threats | Coordinated attacks | Weapon upgrades | Combat events |
| **Discover** | Explore the unknown | Clear to explore | — | Crew reacts | Find materials | Discovery events |
| **Lead** | Companions follow | Team combat | Crew comments | — | Gear preferences | Crew events |
| **Craft** | Ship handling changes | New weapons + energy | Better scanners | Revival cost | — | Upgrade events |
| **Log** | Area entries | Battle records | Discovery records | Crew stories | Milestone records | — |

### Controls and Input

Controls are defined in the Target Platforms section (Step 3). Summary:

- **Classic Asteroids rotation model** — ship rotates to aim, no independent aim direction
- **7 core actions:** Thrust, Rotate, Fire, Switch weapon, Wingman command, Interact/Dock, Toggle map
- **Consistent across keyboard and controller**
- **Key/button remapping:** Post-MVP

### Input Feel

- **Responsiveness:** Zero perceived input lag. Rotation and thrust respond on the frame of input
- **Inertia:** Present but player-friendly. Drift enhances gameplay, never frustrates. Stopping takes ~0.5-1 second of gentle drift
- **Soft speed cap:** No hard wall — thrust naturally becomes less effective at high speed. Feels like reaching cruising velocity, not hitting a wall
- **Weapon feedback:** Every shot produces visual flash (bloom), impact particles, and screen shake on hit. The player always knows they hit something
- **Energy feedback:** Energy bar visible on HUD. Low energy triggers subtle visual warning. Empty energy = primary weapon still works (laser), but special weapons unavailable
- **UI feedback:** Minimap blips pulse gently. New discoveries get a brief highlight. Companion barks appear as floating text near the companion ship

---

## Shooter Specific Design

### Weapon Systems

**Weapon Philosophy:** Two weapon categories with fundamentally different feel and resource management.

**Category 1: Laser Weapons (Energy-Free)**
- **Mechanic:** Hitscan — instant ray in facing direction. Pulsed fire — rhythmic individual shots (*pew... pew... pew...*), not continuous beam or held fire. Each pulse is a conscious shot.
- **Cost:** None — the reliable baseline, always available
- **Feel:** Precise, snappy, rhythmic. Each pulse produces a sharp visual flash with immediate hit feedback.
- **Upgrades:**
  - Pulse strength (damage per pulse) — visually represented by a thicker/longer beam flash
  - Fire rate (pulses per second) — faster rhythm, higher DPS
  - Post-MVP: Sustained beam variant (0.1-0.3s active hitscan that hits everything passing through)
- **Role:** The workhorse. Always works, never runs out. Lower damage compensated by reliability and zero energy cost.

**Category 2: Projectile Weapons (Energy-Cost)**
- **Mechanic:** Physical projectiles that fly through space. Visible, dodge-able by enemies AND the player.
- **Cost:** Energy — different weapons drain different amounts
- **Feel:** Weighty, impactful, tactical. You see your shots fly and hear them hit.

| Weapon | Energy Cost | Projectile Speed | Damage | Role | MVP? |
|--------|-----------|-----------------|--------|------|------|
| **Spread** | Medium | Fast | Medium (per projectile) | Wide arc, crowd control | Yes |
| **Rockets** | High | Slow | High + area effect | Heavy hitters, boss damage | Post-MVP |
| **Mines** | Medium | Stationary | High on contact | Area denial, traps | Post-MVP |

**Weapon Progression:**
- **Find:** New weapon types discovered as loot drops or in station shops. First Spread found relatively early to teach weapon switching.
- **Upgrade:** Each weapon type upgradeable independently — damage, energy efficiency, projectile speed, special effects
- **Buy:** Some weapon upgrades only available at specialized stations — creates reason to seek specific stations

**Weapon Feel Differentiation:**
- Laser: Sharp white/blue flash, precise sound, minimal screen shake — rhythmic pulsing
- Spread: Colorful burst, scatter sound, light screen shake
- Rockets: Bright trail, bass-heavy launch sound, strong screen shake on impact
- Mines: Quiet placement, pulsing glow while active, massive explosion on trigger

### Aiming and Combat Mechanics

**Aiming System:** Classic Asteroids rotation — the ship IS the crosshair. No independent aim direction. The player rotates the entire ship to aim. This makes positioning a core combat skill — you can't fly in one direction and shoot in another.

**Hit Detection:**
- **Laser:** Hitscan — instant ray in facing direction (raycast via physics engine). Hits first target in line. Simple, reliable, zero latency feel.
- **Projectiles:** Physics-based — spawned as entities with velocity, collide with targets. Can miss. Can be dodged. Travel time creates lead-shooting skill.

**Enemy Telegraphing:**
- Enemy ships face their shooting direction — the player reads enemy orientation to predict incoming fire
- Laser enemies: Face toward player before firing → player sees the threat alignment and maneuvers away
- Projectile enemies: Visible projectiles in flight → player dodges actively
- This creates natural difficulty scaling: laser enemies require positional reading, projectile enemies require reactive dodging

**Damage Model (MVP):**
- Flat damage per hit — no critical hits, no weak points, no damage types
- Ship health as single HP bar
- Damage numbers not displayed — feedback through visual/audio impact effects
- **Damage taken feedback:** Ship blinks red briefly + screen shake on hit. The player always knows they're being hurt.
- **Damage dealt feedback:** Enemy blinks white on hit. The player always knows they hit something.
- Post-MVP consideration: Damage types (energy vs kinetic) and enemy resistances

### Enemy Design and AI

**Enemy Categories:**

| Type | Behavior | Speed | Health | Threat Level |
|------|----------|-------|--------|-------------|
| **Scout Drone** | Fast, erratic movement, weak shots | High | Low | Fodder — teaches basic combat |
| **Fighter** | Aggressive pursuit, moderate firepower | Medium | Medium | Standard threat — the bread and butter enemy |
| **Heavy Cruiser** | Slow, tanky, strong weapons, area fire | Low | High | Elite — requires strategy or upgraded weapons |
| **Sniper** | Keeps distance, precise long-range laser | Low | Low | Punishes stationary play — forces movement |
| **Swarm** | Groups of 3-5, coordinated flanking | High | Very Low | Overwhelming in numbers, weak individually |

**Neutral Entities:**

Not all encounters are hostile. Neutral entities populate the world and make it feel alive, supporting the positive and inviting atmosphere.

| Type | Behavior | Interaction |
|------|----------|-------------|
| **Trader Ship** | Flies between stations along routes | Approach to trade without docking at a station. Carries unique wares. |
| **Civilian / Refugee** | Flees from danger, follows safe routes | Protect them for faction reputation boost. Ignore or harm them for reputation penalty. |
| **Explorer** | Wanders the world, visits points of interest | Potential companion candidate. May share map information. |

**MVP Minimum:** Trader Ships as a single neutral entity type — a ship entity with `PatrolRoute` AI that moves between stations. Same ECS architecture as enemies, different behavior component.

**AI Behavior by Faction:**
- **Pirates:** Aggressive — pursue player, attempt to flank, retreat when damaged. Primarily Fighters and Scout Drones.
- **Aliens:** Varied — some peaceful until provoked, some territorial, some aggressive on sight. Full range of types including Swarm behavior unique to aliens.
- **Military/Patrol:** Defensive — guard territories, warn before attacking, call reinforcements. Heavy Cruisers and Snipers common.
- **Rogue Drones:** Erratic — unpredictable movement patterns, no faction loyalty, attack anything. Scout Drones and Swarm.

**Spawn System:**
- **Faction Territories (primary):** Enemies spawn at defined patrol points within their faction's territory. Faction ownership is determined by a noise-based faction map layer — each chunk gets a deterministic faction affiliation from `noise(chunk_x, chunk_y, seed + FACTION_OFFSET)`. Borders emerge organically from the noise function. Density increases closer to faction centers. Respawn after destruction on a tunable `respawn_delay` timer — configurable, not hardcoded.
- **Random Encounters (rare):** Occasional lone ships or small groups outside territories. Creates surprise in otherwise quiet areas. Prevents the open void from feeling completely safe.
- **Neutral Routes:** Trader ships follow paths between stations. Civilians appear near stations and along trade routes. Explorers wander freely.
- **Difficulty Scaling:** Enemy count, type distribution, and AI sophistication scale with `distance_from_spawn` gradient. Near spawn: mostly Scout Drones and Traders. Far from spawn: Heavy Cruisers, coordinated Swarms, mixed faction patrols.

### Arena and Level Design

**Combat Environments:** Void Drifter has no discrete arenas or levels. Combat happens in the open world, but the environment shapes every encounter.

**Environment Types:**

**1. Deep Space (Open)**
- No obstacles, no cover — pure piloting skill
- Full freedom of movement, but nowhere to hide
- Favors fast ships and long-range weapons
- Encounters here are about positioning and speed

**2. Asteroid Fields (Dynamic Cover)**
- Asteroids provide cover from enemy fire — break line of sight
- Asteroids MOVE — cover is temporary and unpredictable
- Asteroids are destructible and dangerous — collision damages the player
- Creates tactical choices: use the field as cover, or clear it with weapons for open space
- Tight maneuvering required — rewards skilled pilots

**3. Wreck Fields (Static Structure)**
- Remnants of destroyed stations and large ships
- Static geometry — reliable cover that doesn't move
- Tighter spaces, more predictable combat
- Often contain loot and resources among the wreckage
- Natural points of interest that combine combat with exploration

**Environment Impact on Combat:**
- Deep Space → long-range engagements, kiting, speed-based evasion
- Asteroid Fields → close-range dogfighting, cover-shooting, environmental hazards
- Wreck Fields → ambush opportunities, exploration-combat mix, loot incentive to fight

**Post-MVP Environment Consideration:** Nebulae or energy fields that affect visibility or scanner range could add variety, but require careful UX design for 2D space. Deferred for evaluation.

### Multiplayer Considerations

**Deferred.** Online/multiplayer features are not in scope for MVP or near-term post-MVP. Architecture should not preclude future multiplayer, but no implementation effort is allocated.

---

## Progression and Balance

### Player Progression

Void Drifter respects the player's time. The core principle: **Exploration IS progression.** Flying further makes the player stronger, reveals new content, and generates stories — automatically.

#### Progression Types

| Type | How It Works | Tempo |
|------|-------------|-------|
| **Power** | Ship upgrades, weapons, scanner improvements — every station visit offers visible improvement | Every session |
| **Skill** | Better flying, combat positioning, drift maneuvers — emerges naturally from play | Gradual |
| **Content** | New biomes, station types, factions — discovered by flying further from spawn | Every session |
| **Collection** | Companions, weapon arsenal, logbook entries, explored map coverage | Every session |

#### Progression Pacing

- **Session-positive design.** Station density and reward frequency are tuned so that exploration naturally produces visible progress every session. Concrete pacing values are calibrated through playtesting in Sprint 0.
- **No grind walls.** Upgrades cost resources, but never so much that repetitive farming is required. Resources come naturally through exploration — not by circling the same asteroid field. **Acceptance criterion:** No upgrade should cost more than 2 complete outer-loop cycles (~20 minutes) of resources. If it does, it is a grind wall regardless of intent.
- **Progress is never lost.** Upgrades, companions, map data, logbook entries — all permanent. Only credits are partially lost on death.
- **No meta-progression.** There are no runs. One continuous save. Pick up where you left off every time.
- **No time gates.** No cooldowns, no energy timers, no "come back tomorrow" mechanics.

#### Upgrade Structure

Each ship system has **5 upgrade tiers** (Tier 1 = starting, Tier 5 = fully upgraded):

- Thrust Power
- Max Speed
- Rotation Speed
- Energy Capacity
- Energy Regeneration
- Scanner Range
- Hull Strength
- Cargo Capacity

Each weapon type is independently upgradeable through the same 5-tier structure (damage, fire rate, energy efficiency, special effects).

### Difficulty Curve

**Player-Controlled Through Exploration** — the player chooses their difficulty by choosing how far to fly.

| Aspect | Design |
|--------|--------|
| **Core Principle** | `distance_from_spawn` determines everything — further = harder + better rewards |
| **Player Agency** | The player selects difficulty by deciding how far to fly. Too hard? Fly back. Ready for more? Push further. |
| **Smooth Gradient** | No discrete zones, no sudden spikes. Difficulty increases as a continuous function of distance |
| **Safety Net** | Stations are safe havens. Respawn at last visited station. The spawn area always remains easy |
| **Stuck Solution** | A player who is stuck can always retreat toward spawn. The game never becomes impossible |

#### Boss Encounters

Bosses provide natural highlights within the smooth difficulty curve — optional challenges that reward courage.

| Aspect | Design |
|--------|--------|
| **Spawn** | Deterministic via noise layer — `boss_chance = noise(chunk_x, chunk_y, seed + BOSS_OFFSET) * distance_factor`. Positions are surprising but consistent per seed |
| **Faction Bosses** | Each faction has unique boss types (Pirate Flagship, Alien Mothership, Military Cruiser, Drone Swarm Core) |
| **Telegraphing** | Bosses announce themselves — minimap warning, companions react nervously via barks |
| **Reward** | Boss loot scales with `distance_from_spawn` like everything else. Nearby bosses give good but not overpowered loot. Distant bosses give the best rewards in the game |
| **Scaling** | Boss difficulty scales with `distance_from_spawn` — nearby bosses are manageable, distant bosses are true challenges |
| **No Gate** | Bosses are always optional. No boss blocks progression. Fighting is rewarded, fleeing costs nothing — but the boss remains in that chunk (cooldown timer before re-encounter) |
| **Logbook** | Tier 1 event — every boss encounter is recorded, whether victory or retreat |

#### Challenge Scaling

- **Near Spawn:** Scout Drones, few enemies, common resources. Bosses are rare and manageable.
- **Mid-Distance:** Fighters, mixed factions, crafting materials. Bosses appear occasionally with good loot.
- **Far From Spawn:** Heavy Cruisers, coordinated Swarms, rare materials. Bosses are frequent and formidable.
- The gradient is continuous — these are descriptions of regions on a smooth curve, not discrete zones.

#### Difficulty Options

No explicit difficulty selection. The open world IS the difficulty selector. Players who want a relaxed experience stay closer to spawn. Players seeking challenge push into the unknown. This design inherently accommodates all skill levels without a settings menu.

### Economy and Resources

#### Resources

| Resource | Source | Use |
|----------|--------|-----|
| **Credits** | Enemy drops, trading, discovery rewards | Buy items at stations. Partially lost on death |
| **Materials (Tiered)** | Asteroids, wrecks, enemy drops | Crafting upgrades and weapons |

Materials use a tier system (Common, Uncommon, Rare) rather than separate resource types. Higher-tier material drop probability increases with `distance_from_spawn`. All tiers share the same inventory — no separate "rare material" storage.

#### Economy Flow

- **Earn:** Resources flow naturally from exploration. Flying further yields better drops. Boss encounters guarantee valuable loot.
- **Spend:** Crafting and buying at stations. Some items craft-only, some buy-only — creates reason to both explore and visit specialized stations.
- **Scaling Rule:** `reward_value` scales faster than `station_price` with distance. The player becomes net richer the further they push — exploration is always economically rewarded. The exact ratio is a tunable parameter calibrated in Sprint 0.
- **Sinks:** Death (partial credit loss) + increasing upgrade costs keep the economy balanced. The player always has a reason to earn more without feeling punished.
- **No Premium Currency.** No microtransactions. No paid shortcuts. One economy, fully earnable through play.

#### Balance Structure (Sprint 0)

A balance table mapping `distance_tier → drop_value → upgrade_cost → expected_time_to_next_upgrade` must be established during Sprint 0. The specific numbers will change through playtesting, but the structural framework must exist from the start to enable data-driven tuning.

---

## World Design Framework

### Structure Type

**Procedural Open World + Mini-Level Pockets** — two complementary spatial systems.

The primary world is an infinite, seed-generated open space with no discrete level boundaries. The secondary system consists of self-contained mini-levels accessed through wormholes — procedurally generated pocket dimensions that provide focused, finite challenges as contrast to the endless sandbox.

### Tutorial Zone

A contained starting region inspired by Breath of the Wild's Great Plateau. The player begins here and cannot leave until core combat abilities are unlocked through play. The player never realizes they are in a tutorial.

#### Gravity Well Boundary

The tutorial station has a **defective tractor beam** that pulls the player back when they drift too far. This is the tutorial boundary — organic, diegetic, not a game-y invisible wall.

- **Pull Formula:** `pull_force = max(0, (distance - safe_radius) * pull_strength)` — linear, predictable, learnable
- **Safe Radius:** Immediate spawn area is field-free. The player learns to fly before encountering the pull
- **Visual:** The station sparks and flickers — the defect is visible. The pull field has a subtle visual distortion at its edge

#### Ability Unlock Sequence

The tutorial teaches only the core verbs: **Fly, Shoot, Switch Weapon.** All other mechanics (Minimap active from start, Companions, Craft, Log) are introduced after the tutorial in the open world.

```
Spawn → Fly lernen → Wrack finden → Laser erhalten
  → Gegner bekämpfen → Station docken → Spread erhalten
    → Generator angreifen → Zerstörung → Freiheit!
```

| Phase | Ability Unlocked | How the Player Learns |
|-------|-----------------|----------------------|
| 1 | **Fly** (Thrust + Rotate) | Starting state — only ability available. Nearby points of interest encourage movement |
| 2 | **Laser** | Found at a nearby wreck — auto-docks on approach (visual animation + sound, no inventory UI). First enemies spawn after laser is available |
| 3 | **Spread (Projectile)** | Received at the tutorial station when docking. Teaches weapon switching and energy management. The generator is only destructible by Spread — this is the key to freedom |

#### Generator Destruction (Tutorial Finale)

The defective generator is the tutorial's climactic moment — a mini-boss fight that tests everything the player learned.

- **Generator Defenses:** Defensive drones or energy pulses that require dodging (Fly), weakening (Laser), and destroying (Spread)
- **Vulnerability:** `requires_projectile: true` flag — only Spread damages the generator. This is a special-case flag on the generator entity, NOT a general damage-type system
- **Generator is invulnerable until Spread is obtained** — mechanically impossible to attack prematurely (no visible weak point before Spread)
- **Destruction Feedback:** Generator explodes → shockwave spreads visually outward → gravity field dissolves from inside out → space "opens up" → **Tier 1 Logbook Event** (first entry ever recorded)

#### Station Transformation

After generator destruction, the tutorial station **becomes the player's home station**. The defect is gone — it's now a fully functional station for docking, trading, and crafting. The player has emotional attachment because they "freed" it. This also serves as the first respawn point for death recovery.

#### Tutorial Zone Generation

The tutorial zone is **procedurally generated from the seed** like the rest of the world, but with constraints that guarantee playability:

- Station must spawn within reachable distance from player spawn
- Laser wrack must be placed within safe radius
- Enemy spawn points must be positioned after laser wrack area
- Constraint validation: If layout fails validation, regenerate with modified arrangement offset (same seed)
- **Sprint 0 Acceptance Criterion:** Generate 100 seeds, all 100 tutorial zones must be completable
- **Playtest Criterion:** A blind tester (no game knowledge) must complete the tutorial zone in under 10 minutes without hints

**Scope Note:** Tutorial Zone is a **Sprint 1 Epic** (3-4 stories). Sprint 0 builds the arcade prototype with direct open-world spawn — no tutorial zone. Sprint 1 inserts the tutorial zone before the open world. This prevents the tutorial zone from blocking open-world testing.

### Open World (Post-Tutorial)

The infinite procedural universe. After escaping the tutorial zone, all directions are open.

| Region Type | Characteristics |
|-------------|----------------|
| **Deep Space** | No obstacles, no cover — pure piloting skill. Long-range engagements |
| **Asteroid Fields** | Moving cover, destructible, collision damage. Close-range dogfighting |
| **Wreck Fields** | Static structures from destroyed stations. Ambush opportunities, loot |
| **Stations** | Safe havens — docking, trading, crafting, companion recruitment |
| **Faction Territories** | Noise-based faction zones that determine enemy types and AI behavior |

Region distribution is determined procedurally by noise layers. The player encounters all region types through natural exploration.

### Wormhole Mini-Levels (Pocket Dimensions)

Self-contained, procedurally generated challenges accessed through wormholes found in the open world. Each wormhole leads to a finite space with a clear objective and exit.

**Visual Identity:** Wormholes are the **most visually prominent element** in the world — pulsing light ring, unique color scheme, dedicated minimap icon. No player should fly past a wormhole without noticing it.

**Architecture:** Mini-levels are isolated chunk sets that do NOT exist in the main world. The player is loaded into a separate scene on entry, returned to the wormhole entrance on exit. State is stored as seed + cleared flag + collected items (delta-save, same as main world).

**Crash Recovery:** Crash or disconnect inside a mini-level = respawn at wormhole entrance in main world, mini-level state reset. The player is never trapped in a broken mini-level state.

**Mini-Level Types:**

| Type | Description | Objective | Reward | Scope |
|------|------------|-----------|--------|-------|
| **Combat Arena** | Enclosed space with enemy waves | Survive all waves or defeat the boss | Weapons, rare materials | **MVP** |
| **Exploration Maze** | Asteroid labyrinth or wreck interior | Find the exit or reach the treasure | Map data, unique loot | Post-MVP |
| **Loot Vault** | Resource-rich pocket with time pressure or guardian | Collect as much as possible | High-value materials, credits | Post-MVP |
| **Mixed Challenge** | Combination of combat, navigation, and collection | Complete all objectives | Best rewards — scales with difficulty | Post-MVP |

**Mini-Level Design Rules:**
- Procedurally generated from templates — seed-deterministic (same wormhole = same mini-level per seed)
- Difficulty scales with `distance_from_spawn` of the wormhole entrance
- Finite and completable — clear entry, clear exit, clear objective
- Companions enter with the player — barks react to the environment
- Death inside a mini-level = respawn at last station (same as open world)
- Completed mini-levels remain accessible but marked as cleared on the map
- Wormhole frequency increases with distance from spawn — more risk, more opportunity

### World Progression

**Ungated Exploration** — no abilities, keys, or items are required to access any part of the open world after the tutorial. The only natural gate is difficulty through distance.

| Progression Model | How It Works |
|-------------------|-------------|
| **Tutorial Zone** | One-time gated area — ability unlocks + generator destruction opens the exit. Never revisited as tutorial (becomes home station) |
| **Open World** | Fly in any direction. Distance = difficulty = reward quality |
| **Wormhole Mini-Levels** | Found through exploration. Optional. Increasing frequency and difficulty further from spawn |

#### Replayability

- **Seed-based universe:** Different seed = completely different world layout, wormhole positions, faction territories, tutorial zone arrangement
- **Mini-levels:** Same seed = same mini-levels, but different strategies with upgraded ship
- **No procedural repetition ceiling:** The infinite world and procedural mini-levels ensure no two sessions feel identical

### World Design Principles

1. **Teach through play, never through text** — the tutorial zone sets the standard. If the player can't figure it out by doing it, the design has failed.
2. **The world should never feel empty between points of interest** — station density, enemy patrols, wormholes, and resource nodes are tuned through playtesting so the player always has a visible next destination.
3. **Contrast creates engagement** — the open world is vast and free; mini-levels are tight and focused. The shift between these modes keeps gameplay fresh.
4. **The world rewards curiosity** — every detour, every "what's that blip?", every wormhole entry should feel worth the player's time.

---

## Art and Audio Direction

### Art Style

**Procedural Vector Art — Warm Space** — recognizable ships and structures built from code-generated vector shapes, set against a vibrant, inviting universe.

The visual identity rejects the cold, dark space trope. Void Drifter's universe is colorful and warm — a place players want to explore, not survive. Think No Man's Sky's color warmth applied to 2D vector art.

#### Visual Approach

| Element | Direction |
|---------|-----------|
| **Ships (Player)** | Recognizable spacecraft silhouette — cockpit, hull, thrusters visible. Procedurally assembled from vector shapes via `lyon`. Thrusters glow when active. Ship visually changes with upgrades |
| **Ships (Enemies)** | Faction-specific silhouettes — each faction has a distinct shape language (angular pirates, organic aliens, blocky military, erratic drones). Recognizable at a glance. Simpler Bevy Mesh primitives for performance |
| **Ships (Neutral)** | Friendly shapes — rounded, non-threatening. Trader ships are bulky and slow-looking. Explorers are sleek |
| **Stations** | Largest structures in the world. Warm light emanating from windows/docking bays. Feels like a safe harbor. Detailed via `lyon` |
| **Asteroids** | Irregular polygons with subtle color variation. Moving asteroids rotate visually. Simple Bevy Mesh primitives |
| **Wreck Fields** | Broken geometry — recognizable as former structures. Darker tones than active stations |
| **Wormholes** | Most visually prominent element — pulsing light ring, unique color, visible from far away |
| **Background** | Deep space is NOT black. This is the most important visual statement — it alone separates Void Drifter from 90% of 2D space games. Example: Soft gradient from deep blue to violet, with star fields at varying density and occasional nebula layers in warm colors (amber, rose, soft orange). The void feels alive and welcoming |

#### Mesh Detail Budget

Performance dictates detail allocation. Complex `lyon` paths cost more CPU than simple Bevy Mesh primitives.

| Priority | Element | Technique | Detail Level |
|----------|---------|-----------|-------------|
| 1 | Player Ship | `lyon` vector paths | High — most detailed mesh in the game |
| 2 | Stations | `lyon` vector paths | High — large, detailed, few on screen |
| 3 | Bosses | `lyon` vector paths | High — visually impressive, rare |
| 4 | Enemies | Bevy Mesh primitives | Medium — recognizable silhouette, but simpler geometry |
| 5 | Projectiles | Bevy Mesh primitives | Low — simple shapes, many on screen |
| 6 | Asteroids | Bevy Mesh primitives | Low — irregular polygons, bulk objects |

#### Color Palette

**Warm, vibrant, high-contrast** — inviting rather than threatening.

The seed influences color distribution within a **curated palette set** — not arbitrary colors. A set of 5-6 harmonious color schemes is defined. The seed selects one and varies saturation/brightness within it. This ensures every seed produces a visually harmonious world.

| Context | Palette Direction |
|---------|------------------|
| **Background** | Deep blues, purples, warm dark tones — never pure black |
| **Player Ship** | Bright, warm colors — stands out against any background |
| **Friendly Elements** | Warm tones — yellows, oranges, soft greens. Stations glow warmly |
| **Hostile Elements** | Cooler or more saturated tones — still colorful, but visually distinct from friendlies |
| **UI / HUD** | Clean, readable, minimal. Contrast hierarchy: gameplay elements always more prominent than UI |
| **Loot / Pickups** | Bright, eye-catching — the player's eye is drawn to rewards naturally |

**Contrast Hierarchy (priority order):**
1. Player ship — always the most visible element
2. Threats (enemies, projectiles) — must be immediately readable
3. Rewards (loot, wormholes) — eye-catching, inviting
4. Environment (asteroids, wrecks) — present but not distracting
5. Background — atmospheric, never competing with gameplay

#### Camera and Perspective

**Top-down 2D** — camera follows the player ship, centered. Zoom level tuned so the player sees enough to react but not so much that the ship feels small. Minimap handles awareness beyond camera range.

#### Visual Juice

Visual juice is a **requirement, not a nice-to-have** — but with performance constraints:

- Thruster particles when flying (via `bevy_hanabi`)
- Weapon impact flashes (bloom effect)
- Screen shake on damage taken and heavy weapon hits
- Explosion particles on enemy/asteroid destruction
- Drift trails when the ship moves at high speed
- Generator destruction: full-screen shockwave cascade

**Performance Guard:** Hard cap on simultaneous particles. Visual juice intensity as settings option (Low/Medium/High). WASM defaults to "Low" automatically.

#### Technical Art Pipeline

All visuals are **code-generated** — no external sprite sheets or image assets:
- **Bevy Mesh API** for simple shapes (enemies, asteroids, projectiles)
- **`lyon`** for complex vector paths (player ship, stations, bosses)
- **`bevy_hanabi`** for particle effects (thrusters, explosions, wormhole shimmer)
- Procedural generation ensures visual variety without asset bloat

### Audio and Music

#### Music Style

**Interactive Synthwave** — warm, driving electronic music that responds to gameplay state.

**MVP Implementation:** Two tracks (Exploration + Combat) with crossfade triggered by `enemies_in_range > 0`. Simple and effective.

**Post-MVP Target:** Full music state machine as a dedicated audio system:

| State | Music Character | Trigger |
|-------|----------------|---------|
| **Exploration** | Relaxed synthwave — melodic pads, gentle arpeggios, warmth | Default state, no threats nearby |
| **Tension** | Bass deepens, melody thins, rhythm picks up | Enemies detected at range |
| **Combat** | Full energy — driving beat, intense synths | Enemies in combat range |
| **Station / Docking** | Calm ambient synth — safe harbor feeling | Docked at station |
| **Boss Encounter** | Unique intensity — heavier than combat, distinctive motif | Boss entity in range |
| **Tutorial Zone** | Curious, exploratory — lighter, sense of wonder | In tutorial zone |
| **Wormhole Mini-Level** | Compressed, focused — tighter rhythm | Inside mini-level |

**Music State Machine Architecture (Post-MVP):** `MusicState` enum with event-driven transitions. Crossfade duration as tunable parameter. States triggered by game events, not hardcoded checks.

**WASM Audio Fallback:** Web Audio API constraints (no auto-play, limited channels) require simplified layering — pre-mixed tracks instead of real-time crossfade on Tier 3 platform.

#### Sound Design

**Punchy, clear, satisfying** — every action has audio feedback.

| Sound | Direction |
|-------|-----------|
| **Thrust** | Soft, continuous hum — intensifies with speed |
| **Laser** | Sharp, clean zap — rhythmic pulsing matches fire rate |
| **Spread** | Chunky burst — heavier, more impactful than laser |
| **Impact (dealt)** | Satisfying crunch — player knows they hit something |
| **Impact (taken)** | Distinct warning tone — different from dealt damage |
| **Pickup / Loot** | Bright, rewarding chime — dopamine trigger |
| **Docking** | Mechanical clunk + warm ambient shift — arrival feeling |
| **Wormhole Entry** | Whoosh + frequency shift — dimensional transition |
| **Generator Destruction** | Bass-heavy explosion cascade — the most epic sound in the game |
| **Companion Barks** | Text-only (no voice acting) — sound cue when bark appears |

#### Audio Asset Strategy

Solo-dev realistic approach — no custom-composed soundtrack for MVP:
- **Music:** Licensed synthwave tracks from royalty-free libraries (CC-licensed or purchased). Custom composition is a post-launch consideration.
- **Sound Effects:** Combination of sound libraries and procedurally generated effects via tools like `sfxr`/`bfxr`. Many Bevy-ecosystem audio tools exist for runtime sound generation.
- **No Voice Acting.** All dialogue is text-only with accompanying sound cues.

#### Voice / Dialogue

**No voice acting.** All companion barks, station text, and logbook entries are text-only. Sound cues accompany text appearance (subtle chime for barks, notification sound for logbook entries). This keeps scope manageable for solo development and avoids localization complexity.

### Aesthetic Goals

The art and audio work together to deliver the game's core emotional arc: **loneliness to belonging, danger to mastery, unknown to home.**

| Pillar | How Art & Audio Support It |
|--------|---------------------------|
| **Arcade Action** | Visual juice + punchy sound design make every shot and hit satisfying. Combat music drives adrenaline |
| **Exploration** | Warm colors + inviting synthwave make the void feel welcoming. The background is never empty or hostile |
| **Crew Bond** | Companion ships have distinct visual silhouettes. Bark sound cues create a sense of presence |
| **Emergent Stories** | The visual and audio memory of moments (first boss explosion, first wormhole entry) makes logbook entries feel richer in retrospect |

**Hidden Differentiator:** The positive, inviting atmosphere. Most space games default to dark, hostile, survival-focused aesthetics. Void Drifter's warm visual palette and melodic synthwave say "come explore" instead of "survive if you can."

---

## Technical Specifications

### Performance Requirements

#### Frame Rate Target

| Tier | Target | Priority |
|------|--------|----------|
| **Tier 1 (Linux)** | 60fps @ 1080p | Non-negotiable |
| **Tier 2 (Win/Mac)** | 60fps @ 1080p | Non-negotiable |
| **Tier 3 (WASM)** | 60fps, reduced particle density | Adaptive — 30fps acceptable as fallback |
| **Stretch (Steam Deck)** | 60fps @ 720p (native screen) | Target |

**Performance Acceptance Test:** 60fps with 200 active entities, 50 particle systems, on Tier 1 reference hardware. Reference hardware is defined in Sprint 0 (developer's machine).

**Design Target:** Maximum 200 simultaneously active entities on screen (player + companions + enemies + projectiles + asteroids + pickups). This defines the performance budget for architecture decisions.

#### Resolution Support

Vector graphics scale naturally — no pixel-perfect constraints. UI must scale correctly.

| Resolution | Support Level |
|-----------|--------------|
| **1080p** | Primary design target |
| **720p** | Supported (Steam Deck, smaller screens) |
| **1440p / 4K** | Works automatically (vector scaling) — UI scaling tested |
| **Ultrawide** | Not targeted for MVP. Camera can letterbox or extend view |

#### Load Times

| Scenario | Target |
|----------|--------|
| **Warm Start (load save)** | Under 5 seconds to gameplay |
| **Cold Start (first play, tutorial generation)** | May exceed 5 seconds — minimal loading screen with seed visualization allowed |
| **Chunk generation** | Seamless — no loading screens, generated ahead of player movement |
| **Wormhole entry** | Under 2 seconds transition |
| **Death respawn** | Instant (under 1 second) |
| **Save / Load** | Under 3 seconds — delta-saves are small |

### Platform-Specific Details

#### PC (Tier 1 + 2)

| Aspect | Requirement |
|--------|-------------|
| **Minimum Spec** | Any hardware capable of running WebGL2 in a modern browser at 60fps. This is the baseline — if the browser version runs smoothly, the native version will too |
| **GPU Requirement** | OpenGL 3.3+ support (required by Bevy) |
| **Windowed / Fullscreen** | Both supported |
| **Steam Features** | Achievements, Cloud Saves, Steam Deck verification — planned for full launch, not MVP |
| **Mod Support** | Not planned. Seeds provide user-generated variety |
| **Save Location** | Local saves in standard OS location. Cloud saves via Steam post-MVP |

#### Web / WASM (Tier 3)

| Aspect | Requirement |
|--------|-------------|
| **Target Browsers** | Chrome, Firefox, Edge (latest versions) |
| **WebGL Version** | WebGL2 |
| **Build Size** | Target under 50MB (realistic with Bevy + lyon + hanabi). Stretch goal: under 30MB via `wasm-opt` and tree-shaking |
| **Constraints** | Single-threaded, limited memory (~2-4GB), Web Audio API restrictions (no auto-play) |
| **Feature Subset** | Smaller chunk radius, reduced particle density (auto "Low" visual juice), simplified audio layering |
| **Offline Play** | Not required — web build assumes connectivity |

#### Steam Deck (Stretch)

| Aspect | Requirement |
|--------|-------------|
| **Input** | Controller-only (built-in gamepad). Same Classic Asteroids controls |
| **Display** | 7" screen @ 720p — UI elements must be readable at this size |
| **Performance** | Target 60fps — Steam Deck is capable Linux hardware |
| **Verification** | Steam Deck Verified badge as goal |

### Save System Data

Explicit list of persisted data — defines the scope of the save system:

| Category | Data |
|----------|------|
| **Player State** | Position, ship configuration, unlocked abilities, credits, inventory |
| **Ship** | Installed upgrades (per system, tier level), equipped weapons |
| **Companions** | Roster, opinion scores (HashMap of entity pairs → i32), current positions |
| **World** | Discovered map regions, station states, cleared mini-levels (seed + cleared flag) |
| **Tutorial** | Completion flag, unlocked abilities within tutorial |
| **Logbook** | All recorded events with timestamps and severity |
| **Economy** | Credits, material inventory (by tier), station storage contents |

Save format: Delta-based — seed + player changes. The seed recreates the base world; only deviations from the generated state are saved.

### Asset Requirements

#### Art Assets

**Zero external art assets.** All visuals are code-generated at runtime:

| Category | Approach | Estimated Count |
|----------|----------|----------------|
| **Ship Meshes** | `lyon` (player, stations, bosses) + Bevy Mesh (enemies, asteroids) | ~20-30 unique mesh generators |
| **Particle Effects** | `bevy_hanabi` definitions | ~10-15 effect types |
| **UI Elements** | Bevy UI system, code-generated | ~10 screens/overlays |
| **Color Palettes** | 5-6 curated schemes, seed-selected | Defined in code |
| **Backgrounds** | Procedural gradient + star field layers | Generated per-chunk |

#### Audio Assets

| Category | Source | Estimated Count |
|----------|--------|----------------|
| **Music Tracks** | Licensed synthwave (royalty-free) | MVP: 2 tracks (Exploration + Combat). Post-MVP: 5-7 tracks |
| **Sound Effects** | `sfxr`/`bfxr` generated + sound libraries | ~25-35 unique sounds |
| **Ambient** | Sound libraries | ~5 ambient loops (station, deep space, asteroid field, wreck field, wormhole) |
| **UI Sounds** | `sfxr` generated | ~10 (clicks, chimes, notifications) |

### External Dependencies

| Dependency | Purpose | Risk | Fallback |
|------------|---------|------|----------|
| **Bevy** | Game engine | Medium — breaking changes between versions | Pin version during sprints, upgrade between epics |
| **`lyon`** | Vector path generation | Low — stable crate | None needed — stable |
| **`bevy_hanabi`** | Particle effects | Medium — must track Bevy version | Custom particle system (simple quads with velocity + lifetime) |
| **Licensed Music** | Soundtrack | Low — interchangeable | Any royalty-free synthwave library |

### Technical Constraints

| Constraint | Impact | Mitigation |
|-----------|--------|------------|
| **Bevy breaking changes** | Major version updates may require code migration | Pin Bevy version during sprints, upgrade between epics |
| **WASM single-threaded** | No parallel chunk generation on web | Smaller chunk radius, generation budget per frame |
| **WASM memory limit** | ~2-4GB depending on browser | Aggressive chunk unloading, smaller world cache on WASM |
| **Solo developer** | All code, design, testing by one person | Prioritize procedural generation over hand-crafted content. Leverage Bevy ecosystem crates |
| **No external art pipeline** | No artist, no sprite workflow | All visuals code-generated — this is a feature, not a limitation |

### Architecture Guidance

**Multiplayer Preparation:** Multiplayer is out of scope, but the ECS architecture and event system should be designed so that a future network layer is possible without rewriting core game code. This means: game state changes flow through events, not direct mutation; systems are deterministic where possible.

**Crash Reporting & Telemetry (Post-MVP):** Opt-in crash reports and anonymous gameplay telemetry (session length, death positions, popular routes) are valuable for solo-dev balance tuning. Architecture should not preclude adding this later.

**Note:** These are GDD-level requirements. Detailed architecture decisions (ECS system design, chunk management, networking preparation, etc.) are addressed in the Architecture workflow after GDD completion.

---

## Development Epics

### Epic Overview

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

**Embedded Infrastructure (not separate epics):**
- **Event-Observer Pattern** → Epic 1 (Open World Foundation). All systems emit events from day one.
- **Basic Save System** → Epic 1 (Open World Foundation). Delta-save architecture (seed + changes).

### MVP Categories

- **Must (MVP Core):** Epics 0-4 — the game is playable
- **Should (MVP Complete):** Epics 5-8 — the game feels complete
- **Could (Post-MVP):** Epics 9-11 — polish and extras

### Recommended Sequence

```
Epic 0: Arcade Prototype (Sprint 0)
  └→ Epic 1: Open World Foundation + Event-Observer + Basic Save
      ├→ Epic 2: Tutorial Zone (special chunk on chunk system)
      ├→ Epic 3: Stations & Economy
      │   └→ Epic 5: Progression & Upgrades
      │       └→ Epic 7: Boss Encounters
      ├→ Epic 4: Combat Depth
      │   └→ Epic 7: Boss Encounters
      │   └→ Epic 9: Wormhole Mini-Levels
      ├→ Epic 6a: Companion Core (after Epic 3 + 4)
      │   ├→ Epic 6b: Companion Personality
      │   └→ Epic 8: Logbook UI
      ├→ Epic 10: Art & Audio Polish
      └→ Epic 11: Platform & Release
```

### Vertical Slice

**First playable milestone:** After completing Epics 0 + 1 + 3, the player can: fly with physics-based controls, shoot with laser and spread, explore a procedurally generated infinite world with multiple biome types, dock at stations, buy items, and collect resources. This proves the core loop works and feels good. All progress is saved via the delta-save system.

*Detailed epic breakdown with stories is maintained in the separate [epics.md](epics.md) file.*

---

## Success Metrics

### Technical Metrics

#### Automated Metrics (checked every sprint via CI)

| Metric | Target | Measurement |
|--------|--------|-------------|
| **Frame Rate** | 60fps with 200 entities on Tier 1 hardware | Automated benchmark |
| **Frame Rate (WASM)** | 60fps with 100 entities, 30fps acceptable fallback | Browser profiling |
| **Load Time (Warm Start)** | Under 5 seconds | Automated test |
| **Load Time (Cold Start)** | Under 15 seconds including tutorial generation | Automated test |
| **WASM Build Size** | Under 50MB (stretch: under 30MB) | CI build artifact check |
| **Save File Size** | Under 1MB for 10+ hours of play | Automated extended session test |
| **Crash Rate** | Zero crashes in 4 hours of continuous play | Automated soak test — catches memory leaks and ECS growth issues |
| **Memory Usage** | Under 500MB after 2 hours of continuous exploration in one direction | Automated memory profiling — validates chunk unloading |
| **Tutorial Validity** | 100/100 seeds produce completable tutorial zones | Automated 100-seed validation |
| **Chunk Generation** | No visible stutter at maximum speed | Manual playtest per sprint |

### Gameplay Metrics

#### Metric #0: Developer Joy

**"Do you still voluntarily play your own game?"** — This is the first and most important metric. If the developer doesn't enjoy playing the prototype after a day of development, something fundamental is wrong. This is checked informally but honestly at every sprint boundary.

#### Playtest Metrics (checked at epic boundaries with friends/community volunteers + feedback template)

| Metric | Target | Measurement | Pillar |
|--------|--------|-------------|--------|
| **Tutorial Completion** | Blind tester completes in under 10 min | Playtest session | Arcade Action |
| **Session Length** | Average 30-45 minutes | Playtest observation (post-MVP: telemetry) | All |
| **Return Rate** | Playtester wants to play again next day | Playtest feedback form | All |
| **Time to First "Wow"** | Under 5 minutes (first combat, first discovery) | Playtest observation | Arcade Action, Exploration |
| **Exploration Pull** | Player flies toward minimap blips voluntarily | Playtest observation | Exploration |
| **Companion Attachment** | Player reacts emotionally to companion loss or barks | Playtest observation | Crew Bond |
| **Logbook Engagement** | Player opens logbook voluntarily at least once per session | Playtest observation | Emergent Stories |
| **Upgrade Pacing** | At least one meaningful upgrade per session | Playtest timing | Arcade Action |
| **Grind Check** | No upgrade requires more than 2 outer-loop cycles (~20 min) | Balance spreadsheet + playtest | All |
| **Death Frustration** | Player retries after death without hesitation | Playtest observation | Arcade Action |
| **Boss Engagement** | Player chooses to fight (not flee) at least 50% of boss encounters | Playtest observation | Arcade Action |

### Qualitative Success Criteria

Signs that Void Drifter is achieving its design goals:

| Criterion | What It Means | Related Pillar |
|-----------|--------------|----------------|
| **"Just one more blip"** | Players describe wanting to see what's next before quitting | Exploration |
| **Players name their companions** | Emotional attachment to crew | Crew Bond |
| **Playtesters tell stories unprompted** | The game generates moments worth sharing — pre-launch version of the "players share seeds" vision | Emergent Stories |
| **"It feels good to fly"** | Controls are praised without prompting | Arcade Action |
| **Players describe the atmosphere as "warm" or "inviting"** | Art and audio direction succeeded | Hidden Differentiator |

### Metric Review Cadence

| When | What |
|------|------|
| **Every Sprint** | Automated technical metrics (FPS, load times, crash rate, memory). Developer joy check |
| **After Sprint 0** | **Critical gate:** Minimum 3 external testers play the arcade prototype. If "feels good to fly" is not mentioned, Sprint 0 is not complete |
| **End of Each Epic** | Playtest session with volunteer tester. Use standardized feedback template for comparable results |
| **After MVP Core (Epics 0-4)** | First broader external playtest. Gather qualitative feedback. Validate core loop |
| **After MVP Complete (Epics 0-8)** | Extended playtest (multiple sessions over multiple days). Check return rate and session length |
| **Pre-Release** | Full metric review against all targets. Go/no-go decision |

### Playtest Infrastructure

- **Testers:** Friends, family, online community volunteers. No professional QA team expected for solo development
- **Feedback Template:** Standardized questionnaire covering all gameplay metrics. Created before first playtest (Sprint 0)
- **Post-MVP Telemetry:** Opt-in anonymous gameplay data (session length, death positions, popular routes) for data-driven balance tuning. Architecture allows this but implementation is post-MVP

---

## Out of Scope

The following features are explicitly **not included** in Void Drifter v1.0:

| Feature | Reason |
|---------|--------|
| **Multiplayer / Co-op** | Architecture allows future addition, but no implementation effort allocated |
| **Mod Support** | Seeds provide user-generated variety. No modding API planned |
| **Voice Acting** | All dialogue is text-based (barks, logbook). Keeps scope manageable for solo dev |
| **Localization** | English only for v1.0. Text-based UI allows future localization without code changes |
| **Console Ports** | PC + WASM + Steam Deck only. Console certification is a separate project |
| **Twin-Stick Controls** | May be evaluated post-MVP, but Classic Asteroids rotation is the core identity |
| **Level Editor** | Procedural generation replaces hand-crafted content |
| **In-App Purchases / Monetization** | No microtransactions. No premium currency. One-time purchase |

### Deferred to Post-Launch

| Feature | When | Dependencies |
|---------|------|-------------|
| Interactive Music State Machine | Post-MVP | MVP crossfade system must work first |
| Mini-Level Variety (Maze, Vault, Mixed) | Post-MVP | Combat Arena must prove the concept |
| Damage Types (Energy vs Kinetic) | Post-MVP | Flat damage must feel balanced first |
| Nebula Environment Type | Post-MVP | Requires UX research for 2D visibility |
| Cloud Saves (Steam) | Full Launch | Steam integration |
| Key/Button Remapping | Post-MVP | Bevy input system supports it — architecture ready |
| Narrative Text Generation (Logbook) | Post-MVP | Raw event log must exist first |
| Crash Reporting & Telemetry | Post-MVP | Architecture allows, not implemented |
| Custom Composed Soundtrack | Post-Launch | Licensed tracks for MVP |
| Export/Share Logbook | Post-Launch | Logbook UI must exist first |

---

## Assumptions and Dependencies

### Key Assumptions

| Assumption | Impact if Wrong | Mitigation |
|-----------|----------------|------------|
| **Bevy remains stable enough for production** | Major — engine switch would restart the project | Pin Bevy version during sprints, upgrade between epics. Community is active and growing |
| **Solo developer capacity is sufficient** | High — burnout or scope creep stalls the project | Strict MVP scope. Must/Should/Could categories. Cut "Could" without guilt |
| **Procedural generation produces fun content** | Critical — the entire game relies on it | Sprint 0 validates core generation. 100-seed automated tests. Playtest at every epic boundary |
| **Classic Asteroids controls feel good on modern hardware** | High — controls are the foundation | Sprint 0 validates with 3+ external testers before any other work |
| **Players find procedural vector art visually appealing** | Medium — art style is unconventional | Curated color palettes prevent ugly seeds. Playtest feedback on visuals from Sprint 0 |
| **Licensed synthwave tracks fit the game's mood** | Low — music is interchangeable | Multiple royalty-free libraries available. Tracks can be swapped without code changes |
| **Tutorial zone constraint generation reliably produces playable layouts** | Medium — broken tutorials lose players immediately | 100-seed automated validation. Fallback: regenerate with arrangement offset |

### External Dependencies

| Dependency | Version Strategy | Risk | Fallback |
|-----------|-----------------|------|----------|
| **Bevy** | Pin during sprints, upgrade between epics | Medium | No fallback — Bevy is the foundation |
| **`lyon`** | Latest compatible with pinned Bevy | Low | Stable crate, rarely breaks |
| **`bevy_hanabi`** | Track Bevy version compatibility | Medium | Custom particle system (quads + velocity + lifetime) |
| **Licensed Music** | Royalty-free synthwave libraries | Low | Any royalty-free synthwave works |
| **Sound Effects** | `sfxr`/`bfxr` + sound libraries | Low | Fully replaceable |
| **Rust Toolchain** | Stable channel | Very Low | Rust stability guarantees |

### Risk Factors

| Risk | Probability | Impact | Mitigation |
|------|------------|--------|------------|
| **Fun Gap** — game isn't fun despite good mechanics | Medium | Critical | Sprint 0 validation. "Developer joy" as Metric #0. Kill the project early if core loop fails |
| **Bevy Breaking Changes** — major update mid-development | High | Medium | Version pinning. Upgrade only between epics. Community migration guides |
| **Scope Creep** — "just one more feature" | High | High | Must/Should/Could categories. Cut Could without guilt. Every feature must pass: "Does this make moment-to-moment gameplay worse?" |
| **Solo Dev Burnout** — motivation loss over long development | Medium | Critical | Short sprints with playable results. Developer joy metric. Ship early (itch.io), iterate on feedback |
| **Procedural Monotony** — generated world feels samey after hours | Medium | High | Environment variety (3 types + wormholes). Faction territory variation. Playtest sessions >2 hours to detect |

---

## Document Information

**Document:** Void Drifter - Game Design Document
**Version:** 1.0
**Created:** 2026-02-25
**Author:** Simon
**Status:** Complete

### Change Log

| Version | Date | Changes |
|---------|------|---------|
| 1.0 | 2026-02-25 | Initial GDD complete |
