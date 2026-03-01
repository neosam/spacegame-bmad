# Epic 6c Retrospective — Companion Combat

**Date:** 2026-02-28
**Epic:** 6c — Companion Combat
**Participants:** Simon (Project Lead), Max (Scrum Master), Samus (Game Designer), Link (Dev Lead), GLaDOS (QA), Cloud (Architect)
**Tests at completion:** 712 (was 681, +31)

---

## Epic Summary

| Metric | Value |
|--------|-------|
| Stories completed | 5 / 5 (100%) |
| Tests added | +31 |
| New components | `CompanionFlight`, `CompanionTarget`, `CompanionWeapon`, `CompanionPrevHealth` |
| New systems | `update_companion_rotation`, `update_companion_thrust_and_drag`, `update_companion_target`, `fire_companion_weapon`, `detect_companion_damage` |
| New pure functions | `rotate_toward_angle`, `angle_toward`, `facing_from_angle`, `nearest_enemy`, `should_fire`, `format_opinion_score` |
| Post-ship fixes | 3 (rotation missing, self-hit projectile, balance tuning) |

---

## 6b-Retro Action Items — Follow-Through

| Action Item | Status | Notes |
|-------------|--------|-------|
| Wire up DamageTaken bark (6c-4) | ✅ Erledigt | `CompanionPrevHealth` + `detect_companion_damage` |
| Opinion sichtbar machen (6c-5) | ✅ Erledigt | Score im Bark-Text: `Wing-1 (+12): "Target down!"` |
| Schiffsphysik (6c-1) | ✅ Erledigt | rotate + thrust + drag — echter Schiffsflug |

**100% follow-through** auf alle 3 committed Action Items.

---

## What Went Well

- **Vollständige 6b-Abdeckung** — Alle offenen Punkte aus der 6b-Retro wurden erledigt.
- **Saubere Architektur** — Zwei getrennte Systeme (`rotation` + `thrust`) statt einem monolithischen. Leicht erweiterbar.
- **Pure Functions** — 6 neue testbare Pure Functions. Logik vollständig von ECS getrennt.
- **Schnelle Bug-Recovery** — Beide Post-Ship-Bugs (Transform-Rotation vergessen, Selbsttreffer) wurden in derselben Session entdeckt und gefixt.
- **Companion kämpft jetzt wirklich mit** — Player-Feedback: "Letztendlich recht cool."

---

## What Didn't Go Well

- **Transform.rotation vergessen** — `flight.angle` wurde berechnet aber nie auf `transform.rotation` angewendet. Visuell rotierte der Companion nicht, obwohl die Logik korrekt war. Erst durch Spielertest entdeckt.
- **Selbsttreffer-Bug** — Projektil spawnte bei `companion_pos` (innerhalb des eigenen Colliders). Beim nächsten Frame: Selbsttreffer, Schaden, Bark-Loop. Fix: Spawn-Offset `direction * 20.0`.
- **Balance nicht getestet** — `turn_rate: 3.5`, `thrust: 220`, `max_speed: 200` fühlten sich im Spiel zu langsam an. Simon: "Die NPCs sind sehr langsam, ich muss immer auf sie warten." Fix: turn_rate 6.0, thrust 400, max_speed 320, drag 1.5.
- **Kein Playtest-Schritt im DoD** — Unit Tests prüfen Logik, aber nicht Gameplay-Feel. Balance-Parameter brauchen manuelles Spieltesting.

---

## Key Insights

1. **Unit Tests ≠ Gameplay-Feel** — `should_fire`, `nearest_enemy` etc. sind korrekt und getestet, aber die Parameter dahinter (Geschwindigkeit, Reichweite) müssen im echten Spiel validiert werden.
2. **Output-Steps vergessen** — Der häufigste Bug-Typ: Berechnung richtig, Ergebnis nicht angewendet. Checkliste: "Wird das Ergebnis auch tatsächlich geschrieben?"
3. **Spawn-Offsets bei Projektilen** — Immer Projektile außerhalb des Spawner-Colliders spawnen. Faustregel: `spawn_pos = origin + direction * (collider_radius + buffer)`.

---

## Bugs Entdeckt (nicht 6c-spezifisch)

### Bug 1: Respawn in falschem Biom
**Symptom:** Nach dem Tod bei `Vec3::ZERO` respawnen — aber das Biom ist anders als beim Spielstart (Asteroiden-Feld statt Tutorial Zone).
**Ursache:** `spawn_tutorial_zone` spawnt einmalig handgefertigte Entities. Wenn der Spieler die Zone verlässt, werden die Chunks entladen. Beim Respawn regeneriert das Chunk-Loading-System Chunk (0,0) **prozedural** — mit dem Biom-Seed, der ein Asteroiden-Feld ergibt. Die Tutorial Zone ist ein Einweg-Spawn.
**Empfehlung:** Zwei unabhängige Fixes:
- Respawn an letzter besuchter Station statt `Vec3::ZERO`
- Tutorial-Zone-Chunks als "permanent" markieren (kein prozedurales Überschreiben)

---

## Action Items

| Item | Priorität | Ziel |
|------|-----------|------|
| Playtest-Schritt in DoD aufnehmen: Balance-Parameter manuell spielen | Medium | ab sofort |
| Output-Step-Checkliste: "Wird das Ergebnis geschrieben?" | Low | ab sofort |
| Respawn an letzter Station statt Vec3::ZERO | **High** | nächster Sprint |
| Tutorial Zone Chunks vor prozeduralem Überschreiben schützen | Medium | nächster Sprint |

---

## Readiness Assessment

| Bereich | Status |
|---------|--------|
| Tests | ✅ 712, alle grün |
| Rotation | ✅ gefixt |
| Selbsttreffer | ✅ gefixt |
| Balance | ✅ gefixt (turn_rate 6.0, thrust 400) |
| Bekannte offene Bugs | ⚠️ Respawn/Biom-Bug (seit Epic 1, nicht 6c) |

Epic 6c: **ship-ready**. Der Respawn/Biom-Bug ist pre-existing und blockiert nicht den 6c-Abschluss.

---

## Next Steps

1. Sprint-Status für Epic 6c auf `done` setzen
2. Respawn + Tutorial-Zone-Bugs als eigene Stories für einen Bugfix-Sprint anlegen
3. Nächstes Epic planen (Epic 7: Boss Encounters oder Epic 8: Logbook UI)

---

*Retrospective conducted with Simon, 2026-02-28*
