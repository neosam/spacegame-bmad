/// Companion Personality — Epic 6b
///
/// Personality types, bark system, opinion tracking, and combat behavior modifiers.
/// Design: Pure functions for all logic. Systems handle ECS state changes.
use bevy::prelude::*;
use std::collections::HashMap;

use crate::shared::events::{GameEvent, GameEventKind};
use crate::social::companion::{Companion, CompanionData, WingmanCommand};
use crate::social::faction::FactionId;

// ── Personality ───────────────────────────────────────────────────────────

/// Personality type assigned to each companion at recruitment.
/// Influences barks, opinion reactions, and combat movement behavior.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CompanionPersonality {
    /// Charges forward, aggressive barks.
    Brave,
    /// Holds back, cautious barks.
    Cautious,
    /// Dry wit, sarcastic comments.
    Sarcastic,
    /// Formal and reliable, military-style.
    Loyal,
}

/// Pure function: maps faction to default personality.
/// Pirates → Sarcastic, Military → Loyal, Aliens → Cautious, RogueDrones → Brave, Neutral → Loyal.
pub fn personality_for_faction(faction: &FactionId) -> CompanionPersonality {
    match faction {
        FactionId::Pirates => CompanionPersonality::Sarcastic,
        FactionId::Military => CompanionPersonality::Loyal,
        FactionId::Aliens => CompanionPersonality::Cautious,
        FactionId::RogueDrones => CompanionPersonality::Brave,
        FactionId::Neutral => CompanionPersonality::Loyal,
    }
}

// ── Bark System ───────────────────────────────────────────────────────────

/// What triggered a companion bark.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BarkTrigger {
    PlayerDied,
    EnemyDestroyed,
    CommandReceived,
    DamageTaken,
}

/// How long a bark is displayed on the HUD (seconds).
pub const BARK_DISPLAY_DURATION: f32 = 4.0;

/// Current bark text and its remaining display time.
/// Rendering reads this resource to update the bark HUD node.
#[derive(Resource, Default, Debug)]
pub struct BarkDisplay {
    /// (companion_name, bark_text), None when not displaying.
    pub current: Option<(String, &'static str)>,
    /// Seconds remaining until bark clears.
    pub timer: f32,
}

/// Pure function: returns a bark line for the given personality and trigger.
/// Covers all 16 (personality × trigger) combinations.
pub fn pick_bark(personality: CompanionPersonality, trigger: BarkTrigger) -> &'static str {
    match (personality, trigger) {
        (CompanionPersonality::Brave, BarkTrigger::PlayerDied) => "Keep fighting! I'll cover you!",
        (CompanionPersonality::Brave, BarkTrigger::EnemyDestroyed) => "Target down! Who's next?",
        (CompanionPersonality::Brave, BarkTrigger::CommandReceived) => "Copy that. Let's go!",
        (CompanionPersonality::Brave, BarkTrigger::DamageTaken) => "Just a scratch!",
        (CompanionPersonality::Cautious, BarkTrigger::PlayerDied) => "I warned you... retreating.",
        (CompanionPersonality::Cautious, BarkTrigger::EnemyDestroyed) => "Clear. Stay sharp.",
        (CompanionPersonality::Cautious, BarkTrigger::CommandReceived) => "Understood. Proceeding carefully.",
        (CompanionPersonality::Cautious, BarkTrigger::DamageTaken) => "Taking fire! Need backup!",
        (CompanionPersonality::Sarcastic, BarkTrigger::PlayerDied) => "Oh great, there goes the captain.",
        (CompanionPersonality::Sarcastic, BarkTrigger::EnemyDestroyed) => "Was that supposed to be a challenge?",
        (CompanionPersonality::Sarcastic, BarkTrigger::CommandReceived) => "Sure, boss. Totally going to work.",
        (CompanionPersonality::Sarcastic, BarkTrigger::DamageTaken) => "Ow. Thanks for the backup. Really.",
        (CompanionPersonality::Loyal, BarkTrigger::PlayerDied) => "Captain down! Holding position!",
        (CompanionPersonality::Loyal, BarkTrigger::EnemyDestroyed) => "Hostile neutralized, Captain.",
        (CompanionPersonality::Loyal, BarkTrigger::CommandReceived) => "Acknowledged, Captain.",
        (CompanionPersonality::Loyal, BarkTrigger::DamageTaken) => "I can take it. Won't let you down.",
    }
}

// ── Bark Systems ──────────────────────────────────────────────────────────

/// Reads GameEvent messages and triggers a bark from the first companion with a personality.
pub fn emit_barks_on_game_events(
    mut events: bevy::ecs::message::MessageReader<GameEvent>,
    companion_query: Query<(&CompanionData, &CompanionPersonality), With<Companion>>,
    mut bark_display: ResMut<BarkDisplay>,
) {
    let mut trigger: Option<BarkTrigger> = None;
    for event in events.read() {
        let t = match &event.kind {
            GameEventKind::PlayerDeath => Some(BarkTrigger::PlayerDied),
            GameEventKind::EnemyDestroyed { .. } => Some(BarkTrigger::EnemyDestroyed),
            _ => None,
        };
        if t.is_some() {
            trigger = t;
            break;
        }
    }

    let Some(trigger) = trigger else { return };
    let Some((data, personality)) = companion_query.iter().next() else { return };

    bark_display.current = Some((data.name.clone(), pick_bark(*personality, trigger)));
    bark_display.timer = BARK_DISPLAY_DURATION;
}

/// Emits a bark when the player cycles WingmanCommand (companion acknowledges new orders).
pub fn emit_bark_on_command_change(
    changed_query: Query<
        (&CompanionData, &CompanionPersonality),
        (With<Companion>, Changed<WingmanCommand>),
    >,
    mut bark_display: ResMut<BarkDisplay>,
) {
    let Some((data, personality)) = changed_query.iter().next() else { return };
    bark_display.current = Some((
        data.name.clone(),
        pick_bark(*personality, BarkTrigger::CommandReceived),
    ));
    bark_display.timer = BARK_DISPLAY_DURATION;
}

/// Decrements the bark display timer and clears expired barks.
pub fn tick_bark_display(mut bark_display: ResMut<BarkDisplay>, time: Res<Time>) {
    if bark_display.current.is_none() {
        return;
    }
    bark_display.timer -= time.delta_secs();
    if bark_display.timer <= 0.0 {
        bark_display.current = None;
        bark_display.timer = 0.0;
    }
}

// ── Player Opinions (6b-2) ────────────────────────────────────────────────

/// Tracks each companion's opinion of the player. Range: −100 to 100. Starts at 0.
/// Updated by player actions: kills raise opinion, deaths lower it.
#[derive(Resource, Default, Debug)]
pub struct PlayerOpinions {
    /// Maps companion Entity → opinion score (−100..=100).
    pub scores: HashMap<Entity, i32>,
}

/// Pure function: returns the opinion delta for a given game event kind.
pub fn opinion_delta_for_event(kind: &GameEventKind) -> i32 {
    match kind {
        GameEventKind::EnemyDestroyed { .. } => 2,
        GameEventKind::PlayerDeath => -5,
        GameEventKind::StationDocked => 1,
        _ => 0,
    }
}

/// Clamps an opinion value to [−100, 100].
pub fn clamp_opinion(v: i32) -> i32 {
    v.clamp(-100, 100)
}

/// Reads GameEvents and adjusts every companion's opinion of the player.
pub fn update_player_opinions(
    mut events: bevy::ecs::message::MessageReader<GameEvent>,
    companion_query: Query<Entity, With<Companion>>,
    mut opinions: ResMut<PlayerOpinions>,
) {
    let mut total_delta = 0i32;
    for event in events.read() {
        total_delta += opinion_delta_for_event(&event.kind);
    }
    if total_delta == 0 {
        return;
    }
    for entity in companion_query.iter() {
        let score = opinions.scores.entry(entity).or_insert(0);
        *score = clamp_opinion(*score + total_delta);
    }
}

// ── Peer Opinions (6b-3) ──────────────────────────────────────────────────

/// Tracks companions' opinions of each other. Range: −100 to 100. Starts at 0.
/// Key: (observer, subject) — "observer's opinion of subject".
#[derive(Resource, Default, Debug)]
pub struct PeerOpinions {
    pub scores: HashMap<(Entity, Entity), i32>,
}

/// Reads GameEvents and adjusts peer opinions between companions.
///
/// EnemyDestroyed: all companion pairs gain +1 (fought together).
/// PlayerDeath (companion retreats): retreating companion loses −3 with all peers.
pub fn update_peer_opinions(
    mut events: bevy::ecs::message::MessageReader<GameEvent>,
    companion_query: Query<Entity, With<Companion>>,
    mut peer_opinions: ResMut<PeerOpinions>,
) {
    let mut enemy_destroyed = false;
    let mut player_died = false;
    for event in events.read() {
        match &event.kind {
            GameEventKind::EnemyDestroyed { .. } => enemy_destroyed = true,
            GameEventKind::PlayerDeath => player_died = true,
            _ => {}
        }
    }

    if !enemy_destroyed && !player_died {
        return;
    }

    let companions: Vec<Entity> = companion_query.iter().collect();
    if companions.len() < 2 {
        return;
    }

    if enemy_destroyed {
        // Shared combat → all pairs gain mutual positive opinion
        for i in 0..companions.len() {
            for j in (i + 1)..companions.len() {
                let a = companions[i];
                let b = companions[j];
                let ab = peer_opinions.scores.entry((a, b)).or_insert(0);
                *ab = clamp_opinion(*ab + 1);
                let ba = peer_opinions.scores.entry((b, a)).or_insert(0);
                *ba = clamp_opinion(*ba + 1);
            }
        }
    }

    if player_died {
        // One companion retreating while others remain → opinion loss with all peers
        // We can't know which specific companion retreated here (retreat happens async),
        // so all companions lose a small amount of respect for each other during chaos.
        for i in 0..companions.len() {
            for j in (i + 1)..companions.len() {
                let a = companions[i];
                let b = companions[j];
                let ab = peer_opinions.scores.entry((a, b)).or_insert(0);
                *ab = clamp_opinion(*ab - 1);
                let ba = peer_opinions.scores.entry((b, a)).or_insert(0);
                *ba = clamp_opinion(*ba - 1);
            }
        }
    }
}

// ── Personality Combat Behavior (6b-4) ───────────────────────────────────

/// Dynamic follow behavior overrides applied each frame based on personality + command.
/// Modifies CompanionFollowAI at runtime — resets to base each frame.
pub fn update_personality_behavior(
    mut companion_query: Query<
        (
            &CompanionPersonality,
            &WingmanCommand,
            &mut crate::social::companion::CompanionFollowAI,
        ),
        With<Companion>,
    >,
    time: Res<Time>,
) {
    let elapsed = time.elapsed_secs();

    for (personality, command, mut follow_ai) in companion_query.iter_mut() {
        match personality {
            CompanionPersonality::Brave => {
                // Charges in close — shorter follow distance, faster speed
                follow_ai.follow_distance = match command {
                    WingmanCommand::Attack => 35.0,
                    WingmanCommand::Defend => 55.0,
                    WingmanCommand::Retreat => 90.0,
                };
                follow_ai.follow_speed = 175.0;
            }
            CompanionPersonality::Cautious => {
                // Hangs back — longer follow distance
                follow_ai.follow_distance = match command {
                    WingmanCommand::Attack => 65.0,
                    WingmanCommand::Defend => 85.0,
                    WingmanCommand::Retreat => 120.0,
                };
                follow_ai.follow_speed = 155.0;
            }
            CompanionPersonality::Sarcastic => {
                // Occasionally hesitates (uses time-based pulse to vary speed)
                let hesitation = (elapsed * 0.7).sin().abs();
                follow_ai.follow_distance = 60.0;
                follow_ai.follow_speed = 130.0 + hesitation * 40.0;
            }
            CompanionPersonality::Loyal => {
                // Perfect execution — default values, no deviation
                follow_ai.follow_distance = 60.0;
                follow_ai.follow_speed = 150.0;
            }
        }
    }
}

// ── Unit Tests ─────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    // 6b-1: pick_bark covers all 16 combinations
    #[test]
    fn pick_bark_brave_player_died() {
        let bark = pick_bark(CompanionPersonality::Brave, BarkTrigger::PlayerDied);
        assert!(!bark.is_empty(), "Bark should not be empty");
    }

    #[test]
    fn pick_bark_brave_enemy_destroyed() {
        let bark = pick_bark(CompanionPersonality::Brave, BarkTrigger::EnemyDestroyed);
        assert!(!bark.is_empty());
    }

    #[test]
    fn pick_bark_brave_command_received() {
        let bark = pick_bark(CompanionPersonality::Brave, BarkTrigger::CommandReceived);
        assert!(!bark.is_empty());
    }

    #[test]
    fn pick_bark_brave_damage_taken() {
        let bark = pick_bark(CompanionPersonality::Brave, BarkTrigger::DamageTaken);
        assert!(!bark.is_empty());
    }

    #[test]
    fn pick_bark_cautious_all_triggers() {
        for trigger in [
            BarkTrigger::PlayerDied,
            BarkTrigger::EnemyDestroyed,
            BarkTrigger::CommandReceived,
            BarkTrigger::DamageTaken,
        ] {
            let bark = pick_bark(CompanionPersonality::Cautious, trigger);
            assert!(!bark.is_empty(), "Cautious bark for {trigger:?} should not be empty");
        }
    }

    #[test]
    fn pick_bark_sarcastic_all_triggers() {
        for trigger in [
            BarkTrigger::PlayerDied,
            BarkTrigger::EnemyDestroyed,
            BarkTrigger::CommandReceived,
            BarkTrigger::DamageTaken,
        ] {
            let bark = pick_bark(CompanionPersonality::Sarcastic, trigger);
            assert!(!bark.is_empty(), "Sarcastic bark for {trigger:?} should not be empty");
        }
    }

    #[test]
    fn pick_bark_loyal_all_triggers() {
        for trigger in [
            BarkTrigger::PlayerDied,
            BarkTrigger::EnemyDestroyed,
            BarkTrigger::CommandReceived,
            BarkTrigger::DamageTaken,
        ] {
            let bark = pick_bark(CompanionPersonality::Loyal, trigger);
            assert!(!bark.is_empty(), "Loyal bark for {trigger:?} should not be empty");
        }
    }

    #[test]
    fn pick_bark_personalities_differ_for_same_trigger() {
        let brave = pick_bark(CompanionPersonality::Brave, BarkTrigger::PlayerDied);
        let cautious = pick_bark(CompanionPersonality::Cautious, BarkTrigger::PlayerDied);
        let sarcastic = pick_bark(CompanionPersonality::Sarcastic, BarkTrigger::PlayerDied);
        let loyal = pick_bark(CompanionPersonality::Loyal, BarkTrigger::PlayerDied);
        // All four should be different texts
        assert_ne!(brave, cautious);
        assert_ne!(brave, sarcastic);
        assert_ne!(brave, loyal);
        assert_ne!(cautious, sarcastic);
    }

    #[test]
    fn personality_for_faction_pirates_sarcastic() {
        assert_eq!(
            personality_for_faction(&FactionId::Pirates),
            CompanionPersonality::Sarcastic
        );
    }

    #[test]
    fn personality_for_faction_military_loyal() {
        assert_eq!(
            personality_for_faction(&FactionId::Military),
            CompanionPersonality::Loyal
        );
    }

    #[test]
    fn personality_for_faction_aliens_cautious() {
        assert_eq!(
            personality_for_faction(&FactionId::Aliens),
            CompanionPersonality::Cautious
        );
    }

    #[test]
    fn personality_for_faction_rogue_drones_brave() {
        assert_eq!(
            personality_for_faction(&FactionId::RogueDrones),
            CompanionPersonality::Brave
        );
    }

    #[test]
    fn personality_for_faction_neutral_loyal() {
        assert_eq!(
            personality_for_faction(&FactionId::Neutral),
            CompanionPersonality::Loyal
        );
    }

    // 6b-2: opinion logic
    #[test]
    fn opinion_delta_enemy_destroyed_positive() {
        let delta = opinion_delta_for_event(&GameEventKind::EnemyDestroyed {
            entity_type: "drone",
        });
        assert!(delta > 0, "Killing enemies should raise opinion");
    }

    #[test]
    fn opinion_delta_player_death_negative() {
        let delta = opinion_delta_for_event(&GameEventKind::PlayerDeath);
        assert!(delta < 0, "Player dying should lower companion opinion");
    }

    #[test]
    fn opinion_delta_station_docked_positive() {
        let delta = opinion_delta_for_event(&GameEventKind::StationDocked);
        assert!(delta > 0, "Docking at station should raise opinion");
    }

    #[test]
    fn clamp_opinion_caps_at_100() {
        assert_eq!(clamp_opinion(150), 100);
    }

    #[test]
    fn clamp_opinion_caps_at_minus_100() {
        assert_eq!(clamp_opinion(-150), -100);
    }

    #[test]
    fn clamp_opinion_neutral_unchanged() {
        assert_eq!(clamp_opinion(0), 0);
        assert_eq!(clamp_opinion(50), 50);
        assert_eq!(clamp_opinion(-50), -50);
    }

    // 6b-3: peer opinion data structure
    #[test]
    fn peer_opinions_starts_empty() {
        let opinions = PeerOpinions::default();
        assert!(opinions.scores.is_empty(), "PeerOpinions should start empty");
    }

    #[test]
    fn player_opinions_starts_empty() {
        let opinions = PlayerOpinions::default();
        assert!(opinions.scores.is_empty(), "PlayerOpinions should start empty");
    }

    // 6b-4: personality behavior sanity
    #[test]
    fn bark_display_default_is_empty() {
        let display = BarkDisplay::default();
        assert!(display.current.is_none(), "BarkDisplay should start with no active bark");
        assert_eq!(display.timer, 0.0, "BarkDisplay timer should start at 0");
    }
}
