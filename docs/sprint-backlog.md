# Sprint Backlog — Epic 6c: Companion Combat

**Sprint:** 8
**Epic:** 6c — Companion Combat
**Datum:** 2026-02-28
**Dependencies:** Epic 6b ✅

---

## Stories

| Story ID | Titel | Status | Abhängigkeiten |
|----------|-------|:------:|----------------|
| 6c-1 | Companion Ship Flight | todo | keine |
| 6c-2 | Companion Weapon | todo | 6c-1 |
| 6c-3 | Target Acquisition | todo | 6c-1 |
| 6c-4 | DamageTaken Bark | todo | 6c-1, 6c-2 |
| 6c-5 | Opinion HUD | todo | keine |

---

## Story Beschreibungen

### 6c-1: Companion Ship Flight
Als Spieler sehe ich meinen Companion sich realistisch drehen und in Flugrichtung beschleunigen, damit er wie ein echtes Schiff wirkt und nicht wie ein schwebendes Objekt.

**Technisch:** Companion braucht eigene Rotationslogik (dreht sich zum Zielvektor), Thrust in Blickrichtung, Drag — analog zu `apply_rotation` + `apply_thrust` + `apply_drag` des Players, aber KI-gesteuert.

### 6c-2: Companion Weapon
Als Spieler sieht mein Companion im Attack-Modus auf Feinde in Reichweite und feuert mit eigenem Cooldown, damit er aktiv zum Kampf beiträgt.

**Technisch:** Companion bekommt `CompanionWeapon { damage, range, cooldown_secs }` Component. System feuert Projektile auf das aktuelle Ziel.

### 6c-3: Target Acquisition
Als Spieler richtet sich mein Companion im Attack-Modus auf den nächsten Feind aus und verfolgt ihn, damit das Kampfverhalten glaubwürdig ist.

**Technisch:** `CompanionTarget { entity: Option<Entity> }` Component. System sucht nächsten Feind innerhalb `aggro_range`, setzt Target, dreht Companion dorthin.

### 6c-4: DamageTaken Bark
Als Spieler höre ich meinen Companion reagieren wenn er getroffen wird, damit er sich lebendig anfühlt.

**Technisch:** `DamageTaken`-BarkTrigger in `pick_bark()` ist bereits implementiert — nur das auslösende System fehlt. Health-Change-Detektion auf Companion-Entities.

### 6c-5: Opinion HUD
Als Spieler sehe ich kurz die aktuelle Stimmung meines Companions (z.B. "Wing-1 mag dich sehr"), damit die Opinion-Werte spürbar werden.

**Technisch:** Opinion-Score neben Bark-Text anzeigen, oder als separates kleines Label neben dem Companion-Namen.
