# Story 6a-4: Companion Visuals

**Epic:** 6a — Companion Core
**Status:** done

## User Story
As a player, my companion has a distinct visual silhouette based on their faction origin so that they feel unique.

## Acceptance Criteria
- [x] `NeedsCompanionVisual` marker component exists
- [x] `generate_companion_mesh()` creates a smaller ship triangle
- [x] `CompanionAssets` resource caches mesh + per-faction materials
- [x] `render_companions` system attaches Mesh2d + MeshMaterial2d by faction
- [x] Core spawns NeedsCompanionVisual, Rendering removes it

## Technical Notes
- Companion mesh: 7-point ship silhouette, ~60% of player size
- Color palette: Neutral=cyan, Pirates=red-orange, Military=blue-green, Aliens=purple, RogueDrones=orange-yellow
