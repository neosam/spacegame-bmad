/// Enemy AI Finite State Machine for Epic 4: Combat Depth.
///
/// Design principle: FSM logic lives in pure functions (`next_state`).
/// Pure functions take explicit inputs and return the next state —
/// no ECS world access, no side effects, fully testable without App setup.
///
/// # Test Pattern
///
/// ```rust
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
}
