# Story 6c-4: DamageTaken Bark

## Goal
Companion reacts with a bark when hit, making it feel responsive and alive.

## Acceptance Criteria
- Companion spawned with `Health { current: 100.0, max: 100.0 }` and `Collider { radius: 14.0 }`
- `CompanionPrevHealth { value: f32 }` component tracks last-frame health
- `detect_companion_damage` system: if health decreased → emit DamageTaken bark via BarkDisplay
- Existing `DamageTaken` bark lines in `pick_bark()` now fire
- Unit test: detect_companion_damage fires when health decreases

## Implementation
Modified `handle_recruit_companion` to add Health + Collider + CompanionPrevHealth.
New system `detect_companion_damage` added to `src/social/companion_personality.rs`.
Registered in `src/social/mod.rs`.
