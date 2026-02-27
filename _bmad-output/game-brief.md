---
stepsCompleted: [1, 2, 3, 4, 5, 6, 7, 8]
inputDocuments:
  - '_bmad-output/brainstorming-session-2026-02-25.md'
documentCounts:
  brainstorming: 1
  research: 0
  notes: 0
workflowType: 'game-brief'
lastStep: 8
project_name: 'spacegame-bmad'
user_name: 'Simon'
date: '2026-02-25'
game_name: 'Void Drifter'
---

# Game Brief: Void Drifter

**Date:** 2026-02-25
**Author:** Simon
**Status:** Complete — Ready for GDD Development

---

## Executive Summary

Void Drifter is Asteroids meets open-world exploration in a living, seed-generated universe — where your crew becomes family and every journey writes itself.

**Target Audience:** Adult and young-adult gamers (16+) who are Explorer-Collectors — driven by discovery, emotional bonds with companions, and the joy of exploring an infinite universe.

**Core Pillars:** Arcade Action, Exploration, Crew Bond, Emergent Stories

**Key Differentiators:**
- An automatic event logbook that captures and preserves the player's emergent story
- Companions with opinions about the player and each other — personality through contextual barks, not dialogue trees
- Infinite persistent Asteroids — tight arcade controls in a seed-generated world that remembers
- The emotional arc from solitude to family — no other game in this space offers this journey

**Platform:** Windows, Linux, macOS (native), Web (WASM), Steam Deck (stretch goal)

**Tech Stack:** Bevy Engine + Rust — procedural vector graphics, no external art pipeline

**Success Vision:** A game that is widely played and shared — where every seed creates a unique story worth telling.

---

## Game Vision

### Core Concept

Asteroids meets open-world exploration in a living, seed-generated universe — where your crew becomes family and every journey writes itself.

### Elevator Pitch

A 2D arcade space adventure with procedural vector visuals in an infinite universe. Fight through asteroid fields, discover stations, recruit living wingmen with their own personalities, and navigate faction conflicts — while the game writes your unique story as you play. Every seed creates a living universe that remembers what you did.

### Vision Statement

Your crew becomes family — that's the heart of Void Drifter. The game creates a space adventure where players feel genuine accomplishment through exploration and survival, while building emotional bonds with a crew of unique companions. Procedural generation turns every journey into a personal story — every pilot's path is different, but every pilot fights for the people flying beside them.

---

## Target Market

### Primary Audience

Adult and young-adult gamers (16+) who are Explorer-Collectors at heart — they live for discovering new places, finding hidden treasures, and building a crew they genuinely care about. These are the players who get attached to their RPG party members, who never swap out a companion once they've bonded, and who explore every corner of a map before moving on.

**Demographics:**
- Age: 16+, skewing toward adult players
- Experience: Low barrier to entry — no genre knowledge required
- Platform: PC (itch.io for Early Access, Steam for full launch)
- Player Type: Explorer-Collectors (Bartle's Taxonomy) — driven by discovery and attachment

**Gaming Preferences:**
- Exploration-driven games with emergent stories
- Accessible controls with hidden depth
- Games that respect their time (save anywhere, mild death penalty)
- Players who get attached to their RPG party members — they want companions that feel alive

**Motivations:**
- The joy of discovery — "What's behind that next blip on the minimap?"
- Building emotional bonds with companions who have their own personalities and opinions
- Feeling accomplishment through progression and world impact
- Creating a unique personal story through gameplay — and being able to read it back

### Secondary Audience

Nostalgic Asteroids fans and indie space game enthusiasts who enjoyed FTL, Everspace, or Nova Drift but want a persistent open-world experience. Players who loved tight arcade controls and physics-based movement but want more depth — exploration, progression, and a reason to keep playing beyond the next highscore.

### Market Context

The procedural storytelling space is proven but underserved in key areas. Games like RimWorld and Dwarf Fortress demonstrate massive demand for emergent narratives, but share a common gap: the stories happen in the player's head and are never captured by the game itself. Void Drifter's automatic event logbook directly addresses this unmet need.

**Similar Successful Games:**
- **RimWorld** — Proves appetite for emergent stories in procedural worlds, but player is observer not protagonist, and stories are never captured
- **FTL: Faster Than Light** — Closest direct competitor: 2D, space, crew, procedural, indie. Key difference: FTL is linear with short runs. Void Drifter is open-world and persistent — your universe grows with you instead of resetting
- **Terraria** — Proves infinite procedural worlds with progression and mild death penalties retain players long-term
- **Asteroids** — The genre foundation, proven for decades but never evolved into open-world

**Market Opportunity:**
- No game currently combines Asteroids-style arcade combat with open-world exploration and companion systems
- The "story that writes itself AND remembers" is an unfilled niche — RimWorld players actively ask for this feature
- Seed-sharing creates organic community growth and content discovery (platform TBD — Discord, Reddit, or in-game browser)
- Streaming and content creator potential — seed-based games are streaming gold. "I'm playing my community's seed" is a proven format for Twitch and YouTube organic reach
- The Bevy/Rust tech stack provides a free marketing channel — the Rust gamedev community is small but passionate, and a polished Bevy game will attract organic attention
- Distribution strategy: itch.io first as demo and prototype feedback channel, then Early Access for community building, Steam for full launch

---

## Game Fundamentals

### Core Gameplay Pillars

1. **Exploration** — The infinite procedural universe is the stage. Every direction promises discovery — new biomes, stations, wormholes, and secrets. The minimap's unknown blips are the primary driver of player engagement.

2. **Arcade Action** — Asteroids-DNA at the core. Physics-based flight with inertia, fast weapon switching, reactive combat. Controls must feel tight and satisfying from the first second. 60fps is non-negotiable.

3. **Crew Bond** — Companions are not gameplay buffs — they are characters the player cares about. Wingmen with personalities, opinions about each other, and memories of shared experiences. Losing one hurts. Saving one feels heroic.

4. **Emergent Stories** — The game writes itself. Faction wars, crew conflicts, narrow escapes, unexpected alliances — all captured in an automatic logbook. Every playthrough generates a unique narrative worth sharing.

**Pillar Priority:** When pillars conflict, prioritize:
1. Arcade Action (if it doesn't feel good to fly and shoot, nothing else matters)
2. Exploration (the pull to discover drives everything forward)
3. Crew Bond (emotional investment keeps players coming back)
4. Emergent Stories (the logbook captures what the other pillars create)

### Primary Mechanics

- **Fly** — Physics-based movement with inertia. Rotate, thrust, drift. The core verb inherited from Asteroids — always active, always satisfying.
- **Discover** — Follow minimap blips into the unknown. Uncover biomes, stations, wormholes, wrecks. Expand the world map. Each discovery reveals new possibilities.
- **Shoot** — Arcade combat with quick weapon switching. Laser, spread, rockets, mines — each weapon fits different situations. Simple to use, deep to master.
- **Lead** — Recruit companions, command wingmen in combat, protect them from danger, mediate crew conflicts. The emotional bond with the crew emerges from these concrete leadership actions.
- **Craft** — Collect resources from asteroids and structures. Upgrade ship systems, mount new weapons, improve scanners. Visible progression — the ship tells its own story.

**Core Loop:**
**Inner Loop** (seconds): Fly → Shoot → Survive — always active, the arcade heartbeat
**Outer Loop** (minutes): Discover → Engage (fight, explore, or dock) → Reward (loot, companion, lore, upgrade) → Decide (push further or return to station) → Repeat

### Player Experience Goals

- **Discovery & Surprise** — "What's behind that blip?" Every session delivers at least one moment of genuine surprise — a new biome, an unexpected faction event, a hidden wormhole.
- **Tension & Relief** — Heart-pounding combat and narrow escapes, followed by the relief of reaching a station with a full cargo hold. The contrast makes both halves stronger.
- **Mastery & Growth** — From struggling in the spawn area to cruising through it with a fleet of wingmen. The player feels their power grow through upgrades, crew size, and map knowledge.
- **Connection & Belonging** — The crew becomes family. Players remember their companions by name, celebrate victories together, mourn losses. The logbook preserves these bonds.
- **Relaxation & Flow** — Quiet zones compose 15-25% of traversal time, providing intentional pacing between encounters. Drifting through empty space with the crew, scanning the minimap, planning the next move. The calm makes the action hit harder.

**Emotional Journey:** A typical session flows between tension and calm — discover something on the minimap (curiosity), fly toward it (anticipation), encounter danger or wonder (tension/awe), overcome or explore (mastery/discovery), return to safety (relief/satisfaction), read the logbook entry about what just happened (reflection/pride), spot the next blip on the minimap (curiosity renewed). The circle closes — that's the "one more turn" factor.

---

## Scope and Constraints

### Target Platforms

- **Primary:** Windows, Linux, macOS (native builds via Bevy/Rust)
- **Web:** Browser via WASM (WebAssembly) — enables itch.io distribution and instant demos
- **Stretch Goal:** Steam Deck (native Linux + controller support, UI scaling for small screen)
- **Distribution:** itch.io for early prototypes and feedback → Steam Early Access → Steam full launch

**Platform Priority:** Web (WASM) and Linux first (developer's primary environment), then Windows and macOS. Steam Deck compatibility as stretch goal after core platforms are stable.

### Development Timeline

**Developer Profile:** Solo developer, part-time (~10-15h/week effective dev time alongside 40h/week main job)

**MVP Epics (Prototype → Playable Demo):**

| Epic | Scope | Est. Sprints (2 weeks) |
|------|-------|------------------------|
| 🎮 Core Flight | Movement, shooting, collision, camera, basic HUD | 2-3 |
| 🌌 Procedural World | Chunk generation, seed system, asteroids, 1 biome, minimap | 3-4 |
| 🏠 Station Loop | 1 station type, docking, basic trading/upgrading | 2-3 |

**Estimated MVP timeline:** 7-10 sprints → ~3.5-5 months. With buffer for real life: **~6 months to playable prototype.**

**Post-MVP Phases:**
- Phase 2: Companions (1 wingman as proof of concept), additional biomes, weapon variety
- Phase 3: Faction system (static affiliations first), event logbook
- Phase 4: NPC interactions, crew opinions, faction dynamics
- Phase 5: Audio, polish, Steam release preparation

### Budget Considerations

- **Self-funded** — zero external budget, zero financial risk tolerance
- **Engine:** Bevy (open-source, free) — no licensing costs
- **Art:** Procedural vector graphics generated in code — no artist needed, no asset costs
- **Audio:** Biggest skill gap. Strategy: free SFX from freesound.org for prototype, `bevy_audio` for integration. Music deferred to late development or post-launch — not a blocker for MVP
- **Distribution:** itch.io (free), Steam ($100 one-time fee for Steamworks)
- **Tools:** All open-source (Rust, Bevy, cargo, git)

### Team Resources

- **Solo Developer:** Simon — intermediate Bevy/Rust experience, full-stack capability (design, code, procedural art)
- **AI-Assisted Development:** BMAD workflow for planning and documentation, AI pair programming for implementation
- **Skill Strengths:** Rust/Bevy, systems programming, procedural generation, game design
- **Skill Gaps:** Audio design/music (low priority — deferred), marketing (community-driven via seed-sharing and Rust gamedev community)
- **No external dependencies:** No team coordination overhead, no blocking on other people

### Technical Constraints

**Engine & Language:**
- Bevy Engine (latest stable) + Rust — ECS architecture is ideal for entity-heavy procedural worlds
- 60fps non-negotiable — all systems must be profiled for the hot path

**Architecture-Critical Decisions (must be solved early):**
- **Persistenz-Architektur:** Seed + delta-based saves. Serialization format must be designed from day 1 — retrofitting is architecturally destructive
- **Chunk-System:** Minecraft-style chunk generation/despawning. Must account for WASM memory limits from the start
- **WASM Constraints:** No multi-threading, limited memory. Chunk loading strategy must work single-threaded

**MVP Technical Boundaries:**
- Factions: Static affiliations only (enemy/neutral/friendly) — no autonomous simulation until post-MVP
- Companions: 1 wingman with simple Follow/Attack/Flee state machine — full personality system is post-MVP
- Event Logbook: Simple event list — natural language generation is post-MVP
- Online/Multiplayer: Fully deferred — architecture should not preclude it, but no implementation effort in MVP

**Cross-Platform Strategy:**
- Develop on Linux, CI builds for all targets
- WASM build tested alongside native from sprint 1 — no "port later" approach

---

## Reference Framework

### Inspiration Games

**Asteroids (1979)**
- Taking: The core flight model — physics-based rotation, thrust, inertia. The feeling of piloting a ship in space where momentum matters.
- Not Taking: The static single-screen arena, the repetitive gameplay loop, the lack of progression or world.

**FTL: Faster Than Light (2012)**
- Taking: The sense of journeying through space, encountering events, managing risk vs reward at each stop.
- Not Taking: The ship interior management focus, the linear run structure, the permadeath reset, the top-down crew micromanagement.

**Terraria (2011)**
- Taking: The joy of exploring a procedural world full of discoveries. The diverse NPC roster that populates your world and makes it feel alive.
- Not Taking: The finite world boundaries. The flat, opinion-less NPCs who are just service dispensers with no personality or relationships.

**RimWorld (2018)**
- Taking: The rich character variety with individual traits and stories. The emergent storytelling where every playthrough generates unique narratives.
- Not Taking: The observer role — in Void Drifter, you ARE the protagonist. The ephemeral stories that only exist in the player's head and are never captured by the game.

### Competitive Analysis

**Direct Competitors:**
- **FTL: Faster Than Light** — Closest match: 2D, space, crew, procedural, indie. But linear with short runs, no persistent world.
- **Nova Drift** — Asteroids-evolved with builds and progression. But arena-based, no exploration or companions.
- **Everspace (1 & 2)** — Space combat with exploration. But 3D, roguelike structure, no companion bonds.
- **Reassembly** — 2D space with procedural ships and factions. Closer to Void Drifter than it appears, but no companions, no storytelling, no persistent narrative.

**Competitor Strengths:**
- FTL nails tension and meaningful choices in a compact package
- Nova Drift delivers satisfying arcade combat with deep build variety
- Everspace provides gorgeous space combat with exploration incentives
- Reassembly proves procedural ships and faction systems work in 2D

**Competitor Weaknesses:**
- None offer a persistent open world that remembers your actions
- None capture the emergent story for the player to read back
- None build genuine emotional bonds with companions who have opinions and memories
- All reset progress (roguelike structure) — no long-term world investment

### Key Differentiators

1. **The Story That Writes Itself — And Remembers** — An automatic event logbook captures every meaningful moment. Events are filtered by a severity system: Tier 1 (always logged — companion death, first station visit, faction shifts), Tier 2 (logged when notable — narrow escapes, rare loot), Tier 3 (never logged — routine combat, standard docking). Event-capture architecture is built from day 1; narrative text generation is post-MVP. No other game in this space captures and presents the player's emergent story.

2. **Companions With Opinions** — NPCs are not stat buffs or service dispensers. They express personality through contextual barks and opinion scores — not dialogue trees. They react to events ("Ugh, nicht schon wieder diese Piraten!"), have opinions about the player AND about each other, and remember shared experiences. A bark-trigger-matrix defines who says what in which situation — dozens of contextual one-liners per NPC create the feeling of a living crew without BioWare-scale dialogue systems.

3. **Infinite Persistent Asteroids** — The proven arcade flight model of Asteroids, expanded into an infinite seed-generated universe that persists between sessions. Seed + delta saves mean the world remembers what you did. No other game combines tight arcade controls with open-world persistence.

4. **From Solitude to Family** — The emotional differentiator. You start as a lone pilot in the void. Over time, you build a crew that becomes family. The journey from isolation to belonging is the emotional arc that no competitor offers — and it emerges naturally from gameplay, not from scripted cutscenes.

**Unique Value Proposition:**
Void Drifter is the only game that combines Asteroids-style arcade flight with an infinite persistent universe, emotionally bonded companions, and a story that writes itself and lets you read it back.

**Pitch One-Liner:**
*Asteroids meets crew RPG in an infinite universe that writes your story.*

---

## Content Framework

### World and Setting

An infinite, seed-generated universe with a positive and inviting atmosphere. The void of space is not hostile or lonely — it's a playground full of discoveries waiting to happen. Different biomes provide visual and gameplay variety (asteroid fields, nebulae, enemy territories, quiet zones), while stations serve as warm anchor points in the expanse.

**Setting Tone:** Optimistic sci-fi — danger exists, but the universe feels welcoming. The world invites exploration rather than punishing it. Most space games default to isolation and dread (Dead Space, FTL, Everspace). Void Drifter's warm, inviting atmosphere is a hidden differentiator — the universe says "come explore" instead of "survive if you can."

**World-Building Depth:** Lore plays a background role. The game lives through its mechanics and gameplay, not through exposition or text-heavy world-building. Lore may be layered in post-MVP through environmental details, station flavor text, or NPC barks — but it is never required to enjoy the game.

### Narrative Approach

**Primary:** Emergent narrative — the player's story writes itself through gameplay decisions, crew interactions, faction encounters, and exploration. The event logbook captures and preserves these stories.

**Secondary:** Minimal scripted lore — no linear plot, no cutscenes, no mandatory dialogue. The universe provides context (factions exist, stations have purposes, biomes have character), but the story is what happens to YOU.

**Story Delivery:**
- Event logbook (automatic, always running)
- Companion barks (contextual, personality-driven)
- Station interactions (trading, upgrading, NPC encounters)
- Faction reputation shifts (consequences of player actions)
- Environmental storytelling at zero cost — procedurally generated station names and flavor descriptions ("Abandoned Trading Post Kappa-7") create micro-narratives through string templates
- No cutscenes, no dialogue trees, no scripted narrative

**Future Option:** Lore and story layers can be added post-launch without changing core architecture.

### Content Volume

**MVP Content Targets:**
- 1 biome type (asteroid field with density variation)
- 1 station type (trading/upgrade hub)
- 2 weapon types (laser, spread) — enough to prove weapon switching works
- 1 companion as proof of concept
- 3 faction affiliations (enemy, neutral, friendly — static enums)
- Basic event logbook with Tier 1 events

**Post-MVP Content Expansion:**
- Additional biomes (nebulae, debris fields, enemy strongholds, quiet zones)
- Specialized station types (repair, black market, faction HQ)
- Additional weapons (rockets, mines) and weapon upgrade paths
- 3-4 additional companions with unique personalities and bark sets
- Wormhole mini-dimensions
- Expanded event logbook with Tier 2 events and narrative generation
- Faction dynamics and autonomous behavior

---

## Art and Audio Direction

### Visual Style

**Procedural Vector Graphics** — all visual elements generated dynamically in code. Ships, stations, asteroids, and structures are built from geometric primitives using Bevy's `Mesh` API (simple polygons and circles for MVP, `lyon` for complex shapes when needed).

**Color Palette:** Vibrant and lively with a clear contrast hierarchy:
- **Background:** Dark, subtle star field — the canvas
- **Environment** (asteroids, nebulae): Medium saturation — present but not competing for attention
- **Interactive** (ships, stations, loot, companions): High saturation + glow effects — immediately readable as "things that matter"
- **Danger** (enemies, projectiles, hazards): Warm signal colors (red, orange) — instant visual threat recognition

Each biome has a distinct color identity — warm oranges for asteroid fields, cool blues for nebulae, neon greens for alien territory. The universe feels alive and inviting, not cold and sterile.

**Visual Juice (Technical Requirement, not Nice-to-Have):**
- Bloom/glow effects for energy, weapons, and engines (Bevy built-in bloom support)
- Particle effects for thrust, explosions, and environmental hazards (`bevy_hanabi`)
- Screen shake on impacts and explosions
- These effects are what make arcade games *feel* satisfying — they are core to the Arcade Action pillar

**References:** Geometry Wars (glow aesthetic), Asteroids (ship silhouette language), Reassembly (procedural ship construction)

### Audio Style

**Music:** Synthwave — pulsing, retro-futuristic beats that match the arcade action and complement the vibrant visuals. Upbeat during exploration, intensifying during combat, ambient during quiet zones.

**Sound Effects:** Punchy arcade SFX for weapons, thrusters, explosions, and docking. Satisfying feedback on every player action. Prototype uses free SFX from freesound.org via `bevy_audio`.

**Voice Acting:** None. Companions communicate through text barks only. This is a deliberate design choice that keeps scope manageable and lets the player imagine their crew's voices.

### Production Approach

**In-House (Simon):**
- All code (Bevy/Rust)
- All procedural visuals (code-generated)
- Game design and balancing
- Event logbook system

**Deferred / External:**
- Music: Deferred to late development. Options: royalty-free synthwave tracks, commission from indie musician, or AI-generated placeholder
- SFX: Free libraries (freesound.org) for prototype, potentially upgraded for release
- Marketing: Community-driven via seed-sharing, Rust gamedev community, itch.io visibility

**Tools:** All open-source — Rust, Bevy, cargo, git. No paid tools or licenses required.

---

## Risk Assessment

### Key Risks

| Risk | Likelihood | Impact | Priority |
|------|-----------|--------|----------|
| Scope creep (companion system, factions) | High | High | Critical |
| Solo-dev burnout (main job + game dev) | Medium | High | Critical |
| The Fun Gap (systems before fun) | Medium | High | Critical |
| Chunk persistence + WASM memory limits | Medium | High | High |
| Bevy breaking changes (pre-1.0 engine) | Medium | Medium | High |
| NPC AI complexity exceeding estimates | Medium | Medium | Medium |
| Discoverability as unknown indie | Medium | Medium | Medium |
| Audio skill gap delaying polish | Low | Low | Low |

### Technical Challenges

- **Chunk-Persistenz mit WASM:** Single-threaded chunk loading must work within browser memory limits. Architecture must be designed for WASM constraints from day 1 — retrofitting is not an option.
- **Seed + Delta Serialization:** Save format must be stable and forward-compatible. Include format version number from day 1. Breaking changes to serialization corrupt all existing saves.
- **NPC State Machines:** Even simple Follow/Attack/Flee AI requires extensive tuning. Companion opinion systems (N×N matrix) must be constrained to 3-4 NPCs in MVP.
- **60fps with Many Entities:** Procedural worlds spawn many asteroids, projectiles, and NPCs simultaneously. ECS architecture helps, but profiling must be continuous.
- **Bevy Pre-1.0 Instability:** Breaking changes between minor versions can require significant migration effort. Pin Bevy version in `Cargo.toml`, upgrade only between sprints — never mid-epic.
- **Visual Juice Pipeline:** Bloom, particles (`bevy_hanabi`), and screen shake require post-processing setup in Bevy. Not trivial — allocate dedicated time in Core Flight epic.

### Market Risks

- **Discoverability:** No marketing budget. Mitigation through organic channels: Rust gamedev community, seed-sharing virality, itch.io prototype feedback, content creator potential.
- **Genre Expectations:** "Asteroids-like" may set wrong expectations (simple arcade). Messaging must emphasize the open-world and companion aspects.

### Mitigation Strategies

- **The Fun Gap → Sprint 0 (mandatory).** Before any MVP epic begins: one weekend building pure fly + shoot in empty space. If it doesn't make you grin in 30 seconds, re-evaluate before investing months. The arcade heartbeat must be proven before systems are built around it.
- **Scope Creep → Ruthless MVP discipline.** Only Priority 1 features in MVP. Each post-MVP feature must prove its value before implementation. The Scrum Master's epic breakdown is the guardrail.
- **Burnout → Sustainable pace.** 2-week sprints with realistic goals. Every sprint produces a playable build — visible progress prevents motivation death spirals. Take breaks without guilt.
- **WASM Limits → Test WASM from sprint 1.** Never "port later." CI builds for both native and WASM from the start. If something doesn't work in WASM, discover it immediately.
- **Serialization → Design save format early, version it.** Include a format version number from day 1. Migrations between versions are explicit and tested.
- **NPC Complexity → Start with 1 companion.** Prove the bark system and opinion scores work with a single NPC before adding more. Scale horizontally only after the pattern is proven.
- **Bevy Instability → Pin and upgrade deliberately.** Lock Bevy version, upgrade only between sprints with a dedicated migration task. Monitor Bevy release notes proactively.
- **Discoverability → Build community early.** Publish on itch.io as soon as a playable prototype exists. Share development progress in Rust/Bevy communities. Seed-sharing as built-in viral mechanic.

---

## Success Criteria

### MVP Definition

The Minimum Playable Version of Void Drifter is the smallest build that delivers a complete, fun core gameplay loop:

**Sprint 0 (Pre-MVP — mandatory):**
- Pure arcade prototype: fly and shoot in empty space
- Must feel satisfying within 30 seconds — if not, re-evaluate before proceeding

**MVP Scope:**
- Physics-based flight with inertia (rotate, thrust, drift)
- 2 weapon types (laser, spread) with weapon switching
- Procedural chunk-based world generation from seed
- 1 biome (asteroid field with density variation)
- 1 station type (trading/upgrade hub) with docking
- Basic HUD and minimap
- 1 companion as proof of concept (Follow/Attack/Flee AI + bark system)
- 3 static faction affiliations (enemy, neutral, friendly)
- Basic event logbook with Tier 1 events
- Save/load with seed + delta persistence
- WASM build alongside native from sprint 1
- Visual juice: bloom, particles, screen shake

**Explicitly NOT in MVP:**
- Additional biomes, stations, or weapons (Post-MVP Phase 2)
- Faction dynamics or autonomous simulation (Post-MVP Phase 3)
- NPC opinion systems beyond 1 companion (Post-MVP Phase 3)
- Wormhole mini-dimensions (Post-MVP Phase 4)
- Audio/music (Post-MVP Phase 5)
- Online/multiplayer features (fully deferred)

### Success Metrics

**Primary Metric:** Player reach — the game is widely played and shared.

| Metric | Target | How to Measure |
|--------|--------|----------------|
| itch.io downloads (prototype) | 500+ | itch.io analytics |
| Steam wishlists (Early Access) | 5,000+ | Steamworks dashboard |
| Average session length | 20+ minutes | In-game telemetry (opt-in) |
| Seed-sharing activity | Organic sharing observed | Community channels (Discord, Reddit) |
| Rust/Bevy community attention | Dev log engagement | Reddit/Discord post engagement |
| Player retention (return within 7 days) | 30%+ | In-game telemetry (opt-in) |

**Personal Success:** The game is fun to play, people enjoy it, and seeds get shared organically.

### Launch Goals

**itch.io Prototype Launch:**
- Playable MVP with core loop complete
- Gather feedback from early players
- Build initial community

**Steam Early Access:**
- MVP + at least 2 additional biomes and 2+ companions
- Event logbook with Tier 1 + Tier 2 events
- Multiple weapon types and upgrade paths
- Community feedback loop established

**Steam Full Launch:**
- Complete content suite (biomes, companions, factions, wormholes)
- Full event logbook with narrative generation
- Audio/music integrated
- Polished UI and onboarding

---

## Next Steps

### Immediate Actions

1. **Create Game Design Document (GDD)** — Transform this brief into detailed design specifications using the `bmad-gds-gdd` workflow
2. **Sprint 0: Pure Arcade Prototype** — One weekend: fly + shoot in empty space. Validate the core feeling before investing further
3. **Technical Architecture** — Design chunk system, persistence architecture, and WASM strategy using the `bmad-gds-game-architecture` workflow

### Research Needs

- Bevy 0.18 bloom/post-processing capabilities and `bevy_hanabi` particle system maturity
- WASM memory limits and chunk loading strategies for browser targets
- Procedural vector generation approaches in Bevy (Mesh API vs `lyon` performance characteristics)
- Synthwave music licensing options (royalty-free libraries, indie musician commissions)

### Open Questions

No open questions at this time. All major design decisions have been made through the Game Brief workflow. Detailed implementation questions will be resolved during GDD creation and technical architecture planning.

---

## Appendices

### A. Input Documents

- **Brainstorming Session (2026-02-25):** `_bmad-output/brainstorming-session-2026-02-25.md` — 117 ideas across 11 themes, organized into 3 priority tiers

### B. Party Mode Reviews

Party Mode multi-agent reviews were conducted for:
- Game Vision (2 rounds)
- Target Market (2 rounds)
- Game Fundamentals (2 rounds)
- Scope & Constraints (1 round)
- Reference Framework (2 rounds)
- Content & Production (1 round)

Agents consulted: Game Architect (Cloud Dragonborn), Game Designer (Samus Shepard), Game Developer (Link Freeman), Scrum Master (Max), Game QA (GLaDOS)

### C. Design Principles (from Brainstorming)

1. If the core loop isn't fun, nothing else matters
2. Procedural generation serves gameplay, not the other way around
3. NPCs are characters, not mechanics
4. The player's story is the game's story
5. Complexity through emergence, not through systems

---

_This Game Brief is complete and serves as the foundational input for Game Design Document (GDD) creation._

_Next Steps: Use the `bmad-gds-gdd` command to create detailed game design documentation._
