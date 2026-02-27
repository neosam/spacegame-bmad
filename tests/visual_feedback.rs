mod helpers;

use bevy::prelude::*;
use helpers::{spawn_asteroid, spawn_player, test_app};
use void_drifter::core::collision::{
    apply_damage, despawn_destroyed, DamageQueue, DestroyedPositions, LaserHitPositions,
};
use void_drifter::rendering::effects::ScreenShake;
use void_drifter::shared::components::JustDamaged;

#[test]
fn apply_damage_inserts_just_damaged_component() {
    let mut app = test_app();
    app.init_resource::<ScreenShake>();
    
    // Spawn an entity with health
    let entity = spawn_asteroid(&mut app, Vec2::ZERO, 10.0, 100.0);
    
    // Add damage to queue
    let mut damage_queue = app.world_mut().resource_mut::<DamageQueue>();
    damage_queue.entries.push((entity, 10.0));
    drop(damage_queue);
    
    // Run apply_damage
    app.add_systems(FixedUpdate, apply_damage);
    app.update();
    
    // Check that JustDamaged component was inserted
    let just_damaged = app.world().entity(entity).get::<JustDamaged>();
    assert!(just_damaged.is_some(), "JustDamaged component should be inserted");
    assert_eq!(
        just_damaged
            .expect("JustDamaged should be inserted on damaged entity")
            .amount,
        10.0
    );
}

#[test]
fn destroyed_entity_position_recorded_in_destroyed_positions() {
    let mut app = test_app();
    app.init_resource::<ScreenShake>();
    
    // Spawn an entity at a known position
    let position = Vec2::new(100.0, 200.0);
    let entity = spawn_asteroid(&mut app, position, 10.0, 10.0);
    
    // Deal enough damage to destroy it
    let mut damage_queue = app.world_mut().resource_mut::<DamageQueue>();
    damage_queue.entries.push((entity, 20.0));
    drop(damage_queue);
    
    // Run damage and despawn systems
    app.add_systems(FixedUpdate, (apply_damage, despawn_destroyed).chain());
    app.update();
    
    // Check that position was recorded
    let destroyed_positions = app.world().resource::<DestroyedPositions>();
    assert!(!destroyed_positions.positions.is_empty(), "Destroyed position should be recorded");
    let recorded_pos = destroyed_positions.positions[0];
    assert!((recorded_pos.x - position.x).abs() < 0.1);
    assert!((recorded_pos.y - position.y).abs() < 0.1);
}

/// Helper: position a player at `pos` facing direction `dir`.
fn position_player_facing(app: &mut App, player: Entity, pos: Vec2, dir: Vec2) {
    let angle = dir.y.atan2(dir.x) - std::f32::consts::FRAC_PI_2;
    let mut binding = app.world_mut().entity_mut(player);
    let mut transform = binding
        .get_mut::<Transform>()
        .expect("Player should have Transform");
    transform.translation = pos.extend(0.0);
    transform.rotation = Quat::from_rotation_z(angle);
}

#[test]
fn laser_hit_position_recorded_in_laser_hit_positions() {
    let mut app = test_app();
    app.init_resource::<ScreenShake>();
    
    // Spawn player and target entity
    let player = spawn_player(&mut app);
    let target_pos = Vec2::new(0.0, 50.0);
    let _target = spawn_asteroid(&mut app, target_pos, 10.0, 100.0);
    
    // Position player at origin facing +Y (toward target)
    position_player_facing(&mut app, player, Vec2::ZERO, Vec2::Y);
    
    // Fire laser using ActionState
    app.world_mut().resource_mut::<void_drifter::core::input::ActionState>().fire = true;
    
    // Run full update cycle (fire_weapon → check_laser_collisions)
    app.update();
    
    // Check that hit position was recorded
    let laser_hit_positions = app.world().resource::<LaserHitPositions>();
    assert!(!laser_hit_positions.positions.is_empty(), "Laser hit position should be recorded");
}

#[test]
fn screen_shake_trauma_increases_when_player_has_just_damaged() {
    let mut app = test_app();
    app.init_resource::<ScreenShake>();
    app.init_resource::<DestroyedPositions>();
    
    // Spawn player
    let player = spawn_player(&mut app);
    
    // Add JustDamaged component to player
    app.world_mut().entity_mut(player).insert(JustDamaged { amount: 10.0 });
    
    // Get initial trauma
    let initial_trauma = {
        let screen_shake = app.world().resource::<ScreenShake>();
        screen_shake.trauma
    };
    
    // Run trigger_screen_shake
    app.add_systems(Update, void_drifter::rendering::effects::trigger_screen_shake);
    app.update();
    
    // Check that trauma increased
    let screen_shake = app.world().resource::<ScreenShake>();
    assert!(screen_shake.trauma > initial_trauma, "Trauma should increase when player takes damage");
    assert!(screen_shake.trauma >= 0.3, "Trauma should increase by at least 0.3");
}

#[test]
fn screen_shake_trauma_increases_on_nearby_destruction() {
    let mut app = test_app();
    app.init_resource::<ScreenShake>();

    // Spawn player at origin
    let _player = spawn_player(&mut app);

    // Simulate a nearby destruction (within 200 world units of player at origin)
    app.world_mut()
        .resource_mut::<DestroyedPositions>()
        .positions
        .push(Vec2::new(50.0, 0.0));

    // Run trigger_screen_shake
    app.add_systems(Update, void_drifter::rendering::effects::trigger_screen_shake);
    app.update();

    let screen_shake = app.world().resource::<ScreenShake>();
    assert!(
        screen_shake.trauma >= 0.2,
        "Trauma should increase by at least 0.2 for nearby destruction, got {}",
        screen_shake.trauma
    );
}

#[test]
fn screen_shake_no_trauma_from_distant_destruction() {
    let mut app = test_app();
    app.init_resource::<ScreenShake>();

    // Spawn player at origin
    let _player = spawn_player(&mut app);

    // Simulate destruction far away (beyond 200 world units)
    app.world_mut()
        .resource_mut::<DestroyedPositions>()
        .positions
        .push(Vec2::new(500.0, 0.0));

    app.add_systems(Update, void_drifter::rendering::effects::trigger_screen_shake);
    app.update();

    let screen_shake = app.world().resource::<ScreenShake>();
    assert!(
        screen_shake.trauma < 0.01,
        "Trauma should not increase for distant destruction, got {}",
        screen_shake.trauma
    );
}

#[test]
fn damage_flash_swaps_material_to_white_for_non_player() {
    use void_drifter::rendering::effects::{trigger_damage_flash, DamageFlash, FlashMaterials};

    let mut app = App::new();
    app.add_plugins(MinimalPlugins);

    // Create materials
    let mut materials = Assets::<ColorMaterial>::default();
    let original = materials.add(ColorMaterial::from(Color::srgb(0.5, 0.5, 0.5)));
    let white = materials.add(ColorMaterial::from(Color::srgb(1.0, 1.0, 1.0)));
    let red = materials.add(ColorMaterial::from(Color::srgb(1.0, 0.2, 0.2)));
    app.insert_resource(materials);
    app.insert_resource(FlashMaterials {
        white: white.clone(),
        red,
    });

    // Spawn non-player entity with JustDamaged + MeshMaterial2d
    let entity = app
        .world_mut()
        .spawn((
            JustDamaged { amount: 10.0 },
            MeshMaterial2d(original.clone()),
        ))
        .id();

    app.add_systems(Update, trigger_damage_flash);
    app.update();

    // DamageFlash should be inserted with original material stored
    let flash = app
        .world()
        .entity(entity)
        .get::<DamageFlash>()
        .expect("DamageFlash should be inserted on damaged entity");
    assert_eq!(
        flash.original_material, original,
        "Original material should be stored in DamageFlash"
    );

    // Material should be swapped to white (non-player)
    let material = app
        .world()
        .entity(entity)
        .get::<MeshMaterial2d<ColorMaterial>>()
        .expect("Entity should still have MeshMaterial2d");
    assert_eq!(
        material.0, white,
        "Material should be swapped to white for non-player entity"
    );
}

#[test]
fn damage_flash_swaps_material_to_red_for_player() {
    use void_drifter::core::flight::Player;
    use void_drifter::rendering::effects::{trigger_damage_flash, DamageFlash, FlashMaterials};

    let mut app = App::new();
    app.add_plugins(MinimalPlugins);

    // Create materials
    let mut materials = Assets::<ColorMaterial>::default();
    let original = materials.add(ColorMaterial::from(Color::srgb(1.0, 0.85, 0.2)));
    let white = materials.add(ColorMaterial::from(Color::srgb(1.0, 1.0, 1.0)));
    let red = materials.add(ColorMaterial::from(Color::srgb(1.0, 0.2, 0.2)));
    app.insert_resource(materials);
    app.insert_resource(FlashMaterials {
        white,
        red: red.clone(),
    });

    // Spawn player entity with JustDamaged + MeshMaterial2d
    let entity = app
        .world_mut()
        .spawn((
            Player,
            JustDamaged { amount: 15.0 },
            MeshMaterial2d(original.clone()),
        ))
        .id();

    app.add_systems(Update, trigger_damage_flash);
    app.update();

    // DamageFlash should be inserted
    let flash = app
        .world()
        .entity(entity)
        .get::<DamageFlash>()
        .expect("DamageFlash should be inserted on damaged player");
    assert_eq!(
        flash.original_material, original,
        "Original material should be stored in DamageFlash"
    );

    // Material should be swapped to red (player)
    let material = app
        .world()
        .entity(entity)
        .get::<MeshMaterial2d<ColorMaterial>>()
        .expect("Player should still have MeshMaterial2d");
    assert_eq!(
        material.0, red,
        "Material should be swapped to red for player entity"
    );
}

