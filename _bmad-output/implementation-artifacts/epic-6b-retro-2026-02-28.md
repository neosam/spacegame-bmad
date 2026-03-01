# Epic 6b Retrospective — Companion Personality

**Date:** 2026-02-28
**Epic:** 6b — Companion Personality
**Participants:** Simon (Project Lead), Max (Scrum Master), Samus (Game Designer), Link (Dev Lead), GLaDOS (QA), Cloud (Architect)
**Tests at completion:** 681 (was 659, +22)

---

## Epic Summary

| Metric | Value |
|--------|-------|
| Stories completed | 4 / 4 (100%) |
| Tests added | +22 |
| New files | `src/social/companion_personality.rs` |
| Modified files | `companion.rs`, `social/mod.rs`, `rendering/mod.rs` |

---

## What Went Well

- **Bark system works** — Companions react with contextual one-liners. Immediately visible and satisfying to the player.
- **Clean architecture** — All 4 stories fit into one new module (`companion_personality.rs`). Pure functions, good test coverage.
- **HUD lesson applied** — Epic 5 action item "HUD-Checkliste" was partially applied: the bark HUD was built at startup.
- **Fast implementation** — All 4 stories in one coding run, zero regressions, clean compile.

---

## What Didn't Go Well

- **Opinion system is invisible** — `PlayerOpinions` and `PeerOpinions` resources exist in memory but the player can never see or feel them. No UI, no feedback.
- **Combat behavior was shallow** — Story 6b-4 "Personality Combat Behavior" only tweaked `follow_distance` and `follow_speed`. The player fantasy of "my wingman fights alongside me" was never delivered.
- **Companion doesn't attack** — No `Weapon`, `FireCooldown`, or `Target` component exists. Combat behavior is structurally impossible without these foundations.
- **Companion flight is unrealistic** — Velocity is set directly to a target vector (teleport-style). The companion doesn't rotate first and thrust in its facing direction like a real ship. This breaks immersion.
- **Dead code: `DamageTaken` trigger** — `pick_bark()` handles `BarkTrigger::DamageTaken` but no system ever emits it.

---

## Key Insights

1. **"Behavior" without physics feels wrong** — A companion that slides toward targets instead of flying like a ship breaks the player's mental model immediately.
2. **Invisible systems don't count as features** — Opinion tracking is technically correct but delivers zero player value until there's UI.
3. **Story 6b-4 needed 6c-1 as a prerequisite** — Combat behavior can't be real without proper ship flight. Scope was defined before the dependency was visible.

---

## Action Items

| Item | Priority | Target |
|------|----------|--------|
| Wire up `DamageTaken` bark in Epic 6c when companion takes damage | Medium | 6c-4 |
| Make opinion visible to player (bark or HUD) | Medium | 6c-5 |
| Implement realistic ship flight for companion (rotate + thrust) | **Critical** | 6c-1 |

---

## Previous Retro Follow-Through (Epic 5)

| Action Item | Status | Notes |
|-------------|--------|-------|
| HUD-Checkliste vor Story-Abschluss | ⏳ Partial | Bark HUD built ✅, opinion/personality HUD not built ❌ |
| Sichtbarkeits-Check in DoD | ⏳ Partial | Applied for barks, missed for opinion system |

---

## Significant Discovery — Epic Update Required

Epic 6b revealed that the companion needs proper ship physics as a prerequisite for any meaningful combat behavior. This was not scoped in 6a or 6b.

**Wrong assumption:** "Companion combat behavior can be delivered by modifying follow parameters."
**Reality:** Companion needs rotate + thrust physics, target acquisition, and a weapon system before any combat behavior is meaningful.

**Recommendation:** Implement **Epic 6c: Companion Combat** before moving to Epic 7 or 8.

---

## Epic 6c: Companion Combat — Planned Scope

| Story | Title | Dependency |
|-------|-------|-----------|
| 6c-1 | Companion Ship Flight — rotates, thrusts in facing direction, drifts | none |
| 6c-2 | Companion Weapon — fires at enemies in range with cooldown | 6c-1 |
| 6c-3 | Target Acquisition — tracks nearest enemy in Attack mode | 6c-1 |
| 6c-4 | DamageTaken Bark — companion reacts when hit | 6c-1, 6c-2 |
| 6c-5 | Opinion HUD — shows companion mood briefly as bark extension | none |

---

## Next Steps

1. Create `docs/sprint-backlog.md` for Epic 6c
2. Implement 6c-1 (Companion Ship Flight) first — load-bearing for all other stories
3. Review `DamageTaken` dead code in `pick_bark()` when wiring 6c-4

---

*Retrospective conducted with Simon, 2026-02-28*
