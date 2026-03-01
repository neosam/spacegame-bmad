/// Companion Core — Epic 6a
///
/// All companion logic: recruitment, follow AI, wingman commands, survival, and visual markers.
/// Design: Pure functions for testable logic. Systems handle ECS state changes.
/// Core/Rendering separation: Core spawns NeedsCompanionVisual, Rendering attaches mesh.
use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::core::flight::Player;
use crate::core::input::ActionState;
use crate::core::station::{Docked, Station};
use crate::infrastructure::events::EventSeverityConfig;
use crate::shared::components::Velocity;
use crate::shared::events::{GameEvent, GameEventKind};
use crate::social::companion_personality::personality_for_faction;
use crate::social::faction::FactionId;

// ── Components ────────────────────────────────────────────────────────────

/// Marker component for companion entities.
#[derive(Component, Debug, Clone)]
pub struct Companion;

/// Identity and faction of a companion entity.
#[derive(Component, Debug, Clone)]
pub struct CompanionData {
    pub name: String,
    pub faction: FactionId,
}

/// Companion follow AI configuration.
#[derive(Component, Debug, Clone)]
pub struct CompanionFollowAI {
    /// World-units per second movement speed.
    pub follow_speed: f32,
    /// How far the companion tries to stay from the player (world units).
    pub follow_distance: f32,
}

impl Default for CompanionFollowAI {
    fn default() -> Self {
        Self {
            follow_speed: 150.0,
            follow_distance: 60.0,
        }
    }
}

/// Wingman command issued by the player.
/// Cycles: Attack → Defend → Retreat → Attack.
#[derive(Component, Debug, Clone, PartialEq)]
pub enum WingmanCommand {
    Attack,
    Defend,
    Retreat,
}

impl WingmanCommand {
    /// Returns the next command in the cycle.
    pub fn next(&self) -> Self {
        match self {
            WingmanCommand::Attack => WingmanCommand::Defend,
            WingmanCommand::Defend => WingmanCommand::Retreat,
            WingmanCommand::Retreat => WingmanCommand::Attack,
        }
    }
}

/// Marker: companion needs its visual mesh attached by the RenderingPlugin.
/// Core never touches rendering — RenderingPlugin reads this and removes it.
#[derive(Component, Debug)]
pub struct NeedsCompanionVisual;

/// Companion is retreating toward a station after player death.
/// Removed when the companion arrives at the station.
#[derive(Component, Debug, Clone)]
pub struct CompanionRetreating {
    /// World position to retreat toward (nearest station).
    pub target: Vec2,
}

// ── Resources ─────────────────────────────────────────────────────────────

/// Tracks all active companion entities in the world.
#[derive(Resource, Default, Debug)]
pub struct CompanionRoster {
    pub companions: Vec<Entity>,
}

// ── Save Types ────────────────────────────────────────────────────────────

/// Serializable snapshot of a single companion.
/// Stored inside `PlayerSave.companions`.
#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct CompanionSaveEntry {
    pub name: String,
    /// String representation of FactionId for forwards-compatible serialization.
    pub faction: String,
    pub x: f32,
    pub y: f32,
}

impl CompanionSaveEntry {
    /// Create from companion components.
    pub fn from_components(data: &CompanionData, transform: &Transform) -> Self {
        Self {
            name: data.name.clone(),
            faction: faction_id_to_str(&data.faction).to_string(),
            x: transform.translation.x,
            y: transform.translation.y,
        }
    }
}

/// Convert FactionId to a stable string key for serialization.
pub fn faction_id_to_str(faction: &FactionId) -> &'static str {
    match faction {
        FactionId::Pirates => "Pirates",
        FactionId::Military => "Military",
        FactionId::Aliens => "Aliens",
        FactionId::RogueDrones => "RogueDrones",
        FactionId::Neutral => "Neutral",
    }
}

/// Convert string key back to FactionId (defaults to Neutral on unknown).
pub fn str_to_faction_id(s: &str) -> FactionId {
    match s {
        "Pirates" => FactionId::Pirates,
        "Military" => FactionId::Military,
        "Aliens" => FactionId::Aliens,
        "RogueDrones" => FactionId::RogueDrones,
        _ => FactionId::Neutral,
    }
}

// ── Pure Functions ─────────────────────────────────────────────────────────

/// Compute the desired velocity for a companion following the player.
/// Pure function — no ECS access, fully testable.
///
/// The companion tries to stay at `player_pos + lateral_offset + behind_offset`.
/// Returns the desired velocity vector (absolute, not a delta).
pub fn companion_follow_velocity(
    companion_pos: Vec2,
    player_pos: Vec2,
    follow_distance: f32,
    follow_speed: f32,
) -> Vec2 {
    // Target: behind and slightly to the side of the player
    let target = player_pos + Vec2::new(-follow_distance * 0.5, -follow_distance);
    let to_target = target - companion_pos;
    let distance = to_target.length();

    if distance < 5.0 {
        // Already at target — no movement
        return Vec2::ZERO;
    }

    // Proportional speed: full speed at distance >= follow_speed, ramp down when close
    let direction = to_target.normalize_or_zero();
    let speed = follow_speed.min(distance * 3.0);
    direction * speed
}

// ── Systems ────────────────────────────────────────────────────────────────

/// Handles companion recruitment when the player is docked and presses the recruit action.
///
/// Spawns a companion entity near the player with Companion + CompanionData + NeedsCompanionVisual.
/// Emits `GameEventKind::CompanionRecruited` on success.
pub fn handle_recruit_companion(
    mut action_state: ResMut<ActionState>,
    player_query: Query<(&Transform, &Docked), With<Player>>,
    mut roster: ResMut<CompanionRoster>,
    mut commands: Commands,
    mut game_events: bevy::ecs::message::MessageWriter<GameEvent>,
    time: Res<Time>,
    severity_config: Res<EventSeverityConfig>,
) {
    if !action_state.recruit {
        return;
    }
    // Consume recruit so it doesn't repeat next frame
    action_state.recruit = false;

    let Ok((player_transform, _docked)) = player_query.single() else {
        return;
    };

    let companion_pos = player_transform.translation.truncate() + Vec2::new(35.0, 0.0);
    let name = format!("Wing-{}", roster.companions.len() + 1);
    let faction = FactionId::Neutral;

    let personality = personality_for_faction(&faction);
    let entity = commands
        .spawn((
            Companion,
            CompanionData {
                name: name.clone(),
                faction: faction.clone(),
            },
            CompanionFollowAI::default(),
            WingmanCommand::Defend,
            personality,
            NeedsCompanionVisual,
            Velocity::default(),
            Transform::from_translation(companion_pos.extend(0.0)),
        ))
        .id();

    roster.companions.push(entity);

    let kind = GameEventKind::CompanionRecruited { name };
    game_events.write(GameEvent {
        severity: severity_config.severity_for(&kind),
        kind,
        position: companion_pos,
        game_time: time.elapsed_secs_f64(),
    });
}

/// Moves companions toward the player using follow AI.
/// Companions with `CompanionRetreating` are excluded (handled separately).
pub fn update_companion_follow(
    player_query: Query<&Transform, (With<Player>, Without<Companion>)>,
    mut companion_query: Query<
        (&mut Velocity, &Transform, &CompanionFollowAI),
        (With<Companion>, Without<CompanionRetreating>),
    >,
    time: Res<Time>,
) {
    let Ok(player_transform) = player_query.single() else {
        return;
    };
    let player_pos = player_transform.translation.truncate();
    let dt = time.delta_secs();

    for (mut velocity, companion_transform, follow_ai) in companion_query.iter_mut() {
        let companion_pos = companion_transform.translation.truncate();
        let target_vel = companion_follow_velocity(
            companion_pos,
            player_pos,
            follow_ai.follow_distance,
            follow_ai.follow_speed,
        );
        // Smooth blend toward desired velocity
        velocity.0 = velocity.0.lerp(target_vel, (dt * 5.0).min(1.0));
    }
}

/// Applies velocity to companion entities (position integration).
/// Separate from Player's apply_velocity (which includes thrust/drag).
pub fn update_companion_positions(
    time: Res<Time>,
    mut query: Query<(&Velocity, &mut Transform), With<Companion>>,
) {
    let dt = time.delta_secs();
    for (velocity, mut transform) in query.iter_mut() {
        transform.translation.x += velocity.0.x * dt;
        transform.translation.y += velocity.0.y * dt;
    }
}

/// Cycles wingman commands on all companions when the player presses the command button.
pub fn handle_wingman_commands(
    action_state: Res<ActionState>,
    mut companion_query: Query<&mut WingmanCommand, With<Companion>>,
) {
    if !action_state.wingman_command {
        return;
    }
    for mut command in companion_query.iter_mut() {
        *command = command.next();
    }
}

/// Responds to PlayerDeath event by setting all companions to retreat toward the nearest station.
pub fn handle_companion_survival(
    mut events: bevy::ecs::message::MessageReader<GameEvent>,
    companion_query: Query<Entity, (With<Companion>, Without<CompanionRetreating>)>,
    station_query: Query<&Transform, With<Station>>,
    companion_transforms: Query<&Transform, With<Companion>>,
    mut commands: Commands,
) {
    let mut player_died = false;
    for event in events.read() {
        if matches!(event.kind, GameEventKind::PlayerDeath) {
            player_died = true;
        }
    }

    if !player_died {
        return;
    }

    for entity in companion_query.iter() {
        let companion_pos = companion_transforms
            .get(entity)
            .map(|t| t.translation.truncate())
            .unwrap_or(Vec2::ZERO);

        // Find nearest station
        let target = station_query
            .iter()
            .map(|t| t.translation.truncate())
            .min_by_key(|pos| (pos.distance(companion_pos) * 10.0) as i64)
            .unwrap_or(Vec2::ZERO);

        commands.entity(entity).insert(CompanionRetreating { target });
    }
}

/// Moves retreating companions toward their station target.
/// Removes `CompanionRetreating` when the companion arrives.
pub fn update_retreating_companions(
    mut companion_query: Query<
        (Entity, &mut Velocity, &Transform, &CompanionRetreating),
        With<Companion>,
    >,
    mut commands: Commands,
    time: Res<Time>,
) {
    let dt = time.delta_secs();
    for (entity, mut velocity, transform, retreating) in companion_query.iter_mut() {
        let companion_pos = transform.translation.truncate();
        let to_target = retreating.target - companion_pos;
        let distance = to_target.length();

        if distance < 25.0 {
            // Arrived — stop and wait at station
            velocity.0 = Vec2::ZERO;
            commands.entity(entity).remove::<CompanionRetreating>();
        } else {
            let direction = to_target.normalize_or_zero();
            let speed = 180.0_f32.min(distance * 2.5);
            velocity.0 = velocity.0.lerp(direction * speed, (dt * 5.0).min(1.0));
        }
    }
}

// ── Unit Tests ─────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wingman_command_cycles_attack_to_defend() {
        assert_eq!(WingmanCommand::Attack.next(), WingmanCommand::Defend);
    }

    #[test]
    fn wingman_command_cycles_defend_to_retreat() {
        assert_eq!(WingmanCommand::Defend.next(), WingmanCommand::Retreat);
    }

    #[test]
    fn wingman_command_cycles_retreat_to_attack() {
        assert_eq!(WingmanCommand::Retreat.next(), WingmanCommand::Attack);
    }

    #[test]
    fn companion_follow_velocity_moves_toward_player() {
        let companion_pos = Vec2::new(0.0, 0.0);
        let player_pos = Vec2::new(300.0, 0.0);
        let vel = companion_follow_velocity(companion_pos, player_pos, 60.0, 150.0);
        // Target is at (270.0, -60.0), which has positive x from origin
        assert!(vel.x > 0.0, "Should have positive x component toward player: {vel:?}");
    }

    #[test]
    fn companion_follow_velocity_stops_when_at_target() {
        // When companion is exactly at the computed target position, velocity should be ~0
        let player_pos = Vec2::new(200.0, 0.0);
        // target = player_pos + Vec2::new(-30.0, -60.0) = Vec2::new(170.0, -60.0)
        let companion_pos = Vec2::new(170.0, -60.0);
        let vel = companion_follow_velocity(companion_pos, player_pos, 60.0, 150.0);
        assert!(vel.length() < 5.0, "Should return near-zero velocity at target: {vel:?}");
    }

    #[test]
    fn companion_follow_velocity_caps_at_follow_speed() {
        let companion_pos = Vec2::ZERO;
        let player_pos = Vec2::new(10_000.0, 0.0);
        let vel = companion_follow_velocity(companion_pos, player_pos, 60.0, 150.0);
        assert!(
            vel.length() <= 150.0 + f32::EPSILON,
            "Speed should not exceed follow_speed: {}",
            vel.length()
        );
    }

    #[test]
    fn companion_roster_starts_empty() {
        let roster = CompanionRoster::default();
        assert!(roster.companions.is_empty(), "Roster should start empty");
    }

    #[test]
    fn needs_companion_visual_is_a_component() {
        // Compile-time check: NeedsCompanionVisual implements Component
        let _marker = NeedsCompanionVisual;
    }

    #[test]
    fn companion_save_entry_round_trip() {
        let entry = CompanionSaveEntry {
            name: "Wing-1".to_string(),
            faction: "Neutral".to_string(),
            x: 100.0,
            y: -50.0,
        };
        let ron_str =
            ron::ser::to_string(&entry).expect("Should serialize CompanionSaveEntry to RON");
        let restored: CompanionSaveEntry =
            ron::from_str(&ron_str).expect("Should deserialize CompanionSaveEntry from RON");
        assert_eq!(restored.name, "Wing-1");
        assert_eq!(restored.faction, "Neutral");
        assert!((restored.x - 100.0).abs() < f32::EPSILON);
    }

    #[test]
    fn faction_id_string_roundtrip() {
        for faction in [
            FactionId::Pirates,
            FactionId::Military,
            FactionId::Aliens,
            FactionId::RogueDrones,
            FactionId::Neutral,
        ] {
            let s = faction_id_to_str(&faction);
            let restored = str_to_faction_id(s);
            assert_eq!(faction, restored, "FactionId roundtrip failed for {s}");
        }
    }
}
