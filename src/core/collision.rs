use bevy::ecs::message::MessageReader;
use bevy::prelude::*;

use crate::shared::components::JustDamaged;
use super::weapons::{LaserFired, SpreadProjectile, WeaponConfig};

/// Radius used for spread projectile collision checks.
pub const PROJECTILE_RADIUS: f32 = 2.0;

// ── Collision math ──────────────────────────────────────────────────────

/// Checks if a ray intersects a circle, returns hit point if found.
/// Returns None if ray misses or circle is behind origin or beyond range.
pub fn ray_circle_intersection(
    origin: Vec2,
    direction: Vec2,
    range: f32,
    center: Vec2,
    radius: f32,
) -> Option<Vec2> {
    // Vector from origin to circle center
    let to_center = center - origin;

    // Project to_center onto direction to find closest approach on ray
    let projection_length = to_center.dot(direction);

    // Circle is behind the ray origin
    if projection_length < 0.0 {
        return None;
    }

    // Circle is beyond ray range
    if projection_length > range {
        return None;
    }

    // Closest point on ray to circle center
    let closest_point = origin + direction * projection_length;

    // Distance from closest point to circle center
    let distance_to_center = closest_point.distance(center);

    // Ray hits circle if distance is within radius
    if distance_to_center <= radius {
        Some(closest_point)
    } else {
        None
    }
}

/// Checks if two circles intersect (overlap or touch).
pub fn circle_circle_intersection(
    center1: Vec2,
    radius1: f32,
    center2: Vec2,
    radius2: f32,
) -> bool {
    let distance = center1.distance(center2);
    distance <= (radius1 + radius2)
}

// ── Components ──────────────────────────────────────────────────────────

/// Circle collider for entities that participate in collision detection.
#[derive(Component, Debug, Clone)]
pub struct Collider {
    pub radius: f32,
}

/// Health pool for destructible entities (asteroids, enemies).
#[derive(Component, Debug, Clone)]
pub struct Health {
    pub current: f32,
    pub max: f32,
}

// ── Damage queue (resource) ─────────────────────────────────────────────

/// Buffers damage to be applied in the Damage set.
/// Collision systems push entries; `apply_damage` consumes and clears them.
#[derive(Resource, Default)]
pub struct DamageQueue {
    pub entries: Vec<(Entity, f32)>,
}

/// Positions of entities destroyed this frame. Cleared after rendering consumes them.
#[derive(Resource, Default)]
pub struct DestroyedPositions {
    pub positions: Vec<Vec2>,
}

/// Positions of laser hits this frame. Cleared after rendering consumes them.
#[derive(Resource, Default)]
pub struct LaserHitPositions {
    pub positions: Vec<Vec2>,
}

// ── Systems ─────────────────────────────────────────────────────────────

/// Reads `LaserFired` messages and checks ray-circle intersection against
/// all entities with `Collider` and `Health`. Finds the closest hit and
/// stores its damage in the `DamageQueue` and hit position in `LaserHitPositions`.
pub fn check_laser_collisions(
    mut laser_reader: MessageReader<LaserFired>,
    config: Res<WeaponConfig>,
    colliders: Query<(Entity, &Transform, &Collider), With<Health>>,
    mut damage_queue: ResMut<DamageQueue>,
    mut laser_hit_positions: ResMut<LaserHitPositions>,
) {
    for laser in laser_reader.read() {
        let mut closest_hit: Option<Entity> = None;
        let mut closest_hit_point: Option<Vec2> = None;
        let mut closest_distance = laser.range;

        for (entity, transform, collider) in colliders.iter() {
            let center = Vec2::new(transform.translation.x, transform.translation.y);

            if let Some(hit_point) = ray_circle_intersection(
                laser.origin,
                laser.direction,
                laser.range,
                center,
                collider.radius,
            ) {
                let distance = laser.origin.distance(hit_point);
                if distance < closest_distance {
                    closest_distance = distance;
                    closest_hit = Some(entity);
                    closest_hit_point = Some(hit_point);
                }
            }
        }

        // Apply damage to closest hit and record hit position
        if let Some(entity) = closest_hit {
            damage_queue.entries.push((entity, config.laser_damage));
            if let Some(hit_point) = closest_hit_point {
                laser_hit_positions.positions.push(hit_point);
            }
        }
    }
}

/// Checks circle-circle intersection between `SpreadProjectile` entities
/// and entities with `Collider` + `Health`. Stores damage in `DamageQueue`
/// and despawns projectiles on hit.
pub fn check_projectile_collisions(
    mut commands: Commands,
    projectiles: Query<(Entity, &Transform, &SpreadProjectile)>,
    colliders: Query<(Entity, &Transform, &Collider), With<Health>>,
    mut damage_queue: ResMut<DamageQueue>,
) {
    for (proj_entity, proj_transform, projectile) in projectiles.iter() {
        let proj_center = Vec2::new(
            proj_transform.translation.x,
            proj_transform.translation.y,
        );

        for (target_entity, target_transform, collider) in colliders.iter() {
            let target_center = Vec2::new(
                target_transform.translation.x,
                target_transform.translation.y,
            );

            if circle_circle_intersection(
                proj_center,
                PROJECTILE_RADIUS,
                target_center,
                collider.radius,
            ) {
                damage_queue.entries.push((target_entity, projectile.damage));
                commands.entity(proj_entity).despawn();
                break; // Projectile can only hit one target
            }
        }
    }
}

/// Applies accumulated damage from `DamageQueue` to `Health` components.
/// Allows multiple hits to the same entity in one frame (no invincibility).
/// Inserts `JustDamaged` component on entities that receive damage for visual feedback.
pub fn apply_damage(
    mut commands: Commands,
    mut query: Query<&mut Health>,
    mut damage_queue: ResMut<DamageQueue>,
) {
    for (entity, amount) in damage_queue.entries.drain(..) {
        if let Ok(mut health) = query.get_mut(entity) {
            health.current = (health.current - amount).max(0.0);
            commands.entity(entity).insert(JustDamaged { amount });
        }
    }
}

/// Despawns entities whose health has reached zero or below.
/// Records positions of destroyed entities in `DestroyedPositions` for visual effects.
pub fn despawn_destroyed(
    mut commands: Commands,
    query: Query<(Entity, &Health, &Transform)>,
    mut destroyed_positions: ResMut<DestroyedPositions>,
) {
    for (entity, health, transform) in query.iter() {
        if health.current <= 0.0 {
            let position = Vec2::new(transform.translation.x, transform.translation.y);
            destroyed_positions.positions.push(position);
            commands.entity(entity).despawn();
        }
    }
}

// ── Unit tests ──────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    // ── ray_circle_intersection ──

    #[test]
    fn ray_hits_circle() {
        // Ray fires right, circle at (50, 0), radius 10
        let hit = ray_circle_intersection(
            Vec2::ZERO,
            Vec2::X,
            100.0,
            Vec2::new(50.0, 0.0),
            10.0,
        );
        assert!(hit.is_some(), "Ray should hit the circle");
        let point = hit.expect("checked above");
        // Hit point should be approximately at (50, 0) ± radius
        assert!((point.x - 50.0).abs() < 10.1);
    }

    #[test]
    fn ray_misses_circle() {
        // Ray fires right, circle at (50, 100), radius 10 — too far off-axis
        let hit = ray_circle_intersection(
            Vec2::ZERO,
            Vec2::X,
            100.0,
            Vec2::new(50.0, 100.0),
            10.0,
        );
        assert!(hit.is_none(), "Ray should miss the circle");
    }

    #[test]
    fn ray_stops_before_circle() {
        // Ray fires right with range 30, circle at (50, 0), radius 10
        let hit = ray_circle_intersection(
            Vec2::ZERO,
            Vec2::X,
            30.0,
            Vec2::new(50.0, 0.0),
            10.0,
        );
        assert!(hit.is_none(), "Ray should stop before reaching the circle");
    }

    // ── circle_circle_intersection ──

    #[test]
    fn circles_overlap() {
        // Two circles close together
        let result = circle_circle_intersection(
            Vec2::ZERO,
            10.0,
            Vec2::new(15.0, 0.0),
            10.0,
        );
        assert!(result, "Circles should overlap");
    }

    #[test]
    fn circles_do_not_overlap() {
        // Two circles far apart
        let result = circle_circle_intersection(
            Vec2::ZERO,
            10.0,
            Vec2::new(100.0, 0.0),
            10.0,
        );
        assert!(!result, "Circles should not overlap");
    }
}

