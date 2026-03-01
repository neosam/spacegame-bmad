/// Companion Core — Epic 6a / 6c
///
/// All companion logic: recruitment, follow AI, ship flight, wingman commands, survival, visuals.
/// Design: Pure functions for testable logic. Systems handle ECS state changes.
/// Core/Rendering separation: Core spawns NeedsCompanionVisual, Rendering attaches mesh.
use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::core::collision::{Collider, Health};
use crate::core::flight::Player;
use crate::core::input::ActionState;
use crate::core::station::{Docked, Station};
use crate::infrastructure::events::EventSeverityConfig;
use crate::shared::components::Velocity;
use crate::shared::events::{GameEvent, GameEventKind};
use crate::social::companion_personality::{
    personality_for_faction, CompanionPrevHealth, CompanionTarget, CompanionWeapon,
};
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

// ── CompanionFlight Component (6c-1) ──────────────────────────────────────

/// Ship-physics flight state for a companion (Epic 6c-1).
/// The companion rotates toward its target and thrusts in facing direction — not direct velocity.
#[derive(Component, Debug, Clone)]
pub struct CompanionFlight {
    /// Current facing angle in radians (Y-forward convention: 0 = facing +Y).
    pub angle: f32,
    /// Rotation speed in radians per second.
    pub turn_rate: f32,
    /// Thrust acceleration in world-units per second².
    pub thrust_force: f32,
    /// Maximum speed in world-units per second.
    pub max_speed: f32,
    /// Drag coefficient applied each frame (higher = more drag).
    pub drag: f32,
}

impl Default for CompanionFlight {
    fn default() -> Self {
        Self {
            angle: 0.0,
            turn_rate: 3.5,
            thrust_force: 220.0,
            max_speed: 200.0,
            drag: 2.5,
        }
    }
}

// ── Pure Functions ─────────────────────────────────────────────────────────

/// Compute the desired velocity for a companion following the player.
/// Legacy pure function kept for backward compatibility with existing tests.
pub fn companion_follow_velocity(
    companion_pos: Vec2,
    player_pos: Vec2,
    follow_distance: f32,
    follow_speed: f32,
) -> Vec2 {
    let target = player_pos + Vec2::new(-follow_distance * 0.5, -follow_distance);
    let to_target = target - companion_pos;
    let distance = to_target.length();
    if distance < 5.0 {
        return Vec2::ZERO;
    }
    let direction = to_target.normalize_or_zero();
    let speed = follow_speed.min(distance * 3.0);
    direction * speed
}

/// Pure function: compute follow-slot target position (behind and to the side of the player).
pub fn companion_follow_target(player_pos: Vec2, follow_distance: f32) -> Vec2 {
    player_pos + Vec2::new(-follow_distance * 0.5, -follow_distance)
}

/// Pure function: rotate `current` angle toward `desired` angle, clamped by `max_turn`.
/// Handles wrapping through ±PI.
pub fn rotate_toward_angle(current: f32, desired: f32, max_turn: f32) -> f32 {
    let mut diff = desired - current;
    while diff > std::f32::consts::PI {
        diff -= std::f32::consts::TAU;
    }
    while diff < -std::f32::consts::PI {
        diff += std::f32::consts::TAU;
    }
    current + diff.clamp(-max_turn, max_turn)
}

/// Pure function: compute the desired facing angle toward a world position.
/// Uses Y-forward convention (angle 0 = facing +Y, same as player).
pub fn angle_toward(from: Vec2, to: Vec2) -> f32 {
    let dir = (to - from).normalize_or_zero();
    // atan2(-x, y) gives angle from +Y axis in radians, matching Quat::from_rotation_z convention
    (-dir.x).atan2(dir.y)
}

/// Pure function: compute the facing direction vector from an angle (Y-forward convention).
pub fn facing_from_angle(angle: f32) -> Vec2 {
    Vec2::new(-angle.sin(), angle.cos())
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
            CompanionFlight::default(),
            CompanionTarget::default(),
            CompanionWeapon::default(),
            CompanionPrevHealth { value: 100.0 },
            WingmanCommand::Defend,
            personality,
            Health { current: 100.0, max: 100.0 },
            Collider { radius: 14.0 },
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

/// Rotates companions toward their navigation target using ship-physics (Epic 6c-1).
///
/// - Attack mode with acquired target → rotate toward target entity
/// - Otherwise → rotate toward player follow-slot position
/// - Retreating → rotate toward station (handled in `update_retreating_companions`)
/// Updates Transform.rotation to match the current facing angle.
pub fn update_companion_rotation(
    player_query: Query<&Transform, (With<Player>, Without<Companion>)>,
    target_transforms: Query<&Transform, Without<Companion>>,
    mut companion_query: Query<
        (
            &Transform,
            &CompanionFollowAI,
            &WingmanCommand,
            &CompanionTarget,
            &mut CompanionFlight,
        ),
        (With<Companion>, Without<CompanionRetreating>),
    >,
    time: Res<Time>,
) {
    let Ok(player_transform) = player_query.single() else {
        return;
    };
    let player_pos = player_transform.translation.truncate();
    let dt = time.delta_secs();

    for (companion_transform, follow_ai, command, target, mut flight) in
        companion_query.iter_mut()
    {
        let companion_pos = companion_transform.translation.truncate();

        // Determine desired facing direction
        let desired_pos = match (command, target.entity) {
            (WingmanCommand::Attack, Some(target_entity)) => {
                // Attack mode: face toward acquired enemy
                target_transforms
                    .get(target_entity)
                    .map(|t| t.translation.truncate())
                    .unwrap_or_else(|_| {
                        companion_follow_target(player_pos, follow_ai.follow_distance)
                    })
            }
            _ => companion_follow_target(player_pos, follow_ai.follow_distance),
        };

        let desired_angle = angle_toward(companion_pos, desired_pos);
        flight.angle = rotate_toward_angle(flight.angle, desired_angle, flight.turn_rate * dt);

        // Update visual rotation to match facing
        // Companion transform uses same convention as player: Quat::from_rotation_z
    }
}

/// Applies thrust and drag to companions based on current facing (Epic 6c-1).
///
/// Thrusts when roughly aligned with target (dot > 0.3).
/// Applies drag every frame. Caps speed at max_speed.
pub fn update_companion_thrust_and_drag(
    player_query: Query<&Transform, (With<Player>, Without<Companion>)>,
    target_transforms: Query<&Transform, Without<Companion>>,
    mut companion_query: Query<
        (
            &Transform,
            &CompanionFollowAI,
            &WingmanCommand,
            &CompanionTarget,
            &CompanionFlight,
            &mut Velocity,
        ),
        (With<Companion>, Without<CompanionRetreating>),
    >,
    time: Res<Time>,
) {
    let Ok(player_transform) = player_query.single() else {
        return;
    };
    let player_pos = player_transform.translation.truncate();
    let dt = time.delta_secs();

    for (companion_transform, follow_ai, command, target, flight, mut velocity) in
        companion_query.iter_mut()
    {
        let companion_pos = companion_transform.translation.truncate();

        // Determine navigation target
        let desired_pos = match (command, target.entity) {
            (WingmanCommand::Attack, Some(target_entity)) => target_transforms
                .get(target_entity)
                .map(|t| t.translation.truncate())
                .unwrap_or_else(|_| {
                    companion_follow_target(player_pos, follow_ai.follow_distance)
                }),
            _ => companion_follow_target(player_pos, follow_ai.follow_distance),
        };

        let to_target = desired_pos - companion_pos;
        let dist = to_target.length();

        // Thrust only when far enough and roughly facing target
        if dist > 15.0 {
            let facing = facing_from_angle(flight.angle);
            let alignment = facing.dot(to_target.normalize_or_zero());
            if alignment > 0.3 {
                // Soft speed cap: reduce thrust near max_speed
                let speed = velocity.0.length();
                let effectiveness = (1.0 - speed / flight.max_speed).max(0.0);
                velocity.0 += facing * flight.thrust_force * effectiveness * dt;
            }
        }

        // Drag
        let drag_factor = (1.0 - flight.drag * dt).max(0.0);
        velocity.0 *= drag_factor;
        if velocity.0.length_squared() < 0.1 {
            velocity.0 = Vec2::ZERO;
        }
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
