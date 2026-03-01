/// Social systems: enemy AI, factions, neutral entities.
/// Epic 4: Combat Depth stories will register systems here.
pub mod enemy_ai;
pub mod faction;

use bevy::prelude::*;

use self::enemy_ai::{update_scout_drone_ai, PendingEnemyShotQueue};

pub struct SocialPlugin;

impl Plugin for SocialPlugin {
    fn build(&self, app: &mut App) {
        // Story 4-1: Scout Drone AI
        app.init_resource::<PendingEnemyShotQueue>();
        app.add_systems(Update, update_scout_drone_ai);
    }
}
