use bevy::prelude::*;

/// Velocity vector shared across domains (flight, collision, projectiles, enemy AI).
#[derive(Component, Default, Clone, Debug)]
pub struct Velocity(pub Vec2);

/// Cross-domain marker component indicating an entity received damage this frame.
/// Written by core/collision.rs (apply_damage), consumed by rendering/effects.rs (damage flash, screen shake).
#[derive(Component, Debug)]
pub struct JustDamaged {
    pub amount: f32,
}
