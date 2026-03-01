use bevy::prelude::*;
use serde::{Serialize, Deserialize};

/// Velocity vector shared across domains (flight, collision, projectiles, enemy AI).
#[derive(Component, Default, Clone, Debug)]
pub struct Velocity(pub Vec2);

/// Types of material that can be dropped from destroyed enemies.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MaterialType {
    CommonScrap,
    RareAlloy,
    EnergyCore,
}

/// Marker component for material drop entities in the world.
#[derive(Component, Debug)]
pub struct MaterialDrop;

/// Marker component for material drop entities that need their visual mesh attached by RenderingPlugin.
#[derive(Component, Debug)]
pub struct NeedsMaterialDropVisual;

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
