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

/// Cooldown on contact damage to prevent instant death from continuous overlap.
/// While active, player does not take body-collision damage.
#[derive(Component, Debug)]
pub struct ContactDamageCooldown {
    pub timer: f32,
}

/// Player is immune to damage. Removed when timer expires.
/// Written/removed by core/collision.rs, read by rendering/effects.rs for blink visual.
#[derive(Component, Debug)]
pub struct Invincible {
    pub timer: f32,
}
