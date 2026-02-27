use bevy::prelude::*;

use crate::core::collision::{DestroyedPositions, LaserHitPositions};
use crate::core::flight::Player;
use crate::shared::components::JustDamaged;

/// Camera shake using trauma model.
/// Offset = max_offset * trauma² * oscillation_direction.
/// Trauma decays linearly over time.
#[derive(Resource)]
pub struct ScreenShake {
    pub trauma: f32,      // 0.0–1.0, accumulated from damage/destruction
    pub max_offset: f32,  // Maximum camera offset in world units (default: 8.0)
    pub decay_rate: f32,  // Trauma decay per second (default: 3.0)
}

impl Default for ScreenShake {
    fn default() -> Self {
        Self {
            trauma: 0.0,
            max_offset: 8.0,
            decay_rate: 3.0,
        }
    }
}

/// Adds trauma to screen shake when player takes damage or nearby destruction occurs.
pub fn trigger_screen_shake(
    player_query: Query<&Transform, With<Player>>,
    damaged_query: Query<Entity, (With<Player>, With<JustDamaged>)>,
    destroyed_positions: Res<DestroyedPositions>,
    mut screen_shake: ResMut<ScreenShake>,
) {
    // Check if player has JustDamaged component (+0.3 trauma)
    if damaged_query.single().is_ok() {
        screen_shake.trauma = (screen_shake.trauma + 0.3).min(1.0);
    }
    
    // Check for nearby destruction (+0.2 trauma within 200 world units)
    if let Ok(player_transform) = player_query.single() {
        let player_pos = Vec2::new(player_transform.translation.x, player_transform.translation.y);
        const PROXIMITY_THRESHOLD: f32 = 200.0;
        
        for &destroyed_pos in destroyed_positions.positions.iter() {
            let distance = player_pos.distance(destroyed_pos);
            if distance <= PROXIMITY_THRESHOLD {
                screen_shake.trauma = (screen_shake.trauma + 0.2).min(1.0);
                break; // Only add trauma once per frame for nearby destruction
            }
        }
    }
}

/// Applies screen shake to camera transform in PostUpdate (after camera_follow_player).
pub fn apply_screen_shake(
    time: Res<Time>,
    mut screen_shake: ResMut<ScreenShake>,
    mut camera_query: Query<&mut Transform, With<Camera2d>>,
) {
    let Ok(mut camera_transform) = camera_query.single_mut() else {
        return;
    };

    // Calculate shake amount using quadratic formula for better feel
    let shake_amount = screen_shake.trauma * screen_shake.trauma;
    
    // Use time-based oscillation for pseudo-random shake direction
    let t = time.elapsed_secs();
    let offset_x = (t * 113.0).sin() * shake_amount * screen_shake.max_offset;
    let offset_y = (t * 191.7).cos() * shake_amount * screen_shake.max_offset;
    
    camera_transform.translation.x += offset_x;
    camera_transform.translation.y += offset_y;
    
    // Decay trauma
    let dt = time.delta_secs();
    screen_shake.trauma = (screen_shake.trauma - screen_shake.decay_rate * dt).max(0.0);
}

// ── Damage Flash System ────────────────────────────────────────────────────

/// Pre-created flash materials, initialized once at startup.
#[derive(Resource)]
pub struct FlashMaterials {
    pub white: Handle<ColorMaterial>,  // Damage dealt (enemy hit)
    pub red: Handle<ColorMaterial>,    // Damage taken (player hit)
}

/// Active flash state on an entity. Stores original material for restoration.
#[derive(Component)]
pub struct DamageFlash {
    pub timer: f32,
    pub original_material: Handle<ColorMaterial>,
}

const FLASH_DURATION: f32 = 0.1;

/// Initialize flash materials at startup.
pub fn setup_flash_materials(
    mut commands: Commands,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let white = materials.add(ColorMaterial::from(Color::srgb(1.0, 1.0, 1.0)));
    let red = materials.add(ColorMaterial::from(Color::srgb(1.0, 0.2, 0.2)));
    commands.insert_resource(FlashMaterials { white, red });
}

/// Triggers damage flash on entities with JustDamaged component.
pub fn trigger_damage_flash(
    mut commands: Commands,
    flash_materials: Res<FlashMaterials>,
    damaged_query: Query<(Entity, &JustDamaged, &MeshMaterial2d<ColorMaterial>), Without<DamageFlash>>,
    player_query: Query<Entity, With<Player>>,
) {
    let player_entity = player_query.single().ok();
    
    for (entity, _just_damaged, material) in damaged_query.iter() {
        let is_player = player_entity == Some(entity);
        let flash_material = if is_player {
            flash_materials.red.clone()
        } else {
            flash_materials.white.clone()
        };
        
        commands.entity(entity).insert(DamageFlash {
            timer: FLASH_DURATION,
            original_material: material.0.clone(),
        });
        commands.entity(entity).insert(MeshMaterial2d(flash_material));
        commands.entity(entity).remove::<JustDamaged>();
    }
}

/// Removes JustDamaged from entities without MeshMaterial2d (graceful skip).
pub fn remove_just_damaged_without_material(
    mut commands: Commands,
    damaged_query: Query<Entity, (With<JustDamaged>, Without<MeshMaterial2d<ColorMaterial>>)>,
) {
    for entity in damaged_query.iter() {
        commands.entity(entity).remove::<JustDamaged>();
    }
}

/// Updates damage flash timers and restores original materials.
pub fn update_damage_flash(
    time: Res<Time>,
    mut commands: Commands,
    mut flash_query: Query<(Entity, &mut DamageFlash, &MeshMaterial2d<ColorMaterial>)>,
) {
    let dt = time.delta_secs();
    
    for (entity, mut flash, _current_material) in flash_query.iter_mut() {
        flash.timer -= dt;
        
        if flash.timer <= 0.0 {
            // Restore original material
            commands.entity(entity).insert(MeshMaterial2d(flash.original_material.clone()));
            commands.entity(entity).remove::<DamageFlash>();
        }
    }
}

// ── Destruction Effect System ──────────────────────────────────────────────

/// Brief expanding visual burst at destroyed entity position.
#[derive(Component)]
pub struct DestructionEffect {
    pub timer: f32,
    pub max_lifetime: f32,
}

const DESTRUCTION_LIFETIME: f32 = 0.3;

/// Pre-created destruction effect assets.
#[derive(Resource)]
pub struct DestructionAssets {
    pub mesh: Handle<Mesh>,
    pub material: Handle<ColorMaterial>,
}

/// Initialize destruction assets at startup.
pub fn setup_destruction_assets(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    // Create a small circle mesh for destruction effect
    let mesh = meshes.add(Circle::new(5.0));
    let material = materials.add(ColorMaterial::from(Color::srgb(1.0, 0.7, 0.1)));
    commands.insert_resource(DestructionAssets { mesh, material });
}

/// Spawns destruction effects at positions from DestroyedPositions.
pub fn spawn_destruction_effects(
    mut commands: Commands,
    destruction_assets: Res<DestructionAssets>,
    mut destroyed_positions: ResMut<DestroyedPositions>,
) {
    for position in destroyed_positions.positions.drain(..) {
        commands.spawn((
            DestructionEffect {
                timer: DESTRUCTION_LIFETIME,
                max_lifetime: DESTRUCTION_LIFETIME,
            },
            Mesh2d(destruction_assets.mesh.clone()),
            MeshMaterial2d(destruction_assets.material.clone()),
            Transform::from_translation(position.extend(0.0)),
        ));
    }
}

/// Updates destruction effects: expands scale and despawns when expired.
pub fn update_destruction_effects(
    time: Res<Time>,
    mut commands: Commands,
    mut effect_query: Query<(Entity, &mut DestructionEffect, &mut Transform)>,
) {
    let dt = time.delta_secs();
    
    for (entity, mut effect, mut transform) in effect_query.iter_mut() {
        effect.timer -= dt;
        
        // Expand scale linearly from 1x to 5x over lifetime
        let progress = 1.0 - (effect.timer / effect.max_lifetime);
        let scale = 1.0 + (progress * 4.0); // 1.0 → 5.0
        transform.scale = Vec3::splat(scale);
        
        if effect.timer <= 0.0 {
            commands.entity(entity).despawn();
        }
    }
}

// ── Laser Impact Flash ────────────────────────────────────────────────────

/// Brief flash at laser hit position.
#[derive(Component)]
pub struct ImpactFlash {
    pub timer: f32,
}

const IMPACT_FLASH_LIFETIME: f32 = 0.08;

/// Pre-created impact flash assets, initialized once at startup.
#[derive(Resource)]
pub struct ImpactFlashAssets {
    pub mesh: Handle<Mesh>,
    pub material: Handle<ColorMaterial>,
}

/// Initialize impact flash assets at startup.
pub fn setup_impact_flash_assets(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let mesh = meshes.add(Circle::new(3.0));
    let material = materials.add(ColorMaterial::from(Color::srgb(0.4, 0.9, 1.0)));
    commands.insert_resource(ImpactFlashAssets { mesh, material });
}

/// Spawns laser impact flashes at positions from LaserHitPositions.
pub fn spawn_laser_impact_flash(
    mut commands: Commands,
    impact_flash_assets: Res<ImpactFlashAssets>,
    mut laser_hit_positions: ResMut<LaserHitPositions>,
) {
    for position in laser_hit_positions.positions.drain(..) {
        commands.spawn((
            ImpactFlash {
                timer: IMPACT_FLASH_LIFETIME,
            },
            Mesh2d(impact_flash_assets.mesh.clone()),
            MeshMaterial2d(impact_flash_assets.material.clone()),
            Transform::from_translation(position.extend(0.0)),
        ));
    }
}

/// Updates impact flashes and despawns when expired.
pub fn update_impact_flashes(
    time: Res<Time>,
    mut commands: Commands,
    mut flash_query: Query<(Entity, &mut ImpactFlash)>,
) {
    let dt = time.delta_secs();
    
    for (entity, mut flash) in flash_query.iter_mut() {
        flash.timer -= dt;
        
        if flash.timer <= 0.0 {
            commands.entity(entity).despawn();
        }
    }
}

// ── Unit tests ──────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::time::TimeUpdateStrategy;
    use std::time::Duration;

    #[test]
    fn screen_shake_trauma_decays_over_time() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.init_resource::<ScreenShake>();
        app.insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_secs_f64(0.1)));
        app.insert_resource(Time::<Fixed>::from_seconds(0.1));
        
        // Set initial trauma
        let mut screen_shake = app.world_mut().resource_mut::<ScreenShake>();
        screen_shake.trauma = 1.0;
        drop(screen_shake);
        
        // Prime time
        app.update();
        
        // Run apply_screen_shake
        let _camera = app.world_mut().spawn(Camera2d);
        app.add_systems(Update, apply_screen_shake);
        
        // Advance time
        app.update();
        
        let screen_shake = app.world().resource::<ScreenShake>();
        assert!(screen_shake.trauma < 1.0, "Trauma should decay");
        assert!(screen_shake.trauma >= 0.0, "Trauma should not go negative");
    }

    #[test]
    fn screen_shake_trauma_clamps_at_1_0() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.init_resource::<ScreenShake>();
        
        let mut screen_shake = app.world_mut().resource_mut::<ScreenShake>();
        screen_shake.trauma = 0.8;
        drop(screen_shake);
        
        // Trigger screen shake with +0.3 trauma (would exceed 1.0)
        let player = app.world_mut().spawn((Player, Transform::default())).id();
        app.world_mut().entity_mut(player).insert(JustDamaged { amount: 10.0 });
        app.init_resource::<DestroyedPositions>();
        app.add_systems(Update, trigger_screen_shake);
        
        app.update();
        
        let screen_shake = app.world().resource::<ScreenShake>();
        assert!(screen_shake.trauma <= 1.0, "Trauma should clamp at 1.0");
    }

    #[test]
    fn damage_flash_timer_decrements_correctly() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_secs_f64(0.05)));
        app.insert_resource(Time::<Update>::default());
        
        let mut materials = Assets::<ColorMaterial>::default();
        let original_material = materials.add(ColorMaterial::from(Color::srgb(0.5, 0.5, 0.5)));
        let flash_material = materials.add(ColorMaterial::from(Color::srgb(1.0, 1.0, 1.0)));
        app.insert_resource(materials);
        
        let entity = app.world_mut().spawn((
            DamageFlash {
                timer: 0.1,
                original_material: original_material.clone(),
            },
            MeshMaterial2d(flash_material),
        )).id();
        
        app.add_systems(Update, update_damage_flash);
        app.update(); // Prime
        app.update(); // Advance time (0.05s)
        
        let flash = app.world().entity(entity).get::<DamageFlash>();
        assert!(flash.is_some(), "Flash should still exist");
        let flash = flash.expect("Flash should still exist after partial decay");
        assert!(flash.timer < 0.1, "Timer should have decremented");
        assert!(flash.timer > 0.0, "Timer should not have expired yet");
    }
}

