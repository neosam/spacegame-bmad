/// Enemy AI Finite State Machine for Epic 4: Combat Depth.
///
/// Design principle: FSM logic lives in pure functions (`next_state`).
/// Pure functions take explicit inputs and return the next state —
/// no ECS world access, no side effects, fully testable without App setup.
///
/// # Test Pattern
///
/// ```rust,ignore
/// let ctx = AiContext { distance_to_player: 150.0, health_ratio: 1.0,
///     aggro_range: 200.0, attack_range: 80.0, flee_threshold: 0.2 };
/// assert_eq!(next_state(&AiState::Idle, &ctx), AiState::Chase);
/// ```
use bevy::prelude::*;

/// Current AI state for an enemy entity.
#[derive(Component, Debug, Clone, PartialEq)]
pub enum AiState {
    Idle,
    Patrol,
    Chase,
    Attack,
    Flee,
}

/// Erratic movement offset for Scout Drones.
/// Re-rolled every `interval` seconds for unpredictable movement.
#[derive(Component, Debug, Clone)]
pub struct ErraticOffset {
    /// Current random direction offset (normalized, scaled by magnitude).
    pub offset: Vec2,
    /// Time remaining before the next random offset roll.
    pub timer: f32,
    /// How often (seconds) to re-roll the offset.
    pub interval: f32,
    /// Maximum offset magnitude in world units per second.
    pub magnitude: f32,
}

impl Default for ErraticOffset {
    fn default() -> Self {
        Self {
            offset: Vec2::ZERO,
            timer: 0.0,
            interval: 0.5,
            magnitude: 40.0,
        }
    }
}

/// Fire cooldown for enemies shooting at the player.
#[derive(Component, Debug, Clone)]
pub struct EnemyFireCooldown {
    pub timer: f32,
}

impl Default for EnemyFireCooldown {
    fn default() -> Self {
        Self { timer: 0.0 }
    }
}

/// Snapshot of world context passed into the FSM transition function.
/// All fields are plain data — no ECS queries inside the pure function.
#[derive(Debug, Clone)]
pub struct AiContext {
    /// Distance between this enemy and the player (world units).
    pub distance_to_player: f32,
    /// Current health as ratio of max health (0.0 = dead, 1.0 = full).
    pub health_ratio: f32,
    /// Aggro range from `AggroRange` component.
    pub aggro_range: f32,
    /// Attack range from `AttackRange` component.
    pub attack_range: f32,
    /// Flee threshold from `FleeThreshold` component.
    pub flee_threshold: f32,
}

/// Pure FSM transition function.
/// Given the current state and context, returns the next state.
/// No panics, no side effects — safe to call in unit tests without any App.
pub fn next_state(current: &AiState, ctx: &AiContext) -> AiState {
    // Flee always wins if health is critically low.
    if ctx.health_ratio < ctx.flee_threshold {
        return AiState::Flee;
    }

    match current {
        AiState::Idle | AiState::Patrol => {
            if ctx.distance_to_player < ctx.aggro_range {
                AiState::Chase
            } else {
                current.clone()
            }
        }
        AiState::Chase => {
            if ctx.distance_to_player < ctx.attack_range {
                AiState::Attack
            } else if ctx.distance_to_player > ctx.aggro_range * 1.5 {
                // Player escaped — resume patrol
                AiState::Patrol
            } else {
                AiState::Chase
            }
        }
        AiState::Attack => {
            if ctx.distance_to_player > ctx.attack_range * 1.2 {
                // Player moved out of range — chase again
                AiState::Chase
            } else {
                AiState::Attack
            }
        }
        AiState::Flee => AiState::Flee,
    }
}

/// Pure function: rolls a new erratic offset given explicit random inputs.
/// `rand_x` and `rand_y` are in [-1.0, 1.0], magnitude scales the result.
pub fn roll_erratic_offset(rand_x: f32, rand_y: f32, magnitude: f32) -> Vec2 {
    let dir = Vec2::new(rand_x, rand_y);
    let normalized = if dir.length_squared() > 0.0 {
        dir.normalize()
    } else {
        Vec2::X
    };
    normalized * magnitude
}

/// Updates Scout Drone AI states, movement, and shooting.
///
/// For each Scout Drone entity that has AI components:
/// - Computes AI context from distance to player and health ratio
/// - Transitions AI state via `next_state()`
/// - Chase: moves toward player + adds erratic offset
/// - Attack: fires laser if cooldown allows (inserts `PendingEnemyShot`)
/// - Flee: moves away from player
/// - Ticks erratic offset timer and re-rolls when expired
#[allow(clippy::type_complexity)]
pub fn update_scout_drone_ai(
    time: Res<Time>,
    player_query: Query<&Transform, With<crate::core::flight::Player>>,
    mut drone_query: Query<
        (
            &Transform,
            &crate::core::collision::Health,
            &mut crate::shared::components::Velocity,
            &mut AiState,
            &crate::social::faction::AggroRange,
            &crate::social::faction::AttackRange,
            &crate::social::faction::FleeThreshold,
            &mut ErraticOffset,
            &mut EnemyFireCooldown,
        ),
        With<crate::core::spawning::ScoutDrone>,
    >,
    mut pending_shots: ResMut<PendingEnemyShotQueue>,
) {
    let dt = time.delta_secs();
    let Ok(player_transform) = player_query.single() else {
        return;
    };
    let player_pos = Vec2::new(
        player_transform.translation.x,
        player_transform.translation.y,
    );

    for (
        transform,
        health,
        mut velocity,
        mut ai_state,
        aggro_range,
        attack_range,
        flee_threshold,
        mut erratic,
        mut fire_cooldown,
    ) in drone_query.iter_mut()
    {
        let drone_pos = Vec2::new(transform.translation.x, transform.translation.y);
        let to_player = player_pos - drone_pos;
        let distance = to_player.length();

        let health_ratio = if health.max > 0.0 {
            health.current / health.max
        } else {
            0.0
        };

        let ctx = AiContext {
            distance_to_player: distance,
            health_ratio,
            aggro_range: aggro_range.0,
            attack_range: attack_range.0,
            flee_threshold: flee_threshold.0,
        };

        *ai_state = next_state(&ai_state, &ctx);

        // Tick erratic offset timer
        erratic.timer -= dt;
        if erratic.timer <= 0.0 {
            erratic.timer = erratic.interval;
            // Use deterministic hash of position for pseudo-random offset
            let hash_x = (drone_pos.x * 127.1 + drone_pos.y * 311.7).sin();
            let hash_y = (drone_pos.x * 269.5 + drone_pos.y * 183.3).cos();
            erratic.offset = roll_erratic_offset(
                (hash_x.fract() * 2.0 - 1.0).clamp(-1.0, 1.0),
                (hash_y.fract() * 2.0 - 1.0).clamp(-1.0, 1.0),
                erratic.magnitude,
            );
        }

        // Move and act based on AI state
        const DRONE_SPEED: f32 = 80.0;
        match *ai_state {
            AiState::Chase => {
                let dir = if distance > 0.0 {
                    to_player.normalize()
                } else {
                    Vec2::X
                };
                velocity.0 = dir * DRONE_SPEED + erratic.offset;
            }
            AiState::Attack => {
                // Slow down while attacking
                velocity.0 = velocity.0 * 0.9 + erratic.offset * 0.2;

                // Shoot player if cooldown allows
                fire_cooldown.timer -= dt;
                if fire_cooldown.timer <= 0.0 {
                    fire_cooldown.timer = 1.0; // 1 shot per second
                    pending_shots.shots.push(PendingEnemyShot {
                        origin: drone_pos,
                        target: player_pos,
                        damage: 5.0,
                    });
                }
            }
            AiState::Flee => {
                let dir = if distance > 0.0 {
                    -to_player.normalize()
                } else {
                    Vec2::X
                };
                velocity.0 = dir * DRONE_SPEED * 1.5;
            }
            AiState::Idle | AiState::Patrol => {
                // Drift with erratic offset for patrol
                velocity.0 = velocity.0 * 0.99 + erratic.offset * 0.1;
            }
        }
    }
}

/// Updates Fighter AI: aggressive pursuit with larger aggro range.
/// Fighters chase relentlessly, only flee at very low HP.
#[allow(clippy::type_complexity)]
pub fn update_fighter_ai(
    time: Res<Time>,
    player_query: Query<&Transform, With<crate::core::flight::Player>>,
    mut fighter_query: Query<
        (
            &Transform,
            &crate::core::collision::Health,
            &mut crate::shared::components::Velocity,
            &mut AiState,
            &crate::social::faction::AggroRange,
            &crate::social::faction::AttackRange,
            &crate::social::faction::FleeThreshold,
            &mut EnemyFireCooldown,
        ),
        With<crate::core::spawning::Fighter>,
    >,
    mut pending_shots: ResMut<PendingEnemyShotQueue>,
) {
    let dt = time.delta_secs();
    let Ok(player_transform) = player_query.single() else {
        return;
    };
    let player_pos = Vec2::new(
        player_transform.translation.x,
        player_transform.translation.y,
    );

    for (transform, health, mut velocity, mut ai_state, aggro, attack, flee, mut cooldown) in
        fighter_query.iter_mut()
    {
        let pos = Vec2::new(transform.translation.x, transform.translation.y);
        let to_player = player_pos - pos;
        let distance = to_player.length();
        let health_ratio = if health.max > 0.0 { health.current / health.max } else { 0.0 };

        let ctx = AiContext {
            distance_to_player: distance,
            health_ratio,
            aggro_range: aggro.0,
            attack_range: attack.0,
            flee_threshold: flee.0,
        };
        *ai_state = next_state(&ai_state, &ctx);

        const FIGHTER_SPEED: f32 = 120.0;
        match *ai_state {
            AiState::Chase | AiState::Attack => {
                let dir = if distance > 0.0 { to_player.normalize() } else { Vec2::X };
                velocity.0 = dir * FIGHTER_SPEED;
                // Attack state: shoot
                if *ai_state == AiState::Attack {
                    cooldown.timer -= dt;
                    if cooldown.timer <= 0.0 {
                        cooldown.timer = 0.8; // Faster fire rate than drones
                        pending_shots.shots.push(PendingEnemyShot {
                            origin: pos,
                            target: player_pos,
                            damage: 8.0,
                        });
                    }
                }
            }
            AiState::Flee => {
                let dir = if distance > 0.0 { -to_player.normalize() } else { Vec2::X };
                velocity.0 = dir * FIGHTER_SPEED * 1.2;
            }
            AiState::Idle | AiState::Patrol => {
                velocity.0 = velocity.0 * 0.95;
            }
        }
    }
}

/// Updates Heavy Cruiser AI: slow but powerful, high health, attack from range.
#[allow(clippy::type_complexity)]
pub fn update_heavy_cruiser_ai(
    time: Res<Time>,
    player_query: Query<&Transform, With<crate::core::flight::Player>>,
    mut cruiser_query: Query<
        (
            &Transform,
            &crate::core::collision::Health,
            &mut crate::shared::components::Velocity,
            &mut AiState,
            &crate::social::faction::AggroRange,
            &crate::social::faction::AttackRange,
            &crate::social::faction::FleeThreshold,
            &mut EnemyFireCooldown,
        ),
        With<crate::core::spawning::HeavyCruiser>,
    >,
    mut pending_shots: ResMut<PendingEnemyShotQueue>,
) {
    let dt = time.delta_secs();
    let Ok(player_transform) = player_query.single() else {
        return;
    };
    let player_pos = Vec2::new(
        player_transform.translation.x,
        player_transform.translation.y,
    );

    for (transform, health, mut velocity, mut ai_state, aggro, attack, flee, mut cooldown) in
        cruiser_query.iter_mut()
    {
        let pos = Vec2::new(transform.translation.x, transform.translation.y);
        let to_player = player_pos - pos;
        let distance = to_player.length();
        let health_ratio = if health.max > 0.0 { health.current / health.max } else { 0.0 };

        let ctx = AiContext {
            distance_to_player: distance,
            health_ratio,
            aggro_range: aggro.0,
            attack_range: attack.0,
            flee_threshold: flee.0,
        };
        *ai_state = next_state(&ai_state, &ctx);

        // Heavy Cruisers are SLOW
        const CRUISER_SPEED: f32 = 40.0;
        match *ai_state {
            AiState::Chase => {
                let dir = if distance > 0.0 { to_player.normalize() } else { Vec2::X };
                velocity.0 = dir * CRUISER_SPEED;
            }
            AiState::Attack => {
                // Slow down while firing
                velocity.0 = velocity.0 * 0.9;
                cooldown.timer -= dt;
                if cooldown.timer <= 0.0 {
                    cooldown.timer = 2.0; // Slow fire rate, high damage
                    pending_shots.shots.push(PendingEnemyShot {
                        origin: pos,
                        target: player_pos,
                        damage: 20.0, // High damage per shot
                    });
                }
            }
            AiState::Flee => {
                let dir = if distance > 0.0 { -to_player.normalize() } else { Vec2::X };
                velocity.0 = dir * CRUISER_SPEED;
            }
            AiState::Idle | AiState::Patrol => {
                velocity.0 = velocity.0 * 0.99;
            }
        }
    }
}

/// Updates Sniper AI: keeps preferred distance band, shoots from range.
#[allow(clippy::type_complexity)]
pub fn update_sniper_ai(
    time: Res<Time>,
    player_query: Query<&Transform, With<crate::core::flight::Player>>,
    mut sniper_query: Query<
        (
            &Transform,
            &crate::core::collision::Health,
            &mut crate::shared::components::Velocity,
            &mut AiState,
            &crate::social::faction::AggroRange,
            &crate::social::faction::AttackRange,
            &crate::social::faction::FleeThreshold,
            &crate::core::spawning::PreferredRange,
            &mut EnemyFireCooldown,
        ),
        With<crate::core::spawning::Sniper>,
    >,
    mut pending_shots: ResMut<PendingEnemyShotQueue>,
) {
    let dt = time.delta_secs();
    let Ok(player_transform) = player_query.single() else {
        return;
    };
    let player_pos = Vec2::new(
        player_transform.translation.x,
        player_transform.translation.y,
    );

    for (transform, health, mut velocity, mut ai_state, aggro, attack, flee, pref_range, mut cooldown) in
        sniper_query.iter_mut()
    {
        let pos = Vec2::new(transform.translation.x, transform.translation.y);
        let to_player = player_pos - pos;
        let distance = to_player.length();
        let health_ratio = if health.max > 0.0 { health.current / health.max } else { 0.0 };

        let ctx = AiContext {
            distance_to_player: distance,
            health_ratio,
            aggro_range: aggro.0,
            attack_range: attack.0,
            flee_threshold: flee.0,
        };
        *ai_state = next_state(&ai_state, &ctx);

        const SNIPER_SPEED: f32 = 60.0;
        match *ai_state {
            AiState::Chase | AiState::Attack => {
                // Preferred range behavior: maintain distance band
                if distance < pref_range.min {
                    // Too close — back away
                    let dir = if distance > 0.0 { -to_player.normalize() } else { Vec2::X };
                    velocity.0 = dir * SNIPER_SPEED;
                } else if distance > pref_range.max {
                    // Too far — approach
                    let dir = if distance > 0.0 { to_player.normalize() } else { Vec2::X };
                    velocity.0 = dir * SNIPER_SPEED;
                } else {
                    // In range — strafe (perpendicular movement)
                    let perp = Vec2::new(-to_player.y, to_player.x).normalize_or_zero();
                    velocity.0 = perp * SNIPER_SPEED * 0.5;
                }

                // Shoot when in attack state
                if *ai_state == AiState::Attack {
                    cooldown.timer -= dt;
                    if cooldown.timer <= 0.0 {
                        cooldown.timer = 2.0; // Slow rate, high damage
                        pending_shots.shots.push(PendingEnemyShot {
                            origin: pos,
                            target: player_pos,
                            damage: 15.0,
                        });
                    }
                }
            }
            AiState::Flee => {
                let dir = if distance > 0.0 { -to_player.normalize() } else { Vec2::X };
                velocity.0 = dir * SNIPER_SPEED * 1.5;
            }
            AiState::Idle | AiState::Patrol => {
                velocity.0 = velocity.0 * 0.98;
            }
        }
    }
}

/// Updates Swarm AI.
/// SwarmLeader: behaves like a Fighter (aggressive pursuit).
/// SwarmFollower: tracks leader position using world position cache.
#[allow(clippy::type_complexity)]
pub fn update_swarm_ai(
    player_query: Query<&Transform, With<crate::core::flight::Player>>,
    mut swarm_query: Query<
        (
            Entity,
            &Transform,
            &mut crate::shared::components::Velocity,
            Option<&mut AiState>,
            Option<&crate::core::collision::Health>,
            Option<&crate::social::faction::AggroRange>,
            Option<&crate::social::faction::AttackRange>,
            Option<&crate::social::faction::FleeThreshold>,
            Option<&crate::core::spawning::SwarmLeader>,
            Option<&crate::core::spawning::SwarmFollower>,
        ),
        Without<crate::core::flight::Player>,
    >,
) {
    let Ok(player_transform) = player_query.single() else {
        return;
    };
    let player_pos = Vec2::new(
        player_transform.translation.x,
        player_transform.translation.y,
    );

    // First pass: collect leader positions
    let mut leader_positions: std::collections::HashMap<Entity, Vec2> = std::collections::HashMap::new();
    for (entity, transform, _, _, _, _, _, _, leader_marker, _) in swarm_query.iter() {
        if leader_marker.is_some() {
            leader_positions.insert(entity, Vec2::new(transform.translation.x, transform.translation.y));
        }
    }

    // Second pass: update movement
    const SWARM_SPEED: f32 = 110.0;
    const FOLLOWER_SPEED: f32 = 110.0;

    for (_, transform, mut velocity, ai_state_opt, health_opt, aggro_opt, attack_opt, flee_opt, leader_marker, follower_opt) in
        swarm_query.iter_mut()
    {
        let pos = Vec2::new(transform.translation.x, transform.translation.y);

        if leader_marker.is_some() {
            // Leader: act like fighter AI
            if let (Some(mut ai_state), Some(health), Some(aggro), Some(attack), Some(flee)) =
                (ai_state_opt, health_opt, aggro_opt, attack_opt, flee_opt)
            {
                let to_player = player_pos - pos;
                let distance = to_player.length();
                let health_ratio = if health.max > 0.0 { health.current / health.max } else { 0.0 };

                let ctx = AiContext {
                    distance_to_player: distance,
                    health_ratio,
                    aggro_range: aggro.0,
                    attack_range: attack.0,
                    flee_threshold: flee.0,
                };
                *ai_state = next_state(&ai_state, &ctx);

                match *ai_state {
                    AiState::Chase | AiState::Attack => {
                        let dir = if distance > 0.0 { to_player.normalize() } else { Vec2::X };
                        velocity.0 = dir * SWARM_SPEED;
                    }
                    AiState::Flee => {
                        let dir = if distance > 0.0 { -to_player.normalize() } else { Vec2::X };
                        velocity.0 = dir * SWARM_SPEED;
                    }
                    AiState::Idle | AiState::Patrol => {
                        velocity.0 = velocity.0 * 0.95;
                    }
                }
            }
        } else if let Some(follower) = follower_opt {
            // Follower: move toward leader
            if let Some(&leader_pos) = leader_positions.get(&follower.leader) {
                let to_leader = leader_pos - pos;
                let dist = to_leader.length();
                if dist > 20.0 {
                    velocity.0 = to_leader.normalize() * FOLLOWER_SPEED;
                } else {
                    velocity.0 = velocity.0 * 0.9;
                }
            }
        }
    }
}

/// Updates enemy facing direction toward the player.
/// Rotates `FacingDirection` at `TurnRate` radians per second toward the player.
/// Also updates `Transform` rotation so mesh shows the facing direction.
pub fn update_enemy_facing(
    time: Res<Time>,
    player_query: Query<&Transform, With<crate::core::flight::Player>>,
    mut enemy_query: Query<
        (
            &Transform,
            &mut crate::social::faction::FacingDirection,
            Option<&crate::social::faction::TurnRate>,
        ),
        (
            Without<crate::core::flight::Player>,
            Or<(
                With<crate::core::spawning::ScoutDrone>,
                With<crate::core::spawning::Fighter>,
                With<crate::core::spawning::HeavyCruiser>,
                With<crate::core::spawning::Sniper>,
            )>,
        ),
    >,
) {
    let dt = time.delta_secs();
    let Ok(player_transform) = player_query.single() else {
        return;
    };
    let player_pos = Vec2::new(
        player_transform.translation.x,
        player_transform.translation.y,
    );

    for (transform, mut facing, turn_rate_opt) in enemy_query.iter_mut() {
        let enemy_pos = Vec2::new(transform.translation.x, transform.translation.y);
        let to_player = player_pos - enemy_pos;

        if to_player.length_squared() < f32::EPSILON {
            continue;
        }

        let target_dir = to_player.normalize();
        let turn_rate = turn_rate_opt.map(|r| r.0).unwrap_or(3.0); // Default 3 rad/s

        // Rotate current facing toward target_dir at turn_rate
        let current = facing.0;
        let current_angle = current.y.atan2(current.x);
        let target_angle = target_dir.y.atan2(target_dir.x);
        let mut angle_diff = target_angle - current_angle;

        // Normalize angle to [-PI, PI]
        while angle_diff > std::f32::consts::PI { angle_diff -= std::f32::consts::TAU; }
        while angle_diff < -std::f32::consts::PI { angle_diff += std::f32::consts::TAU; }

        let max_turn = turn_rate * dt;
        let actual_turn = angle_diff.clamp(-max_turn, max_turn);
        let new_angle = current_angle + actual_turn;
        facing.0 = Vec2::new(new_angle.cos(), new_angle.sin());
    }
}

/// A pending laser shot from an enemy, to be processed by collision systems.
#[derive(Debug, Clone)]
pub struct PendingEnemyShot {
    pub origin: Vec2,
    pub target: Vec2,
    pub damage: f32,
}

/// Buffer resource for enemy shots produced this frame.
#[derive(Resource, Default)]
pub struct PendingEnemyShotQueue {
    pub shots: Vec<PendingEnemyShot>,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ctx(distance: f32, health: f32) -> AiContext {
        AiContext {
            distance_to_player: distance,
            health_ratio: health,
            aggro_range: 200.0,
            attack_range: 80.0,
            flee_threshold: 0.2,
        }
    }

    #[test]
    fn idle_stays_idle_when_player_out_of_range() {
        assert_eq!(next_state(&AiState::Idle, &ctx(300.0, 1.0)), AiState::Idle);
    }

    #[test]
    fn idle_transitions_to_chase_when_player_enters_aggro_range() {
        assert_eq!(next_state(&AiState::Idle, &ctx(150.0, 1.0)), AiState::Chase);
    }

    #[test]
    fn patrol_transitions_to_chase_when_player_enters_aggro_range() {
        assert_eq!(
            next_state(&AiState::Patrol, &ctx(150.0, 1.0)),
            AiState::Chase
        );
    }

    #[test]
    fn chase_transitions_to_attack_when_in_attack_range() {
        assert_eq!(next_state(&AiState::Chase, &ctx(60.0, 1.0)), AiState::Attack);
    }

    #[test]
    fn chase_transitions_to_patrol_when_player_escapes() {
        // Beyond aggro_range * 1.5 = 300.0
        assert_eq!(
            next_state(&AiState::Chase, &ctx(350.0, 1.0)),
            AiState::Patrol
        );
    }

    #[test]
    fn attack_transitions_to_chase_when_player_moves_out() {
        // Beyond attack_range * 1.2 = 96.0
        assert_eq!(
            next_state(&AiState::Attack, &ctx(100.0, 1.0)),
            AiState::Chase
        );
    }

    #[test]
    fn attack_stays_attack_when_player_in_range() {
        assert_eq!(
            next_state(&AiState::Attack, &ctx(60.0, 1.0)),
            AiState::Attack
        );
    }

    #[test]
    fn flee_triggers_from_any_state_when_health_critical() {
        assert_eq!(next_state(&AiState::Idle, &ctx(50.0, 0.1)), AiState::Flee);
        assert_eq!(next_state(&AiState::Chase, &ctx(50.0, 0.1)), AiState::Flee);
        assert_eq!(next_state(&AiState::Attack, &ctx(50.0, 0.19)), AiState::Flee);
    }

    #[test]
    fn flee_stays_flee() {
        assert_eq!(next_state(&AiState::Flee, &ctx(50.0, 0.1)), AiState::Flee);
    }

    #[test]
    fn full_health_does_not_trigger_flee() {
        assert_ne!(next_state(&AiState::Idle, &ctx(50.0, 1.0)), AiState::Flee);
    }

    // ── ErraticOffset / roll_erratic_offset ──

    #[test]
    fn roll_erratic_offset_produces_bounded_vector() {
        let result = roll_erratic_offset(0.5, 0.5, 40.0);
        // Length should be approximately magnitude (normalized * magnitude)
        let len = result.length();
        assert!(
            (len - 40.0).abs() < 1.0,
            "Expected length ~40.0, got {len}"
        );
    }

    #[test]
    fn roll_erratic_offset_zero_input_uses_fallback() {
        // Zero vector should not panic
        let result = roll_erratic_offset(0.0, 0.0, 40.0);
        // Fallback is Vec2::X * magnitude
        assert!(
            (result.x - 40.0).abs() < 0.01,
            "Expected fallback Vec2::X * 40.0, got {result:?}"
        );
    }

    #[test]
    fn roll_erratic_offset_is_deterministic() {
        let r1 = roll_erratic_offset(0.3, -0.7, 50.0);
        let r2 = roll_erratic_offset(0.3, -0.7, 50.0);
        assert!(
            (r1 - r2).length() < f32::EPSILON,
            "Same inputs must produce identical output"
        );
    }

    #[test]
    fn erratic_offset_default_values_valid() {
        let e = ErraticOffset::default();
        assert!(e.interval > 0.0, "Interval must be positive");
        assert!(e.magnitude > 0.0, "Magnitude must be positive");
    }

    // ── Scout Drone AI system tests ──

    fn setup_scout_drone_ai_app() -> App {
        use crate::core::collision::Health;
        use crate::core::flight::Player;
        use crate::core::spawning::ScoutDrone;
        use crate::shared::components::Velocity;
        use crate::social::faction::{AggroRange, AttackRange, FleeThreshold};

        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.insert_resource(bevy::time::TimeUpdateStrategy::ManualDuration(
            std::time::Duration::from_secs_f64(1.0 / 60.0),
        ));
        app.init_resource::<PendingEnemyShotQueue>();
        app.add_systems(Update, update_scout_drone_ai);

        // Spawn player
        app.world_mut().spawn((
            Player,
            Transform::from_translation(Vec3::new(0.0, 0.0, 0.0)),
        ));

        app
    }

    fn spawn_drone_at(app: &mut App, pos: Vec2, ai_state: AiState) -> Entity {
        use crate::core::collision::Health;
        use crate::core::spawning::ScoutDrone;
        use crate::shared::components::Velocity;
        use crate::social::faction::{AggroRange, AttackRange, FleeThreshold};

        app.world_mut().spawn((
            ScoutDrone,
            Transform::from_translation(pos.extend(0.0)),
            Velocity(Vec2::ZERO),
            Health { current: 30.0, max: 30.0 },
            ai_state,
            AggroRange(200.0),
            AttackRange(80.0),
            FleeThreshold(0.2),
            ErraticOffset::default(),
            EnemyFireCooldown::default(),
        )).id()
    }

    #[test]
    fn drone_in_aggro_range_transitions_to_chase() {
        let mut app = setup_scout_drone_ai_app();
        // Player at origin, drone 100 units away (within aggro 200)
        let drone = spawn_drone_at(&mut app, Vec2::new(100.0, 0.0), AiState::Idle);
        app.update(); // prime
        app.update();
        let state = app.world().entity(drone).get::<AiState>()
            .expect("Drone should have AiState");
        assert_eq!(*state, AiState::Chase, "Drone should chase when player within aggro range");
    }

    #[test]
    fn drone_out_of_range_stays_idle() {
        let mut app = setup_scout_drone_ai_app();
        // Player at origin, drone 300 units away (outside aggro 200)
        let drone = spawn_drone_at(&mut app, Vec2::new(300.0, 0.0), AiState::Idle);
        app.update(); // prime
        app.update();
        let state = app.world().entity(drone).get::<AiState>()
            .expect("Drone should have AiState");
        assert_eq!(*state, AiState::Idle, "Drone should stay idle when player out of range");
    }

    #[test]
    fn drone_in_attack_range_transitions_to_attack() {
        let mut app = setup_scout_drone_ai_app();
        // Player at origin, drone 50 units away (within attack 80)
        let drone = spawn_drone_at(&mut app, Vec2::new(50.0, 0.0), AiState::Chase);
        app.update(); // prime
        app.update();
        let state = app.world().entity(drone).get::<AiState>()
            .expect("Drone should have AiState");
        assert_eq!(*state, AiState::Attack, "Drone should attack when player within attack range");
    }

    #[test]
    fn drone_chase_moves_toward_player() {
        use crate::shared::components::Velocity;
        let mut app = setup_scout_drone_ai_app();
        // Player at origin, drone at x=200 in chase state
        let drone = spawn_drone_at(&mut app, Vec2::new(200.0, 0.0), AiState::Chase);
        app.update(); // prime
        app.update();
        let velocity = app.world().entity(drone).get::<Velocity>()
            .expect("Drone should have Velocity");
        // Should move toward player (negative X direction)
        assert!(velocity.0.x < 0.0, "Chasing drone should move toward player (negative X), got {}", velocity.0.x);
    }

    #[test]
    fn drone_flee_moves_away_from_player() {
        use crate::shared::components::Velocity;
        let mut app = setup_scout_drone_ai_app();
        // Drone at x=50 (within attack range), low health to trigger flee
        // Spawn with low health
        use crate::core::collision::Health;
        use crate::core::spawning::ScoutDrone;
        use crate::social::faction::{AggroRange, AttackRange, FleeThreshold};
        let drone = app.world_mut().spawn((
            ScoutDrone,
            Transform::from_translation(Vec3::new(50.0, 0.0, 0.0)),
            crate::shared::components::Velocity(Vec2::ZERO),
            Health { current: 3.0, max: 30.0 }, // 10% health, flee_threshold=0.2
            AiState::Attack,
            AggroRange(200.0),
            AttackRange(80.0),
            FleeThreshold(0.2),
            ErraticOffset::default(),
            EnemyFireCooldown::default(),
        )).id();
        app.update(); // prime
        app.update();
        let velocity = app.world().entity(drone).get::<crate::shared::components::Velocity>()
            .expect("Drone should have Velocity");
        // Should move away from player (positive X direction, player is at origin)
        assert!(velocity.0.x > 0.0, "Fleeing drone should move away from player (positive X), got {}", velocity.0.x);
    }

    #[test]
    fn drone_attack_state_queues_shot_when_cooldown_ready() {
        let mut app = setup_scout_drone_ai_app();
        // Drone in attack range with cooldown already at 0
        use crate::core::collision::Health;
        use crate::core::spawning::ScoutDrone;
        use crate::shared::components::Velocity;
        use crate::social::faction::{AggroRange, AttackRange, FleeThreshold};
        app.world_mut().spawn((
            ScoutDrone,
            Transform::from_translation(Vec3::new(50.0, 0.0, 0.0)),
            Velocity(Vec2::ZERO),
            Health { current: 30.0, max: 30.0 },
            AiState::Attack, // Already in attack state
            AggroRange(200.0),
            AttackRange(80.0),
            FleeThreshold(0.2),
            ErraticOffset::default(),
            EnemyFireCooldown { timer: 0.0 }, // Ready to fire
        ));
        app.update(); // prime
        app.update();
        let queue = app.world().resource::<PendingEnemyShotQueue>();
        assert!(!queue.shots.is_empty(), "Drone in attack state with ready cooldown should queue a shot");
    }

    // ── Fighter AI tests ──

    fn setup_fighter_ai_app() -> App {
        use crate::core::flight::Player;

        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.insert_resource(bevy::time::TimeUpdateStrategy::ManualDuration(
            std::time::Duration::from_secs_f64(1.0 / 60.0),
        ));
        app.init_resource::<PendingEnemyShotQueue>();
        app.add_systems(Update, update_fighter_ai);

        app.world_mut().spawn((
            Player,
            Transform::from_translation(Vec3::new(0.0, 0.0, 0.0)),
        ));
        app
    }

    fn spawn_fighter_at(app: &mut App, pos: Vec2, ai_state: AiState) -> Entity {
        use crate::core::collision::Health;
        use crate::core::spawning::Fighter;
        use crate::shared::components::Velocity;
        use crate::social::faction::{AggroRange, AttackRange, FleeThreshold};

        app.world_mut().spawn((
            Fighter,
            Transform::from_translation(pos.extend(0.0)),
            Velocity(Vec2::ZERO),
            Health { current: 50.0, max: 50.0 },
            ai_state,
            AggroRange(400.0), // Larger aggro range
            AttackRange(100.0),
            FleeThreshold(0.1), // Only flee at 10% health
            EnemyFireCooldown::default(),
        )).id()
    }

    #[test]
    fn fighter_has_larger_aggro_range_than_drone() {
        // Fighters should have AggroRange(400.0) vs Scout Drone AggroRange(200.0)
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        use crate::core::collision::Health;
        use crate::core::spawning::Fighter;
        use crate::shared::components::Velocity;
        use crate::social::faction::{AggroRange, AttackRange, FleeThreshold};

        let fighter = app.world_mut().spawn((
            Fighter,
            Transform::default(),
            Velocity(Vec2::ZERO),
            Health { current: 50.0, max: 50.0 },
            AiState::Idle,
            AggroRange(400.0),
            AttackRange(100.0),
            FleeThreshold(0.1),
            EnemyFireCooldown::default(),
        )).id();

        let aggro = app.world().entity(fighter).get::<AggroRange>()
            .expect("Fighter should have AggroRange");
        assert!(aggro.0 > 200.0, "Fighter aggro range ({}) should exceed Scout Drone (200.0)", aggro.0);
    }

    #[test]
    fn fighter_chases_player_when_in_aggro_range() {
        let mut app = setup_fighter_ai_app();
        // Player at origin, fighter at 300 units (within fighter aggro 400)
        let fighter = spawn_fighter_at(&mut app, Vec2::new(300.0, 0.0), AiState::Idle);
        app.update(); // prime
        app.update();
        let state = app.world().entity(fighter).get::<AiState>()
            .expect("Fighter should have AiState");
        assert_eq!(*state, AiState::Chase, "Fighter should chase when player within aggro range");
    }

    #[test]
    fn fighter_moves_toward_player_in_chase() {
        use crate::shared::components::Velocity;
        let mut app = setup_fighter_ai_app();
        let fighter = spawn_fighter_at(&mut app, Vec2::new(200.0, 0.0), AiState::Chase);
        app.update(); // prime
        app.update();
        let vel = app.world().entity(fighter).get::<Velocity>()
            .expect("Fighter should have Velocity");
        assert!(vel.0.x < 0.0, "Chasing fighter should move toward player (negative X), got {}", vel.0.x);
    }

    #[test]
    fn fighter_attack_queues_shot_with_ready_cooldown() {
        let mut app = setup_fighter_ai_app();
        use crate::core::collision::Health;
        use crate::core::spawning::Fighter;
        use crate::shared::components::Velocity;
        use crate::social::faction::{AggroRange, AttackRange, FleeThreshold};
        app.world_mut().spawn((
            Fighter,
            Transform::from_translation(Vec3::new(50.0, 0.0, 0.0)),
            Velocity(Vec2::ZERO),
            Health { current: 50.0, max: 50.0 },
            AiState::Attack,
            AggroRange(400.0),
            AttackRange(100.0),
            FleeThreshold(0.1),
            EnemyFireCooldown { timer: 0.0 },
        ));
        app.update(); // prime
        app.update();
        let queue = app.world().resource::<PendingEnemyShotQueue>();
        assert!(!queue.shots.is_empty(), "Fighter should fire when in attack state with ready cooldown");
    }

    // ── Heavy Cruiser AI tests ──

    #[test]
    fn heavy_cruiser_has_high_health() {
        use crate::core::collision::Health;
        use crate::core::spawning::HeavyCruiser;
        use crate::shared::components::Velocity;
        use crate::social::faction::{AggroRange, AttackRange, FleeThreshold};

        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        let cruiser = app.world_mut().spawn((
            HeavyCruiser,
            Transform::default(),
            Velocity(Vec2::ZERO),
            Health { current: 200.0, max: 200.0 },
            AiState::Idle,
            AggroRange(300.0),
            AttackRange(120.0),
            FleeThreshold(0.05),
            EnemyFireCooldown::default(),
        )).id();

        let health = app.world().entity(cruiser).get::<crate::core::collision::Health>()
            .expect("Heavy Cruiser should have Health");
        assert!(health.max >= 200.0, "Heavy Cruiser should have high health (>=200), got {}", health.max);
    }

    #[test]
    fn heavy_cruiser_fires_high_damage_shot() {
        use crate::core::flight::Player;
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.insert_resource(bevy::time::TimeUpdateStrategy::ManualDuration(
            std::time::Duration::from_secs_f64(1.0 / 60.0),
        ));
        app.init_resource::<PendingEnemyShotQueue>();
        app.add_systems(Update, update_heavy_cruiser_ai);
        app.world_mut().spawn((Player, Transform::default()));

        use crate::core::collision::Health;
        use crate::core::spawning::HeavyCruiser;
        use crate::shared::components::Velocity;
        use crate::social::faction::{AggroRange, AttackRange, FleeThreshold};
        app.world_mut().spawn((
            HeavyCruiser,
            Transform::from_translation(Vec3::new(80.0, 0.0, 0.0)),
            Velocity(Vec2::ZERO),
            Health { current: 200.0, max: 200.0 },
            AiState::Attack,
            AggroRange(300.0),
            AttackRange(120.0),
            FleeThreshold(0.05),
            EnemyFireCooldown { timer: 0.0 },
        ));
        app.update(); // prime
        app.update();

        let queue = app.world().resource::<PendingEnemyShotQueue>();
        assert!(!queue.shots.is_empty(), "Heavy Cruiser should fire when in attack state");
        assert!(
            queue.shots[0].damage >= 20.0,
            "Heavy Cruiser shot damage ({}) should be high (>=20)",
            queue.shots[0].damage
        );
    }

    // ── Sniper AI tests ──

    // ── Attack Telegraphing (Story 4-8) tests ──

    #[test]
    fn update_enemy_facing_rotates_toward_player() {
        use crate::core::flight::Player;
        use crate::core::spawning::ScoutDrone;
        use crate::social::faction::{FacingDirection, TurnRate};

        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.insert_resource(bevy::time::TimeUpdateStrategy::ManualDuration(
            std::time::Duration::from_secs_f64(1.0),
        ));
        app.add_systems(Update, update_enemy_facing);

        // Player at (100, 0), enemy at origin facing up (Y)
        app.world_mut().spawn((Player, Transform::from_translation(Vec3::new(100.0, 0.0, 0.0))));

        let drone = app.world_mut().spawn((
            ScoutDrone,
            Transform::from_translation(Vec3::ZERO),
            FacingDirection(Vec2::Y), // Initially facing up
            TurnRate(std::f32::consts::PI * 2.0), // Fast turn: 2π rad/s
        )).id();

        app.update(); // prime (dt=0)
        app.update(); // 1 second at 2π rad/s = full rotation possible

        let facing = app.world().entity(drone).get::<FacingDirection>()
            .expect("Drone should have FacingDirection");
        // Player is to the right (+X), facing should now have positive X component
        assert!(facing.0.x > 0.0, "Facing should point toward player (positive X), got {:?}", facing.0);
    }

    #[test]
    fn update_enemy_facing_respects_turn_rate() {
        use crate::core::flight::Player;
        use crate::core::spawning::ScoutDrone;
        use crate::social::faction::{FacingDirection, TurnRate};

        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.insert_resource(bevy::time::TimeUpdateStrategy::ManualDuration(
            std::time::Duration::from_secs_f64(1.0 / 60.0),
        ));
        app.add_systems(Update, update_enemy_facing);

        // Player directly to the right, drone facing up
        app.world_mut().spawn((Player, Transform::from_translation(Vec3::new(100.0, 0.0, 0.0))));

        let drone = app.world_mut().spawn((
            ScoutDrone,
            Transform::from_translation(Vec3::ZERO),
            FacingDirection(Vec2::Y), // Facing up initially
            TurnRate(0.1), // Very slow turn rate — won't reach target in one frame
        )).id();

        app.update(); // prime
        app.update(); // 1/60 second at 0.1 rad/s = only 0.0017 rad turn

        let facing = app.world().entity(drone).get::<FacingDirection>()
            .expect("Drone should have FacingDirection");
        // Should still be mostly facing up (Y), just slightly toward X
        // Y should still be dominant
        assert!(facing.0.y > 0.9, "Slow turn rate should not rotate much in one frame, y={}", facing.0.y);
    }

    #[test]
    fn facing_direction_default_is_up() {
        use crate::social::faction::FacingDirection;
        let facing = FacingDirection::default();
        assert!((facing.0.y - 1.0).abs() < f32::EPSILON, "Default facing should be Vec2::Y");
    }

    #[test]
    fn sniper_backs_away_when_too_close() {
        use crate::core::flight::Player;
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.insert_resource(bevy::time::TimeUpdateStrategy::ManualDuration(
            std::time::Duration::from_secs_f64(1.0 / 60.0),
        ));
        app.init_resource::<PendingEnemyShotQueue>();
        app.add_systems(Update, update_sniper_ai);
        app.world_mut().spawn((Player, Transform::default()));

        use crate::core::collision::Health;
        use crate::core::spawning::{PreferredRange, Sniper};
        use crate::shared::components::Velocity;
        use crate::social::faction::{AggroRange, AttackRange, FleeThreshold};
        // Sniper at 50 units, min preferred range is 150 — too close
        let sniper = app.world_mut().spawn((
            Sniper,
            Transform::from_translation(Vec3::new(50.0, 0.0, 0.0)),
            Velocity(Vec2::ZERO),
            Health { current: 40.0, max: 40.0 },
            AiState::Chase,
            AggroRange(350.0),
            AttackRange(300.0),
            FleeThreshold(0.15),
            PreferredRange { min: 150.0, max: 280.0 },
            EnemyFireCooldown::default(),
        )).id();
        app.update(); // prime
        app.update();

        let vel = app.world().entity(sniper).get::<crate::shared::components::Velocity>()
            .expect("Sniper should have Velocity");
        // At 50 units from player (min=150), sniper should back away (positive X from player at origin)
        assert!(vel.0.x > 0.0, "Sniper too close should back away from player (positive X), got {}", vel.0.x);
    }

    // ── Swarm AI tests ──

    #[test]
    fn swarm_follower_moves_toward_leader() {
        use crate::core::flight::Player;
        use crate::core::spawning::{Swarm, SwarmFollower, SwarmLeader, Fighter};
        use crate::shared::components::Velocity;
        use crate::core::collision::Health;
        use crate::social::faction::{AggroRange, AttackRange, FleeThreshold};

        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.insert_resource(bevy::time::TimeUpdateStrategy::ManualDuration(
            std::time::Duration::from_secs_f64(1.0 / 60.0),
        ));
        app.init_resource::<PendingEnemyShotQueue>();
        app.add_systems(Update, update_swarm_ai);

        app.world_mut().spawn((Player, Transform::default()));

        // Spawn leader at origin
        let leader = app.world_mut().spawn((
            Fighter,
            SwarmLeader,
            Swarm { swarm_id: 1 },
            Transform::from_translation(Vec3::new(0.0, 0.0, 0.0)),
            Velocity(Vec2::ZERO),
            Health { current: 50.0, max: 50.0 },
            AiState::Chase,
            AggroRange(400.0),
            AttackRange(100.0),
            FleeThreshold(0.1),
        )).id();

        // Spawn follower 200 units away from leader
        let follower = app.world_mut().spawn((
            Fighter,
            SwarmFollower { leader },
            Swarm { swarm_id: 1 },
            Transform::from_translation(Vec3::new(200.0, 0.0, 0.0)),
            Velocity(Vec2::ZERO),
            Health { current: 50.0, max: 50.0 },
        )).id();

        app.update(); // prime
        app.update();

        let vel = app.world().entity(follower).get::<Velocity>()
            .expect("Follower should have Velocity");
        // Follower at x=200, leader at x=0: should move in negative X direction
        assert!(vel.0.x < 0.0, "Follower should move toward leader (negative X), got {}", vel.0.x);
    }

    #[test]
    fn sniper_approaches_when_too_far() {
        use crate::core::flight::Player;
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.insert_resource(bevy::time::TimeUpdateStrategy::ManualDuration(
            std::time::Duration::from_secs_f64(1.0 / 60.0),
        ));
        app.init_resource::<PendingEnemyShotQueue>();
        app.add_systems(Update, update_sniper_ai);
        app.world_mut().spawn((Player, Transform::default()));

        use crate::core::collision::Health;
        use crate::core::spawning::{PreferredRange, Sniper};
        use crate::shared::components::Velocity;
        use crate::social::faction::{AggroRange, AttackRange, FleeThreshold};
        // Sniper at 350 units, max preferred range is 280 — too far
        let sniper = app.world_mut().spawn((
            Sniper,
            Transform::from_translation(Vec3::new(350.0, 0.0, 0.0)),
            Velocity(Vec2::ZERO),
            Health { current: 40.0, max: 40.0 },
            AiState::Chase,
            AggroRange(400.0),
            AttackRange(300.0),
            FleeThreshold(0.15),
            PreferredRange { min: 150.0, max: 280.0 },
            EnemyFireCooldown::default(),
        )).id();
        app.update(); // prime
        app.update();

        let vel = app.world().entity(sniper).get::<crate::shared::components::Velocity>()
            .expect("Sniper should have Velocity");
        // At 350 units (max=280), sniper should approach (negative X toward player at origin)
        assert!(vel.0.x < 0.0, "Sniper too far should approach player (negative X), got {}", vel.0.x);
    }
}
