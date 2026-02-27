use bevy::prelude::*;

/// Velocity vector shared across domains (flight, collision, projectiles, enemy AI).
#[derive(Component, Default, Clone, Debug)]
pub struct Velocity(pub Vec2);
