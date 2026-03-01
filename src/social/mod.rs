/// Social systems: enemy AI, factions, neutral entities.
/// Epic 4: Combat Depth stories will register systems here.
pub mod enemy_ai;
pub mod faction;

use bevy::prelude::*;

use self::enemy_ai::{
    update_fighter_ai, update_heavy_cruiser_ai, update_scout_drone_ai, update_sniper_ai,
    update_swarm_ai, PendingEnemyShotQueue,
};
use self::faction::FactionBehaviorProfiles;

pub struct SocialPlugin;

impl Plugin for SocialPlugin {
    fn build(&self, app: &mut App) {
        // Story 4-1: Scout Drone AI
        app.init_resource::<PendingEnemyShotQueue>();
        app.add_systems(Update, update_scout_drone_ai);
        // Story 4-2: Fighter AI
        app.add_systems(Update, update_fighter_ai);
        // Story 4-3: Heavy Cruiser AI
        app.add_systems(Update, update_heavy_cruiser_ai);
        // Story 4-4: Sniper AI
        app.add_systems(Update, update_sniper_ai);
        // Story 4-5: Swarm AI
        app.add_systems(Update, update_swarm_ai);
        // Story 4-6: Faction Behavior Profiles
        app.init_resource::<FactionBehaviorProfiles>();
    }
}
