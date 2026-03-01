/// Faction identity and behavior parameter components for Epic 4: Combat Depth.
use bevy::prelude::*;

/// Which faction an entity belongs to.
/// Faction determines FSM parameters (aggression, flee threshold, reinforcement calls).
#[derive(Component, Debug, Clone, PartialEq, Eq, Hash)]
pub enum FactionId {
    Pirates,
    Military,
    Aliens,
    RogueDrones,
    Neutral,
}

/// Radius within which an enemy patrols when no player is detected.
#[derive(Component, Debug, Clone)]
pub struct PatrolRadius(pub f32);

/// Distance at which an enemy detects the player and transitions to Chase.
#[derive(Component, Debug, Clone)]
pub struct AggroRange(pub f32);

/// Health ratio (0.0–1.0) below which an enemy transitions to Flee.
#[derive(Component, Debug, Clone)]
pub struct FleeThreshold(pub f32);

/// Distance at which an enemy transitions from Chase to Attack.
#[derive(Component, Debug, Clone)]
pub struct AttackRange(pub f32);
