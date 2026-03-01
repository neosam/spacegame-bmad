/// Social systems: enemy AI, factions, neutral entities.
/// Epic 4: Combat Depth stories will register systems here.
pub mod enemy_ai;
pub mod faction;

use bevy::prelude::*;

pub struct SocialPlugin;

impl Plugin for SocialPlugin {
    fn build(&self, _app: &mut App) {
        // Epic 4 systems registered per story
    }
}
