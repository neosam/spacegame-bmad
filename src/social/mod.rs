/// Social systems: enemy AI, factions, neutral entities, companions.
/// Epic 4: Combat Depth stories will register systems here.
/// Epic 6a: Companion Core systems registered here.
pub mod companion;
pub mod enemy_ai;
pub mod faction;

use bevy::prelude::*;

use self::companion::{
    CompanionRoster, handle_recruit_companion, handle_wingman_commands,
    update_companion_follow, update_companion_positions,
    handle_companion_survival, update_retreating_companions,
};
use self::enemy_ai::{
    update_enemy_facing, update_fighter_ai, update_heavy_cruiser_ai, update_scout_drone_ai,
    update_sniper_ai, update_swarm_ai, PendingEnemyShotQueue,
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
        // Story 4-8: Attack Telegraphing
        app.add_systems(Update, update_enemy_facing);

        // Story 6a-1: Companion Roster resource
        app.init_resource::<CompanionRoster>();
        // Story 6a-1: Recruit companion when docked + recruit action pressed
        app.add_systems(Update, handle_recruit_companion);
        // Story 6a-2: Companion follow AI
        app.add_systems(Update, update_companion_follow);
        // Story 6a-2: Companion position integration (apply velocity)
        app.add_systems(Update, update_companion_positions.after(update_companion_follow));
        // Story 6a-3: Wingman command cycling
        app.add_systems(Update, handle_wingman_commands);
        // Story 6a-5: Companion survival (retreat to station on player death)
        app.add_systems(Update, handle_companion_survival);
        app.add_systems(Update, update_retreating_companions.after(handle_companion_survival));
    }
}
