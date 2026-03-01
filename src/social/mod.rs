/// Social systems: enemy AI, factions, neutral entities, companions.
/// Epic 4: Combat Depth stories will register systems here.
/// Epic 6a: Companion Core systems registered here.
/// Epic 6b: Companion Personality, barks, opinions registered here.
pub mod companion;
pub mod companion_personality;
pub mod enemy_ai;
pub mod faction;

use bevy::prelude::*;

use self::companion::{
    CompanionRoster, handle_recruit_companion, handle_wingman_commands,
    update_companion_rotation, update_companion_thrust_and_drag, update_companion_positions,
    handle_companion_survival, update_retreating_companions,
};
use self::companion_personality::{
    BarkDisplay, PlayerOpinions, PeerOpinions,
    emit_barks_on_game_events, emit_bark_on_command_change, tick_bark_display,
    update_player_opinions, update_peer_opinions, update_personality_behavior,
    update_companion_target, fire_companion_weapon, detect_companion_damage,
};
use self::enemy_ai::{
    update_boss_ai, update_boss_telegraphing, update_boss_flee_bark, tick_boss_retreat_bark,
    update_enemy_facing, update_fighter_ai, update_heavy_cruiser_ai,
    update_scout_drone_ai, update_sniper_ai, update_swarm_ai,
    BossRetreatBark, PendingEnemyShotQueue,
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
        // Story 7-1: Boss AI
        app.add_systems(Update, update_boss_ai);
        // Story 7-2: Boss Telegraphing
        app.add_systems(Update, update_boss_telegraphing.after(update_boss_ai));
        // Story 7-5: Boss Flee Signal
        app.init_resource::<BossRetreatBark>();
        app.add_systems(Update, update_boss_flee_bark.after(update_boss_ai));
        app.add_systems(Update, tick_boss_retreat_bark);

        // Story 6a-1: Companion Roster resource
        app.init_resource::<CompanionRoster>();
        // Story 6a-1: Recruit companion when docked + recruit action pressed
        app.add_systems(Update, handle_recruit_companion);
        // Story 6c-1: Companion ship physics (replaces old update_companion_follow)
        app.add_systems(Update, update_companion_rotation);
        app.add_systems(Update, update_companion_thrust_and_drag.after(update_companion_rotation));
        // Story 6a-2: Companion position integration (apply velocity)
        app.add_systems(Update, update_companion_positions.after(update_companion_thrust_and_drag));
        // Story 6a-3: Wingman command cycling
        app.add_systems(Update, handle_wingman_commands);
        // Story 6a-5: Companion survival (retreat to station on player death)
        app.add_systems(Update, handle_companion_survival);
        app.add_systems(Update, update_retreating_companions.after(handle_companion_survival));

        // Story 6b-1: Companion barks
        app.init_resource::<BarkDisplay>();
        app.add_systems(Update, emit_barks_on_game_events);
        app.add_systems(Update, emit_bark_on_command_change);
        app.add_systems(Update, tick_bark_display);

        // Story 6b-2: Player opinions
        app.init_resource::<PlayerOpinions>();
        app.add_systems(Update, update_player_opinions);

        // Story 6b-3: Peer opinions
        app.init_resource::<PeerOpinions>();
        app.add_systems(Update, update_peer_opinions);

        // Story 6b-4: Personality combat behavior
        app.add_systems(Update, update_personality_behavior.before(update_companion_rotation));

        // Story 6c-3: Target acquisition
        app.add_systems(Update, update_companion_target.before(update_companion_rotation));
        // Story 6c-2: Companion weapon firing
        app.add_systems(Update, fire_companion_weapon.after(update_companion_rotation));
        // Story 6c-4: Damage taken bark
        app.add_systems(Update, detect_companion_damage);
    }
}
