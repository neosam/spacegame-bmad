use bevy::ecs::message::{MessageReader, MessageWriter};
use bevy::prelude::*;

use crate::infrastructure::events::EventSeverityConfig;
use crate::shared::components::{ContactDamageCooldown, Invincible, JustDamaged, Velocity};
use crate::shared::events::{GameEvent, GameEventKind};
use super::flight::Player;
use super::spawning::{Asteroid, BossEnemy, Fighter, HeavyCruiser, ScoutDrone, Sniper};
use super::weapons::{EnemyProjectile, LaserFired, SpreadProjectile, WeaponConfig};
use crate::core::tutorial::GravityWellGenerator;

/// Radius used for spread projectile collision checks.
pub const PROJECTILE_RADIUS: f32 = 2.0;

// ── Collision math ──────────────────────────────────────────────────────

/// AABB pre-filter slop: extra padding added to bounding box checks.
/// Conservative value to avoid false negatives on edge cases.
const AABB_SLOP: f32 = 50.0;

/// Returns true if `pos_a` and `pos_b` are close enough that a collision is possible.
/// Used as a cheap pre-filter before more expensive intersection math.
#[inline]
pub fn aabb_prefilter(pos_a: Vec2, radius_a: f32, pos_b: Vec2, radius_b: f32) -> bool {
    let max_dist = radius_a + radius_b + AABB_SLOP;
    // Use squared distance to avoid sqrt when possible
    pos_a.distance_squared(pos_b) <= max_dist * max_dist
}

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

/// Health pool for entities that can take damage (player, asteroids, enemies).
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
#[allow(clippy::type_complexity)]
pub fn check_laser_collisions(
    mut laser_reader: MessageReader<LaserFired>,
    config: Res<WeaponConfig>,
    colliders: Query<(Entity, &Transform, &Collider), (With<Health>, Without<Player>, Without<GravityWellGenerator>)>,
    mut damage_queue: ResMut<DamageQueue>,
    mut laser_hit_positions: ResMut<LaserHitPositions>,
) {
    for laser in laser_reader.read() {
        let mut closest_hit: Option<Entity> = None;
        let mut closest_hit_point: Option<Vec2> = None;
        let mut closest_distance = laser.range;

        for (entity, transform, collider) in colliders.iter() {
            let center = Vec2::new(transform.translation.x, transform.translation.y);

            // AABB pre-filter: skip if entity is definitely outside laser range
            if !aabb_prefilter(laser.origin, laser.range + collider.radius, center, 0.0) {
                continue;
            }

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
#[allow(clippy::type_complexity)]
pub fn check_projectile_collisions(
    mut commands: Commands,
    projectiles: Query<(Entity, &Transform, &SpreadProjectile)>,
    colliders: Query<(Entity, &Transform, &Collider), (With<Health>, Without<Player>)>,
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

            // AABB pre-filter: skip if projectile is far from target
            if !aabb_prefilter(proj_center, PROJECTILE_RADIUS, target_center, collider.radius) {
                continue;
            }

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

/// Checks enemy projectile collisions against the player only.
/// Enemy shots cannot hit other enemies.
pub fn check_enemy_projectile_collisions(
    mut commands: Commands,
    projectiles: Query<(Entity, &Transform, &EnemyProjectile)>,
    player: Query<(Entity, &Transform, &Collider), With<Player>>,
    mut damage_queue: ResMut<DamageQueue>,
) {
    let Ok((player_entity, player_transform, player_collider)) = player.single() else {
        return;
    };
    let player_pos = Vec2::new(player_transform.translation.x, player_transform.translation.y);
    for (proj_entity, proj_transform, proj) in projectiles.iter() {
        let proj_pos = Vec2::new(proj_transform.translation.x, proj_transform.translation.y);
        // AABB pre-filter: skip if projectile is too far from player
        if !aabb_prefilter(proj_pos, PROJECTILE_RADIUS, player_pos, player_collider.radius) {
            continue;
        }
        if circle_circle_intersection(proj_pos, PROJECTILE_RADIUS, player_pos, player_collider.radius) {
            damage_queue.entries.push((player_entity, proj.damage));
            commands.entity(proj_entity).despawn();
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

/// Contact damage constant — damage per body-collision hit.
pub const CONTACT_DAMAGE: f32 = 20.0;

/// Contact damage cooldown duration in seconds.
const CONTACT_COOLDOWN: f32 = 0.5;

/// Invincibility duration after respawn in seconds.
pub const INVINCIBILITY_DURATION: f32 = 2.0;

/// Checks circle-circle intersection between Player and all non-Player colliders.
/// Pushes contact damage to DamageQueue and inserts ContactDamageCooldown on hit.
/// Skipped if player has Invincible or ContactDamageCooldown.
#[allow(clippy::type_complexity)]
pub fn check_contact_collisions(
    mut commands: Commands,
    player_query: Query<
        (Entity, &Transform, &Collider),
        (With<Player>, Without<Invincible>, Without<ContactDamageCooldown>),
    >,
    targets: Query<(&Transform, &Collider), (With<Health>, Without<Player>)>,
    mut damage_queue: ResMut<DamageQueue>,
) {
    let Ok((player_entity, player_transform, player_collider)) = player_query.single() else {
        return;
    };

    let player_pos = Vec2::new(
        player_transform.translation.x,
        player_transform.translation.y,
    );

    for (target_transform, target_collider) in targets.iter() {
        let target_pos = Vec2::new(
            target_transform.translation.x,
            target_transform.translation.y,
        );

        // AABB pre-filter: skip if target is too far for contact collision
        if !aabb_prefilter(player_pos, player_collider.radius, target_pos, target_collider.radius) {
            continue;
        }

        if circle_circle_intersection(
            player_pos,
            player_collider.radius,
            target_pos,
            target_collider.radius,
        ) {
            damage_queue
                .entries
                .push((player_entity, CONTACT_DAMAGE));
            commands
                .entity(player_entity)
                .insert(ContactDamageCooldown {
                    timer: CONTACT_COOLDOWN,
                });
            break; // Only one contact damage per frame
        }
    }
}

/// Ticks down ContactDamageCooldown timer and removes it when expired.
pub fn tick_contact_cooldown(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut ContactDamageCooldown)>,
) {
    let dt = time.delta_secs();
    for (entity, mut cooldown) in query.iter_mut() {
        cooldown.timer -= dt;
        if cooldown.timer <= 0.0 {
            commands.entity(entity).remove::<ContactDamageCooldown>();
        }
    }
}

/// Handles player death: resets health/position/velocity, triggers destruction effect,
/// grants invincibility, and emits `PlayerDeath` + `PlayerRespawned` events.
/// Runs AFTER apply_damage and BEFORE despawn_destroyed.
/// Respawn priority: WormholeEntrance (if inside arena) > LastDockedStation > Vec3::ZERO.
pub fn handle_player_death(
    mut commands: Commands,
    mut query: Query<(Entity, &mut Health, &mut Transform, &mut Velocity), With<Player>>,
    mut destroyed_positions: ResMut<DestroyedPositions>,
    mut game_events: MessageWriter<GameEvent>,
    time: Res<Time>,
    severity_config: Res<EventSeverityConfig>,
    last_docked: Option<Res<super::station::LastDockedStation>>,
    wormhole_entrance: Option<Res<super::wormhole::WormholeEntrance>>,
) {
    let Ok((entity, mut health, mut transform, mut velocity)) = query.single_mut() else {
        return;
    };

    if health.current <= 0.0 {
        let death_pos = Vec2::new(transform.translation.x, transform.translation.y);

        // Record death position for destruction visual
        destroyed_positions.positions.push(death_pos);

        // Emit PlayerDeath event
        let death_kind = GameEventKind::PlayerDeath;
        game_events.write(GameEvent {
            severity: severity_config.severity_for(&death_kind),
            kind: death_kind,
            position: death_pos,
            game_time: time.elapsed_secs_f64(),
        });

        // Determine respawn position:
        // 1. WormholeEntrance.world_position (if inside arena)
        // 2. LastDockedStation.position (if ever docked)
        // 3. Vec3::ZERO (world origin fallback)
        let respawn_pos = if let Some(entrance) = wormhole_entrance.as_ref() {
            entrance.world_position.extend(0.0)
        } else {
            last_docked
                .as_ref()
                .map(|r| r.position.extend(0.0))
                .unwrap_or(Vec3::ZERO)
        };

        // Reset player state
        health.current = health.max;
        transform.translation = respawn_pos;
        velocity.0 = Vec2::ZERO;

        // Grant invincibility and clear cooldown
        commands.entity(entity).insert(Invincible {
            timer: INVINCIBILITY_DURATION,
        });
        commands
            .entity(entity)
            .remove::<ContactDamageCooldown>();

        // Emit PlayerRespawned event
        let respawn_kind = GameEventKind::PlayerRespawned;
        game_events.write(GameEvent {
            severity: severity_config.severity_for(&respawn_kind),
            kind: respawn_kind,
            position: respawn_pos.truncate(),
            game_time: time.elapsed_secs_f64(),
        });
    }
}

/// Ticks down Invincible timer and removes it when expired.
/// Restores Visibility::Inherited on removal to ensure player is visible.
pub fn tick_invincibility(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut Invincible, Option<&mut Visibility>)>,
) {
    let dt = time.delta_secs();
    for (entity, mut invincible, visibility) in query.iter_mut() {
        invincible.timer -= dt;
        if invincible.timer <= 0.0 {
            commands.entity(entity).remove::<Invincible>();
            if let Some(mut vis) = visibility {
                *vis = Visibility::Inherited;
            }
        }
    }
}

/// Despawns non-Player entities whose health has reached zero or below.
/// Records positions of destroyed entities in `DestroyedPositions` for visual effects.
/// Emits `GameEvent::EnemyDestroyed` for each despawned entity.
/// Emits `GameEvent::BossDestroyed` for Boss entities (Story 7-1).
/// Player entity is NEVER despawned — handled by `handle_player_death` instead.
#[allow(clippy::type_complexity)]
pub fn despawn_destroyed(
    mut commands: Commands,
    query: Query<
        (
            Entity,
            &Health,
            &Transform,
            Option<&Asteroid>,
            Option<&ScoutDrone>,
            Option<&Fighter>,
            Option<&HeavyCruiser>,
            Option<&Sniper>,
            Option<&BossEnemy>,
        ),
        Without<Player>,
    >,
    mut destroyed_positions: ResMut<DestroyedPositions>,
    mut game_events: MessageWriter<GameEvent>,
    time: Res<Time>,
    severity_config: Res<EventSeverityConfig>,
) {
    for (entity, health, transform, asteroid, drone, fighter, heavy, sniper, boss) in query.iter() {
        if health.current <= 0.0 {
            let position = Vec2::new(transform.translation.x, transform.translation.y);
            destroyed_positions.positions.push(position);
            commands.entity(entity).despawn();

            // Boss entities get a BossDestroyed event; others get EnemyDestroyed
            if boss.is_some() {
                use crate::social::faction::FactionId;
                let kind = GameEventKind::BossDestroyed {
                    faction: FactionId::RogueDrones, // Default faction for bosses
                    position,
                };
                game_events.write(GameEvent {
                    severity: severity_config.severity_for(&kind),
                    kind,
                    position,
                    game_time: time.elapsed_secs_f64(),
                });
            } else {
                let entity_type = if asteroid.is_some() {
                    "asteroid"
                } else if drone.is_some() {
                    "drone"
                } else if fighter.is_some() {
                    "fighter"
                } else if heavy.is_some() {
                    "heavy_cruiser"
                } else if sniper.is_some() {
                    "sniper"
                } else {
                    "unknown"
                };
                let kind = GameEventKind::EnemyDestroyed { entity_type };
                game_events.write(GameEvent {
                    severity: severity_config.severity_for(&kind),
                    kind,
                    position,
                    game_time: time.elapsed_secs_f64(),
                });
            }
        }
    }
}

// ── Unit tests ──────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::infrastructure::events::EventSeverityConfig;
    use crate::shared::events::GameEvent;
    use bevy::time::TimeUpdateStrategy;

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

    // ── LastDockedStation respawn ──

    #[test]
    fn player_respawns_at_last_docked_station() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.init_resource::<DestroyedPositions>();
        app.add_message::<GameEvent>();
        app.insert_resource(EventSeverityConfig::default());
        // Insert a LastDockedStation at a non-zero position
        app.insert_resource(crate::core::station::LastDockedStation {
            position: Vec2::new(300.0, 400.0),
        });
        app.add_systems(Update, handle_player_death);

        let entity = app
            .world_mut()
            .spawn((
                Player,
                Health { current: 0.0, max: 100.0 },
                Transform::from_translation(Vec3::new(50.0, 50.0, 0.0)),
                Velocity(Vec2::new(5.0, 5.0)),
            ))
            .id();

        app.update();

        let transform = app
            .world()
            .entity(entity)
            .get::<Transform>()
            .expect("Player should have Transform after death");
        assert!(
            (transform.translation.x - 300.0).abs() < f32::EPSILON,
            "Respawn x should be 300.0, got {}",
            transform.translation.x
        );
        assert!(
            (transform.translation.y - 400.0).abs() < f32::EPSILON,
            "Respawn y should be 400.0, got {}",
            transform.translation.y
        );
        assert!(
            transform.translation.z.abs() < f32::EPSILON,
            "Respawn z should be 0.0, got {}",
            transform.translation.z
        );
    }

    #[test]
    fn player_respawns_at_origin_if_never_docked() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.init_resource::<DestroyedPositions>();
        app.add_message::<GameEvent>();
        app.insert_resource(EventSeverityConfig::default());
        // No LastDockedStation inserted — Option<Res<LastDockedStation>> will be None
        app.add_systems(Update, handle_player_death);

        let entity = app
            .world_mut()
            .spawn((
                Player,
                Health { current: 0.0, max: 100.0 },
                Transform::from_translation(Vec3::new(50.0, 50.0, 0.0)),
                Velocity(Vec2::new(5.0, 5.0)),
            ))
            .id();

        app.update();

        let transform = app
            .world()
            .entity(entity)
            .get::<Transform>()
            .expect("Player should have Transform after death");
        assert!(
            transform.translation.distance(Vec3::ZERO) < f32::EPSILON,
            "Respawn should be at Vec3::ZERO when never docked, got {:?}",
            transform.translation
        );
    }

    // ── contact damage ──

    #[test]
    fn contact_damage_pushes_to_damage_queue() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.init_resource::<DamageQueue>();

        // Player at origin with radius 12
        app.world_mut().spawn((
            Player,
            Transform::default(),
            Collider { radius: 12.0 },
        ));

        // Asteroid overlapping player at (10, 0) with radius 10 — distance 10 < 12+10=22
        app.world_mut().spawn((
            Transform::from_translation(Vec3::new(10.0, 0.0, 0.0)),
            Collider { radius: 10.0 },
            Health { current: 50.0, max: 50.0 },
        ));

        app.add_systems(Update, check_contact_collisions);
        app.update();

        let damage_queue = app.world().resource::<DamageQueue>();
        assert_eq!(
            damage_queue.entries.len(),
            1,
            "Should have one damage entry"
        );
        assert!(
            (damage_queue.entries[0].1 - CONTACT_DAMAGE).abs() < f32::EPSILON,
            "Damage should be CONTACT_DAMAGE"
        );
    }

    #[test]
    fn contact_damage_cooldown_prevents_damage() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.init_resource::<DamageQueue>();

        // Player with active cooldown
        app.world_mut().spawn((
            Player,
            Transform::default(),
            Collider { radius: 12.0 },
            ContactDamageCooldown { timer: 0.5 },
        ));

        // Overlapping asteroid
        app.world_mut().spawn((
            Transform::from_translation(Vec3::new(10.0, 0.0, 0.0)),
            Collider { radius: 10.0 },
            Health { current: 50.0, max: 50.0 },
        ));

        app.add_systems(Update, check_contact_collisions);
        app.update();

        let damage_queue = app.world().resource::<DamageQueue>();
        assert!(
            damage_queue.entries.is_empty(),
            "No damage while cooldown active"
        );
    }

    // ── player death ──

    fn setup_player_death_app() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.init_resource::<DestroyedPositions>();
        app.add_message::<GameEvent>();
        app.insert_resource(EventSeverityConfig::default());
        app.add_systems(Update, handle_player_death);
        app
    }

    fn spawn_dead_player(app: &mut App) -> Entity {
        app.world_mut()
            .spawn((
                Player,
                Health {
                    current: 0.0,
                    max: 100.0,
                },
                Transform::from_translation(Vec3::new(50.0, 50.0, 0.0)),
                Velocity(Vec2::new(10.0, 20.0)),
            ))
            .id()
    }

    #[test]
    fn player_death_resets_health_to_max() {
        let mut app = setup_player_death_app();
        let entity = spawn_dead_player(&mut app);
        app.update();

        let health = app
            .world()
            .entity(entity)
            .get::<Health>()
            .expect("Player should have Health");
        assert!(
            (health.current - 100.0).abs() < f32::EPSILON,
            "Health should be reset to max"
        );
    }

    #[test]
    fn player_death_resets_position_to_origin() {
        let mut app = setup_player_death_app();
        let entity = spawn_dead_player(&mut app);
        app.update();

        let transform = app
            .world()
            .entity(entity)
            .get::<Transform>()
            .expect("Player should have Transform");
        assert!(
            transform.translation.distance(Vec3::ZERO) < f32::EPSILON,
            "Position should be reset to origin"
        );
    }

    #[test]
    fn player_death_resets_velocity_to_zero() {
        let mut app = setup_player_death_app();
        let entity = spawn_dead_player(&mut app);
        app.update();

        let velocity = app
            .world()
            .entity(entity)
            .get::<Velocity>()
            .expect("Player should have Velocity");
        assert!(
            velocity.0.length() < f32::EPSILON,
            "Velocity should be reset to zero"
        );
    }

    #[test]
    fn player_death_inserts_invincible() {
        let mut app = setup_player_death_app();
        let entity = spawn_dead_player(&mut app);
        app.update();

        let invincible = app.world().entity(entity).get::<Invincible>();
        assert!(invincible.is_some(), "Player should have Invincible after death");
        let invincible = invincible.expect("checked above");
        assert!(
            (invincible.timer - INVINCIBILITY_DURATION).abs() < f32::EPSILON,
            "Invincibility timer should be set to INVINCIBILITY_DURATION"
        );
    }

    // ── contact cooldown ──

    #[test]
    fn contact_cooldown_expires_and_removes_component() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.insert_resource(TimeUpdateStrategy::ManualDuration(
            std::time::Duration::from_secs_f64(1.0 / 60.0),
        ));
        app.insert_resource(Time::<Fixed>::from_seconds(1.0 / 60.0));

        let entity = app
            .world_mut()
            .spawn(ContactDamageCooldown { timer: 0.5 })
            .id();

        app.add_systems(Update, tick_contact_cooldown);
        app.update(); // Prime (dt=0)

        // Run enough frames to exceed 0.5s cooldown (60 frames = ~1s)
        for _ in 0..60 {
            app.update();
        }

        let has_cooldown = app
            .world()
            .entity(entity)
            .contains::<ContactDamageCooldown>();
        assert!(
            !has_cooldown,
            "ContactDamageCooldown should be removed after timer expires"
        );
    }

    // ── invincibility ──

    #[test]
    fn invincibility_timer_ticks_down() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.insert_resource(TimeUpdateStrategy::ManualDuration(
            std::time::Duration::from_secs_f64(0.5),
        ));
        app.insert_resource(Time::<Update>::default());

        let entity = app
            .world_mut()
            .spawn(Invincible { timer: 2.0 })
            .id();

        app.add_systems(Update, tick_invincibility);
        app.update(); // Prime
        app.update(); // Advance time

        let invincible = app
            .world()
            .entity(entity)
            .get::<Invincible>()
            .expect("Invincible should still exist");
        assert!(
            invincible.timer < 2.0,
            "Timer should have decremented, got {}",
            invincible.timer
        );
        assert!(
            invincible.timer > 0.0,
            "Timer should not have expired yet, got {}",
            invincible.timer
        );
    }

    #[test]
    fn invincibility_expiry_restores_visibility() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.insert_resource(TimeUpdateStrategy::ManualDuration(
            std::time::Duration::from_secs_f64(1.0 / 60.0),
        ));
        app.insert_resource(Time::<Fixed>::from_seconds(1.0 / 60.0));

        let entity = app
            .world_mut()
            .spawn((Invincible { timer: 2.0 }, Visibility::Hidden))
            .id();

        app.add_systems(Update, tick_invincibility);
        app.update(); // Prime (dt=0)

        // Run enough frames to exceed 2.0s invincibility (180 frames = ~3s)
        for _ in 0..180 {
            app.update();
        }

        let has_invincible = app.world().entity(entity).contains::<Invincible>();
        assert!(
            !has_invincible,
            "Invincible should be removed after timer expires"
        );

        let vis = app
            .world()
            .entity(entity)
            .get::<Visibility>()
            .expect("Entity should have Visibility");
        assert_eq!(
            *vis,
            Visibility::Inherited,
            "Visibility should be restored to Inherited after invincibility expires"
        );
    }
}

