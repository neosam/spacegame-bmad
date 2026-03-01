#![deny(clippy::unwrap_used)]

mod helpers;

use std::time::Duration;
use bevy::prelude::*;
use bevy::time::TimeUpdateStrategy;
use void_drifter::core::flight::Player;
use void_drifter::core::tutorial::{
    advance_phase_on_wreck_shot, apply_gravity_well, check_generator_destroyed,
    check_tutorial_wave_complete, dock_at_station, generate_tutorial_zone, spawn_tutorial_enemies,
    start_destruction_cascade, tick_cascade_timer, unlock_laser_at_wreck, validate_tutorial_config,
    validate_tutorial_seed, CascadeTimer, GravityWellGenerator, SpreadUnlocked, TutorialConfig,
    TutorialEnemy, TutorialEnemyWave, TutorialPhase, TutorialStation, TutorialWreck, TutorialZone,
    WeaponsLocked, WreckShotState,
};
use void_drifter::core::spawning::{ScoutDrone, SpawningConfig};
use void_drifter::shared::components::JustDamaged;
use void_drifter::shared::components::Velocity;
use void_drifter::core::weapons::{ActiveWeapon, Energy, FireCooldown, LaserPulse};
use void_drifter::core::input::ActionState;
use void_drifter::core::collision::Health;
use void_drifter::world::{ActiveChunks, WorldConfig, BiomeConfig, ChunkCoord,
    ChunkEntityIndex, ChunkLoadState, ExploredChunks, PendingChunks};
use void_drifter::infrastructure::save::delta::WorldDeltas;

/// Create a test app with tutorial spawn system registered BEFORE the prime frame.
fn tutorial_test_app() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(bevy::state::app::StatesPlugin);
    app.init_resource::<ActionState>();
    app.insert_resource(void_drifter::core::flight::FlightConfig::default());
    app.insert_resource(void_drifter::core::weapons::WeaponConfig::default());
    app.insert_resource(WorldConfig::default());
    app.insert_resource(BiomeConfig::default());
    app.init_resource::<ActiveChunks>();
    app.init_resource::<ExploredChunks>();
    app.init_resource::<ChunkEntityIndex>();
    app.init_resource::<PendingChunks>();
    app.init_resource::<ChunkLoadState>();
    app.init_resource::<WorldDeltas>();
    app.insert_resource(TutorialConfig::default());
    app.init_state::<TutorialPhase>();
    app.add_message::<void_drifter::shared::events::GameEvent>();
    app.insert_resource(void_drifter::infrastructure::events::EventSeverityConfig::default());
    app.init_resource::<void_drifter::infrastructure::logbook::Logbook>();

    // Register tutorial spawn as Startup
    app.add_systems(Startup, void_drifter::core::tutorial::spawn_tutorial_zone);

    app.insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_secs_f64(
        1.0 / 60.0,
    )));
    app.insert_resource(Time::<Fixed>::from_seconds(1.0 / 60.0));

    // Prime + run Startup systems
    app.update();

    // Simulate rendering's setup_player: spawn player at tutorial layout position.
    // In production, rendering/mod.rs::setup_player does this; tests must replicate it.
    let player_spawn = {
        let zone = app.world().resource::<void_drifter::core::tutorial::TutorialZone>();
        zone.layout.player_spawn
    };
    app.world_mut().spawn((
        Player,
        Velocity::default(),
        void_drifter::core::collision::Health { current: 100.0, max: 100.0 },
        void_drifter::core::collision::Collider { radius: 12.0 },
        FireCooldown::default(),
        Energy::default(),
        ActiveWeapon::default(),
        Transform::from_translation(player_spawn.extend(0.0)),
    ));

    app
}

#[test]
fn hundred_seed_validation() {
    let config = TutorialConfig::default();
    for seed in 0..100 {
        let result = validate_tutorial_seed(seed, &config);
        assert!(
            result.is_ok(),
            "Seed {seed} failed validation: {:?}",
            result.expect_err("Expected Ok")
        );
    }
}

#[test]
fn tutorial_zone_spawns_correct_entities() {
    let mut app = tutorial_test_app();

    // Should have spawned a player
    let player_count = app
        .world_mut()
        .query_filtered::<Entity, With<Player>>()
        .iter(app.world())
        .count();
    assert_eq!(player_count, 1, "Should spawn exactly one player");

    // Should have spawned a tutorial station
    let station_count = app
        .world_mut()
        .query_filtered::<Entity, With<TutorialStation>>()
        .iter(app.world())
        .count();
    assert_eq!(station_count, 1, "Should spawn exactly one tutorial station");

    // Should have spawned a gravity well generator
    let generator_count = app
        .world_mut()
        .query_filtered::<Entity, With<GravityWellGenerator>>()
        .iter(app.world())
        .count();
    assert_eq!(
        generator_count, 1,
        "Should spawn exactly one gravity well generator"
    );

    // TutorialZone resource should exist
    assert!(
        app.world().get_resource::<TutorialZone>().is_some(),
        "TutorialZone resource should be inserted"
    );
}

#[test]
fn tutorial_phase_starts_as_flying() {
    let mut app = tutorial_test_app();
    let phase = app.world().resource::<State<TutorialPhase>>();
    assert_eq!(
        *phase.get(),
        TutorialPhase::Flying,
        "TutorialPhase should start as Flying"
    );
}

#[test]
fn weapon_firing_blocked_in_flying_phase() {
    let mut app = helpers::test_app();

    // Spawn player with WeaponsLocked
    let player = app
        .world_mut()
        .spawn((
            Player,
            FireCooldown::default(),
            Energy::default(),
            ActiveWeapon::default(),
            WeaponsLocked,
            Transform::default(),
        ))
        .id();

    // Set fire input
    app.world_mut().resource_mut::<ActionState>().fire = true;

    app.update();

    // Should not have spawned any laser pulses
    let laser_count = app
        .world_mut()
        .query_filtered::<Entity, With<LaserPulse>>()
        .iter(app.world())
        .count();
    assert_eq!(
        laser_count, 0,
        "No laser should fire when WeaponsLocked is present"
    );

    // Player cooldown should still be 0 (never fired)
    let cooldown = app
        .world()
        .entity(player)
        .get::<FireCooldown>()
        .expect("Should have FireCooldown");
    assert!(
        cooldown.timer == 0.0,
        "Cooldown should remain 0 when weapons locked"
    );
}

#[test]
fn tutorial_zone_occupies_chunks() {
    let mut app = tutorial_test_app();

    let active_chunks = app.world().resource::<ActiveChunks>();
    let origin_coord = ChunkCoord { x: 0, y: 0 };
    assert!(
        active_chunks.chunks.contains_key(&origin_coord),
        "Tutorial zone should mark origin chunk as active"
    );
}

#[test]
fn tutorial_station_is_defective() {
    let mut app = tutorial_test_app();

    let mut query = app.world_mut().query::<&TutorialStation>();
    let station = query
        .iter(app.world())
        .next()
        .expect("Should have a TutorialStation");
    assert!(station.defective, "Tutorial station should be defective");
}

#[test]
fn tutorial_zone_layout_matches_seed() {
    let mut app = tutorial_test_app();

    let world_config = app.world().resource::<WorldConfig>();
    let seed = world_config.seed;
    let tutorial_config = app.world().resource::<TutorialConfig>();
    let expected_layout = generate_tutorial_zone(seed, tutorial_config);

    let zone = app.world().resource::<TutorialZone>();
    assert!(
        (zone.layout.player_spawn.x - expected_layout.player_spawn.x).abs() < f32::EPSILON,
        "Player spawn should match generated layout"
    );
    assert!(
        (zone.layout.station_position.x - expected_layout.station_position.x).abs() < f32::EPSILON,
        "Station position should match generated layout"
    );
}

#[test]
fn player_spawns_at_tutorial_position() {
    let mut app = tutorial_test_app();

    let zone = app.world().resource::<TutorialZone>();
    let expected_pos = zone.layout.player_spawn;

    let mut query = app.world_mut().query_filtered::<&Transform, With<Player>>();
    let player_transform = query
        .iter(app.world())
        .next()
        .expect("Should have a Player");

    assert!(
        (player_transform.translation.x - expected_pos.x).abs() < f32::EPSILON,
        "Player X should match tutorial layout"
    );
    assert!(
        (player_transform.translation.y - expected_pos.y).abs() < f32::EPSILON,
        "Player Y should match tutorial layout"
    );
}

#[test]
fn generator_has_health_from_config() {
    let mut app = tutorial_test_app();

    let config = app.world().resource::<TutorialConfig>();
    let expected_health = config.generator_health;

    let mut query = app.world_mut().query_filtered::<&Health, With<GravityWellGenerator>>();
    let health = query
        .iter(app.world())
        .next()
        .expect("Generator should have Health");

    assert!(
        (health.current - expected_health).abs() < f32::EPSILON,
        "Generator health should match config"
    );
    assert!(
        (health.max - expected_health).abs() < f32::EPSILON,
        "Generator max health should match config"
    );
}

// ── Gravity Well Integration Tests ──────────────────────────────────

/// Create a minimal app with just gravity well system for integration testing.
fn gravity_well_test_app() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.insert_resource(bevy::time::TimeUpdateStrategy::ManualDuration(
        Duration::from_secs_f64(1.0 / 60.0),
    ));
    app.add_systems(FixedUpdate, apply_gravity_well);
    // Prime
    app.update();
    app
}

#[test]
fn gravity_well_player_inside_safe_radius_no_pull() {
    let mut app = gravity_well_test_app();

    // Generator at origin
    app.world_mut().spawn((
        GravityWellGenerator {
            safe_radius: 2000.0,
            pull_strength: 50.0,
            requires_projectile: true,
        },
        Transform::from_translation(Vec3::ZERO),
    ));

    // Player inside safe_radius
    let player = app
        .world_mut()
        .spawn((
            Player,
            Velocity::default(),
            Transform::from_translation(Vec3::new(500.0, 0.0, 0.0)),
        ))
        .id();

    app.update();

    let vel = app
        .world()
        .entity(player)
        .get::<Velocity>()
        .expect("Player should have Velocity");
    assert!(
        vel.0.length() < f32::EPSILON,
        "Player inside safe_radius should have no pull force, got {:?}",
        vel.0
    );
}

#[test]
fn gravity_well_player_outside_pulled_back_over_frames() {
    let mut app = gravity_well_test_app();

    // Generator at origin
    app.world_mut().spawn((
        GravityWellGenerator {
            safe_radius: 100.0,
            pull_strength: 50.0,
            requires_projectile: true,
        },
        Transform::from_translation(Vec3::ZERO),
    ));

    // Player far outside safe_radius at (500, 0)
    let player = app
        .world_mut()
        .spawn((
            Player,
            Velocity::default(),
            Transform::from_translation(Vec3::new(500.0, 0.0, 0.0)),
        ))
        .id();

    // Run several frames
    for _ in 0..10 {
        app.update();
    }

    let vel = app
        .world()
        .entity(player)
        .get::<Velocity>()
        .expect("Player should have Velocity");
    // After many frames of pull, velocity should be significantly toward generator
    assert!(
        vel.0.x < -10.0,
        "Player should be pulled significantly toward generator, got {:?}",
        vel.0
    );
}

#[test]
fn gravity_well_no_effect_when_generator_despawned() {
    let mut app = gravity_well_test_app();

    // Spawn and then despawn generator
    let gen_entity = app
        .world_mut()
        .spawn((
            GravityWellGenerator {
                safe_radius: 100.0,
                pull_strength: 50.0,
                requires_projectile: true,
            },
            Transform::from_translation(Vec3::ZERO),
        ))
        .id();

    let player = app
        .world_mut()
        .spawn((
            Player,
            Velocity::default(),
            Transform::from_translation(Vec3::new(500.0, 0.0, 0.0)),
        ))
        .id();

    // Despawn generator before update
    app.world_mut().entity_mut(gen_entity).despawn();

    app.update();

    let vel = app
        .world()
        .entity(player)
        .get::<Velocity>()
        .expect("Player should have Velocity");
    assert!(
        vel.0.length() < f32::EPSILON,
        "No force when generator is despawned, got {:?}",
        vel.0
    );
}

#[test]
fn gravity_well_with_full_tutorial_zone() {
    let mut app = tutorial_test_app();

    // Add gravity well system
    app.add_systems(FixedUpdate, apply_gravity_well);

    // Move player far outside safe_radius
    let mut player_query = app
        .world_mut()
        .query_filtered::<(Entity, &mut Transform), With<Player>>();
    let (player_entity, _) = player_query
        .iter(app.world())
        .next()
        .expect("Should have player");

    // Set player position far outside safe_radius (default 2000)
    app.world_mut()
        .entity_mut(player_entity)
        .insert(Velocity::default());
    app.world_mut()
        .entity_mut(player_entity)
        .insert(Transform::from_translation(Vec3::new(3000.0, 0.0, 0.0)));

    app.update();

    let vel = app
        .world()
        .entity(player_entity)
        .get::<Velocity>()
        .expect("Player should have Velocity");
    // Player is 1000 units beyond safe_radius (3000 - 2000), should have pull force
    assert!(
        vel.0.x < 0.0,
        "Player outside tutorial zone safe_radius should be pulled back, got {:?}",
        vel.0
    );
}

// ── Laser at Wreck Integration Tests ────────────────────────────────────

/// Create a minimal app with the advance_phase_on_wreck_shot system for testing.
fn wreck_phase_test_app() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(bevy::state::app::StatesPlugin);
    app.insert_resource(bevy::time::TimeUpdateStrategy::ManualDuration(
        Duration::from_secs_f64(1.0 / 60.0),
    ));
    app.init_state::<TutorialPhase>();
    app.add_systems(FixedUpdate, advance_phase_on_wreck_shot);
    // Prime
    app.update();
    app
}

#[test]
fn tutorial_zone_spawns_wreck_entity() {
    let mut app = tutorial_test_app();

    let wreck_count = app
        .world_mut()
        .query_filtered::<Entity, With<TutorialWreck>>()
        .iter(app.world())
        .count();
    assert_eq!(wreck_count, 1, "Should spawn exactly one tutorial wreck");
}

#[test]
fn tutorial_wreck_spawns_with_shot_state_false() {
    let mut app = tutorial_test_app();

    let mut query = app.world_mut().query::<&WreckShotState>();
    let shot_state = query
        .iter(app.world())
        .next()
        .expect("Should have a WreckShotState");
    assert!(
        !shot_state.has_been_shot,
        "WreckShotState should start as not shot"
    );
}

#[test]
fn wreck_spawns_within_safe_radius_all_seeds() {
    let config = TutorialConfig::default();
    for seed in 0..100 {
        let layout = generate_tutorial_zone(seed, &config);
        let dist = (layout.wreck_position - layout.zone_center).length();
        assert!(
            dist <= config.safe_radius,
            "Seed {seed}: wreck at {dist:.1} exceeds safe_radius {}",
            config.safe_radius
        );
    }
}

#[test]
fn phase_advances_shooting_to_spread_unlocked_when_wreck_hit() {
    let mut app = wreck_phase_test_app();

    // Manually set phase to Shooting
    app.world_mut()
        .resource_mut::<bevy::prelude::NextState<TutorialPhase>>()
        .set(TutorialPhase::Shooting);
    app.update(); // Apply state transition: Flying -> Shooting

    // Spawn wreck with JustDamaged (simulates a laser hit this frame)
    app.world_mut().spawn((
        TutorialWreck,
        WreckShotState { has_been_shot: false },
        JustDamaged { amount: 10.0 },
    ));

    app.update(); // advance_phase_on_wreck_shot runs, sets NextState to SpreadUnlocked
    app.update(); // Apply state transition: Shooting -> SpreadUnlocked

    let phase = app.world().resource::<bevy::prelude::State<TutorialPhase>>();
    assert_eq!(
        *phase.get(),
        TutorialPhase::SpreadUnlocked,
        "Phase should advance to SpreadUnlocked after wreck is hit"
    );
}

#[test]
fn phase_does_not_advance_when_wreck_not_hit() {
    let mut app = wreck_phase_test_app();

    // Manually set phase to Shooting
    app.world_mut()
        .resource_mut::<bevy::prelude::NextState<TutorialPhase>>()
        .set(TutorialPhase::Shooting);
    app.update(); // Apply state transition

    // Spawn wreck WITHOUT JustDamaged (not hit this frame)
    app.world_mut().spawn((
        TutorialWreck,
        WreckShotState { has_been_shot: false },
        // No JustDamaged component
    ));

    app.update();

    let phase = app.world().resource::<bevy::prelude::State<TutorialPhase>>();
    assert_eq!(
        *phase.get(),
        TutorialPhase::Shooting,
        "Phase should remain Shooting when wreck is not hit"
    );
}

#[test]
fn phase_advance_is_idempotent_once_spread_unlocked() {
    let mut app = wreck_phase_test_app();

    // Phase already at SpreadUnlocked
    app.world_mut()
        .resource_mut::<bevy::prelude::NextState<TutorialPhase>>()
        .set(TutorialPhase::SpreadUnlocked);
    app.update(); // Apply state transition

    // Wreck gets hit again (already shot)
    app.world_mut().spawn((
        TutorialWreck,
        WreckShotState { has_been_shot: true },
        JustDamaged { amount: 10.0 },
    ));

    app.update();

    let phase = app.world().resource::<bevy::prelude::State<TutorialPhase>>();
    assert_eq!(
        *phase.get(),
        TutorialPhase::SpreadUnlocked,
        "Phase should remain SpreadUnlocked — idempotent on repeat hits"
    );
}

#[test]
fn wreck_shot_state_set_true_on_first_hit() {
    let mut app = wreck_phase_test_app();

    // Set phase to Shooting
    app.world_mut()
        .resource_mut::<bevy::prelude::NextState<TutorialPhase>>()
        .set(TutorialPhase::Shooting);
    app.update();

    // Spawn wreck with JustDamaged
    let wreck = app
        .world_mut()
        .spawn((
            TutorialWreck,
            WreckShotState { has_been_shot: false },
            JustDamaged { amount: 10.0 },
        ))
        .id();

    app.update();

    let shot_state = app
        .world()
        .entity(wreck)
        .get::<WreckShotState>()
        .expect("Wreck should have WreckShotState");
    assert!(
        shot_state.has_been_shot,
        "WreckShotState should be true after first hit"
    );
}

// ── Enemies After Laser Integration Tests ───────────────────────────────

/// Create a minimal app with spawn_tutorial_enemies and check_tutorial_wave_complete
/// for isolated integration testing of the enemy wave.
fn enemy_wave_test_app() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(bevy::state::app::StatesPlugin);
    app.insert_resource(bevy::time::TimeUpdateStrategy::ManualDuration(
        Duration::from_secs_f64(1.0 / 60.0),
    ));
    app.insert_resource(Time::<Fixed>::from_seconds(1.0 / 60.0));
    app.insert_resource(TutorialConfig::default());
    app.insert_resource(SpawningConfig::default());
    app.init_state::<TutorialPhase>();
    app.add_systems(FixedUpdate, check_tutorial_wave_complete);
    app.add_systems(OnEnter(TutorialPhase::SpreadUnlocked), spawn_tutorial_enemies);
    // Prime
    app.update();
    app
}

#[test]
fn tutorial_enemies_spawn_on_spread_unlocked_phase() {
    let mut app = enemy_wave_test_app();

    // Insert the TutorialZone resource so spawn_tutorial_enemies can read it
    let config = TutorialConfig::default();
    use void_drifter::core::tutorial::{TutorialLayout, TutorialZone};
    app.insert_resource(TutorialZone {
        center: Vec2::ZERO,
        seed: 42,
        layout: TutorialLayout {
            player_spawn: Vec2::new(50.0, 0.0),
            station_position: Vec2::new(300.0, 0.0),
            generator_position: Vec2::new(1700.0, 0.0),
            zone_center: Vec2::ZERO,
            wreck_position: Vec2::new(500.0, 0.0),
        },
    });

    // Transition to SpreadUnlocked — triggers OnEnter(SpreadUnlocked) which calls spawn_tutorial_enemies
    app.world_mut()
        .resource_mut::<bevy::prelude::NextState<TutorialPhase>>()
        .set(TutorialPhase::SpreadUnlocked);
    app.update(); // Apply transition + run OnEnter

    let enemy_count = app
        .world_mut()
        .query_filtered::<Entity, With<TutorialEnemy>>()
        .iter(app.world())
        .count();
    assert_eq!(
        enemy_count,
        config.tutorial_enemy_count,
        "Should spawn {} tutorial enemies, got {}",
        config.tutorial_enemy_count,
        enemy_count
    );
}

#[test]
fn tutorial_enemies_have_correct_components() {
    let mut app = enemy_wave_test_app();

    use void_drifter::core::tutorial::{TutorialLayout, TutorialZone};
    app.insert_resource(TutorialZone {
        center: Vec2::ZERO,
        seed: 42,
        layout: TutorialLayout {
            player_spawn: Vec2::new(50.0, 0.0),
            station_position: Vec2::new(300.0, 0.0),
            generator_position: Vec2::new(1700.0, 0.0),
            zone_center: Vec2::ZERO,
            wreck_position: Vec2::new(500.0, 0.0),
        },
    });

    app.world_mut()
        .resource_mut::<bevy::prelude::NextState<TutorialPhase>>()
        .set(TutorialPhase::SpreadUnlocked);
    app.update();

    // Each tutorial enemy should have ScoutDrone, Health, Velocity, Transform
    let mut query = app
        .world_mut()
        .query_filtered::<(&ScoutDrone, &Health, &Velocity, &Transform), With<TutorialEnemy>>();
    let enemies: Vec<_> = query.iter(app.world()).collect();
    let config = TutorialConfig::default();
    assert_eq!(
        enemies.len(),
        config.tutorial_enemy_count,
        "Should have {} tutorial enemies with all components",
        config.tutorial_enemy_count
    );

    let spawning_config = SpawningConfig::default();
    for (_drone, health, _vel, _transform) in &enemies {
        assert!(
            (health.max - spawning_config.drone_health).abs() < f32::EPSILON,
            "Enemy health.max should match spawning_config.drone_health"
        );
        assert!(
            health.current > 0.0,
            "Enemy should spawn with positive health"
        );
    }
}

#[test]
fn tutorial_enemy_wave_resource_tracks_count() {
    let mut app = enemy_wave_test_app();

    use void_drifter::core::tutorial::{TutorialLayout, TutorialZone};
    app.insert_resource(TutorialZone {
        center: Vec2::ZERO,
        seed: 42,
        layout: TutorialLayout {
            player_spawn: Vec2::new(50.0, 0.0),
            station_position: Vec2::new(300.0, 0.0),
            generator_position: Vec2::new(1700.0, 0.0),
            zone_center: Vec2::ZERO,
            wreck_position: Vec2::new(500.0, 0.0),
        },
    });

    app.world_mut()
        .resource_mut::<bevy::prelude::NextState<TutorialPhase>>()
        .set(TutorialPhase::SpreadUnlocked);
    app.update();

    let config = TutorialConfig::default();
    let wave = app
        .world()
        .get_resource::<TutorialEnemyWave>()
        .expect("TutorialEnemyWave resource should exist after spawn");
    assert_eq!(
        wave.remaining,
        config.tutorial_enemy_count,
        "TutorialEnemyWave.remaining should equal tutorial_enemy_count"
    );
}

/// Minimal app with only the wave completion check — no spawn system.
/// Used for tests that manually control the wave state.
fn wave_completion_test_app() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(bevy::state::app::StatesPlugin);
    app.insert_resource(bevy::time::TimeUpdateStrategy::ManualDuration(
        Duration::from_secs_f64(1.0 / 60.0),
    ));
    app.insert_resource(Time::<Fixed>::from_seconds(1.0 / 60.0));
    app.init_state::<TutorialPhase>();
    app.add_systems(FixedUpdate, check_tutorial_wave_complete);
    // Prime
    app.update();
    app
}

#[test]
fn phase_advances_to_complete_when_all_tutorial_enemies_destroyed() {
    let mut app = wave_completion_test_app();

    // Set phase to SpreadUnlocked
    app.world_mut()
        .resource_mut::<bevy::prelude::NextState<TutorialPhase>>()
        .set(TutorialPhase::SpreadUnlocked);
    app.update(); // Apply state transition

    // Insert the wave resource manually (simulating that spawn happened)
    app.insert_resource(TutorialEnemyWave { remaining: 0 });

    // Spawn 0 TutorialEnemy entities — none alive means wave complete
    // (We don't spawn any, so the count is 0)

    app.update(); // check_tutorial_wave_complete runs, should set Complete
    app.update(); // Apply state transition to Complete

    let phase = app.world().resource::<bevy::prelude::State<TutorialPhase>>();
    assert_eq!(
        *phase.get(),
        TutorialPhase::Complete,
        "Phase should advance to Complete when all tutorial enemies are destroyed"
    );
}

#[test]
fn phase_stays_spread_unlocked_while_enemies_alive() {
    let mut app = wave_completion_test_app();

    // Set phase to SpreadUnlocked
    app.world_mut()
        .resource_mut::<bevy::prelude::NextState<TutorialPhase>>()
        .set(TutorialPhase::SpreadUnlocked);
    app.update(); // Apply state transition

    // Insert wave resource and spawn living enemies
    app.insert_resource(TutorialEnemyWave { remaining: 2 });
    app.world_mut().spawn((
        TutorialEnemy,
        Health { current: 30.0, max: 30.0 },
    ));
    app.world_mut().spawn((
        TutorialEnemy,
        Health { current: 30.0, max: 30.0 },
    ));

    app.update(); // check_tutorial_wave_complete runs — enemies alive, no phase change

    let phase = app.world().resource::<bevy::prelude::State<TutorialPhase>>();
    assert_eq!(
        *phase.get(),
        TutorialPhase::SpreadUnlocked,
        "Phase should remain SpreadUnlocked while tutorial enemies are alive"
    );
}

#[test]
fn tutorial_enemies_do_not_trigger_respawn_timers() {
    use void_drifter::core::spawning::{spawn_respawn_timers, RespawnTimer};

    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.insert_resource(SpawningConfig::default());
    app.add_systems(Update, spawn_respawn_timers);
    app.update(); // Prime

    // Spawn a TutorialEnemy ScoutDrone with health = 0 (destroyed)
    app.world_mut().spawn((
        TutorialEnemy,
        ScoutDrone,
        Health { current: 0.0, max: 30.0 },
        Transform::from_translation(Vec3::new(100.0, 200.0, 0.0)),
    ));

    app.update();

    // No RespawnTimer should be created for the TutorialEnemy
    let timer_count = app
        .world_mut()
        .query_filtered::<Entity, With<RespawnTimer>>()
        .iter(app.world())
        .count();
    assert_eq!(
        timer_count,
        0,
        "TutorialEnemy should NOT trigger a RespawnTimer — got {} timers",
        timer_count
    );
}

#[test]
fn normal_scout_drone_still_triggers_respawn_timer() {
    use void_drifter::core::spawning::{spawn_respawn_timers, RespawnTimer};

    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.insert_resource(SpawningConfig::default());
    app.add_systems(Update, spawn_respawn_timers);
    app.update(); // Prime

    // Spawn a normal ScoutDrone (no TutorialEnemy) with health = 0 (destroyed)
    app.world_mut().spawn((
        ScoutDrone,
        Health { current: 0.0, max: 30.0 },
        Transform::from_translation(Vec3::new(50.0, 60.0, 0.0)),
    ));

    app.update();

    let timer_count = app
        .world_mut()
        .query_filtered::<Entity, With<RespawnTimer>>()
        .iter(app.world())
        .count();
    assert_eq!(
        timer_count,
        1,
        "Normal ScoutDrone should still trigger a RespawnTimer"
    );
}

// ── Station Docking Integration Tests ───────────────────────────────────

/// Create a minimal app with dock_at_station system for isolated integration testing.
fn station_docking_test_app() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(bevy::state::app::StatesPlugin);
    app.insert_resource(bevy::time::TimeUpdateStrategy::ManualDuration(
        Duration::from_secs_f64(1.0 / 60.0),
    ));
    app.insert_resource(Time::<Fixed>::from_seconds(1.0 / 60.0));
    app.insert_resource(TutorialConfig::default());
    app.init_state::<TutorialPhase>();
    app.add_message::<void_drifter::shared::events::GameEvent>();
    app.insert_resource(void_drifter::infrastructure::events::EventSeverityConfig::default());
    app.add_systems(FixedUpdate, dock_at_station);
    // Prime
    app.update();
    app
}

#[test]
fn station_docking_advances_phase_to_station_visited_when_complete() {
    let mut app = station_docking_test_app();

    // Manually set phase to Complete
    app.world_mut()
        .resource_mut::<bevy::prelude::NextState<TutorialPhase>>()
        .set(TutorialPhase::Complete);
    app.update(); // Apply state transition to Complete

    // Spawn player at origin
    app.world_mut().spawn((
        Player,
        Transform::from_translation(Vec3::ZERO),
        Velocity::default(),
    ));

    // Spawn station close enough to player (within dock_radius = 150)
    app.world_mut().spawn((
        TutorialStation { defective: true },
        Transform::from_translation(Vec3::new(50.0, 0.0, 0.0)),
    ));

    app.update(); // dock_at_station runs, sets NextState to StationVisited
    app.update(); // Apply state transition to StationVisited

    let phase = app.world().resource::<bevy::prelude::State<TutorialPhase>>();
    assert_eq!(
        *phase.get(),
        TutorialPhase::StationVisited,
        "Phase should advance to StationVisited after docking"
    );
}

#[test]
fn station_docking_sets_station_not_defective() {
    let mut app = station_docking_test_app();

    app.world_mut()
        .resource_mut::<bevy::prelude::NextState<TutorialPhase>>()
        .set(TutorialPhase::Complete);
    app.update();

    app.world_mut().spawn((
        Player,
        Transform::from_translation(Vec3::ZERO),
        Velocity::default(),
    ));

    let station = app
        .world_mut()
        .spawn((
            TutorialStation { defective: true },
            Transform::from_translation(Vec3::new(50.0, 0.0, 0.0)),
        ))
        .id();

    app.update();

    let station_data = app
        .world()
        .entity(station)
        .get::<TutorialStation>()
        .expect("Station should have TutorialStation");
    assert!(
        !station_data.defective,
        "Station should be marked non-defective after docking"
    );
}

#[test]
fn station_docking_adds_spread_unlocked_to_player() {
    let mut app = station_docking_test_app();

    app.world_mut()
        .resource_mut::<bevy::prelude::NextState<TutorialPhase>>()
        .set(TutorialPhase::Complete);
    app.update();

    let player = app
        .world_mut()
        .spawn((
            Player,
            Transform::from_translation(Vec3::ZERO),
            Velocity::default(),
        ))
        .id();

    app.world_mut().spawn((
        TutorialStation { defective: true },
        Transform::from_translation(Vec3::new(50.0, 0.0, 0.0)),
    ));

    app.update();

    let has_spread_unlocked = app
        .world()
        .entity(player)
        .get::<SpreadUnlocked>()
        .is_some();
    assert!(
        has_spread_unlocked,
        "Player should receive SpreadUnlocked component after docking"
    );
}

#[test]
fn station_docking_is_idempotent_when_already_station_visited() {
    let mut app = station_docking_test_app();

    // Phase already at StationVisited
    app.world_mut()
        .resource_mut::<bevy::prelude::NextState<TutorialPhase>>()
        .set(TutorialPhase::StationVisited);
    app.update();

    app.world_mut().spawn((
        Player,
        Transform::from_translation(Vec3::ZERO),
        Velocity::default(),
    ));

    app.world_mut().spawn((
        TutorialStation { defective: false },
        Transform::from_translation(Vec3::new(50.0, 0.0, 0.0)),
    ));

    app.update();

    let phase = app.world().resource::<bevy::prelude::State<TutorialPhase>>();
    assert_eq!(
        *phase.get(),
        TutorialPhase::StationVisited,
        "Phase should remain StationVisited — idempotent"
    );
}

#[test]
fn station_no_dock_when_not_in_complete_phase() {
    let mut app = station_docking_test_app();

    // Phase is SpreadUnlocked (not Complete)
    app.world_mut()
        .resource_mut::<bevy::prelude::NextState<TutorialPhase>>()
        .set(TutorialPhase::SpreadUnlocked);
    app.update();

    app.world_mut().spawn((
        Player,
        Transform::from_translation(Vec3::ZERO),
        Velocity::default(),
    ));

    app.world_mut().spawn((
        TutorialStation { defective: true },
        Transform::from_translation(Vec3::new(50.0, 0.0, 0.0)),
    ));

    app.update();

    let phase = app.world().resource::<bevy::prelude::State<TutorialPhase>>();
    assert_eq!(
        *phase.get(),
        TutorialPhase::SpreadUnlocked,
        "Phase should not change when not in Complete phase"
    );
}

#[test]
fn station_no_dock_when_too_far() {
    let mut app = station_docking_test_app();

    app.world_mut()
        .resource_mut::<bevy::prelude::NextState<TutorialPhase>>()
        .set(TutorialPhase::Complete);
    app.update();

    app.world_mut().spawn((
        Player,
        Transform::from_translation(Vec3::ZERO),
        Velocity::default(),
    ));

    // Station far beyond dock_radius (default 150.0)
    app.world_mut().spawn((
        TutorialStation { defective: true },
        Transform::from_translation(Vec3::new(500.0, 0.0, 0.0)),
    ));

    app.update();

    let phase = app.world().resource::<bevy::prelude::State<TutorialPhase>>();
    assert_eq!(
        *phase.get(),
        TutorialPhase::Complete,
        "Phase should remain Complete when player is too far from station"
    );
}

#[test]
fn hundred_seed_station_dock_radius_fits_station_range() {
    // Station offset is in [200, 400]; dock_radius is 150 — player within 550 can dock.
    // This test checks that the dock_radius is positive and plausible for all seeds.
    let config = TutorialConfig::default();
    assert!(
        config.dock_radius > 0.0,
        "dock_radius must be positive for docking to work"
    );
    // Validate all seeds still produce valid layouts with dock_radius present
    for seed in 0..100 {
        let result = validate_tutorial_seed(seed, &config);
        assert!(
            result.is_ok(),
            "Seed {seed} should still pass layout validation with dock_radius present: {:?}",
            result.expect_err("Expected Ok")
        );
    }
}

// ── Generator Destruction Integration Tests ──────────────────────────────

/// Minimal app with check_generator_destroyed system for isolated testing.
fn generator_destroyed_test_app() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(bevy::state::app::StatesPlugin);
    app.insert_resource(bevy::time::TimeUpdateStrategy::ManualDuration(
        Duration::from_secs_f64(1.0 / 60.0),
    ));
    app.insert_resource(Time::<Fixed>::from_seconds(1.0 / 60.0));
    app.init_state::<TutorialPhase>();
    app.add_message::<void_drifter::shared::events::GameEvent>();
    app.insert_resource(void_drifter::infrastructure::events::EventSeverityConfig::default());
    app.add_systems(FixedUpdate, check_generator_destroyed);
    // Prime
    app.update();
    app
}

#[test]
fn generator_has_collider() {
    let mut app = tutorial_test_app();

    use void_drifter::core::collision::Collider;
    let mut query = app
        .world_mut()
        .query_filtered::<&Collider, With<GravityWellGenerator>>();
    let collider = query
        .iter(app.world())
        .next()
        .expect("GravityWellGenerator should have a Collider");
    assert!(
        collider.radius > 0.0,
        "Generator collider radius should be positive, got {}",
        collider.radius
    );
}

#[test]
fn generator_collider_radius_positive() {
    let mut app = tutorial_test_app();

    use void_drifter::core::collision::Collider;
    let mut query = app
        .world_mut()
        .query_filtered::<&Collider, With<GravityWellGenerator>>();
    let collider = query
        .iter(app.world())
        .next()
        .expect("Generator should have Collider");
    assert!(
        collider.radius >= 1.0,
        "Generator collider radius should be at least 1.0 to be hittable, got {}",
        collider.radius
    );
}

#[test]
fn phase_advances_to_generator_destroyed_when_generator_gone() {
    let mut app = generator_destroyed_test_app();

    // Manually set phase to StationVisited
    app.world_mut()
        .resource_mut::<bevy::prelude::NextState<TutorialPhase>>()
        .set(TutorialPhase::StationVisited);
    app.update(); // Apply state transition to StationVisited

    // No generator entity spawned — check_generator_destroyed should fire
    app.update(); // check_generator_destroyed runs, sets NextState to GeneratorDestroyed
    app.update(); // Apply state transition to GeneratorDestroyed

    let phase = app.world().resource::<bevy::prelude::State<TutorialPhase>>();
    assert_eq!(
        *phase.get(),
        TutorialPhase::GeneratorDestroyed,
        "Phase should advance to GeneratorDestroyed when generator is absent"
    );
}

#[test]
fn phase_stays_station_visited_while_generator_alive() {
    let mut app = generator_destroyed_test_app();

    // Spawn a living generator BEFORE transitioning to StationVisited
    // so that check_generator_destroyed sees it on the same frame as the transition.
    app.world_mut().spawn((
        GravityWellGenerator {
            safe_radius: 2000.0,
            pull_strength: 50.0,
            requires_projectile: true,
        },
        Health { current: 100.0, max: 100.0 },
        Transform::from_translation(Vec3::ZERO),
    ));

    // Set phase to StationVisited and apply the transition
    app.world_mut()
        .resource_mut::<bevy::prelude::NextState<TutorialPhase>>()
        .set(TutorialPhase::StationVisited);
    app.update(); // Apply state transition + check_generator_destroyed runs — generator exists

    let phase = app.world().resource::<bevy::prelude::State<TutorialPhase>>();
    assert_eq!(
        *phase.get(),
        TutorialPhase::StationVisited,
        "Phase should remain StationVisited while generator is alive"
    );
}

#[test]
fn generator_destroyed_is_idempotent() {
    let mut app = generator_destroyed_test_app();

    // Phase already at GeneratorDestroyed
    app.world_mut()
        .resource_mut::<bevy::prelude::NextState<TutorialPhase>>()
        .set(TutorialPhase::GeneratorDestroyed);
    app.update(); // Apply state transition

    // No generator — but phase is already GeneratorDestroyed, should be no-op
    app.update();

    let phase = app.world().resource::<bevy::prelude::State<TutorialPhase>>();
    assert_eq!(
        *phase.get(),
        TutorialPhase::GeneratorDestroyed,
        "Phase should remain GeneratorDestroyed — idempotent"
    );
}

#[test]
fn check_generator_not_triggered_in_non_station_visited_phase() {
    let mut app = generator_destroyed_test_app();

    // Phase is Complete (not StationVisited) — no generator
    app.world_mut()
        .resource_mut::<bevy::prelude::NextState<TutorialPhase>>()
        .set(TutorialPhase::Complete);
    app.update();

    app.update(); // check_generator_destroyed should be a no-op

    let phase = app.world().resource::<bevy::prelude::State<TutorialPhase>>();
    assert_eq!(
        *phase.get(),
        TutorialPhase::Complete,
        "Phase should remain Complete — check_generator_destroyed guards on StationVisited"
    );
}

#[test]
fn gravity_well_stops_when_generator_despawned_integration() {
    let mut app = gravity_well_test_app();

    // Spawn generator
    let generator_entity = app
        .world_mut()
        .spawn((
            GravityWellGenerator {
                safe_radius: 100.0,
                pull_strength: 50.0,
                requires_projectile: true,
            },
            Transform::from_translation(Vec3::ZERO),
        ))
        .id();

    // Player far outside safe_radius
    let player = app
        .world_mut()
        .spawn((
            Player,
            Velocity::default(),
            Transform::from_translation(Vec3::new(500.0, 0.0, 0.0)),
        ))
        .id();

    // Confirm pull is applied when generator exists
    app.update();
    let vel_before = app
        .world()
        .entity(player)
        .get::<Velocity>()
        .expect("Player should have Velocity")
        .0;
    assert!(vel_before.x < 0.0, "Should have pull before despawn");

    // Despawn the generator (simulates destruction)
    app.world_mut().entity_mut(generator_entity).despawn();

    // Reset velocity to zero
    app.world_mut()
        .entity_mut(player)
        .insert(Velocity::default());

    // Run again — no generator, no force
    app.update();

    let vel_after = app
        .world()
        .entity(player)
        .get::<Velocity>()
        .expect("Player should have Velocity");
    assert!(
        vel_after.0.length() < f32::EPSILON,
        "No gravity pull after generator despawned, got {:?}",
        vel_after.0
    );
}

#[test]
fn hundred_seed_validation_still_passes_with_generator_collider() {
    let config = TutorialConfig::default();
    for seed in 0..100 {
        let result = validate_tutorial_seed(seed, &config);
        assert!(
            result.is_ok(),
            "Seed {seed} should still pass validation after generator collider added: {:?}",
            result.expect_err("Expected Ok")
        );
    }
}

// ── Destruction Cascade Integration Tests ───────────────────────────────

/// Minimal app with start_destruction_cascade and tick_cascade_timer for isolated testing.
fn cascade_test_app() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(bevy::state::app::StatesPlugin);
    app.insert_resource(bevy::time::TimeUpdateStrategy::ManualDuration(
        Duration::from_secs_f64(1.0 / 60.0),
    ));
    app.insert_resource(Time::<Fixed>::from_seconds(1.0 / 60.0));
    app.insert_resource(TutorialConfig::default());
    app.init_state::<TutorialPhase>();
    app.add_message::<void_drifter::shared::events::GameEvent>();
    app.insert_resource(void_drifter::infrastructure::events::EventSeverityConfig::default());
    app.add_systems(OnEnter(TutorialPhase::GeneratorDestroyed), start_destruction_cascade);
    app.add_systems(FixedUpdate, tick_cascade_timer);
    // Prime
    app.update();
    app
}

#[test]
fn cascade_timer_inserted_on_generator_destroyed_phase() {
    let mut app = cascade_test_app();

    // Transition to GeneratorDestroyed — triggers OnEnter(GeneratorDestroyed)
    app.world_mut()
        .resource_mut::<bevy::prelude::NextState<TutorialPhase>>()
        .set(TutorialPhase::GeneratorDestroyed);
    app.update(); // Apply transition + run OnEnter(GeneratorDestroyed)

    let timer = app
        .world()
        .get_resource::<CascadeTimer>()
        .expect("CascadeTimer should be inserted on entering GeneratorDestroyed phase");
    assert!(
        timer.remaining > 0.0,
        "CascadeTimer.remaining should be positive after insertion, got {}",
        timer.remaining
    );
}

#[test]
fn cascade_timer_not_inserted_in_other_phases() {
    let mut app = cascade_test_app();

    // Phase is Flying (default) — no cascade timer should be present
    assert!(
        app.world().get_resource::<CascadeTimer>().is_none(),
        "CascadeTimer should not be present in Flying phase"
    );
}

#[test]
fn phase_advances_to_tutorial_complete_after_cascade_delay() {
    let mut app = cascade_test_app();

    // Transition to GeneratorDestroyed
    app.world_mut()
        .resource_mut::<bevy::prelude::NextState<TutorialPhase>>()
        .set(TutorialPhase::GeneratorDestroyed);
    app.update(); // Apply transition + insert CascadeTimer (default 2.0s)

    // Tick enough frames to exceed 2.0s (2.0 / (1/60) = 120 frames + 1 for safety)
    for _ in 0..130 {
        app.update();
    }

    // One more update to apply the state transition
    app.update();

    let phase = app.world().resource::<bevy::prelude::State<TutorialPhase>>();
    assert_eq!(
        *phase.get(),
        TutorialPhase::TutorialComplete,
        "Phase should advance to TutorialComplete after cascade delay expires"
    );
}

#[test]
fn tutorial_wreck_despawned_after_cascade() {
    let mut app = cascade_test_app();

    // Spawn a tutorial wreck
    app.world_mut().spawn((
        TutorialWreck,
        Transform::from_translation(Vec3::new(500.0, 0.0, 0.0)),
    ));

    // Confirm wreck exists before cascade
    let wreck_count_before = app
        .world_mut()
        .query_filtered::<Entity, With<TutorialWreck>>()
        .iter(app.world())
        .count();
    assert_eq!(wreck_count_before, 1, "Should have one wreck before cascade");

    // Transition to GeneratorDestroyed
    app.world_mut()
        .resource_mut::<bevy::prelude::NextState<TutorialPhase>>()
        .set(TutorialPhase::GeneratorDestroyed);
    app.update(); // Apply transition + insert CascadeTimer

    // Tick 130 frames (> 2s at 60fps)
    for _ in 0..130 {
        app.update();
    }

    let wreck_count_after = app
        .world_mut()
        .query_filtered::<Entity, With<TutorialWreck>>()
        .iter(app.world())
        .count();
    assert_eq!(
        wreck_count_after, 0,
        "TutorialWreck should be despawned after cascade fires"
    );
}

#[test]
fn tutorial_station_despawned_after_cascade() {
    let mut app = cascade_test_app();

    // Spawn a tutorial station
    app.world_mut().spawn((
        TutorialStation { defective: false },
        Transform::from_translation(Vec3::new(200.0, 0.0, 0.0)),
    ));

    // Transition to GeneratorDestroyed
    app.world_mut()
        .resource_mut::<bevy::prelude::NextState<TutorialPhase>>()
        .set(TutorialPhase::GeneratorDestroyed);
    app.update();

    // Tick 130 frames
    for _ in 0..130 {
        app.update();
    }

    let station_count = app
        .world_mut()
        .query_filtered::<Entity, With<TutorialStation>>()
        .iter(app.world())
        .count();
    assert_eq!(
        station_count, 0,
        "TutorialStation should be despawned after cascade fires"
    );
}

#[test]
fn cascade_is_idempotent_no_panic_without_wreck_or_station() {
    let mut app = cascade_test_app();

    // No wreck or station entities — cascade should fire without panicking
    app.world_mut()
        .resource_mut::<bevy::prelude::NextState<TutorialPhase>>()
        .set(TutorialPhase::GeneratorDestroyed);
    app.update();

    // Tick 130 frames — cascade fires with empty queries (no-op despawns)
    for _ in 0..130 {
        app.update();
    }
    app.update(); // Apply state transition

    let phase = app.world().resource::<bevy::prelude::State<TutorialPhase>>();
    assert_eq!(
        *phase.get(),
        TutorialPhase::TutorialComplete,
        "Phase should advance to TutorialComplete even when no entities to despawn"
    );
}

#[test]
fn cascade_timer_removed_after_cascade_fires() {
    let mut app = cascade_test_app();

    // Transition to GeneratorDestroyed
    app.world_mut()
        .resource_mut::<bevy::prelude::NextState<TutorialPhase>>()
        .set(TutorialPhase::GeneratorDestroyed);
    app.update();

    // Tick until cascade fires
    for _ in 0..130 {
        app.update();
    }

    // CascadeTimer should be removed after cascade fires
    assert!(
        app.world().get_resource::<CascadeTimer>().is_none(),
        "CascadeTimer should be removed after cascade fires"
    );
}

#[test]
fn hundred_seed_validation_still_passes_after_cascade() {
    let config = TutorialConfig::default();
    for seed in 0..100 {
        let result = validate_tutorial_seed(seed, &config);
        assert!(
            result.is_ok(),
            "Seed {seed} should still pass validation after cascade feature added: {:?}",
            result.expect_err("Expected Ok")
        );
    }
}

// ── Constraint Validation Integration Tests (Story 2-8) ──────────────────

/// Helper: minimal app with just `validate_tutorial_config` as a Startup system.
fn constraint_validation_test_app(config: TutorialConfig) -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.insert_resource(config);
    app.add_systems(Startup, validate_tutorial_config);
    app
}

#[test]
fn validate_tutorial_config_system_runs_without_panic_on_valid_config() {
    // Default config passes all constraints — system must not panic.
    let mut app = constraint_validation_test_app(TutorialConfig::default());
    app.update(); // Runs Startup systems
}

#[test]
fn validate_tutorial_config_system_runs_without_panic_on_invalid_config() {
    // Deliberately violate every constraint the system checks — must only warn, never panic.
    let invalid = TutorialConfig {
        safe_radius: 0.0,                 // violates safe_radius > 0.0
        wreck_offset_max: 5000.0,         // violates wreck_offset_max <= safe_radius
        wreck_offset_min: 6000.0,         // violates wreck_offset_min <= wreck_offset_max
        dock_radius: 0.0,                 // violates dock_radius > 0.0
        tutorial_enemy_count: 0,          // violates tutorial_enemy_count > 0
        tutorial_enemy_spawn_radius: 0.0, // violates tutorial_enemy_spawn_radius > 0.0
        cascade_delay_secs: -1.0,         // violates cascade_delay_secs > 0.0
        // wreck_offset_min is already > 0, but set a known positive value
        // to also trigger wreck_offset_min < 0.0 check we need a negative value:
        ..TutorialConfig {
            wreck_offset_min: -5.0, // violates wreck_offset_min >= 0.0
            ..TutorialConfig::default()
        }
    };
    let mut app = constraint_validation_test_app(invalid);
    app.update(); // Must not panic — only warn
}

#[test]
fn validate_tutorial_config_accepts_valid_safe_radius() {
    // safe_radius exactly at a positive value — no warning expected (no-panic is the test)
    let config = TutorialConfig {
        safe_radius: 1.0,
        wreck_offset_max: 0.5,
        wreck_offset_min: 0.1,
        dock_radius: 0.5,
        generator_offset_max: 0.9,
        generator_offset_min: 0.8,
        station_offset_max: 0.7,
        station_offset_min: 0.6,
        ..TutorialConfig::default()
    };
    let mut app = constraint_validation_test_app(config);
    app.update();
}

#[test]
fn validate_tutorial_config_wreck_offset_max_within_safe_radius_is_valid() {
    let config = TutorialConfig::default();
    // default wreck_offset_max (700) <= safe_radius (2000) — valid
    assert!(
        config.wreck_offset_max <= config.safe_radius,
        "Default wreck_offset_max ({}) should be <= safe_radius ({})",
        config.wreck_offset_max,
        config.safe_radius
    );
}

#[test]
fn validate_tutorial_config_dock_radius_within_safe_radius_is_valid() {
    let config = TutorialConfig::default();
    // default dock_radius (150) <= safe_radius (2000) — valid
    assert!(
        config.dock_radius <= config.safe_radius,
        "Default dock_radius ({}) should be <= safe_radius ({})",
        config.dock_radius,
        config.safe_radius
    );
}

#[test]
fn validate_tutorial_config_wreck_offset_ordering_valid_by_default() {
    let config = TutorialConfig::default();
    assert!(
        config.wreck_offset_min <= config.wreck_offset_max,
        "Default wreck_offset_min ({}) should be <= wreck_offset_max ({})",
        config.wreck_offset_min,
        config.wreck_offset_max
    );
    assert!(
        config.wreck_offset_min >= 0.0,
        "Default wreck_offset_min ({}) should be >= 0.0",
        config.wreck_offset_min
    );
}

#[test]
fn hundred_seed_validation_still_passes_after_constraint_validation() {
    let config = TutorialConfig::default();
    for seed in 0..100 {
        let result = validate_tutorial_seed(seed, &config);
        assert!(
            result.is_ok(),
            "Seed {seed} should still pass validation after constraint validation feature added: {:?}",
            result.expect_err("Expected Ok")
        );
    }
}

// ── Flying → Shooting Trigger Integration Tests (Story 2-10) ────────────

/// Minimal app with unlock_laser_at_wreck and update_weapons_lock for isolated testing.
fn laser_unlock_test_app() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(bevy::state::app::StatesPlugin);
    app.insert_resource(bevy::time::TimeUpdateStrategy::ManualDuration(
        Duration::from_secs_f64(1.0 / 60.0),
    ));
    app.insert_resource(Time::<Fixed>::from_seconds(1.0 / 60.0));
    app.insert_resource(TutorialConfig::default());
    app.init_state::<TutorialPhase>();
    app.add_systems(
        FixedUpdate,
        (
            void_drifter::core::tutorial::update_weapons_lock,
            unlock_laser_at_wreck,
        )
            .chain(),
    );
    // Prime
    app.update();
    app
}

#[test]
fn weapons_locked_removed_after_approaching_wreck() {
    let mut app = laser_unlock_test_app();
    // Phase is Flying (default)

    // Spawn wreck at origin
    app.world_mut().spawn((
        TutorialWreck,
        Transform::from_translation(Vec3::ZERO),
    ));

    // Spawn player inside wreck_dock_radius (default 120.0), with WeaponsLocked
    let player = app
        .world_mut()
        .spawn((
            Player,
            WeaponsLocked,
            Transform::from_translation(Vec3::new(50.0, 0.0, 0.0)),
        ))
        .id();

    // Verify WeaponsLocked is present before update
    assert!(
        app.world().entity(player).get::<WeaponsLocked>().is_some(),
        "Player should have WeaponsLocked initially"
    );

    app.update(); // unlock_laser_at_wreck fires: NextState(Shooting). update_weapons_lock removes WeaponsLocked.
    app.update(); // Apply state transition: Flying -> Shooting

    // After phase transition to Shooting, update_weapons_lock should have removed WeaponsLocked
    app.update(); // Run update_weapons_lock again in Shooting phase

    let has_weapons_locked = app
        .world()
        .entity(player)
        .get::<WeaponsLocked>()
        .is_some();
    assert!(
        !has_weapons_locked,
        "WeaponsLocked should be removed after phase transitions to Shooting"
    );
}

#[test]
fn phase_transitions_flying_to_shooting_on_wreck_proximity() {
    let mut app = laser_unlock_test_app();
    // Phase is Flying (default)

    // Spawn wreck at origin
    app.world_mut().spawn((
        TutorialWreck,
        Transform::from_translation(Vec3::ZERO),
    ));

    // Spawn player inside wreck_dock_radius (default 120.0)
    app.world_mut().spawn((
        Player,
        Transform::from_translation(Vec3::new(80.0, 0.0, 0.0)),
    ));

    app.update(); // unlock_laser_at_wreck runs — sets NextState(Shooting)
    app.update(); // Apply state transition: Flying -> Shooting

    let phase = app.world().resource::<bevy::prelude::State<TutorialPhase>>();
    assert_eq!(
        *phase.get(),
        TutorialPhase::Shooting,
        "Phase should advance to Shooting when player reaches wreck"
    );
}

#[test]
fn phase_stays_flying_when_player_far_from_wreck() {
    let mut app = laser_unlock_test_app();
    // Phase is Flying (default)

    // Spawn wreck at origin
    app.world_mut().spawn((
        TutorialWreck,
        Transform::from_translation(Vec3::ZERO),
    ));

    // Spawn player outside wreck_dock_radius (default 120.0)
    app.world_mut().spawn((
        Player,
        Transform::from_translation(Vec3::new(500.0, 0.0, 0.0)),
    ));

    app.update();
    app.update();

    let phase = app.world().resource::<bevy::prelude::State<TutorialPhase>>();
    assert_eq!(
        *phase.get(),
        TutorialPhase::Flying,
        "Phase should remain Flying when player is far from wreck"
    );
}

#[test]
fn unlock_laser_trigger_is_idempotent_in_shooting_phase() {
    let mut app = laser_unlock_test_app();

    // Set phase to Shooting (skip Flying)
    app.world_mut()
        .resource_mut::<bevy::prelude::NextState<TutorialPhase>>()
        .set(TutorialPhase::Shooting);
    app.update(); // Apply transition to Shooting

    // Spawn wreck and player right next to it
    app.world_mut().spawn((
        TutorialWreck,
        Transform::from_translation(Vec3::ZERO),
    ));
    app.world_mut().spawn((
        Player,
        Transform::from_translation(Vec3::new(10.0, 0.0, 0.0)),
    ));

    app.update(); // unlock_laser_at_wreck — guard returns early (not Flying)
    app.update(); // No state transition should happen

    let phase = app.world().resource::<bevy::prelude::State<TutorialPhase>>();
    assert_eq!(
        *phase.get(),
        TutorialPhase::Shooting,
        "Phase should remain Shooting — unlock_laser_at_wreck is idempotent"
    );
}

#[test]
fn unlock_laser_no_panic_without_wreck_entity() {
    let mut app = laser_unlock_test_app();
    // Phase is Flying (default), no TutorialWreck entity

    app.world_mut().spawn((
        Player,
        Transform::from_translation(Vec3::ZERO),
    ));

    // Should not panic or crash
    app.update();

    let phase = app.world().resource::<bevy::prelude::State<TutorialPhase>>();
    assert_eq!(
        *phase.get(),
        TutorialPhase::Flying,
        "Phase should remain Flying when no wreck exists"
    );
}

#[test]
fn hundred_seed_validation_still_passes_after_laser_unlock_trigger() {
    let config = TutorialConfig::default();
    for seed in 0..100 {
        let result = validate_tutorial_seed(seed, &config);
        assert!(
            result.is_ok(),
            "Seed {seed} should still pass validation after laser unlock trigger added: {:?}",
            result.expect_err("Expected Ok")
        );
    }
}

// ── Tutorial Entity Rendering Tests (Story 2-9) ──────────────────────────

use void_drifter::rendering::{
    render_tutorial_generators, render_tutorial_stations, render_tutorial_wrecks,
    setup_gravity_well_boundary_visual, update_tutorial_station_visual, TutorialGeneratorAssets,
    TutorialStationAssets, TutorialWreckAssets,
};
use void_drifter::rendering::vector_art::{
    generate_circle_outline_mesh, generate_tutorial_generator_mesh, generate_tutorial_station_mesh,
    generate_tutorial_wreck_mesh,
};
use void_drifter::core::tutorial::GravityWellBoundary;
use bevy::ecs::hierarchy::ChildOf;

/// Build a minimal app with Assets<Mesh> + Assets<ColorMaterial> and the tutorial
/// render systems registered. The asset resources (TutorialStationAssets etc.) are
/// pre-inserted so the render systems can run without the full RenderingPlugin.
/// Entities must be spawned AFTER the prime `app.update()` so `Added<T>` fires.
fn tutorial_rendering_app_with_assets() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);

    // Insert bare asset stores — no AssetPlugin needed for in-memory usage
    app.insert_resource(Assets::<Mesh>::default());
    app.insert_resource(Assets::<ColorMaterial>::default());

    // Pre-build asset resources
    {
        let mut meshes = app.world_mut().resource_mut::<Assets<Mesh>>();
        let station_mesh = meshes.add(generate_tutorial_station_mesh(20.0));
        let wreck_mesh = meshes.add(generate_tutorial_wreck_mesh(18.0));
        let generator_mesh = meshes.add(generate_tutorial_generator_mesh(25.0));

        let mut materials = app.world_mut().resource_mut::<Assets<ColorMaterial>>();
        let station_mat_defective = materials.add(ColorMaterial::from(Color::srgb(0.1, 0.5, 0.5)));
        let station_mat_repaired = materials.add(ColorMaterial::from(Color::srgb(0.2, 0.9, 0.9)));
        let wreck_mat = materials.add(ColorMaterial::from(Color::srgb(0.3, 0.3, 0.3)));
        let generator_mat = materials.add(ColorMaterial::from(Color::srgb(1.0, 0.5, 0.1)));

        app.world_mut().insert_resource(TutorialStationAssets {
            mesh: station_mesh,
            material_defective: station_mat_defective,
            material_repaired: station_mat_repaired,
        });
        app.world_mut().insert_resource(TutorialWreckAssets {
            mesh: wreck_mesh,
            material: wreck_mat,
        });
        app.world_mut().insert_resource(TutorialGeneratorAssets {
            mesh: generator_mesh,
            material: generator_mat,
        });
    }

    // Register the rendering systems under test
    app.add_systems(
        Update,
        (
            render_tutorial_stations,
            render_tutorial_wrecks,
            render_tutorial_generators,
            update_tutorial_station_visual,
        ),
    );

    app.insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_secs_f64(
        1.0 / 60.0,
    )));

    // Prime frame — establishes change detection baseline
    app.update();

    app
}

#[test]
fn tutorial_station_gets_mesh2d_after_setup() {
    let mut app = tutorial_rendering_app_with_assets();

    // Spawn AFTER prime so Added<TutorialStation> fires on the next update
    let station = app
        .world_mut()
        .spawn((
            TutorialStation { defective: true },
            Transform::default(),
        ))
        .id();

    app.update(); // render_tutorial_stations runs

    let has_mesh = app.world().entity(station).get::<Mesh2d>().is_some();
    assert!(
        has_mesh,
        "TutorialStation entity should have Mesh2d component after rendering setup"
    );
}

#[test]
fn tutorial_wreck_gets_mesh2d_after_setup() {
    let mut app = tutorial_rendering_app_with_assets();

    let wreck = app
        .world_mut()
        .spawn((TutorialWreck, Transform::default()))
        .id();

    app.update(); // render_tutorial_wrecks runs

    let has_mesh = app.world().entity(wreck).get::<Mesh2d>().is_some();
    assert!(
        has_mesh,
        "TutorialWreck entity should have Mesh2d component after rendering setup"
    );
}

#[test]
fn gravity_well_generator_gets_mesh2d_after_setup() {
    let mut app = tutorial_rendering_app_with_assets();

    let generator = app
        .world_mut()
        .spawn((
            GravityWellGenerator {
                safe_radius: 2000.0,
                pull_strength: 50.0,
                requires_projectile: true,
            },
            Transform::default(),
        ))
        .id();

    app.update(); // render_tutorial_generators runs

    let has_mesh = app.world().entity(generator).get::<Mesh2d>().is_some();
    assert!(
        has_mesh,
        "GravityWellGenerator entity should have Mesh2d component after rendering setup"
    );
}

#[test]
fn tutorial_station_material_updated_when_defective_flips_to_false() {
    let mut app = tutorial_rendering_app_with_assets();

    // Spawn defective station
    let station = app
        .world_mut()
        .spawn((
            TutorialStation { defective: true },
            Transform::default(),
        ))
        .id();

    app.update(); // render_tutorial_stations attaches Mesh2d + MeshMaterial2d

    // Confirm Mesh2d was attached
    assert!(
        app.world().entity(station).get::<Mesh2d>().is_some(),
        "Station should have Mesh2d after initial setup"
    );

    // Record material handle before change
    let mat_handle = app
        .world()
        .entity(station)
        .get::<MeshMaterial2d<ColorMaterial>>()
        .expect("Station should have MeshMaterial2d after setup")
        .clone();

    // Simulate docking: flip defective to false
    app.world_mut()
        .entity_mut(station)
        .get_mut::<TutorialStation>()
        .expect("Station should have TutorialStation")
        .defective = false;

    app.update(); // update_tutorial_station_visual runs on Changed<TutorialStation>

    // Material in Assets should now be bright teal (green channel > 0.7)
    let materials = app.world().resource::<Assets<ColorMaterial>>();
    let mat = materials
        .get(&mat_handle.0)
        .expect("Material should still be present in Assets");
    let green = mat.color.to_srgba().green;
    assert!(
        green > 0.7,
        "After docking (defective=false), station material green should be > 0.7, got {green}"
    );
}

#[test]
fn hundred_seed_validation_still_passes_after_entity_rendering() {
    let config = TutorialConfig::default();
    for seed in 0..100 {
        let result = validate_tutorial_seed(seed, &config);
        assert!(
            result.is_ok(),
            "Seed {seed} should still pass validation after entity rendering added: {:?}",
            result.expect_err("Expected Ok")
        );
    }
}

// ── Gravity Well Boundary Visual Tests (Story 2-11) ──────────────────────

/// Build a minimal app with asset resources and the setup_gravity_well_boundary_visual
/// system registered.  Entities must be spawned AFTER the prime `app.update()` so
/// `Added<GravityWellBoundary>` fires on the subsequent frame.
fn gravity_well_boundary_app() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);

    // Bare in-memory asset stores — no AssetPlugin needed.
    app.insert_resource(Assets::<Mesh>::default());
    app.insert_resource(Assets::<ColorMaterial>::default());

    app.add_systems(Update, setup_gravity_well_boundary_visual);

    app.insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_secs_f64(
        1.0 / 60.0,
    )));

    // Prime frame — establishes change detection baseline.
    app.update();

    app
}

#[test]
fn gravity_well_boundary_spawned_as_child_of_generator() {
    // Verify that spawn_tutorial_zone creates a GravityWellBoundary child entity
    // for each GravityWellGenerator.
    let mut app = tutorial_test_app();

    // Count GravityWellBoundary entities
    let boundary_count = app
        .world_mut()
        .query_filtered::<Entity, With<GravityWellBoundary>>()
        .iter(app.world())
        .count();
    assert_eq!(
        boundary_count, 1,
        "Should spawn exactly one GravityWellBoundary child entity, got {boundary_count}"
    );
}

#[test]
fn gravity_well_boundary_entity_has_parent_that_is_generator() {
    // Verify the GravityWellBoundary entity's ChildOf resolves to a GravityWellGenerator.
    let mut app = tutorial_test_app();

    // Collect all (boundary_entity, parent_entity) pairs via the ChildOf relationship.
    let boundaries: Vec<(Entity, Entity)> = app
        .world_mut()
        .query::<(Entity, &ChildOf)>()
        .iter(app.world())
        .filter_map(|(entity, child_of): (Entity, &ChildOf)| {
            if app.world().entity(entity).get::<GravityWellBoundary>().is_some() {
                Some((entity, child_of.parent()))
            } else {
                None
            }
        })
        .collect();

    assert_eq!(
        boundaries.len(), 1,
        "Should have exactly one GravityWellBoundary with a parent"
    );

    let (_, parent_entity) = boundaries[0];
    let parent_is_generator: bool = app
        .world()
        .entity(parent_entity)
        .get::<GravityWellGenerator>()
        .is_some();
    assert!(
        parent_is_generator,
        "The parent of GravityWellBoundary should be a GravityWellGenerator entity"
    );
}

#[test]
fn gravity_well_boundary_gets_mesh2d_after_visual_setup() {
    // Verify the rendering system attaches Mesh2d to GravityWellBoundary entities.
    let mut app = gravity_well_boundary_app();

    // Spawn generator at origin
    let generator = app
        .world_mut()
        .spawn((
            GravityWellGenerator {
                safe_radius: 2000.0,
                pull_strength: 50.0,
                requires_projectile: true,
            },
            Transform::default(),
            GlobalTransform::default(),
        ))
        .id();

    // Spawn boundary as a child of the generator AFTER the prime frame.
    let boundary = app
        .world_mut()
        .spawn((
            GravityWellBoundary,
            Transform::default(),
            GlobalTransform::default(),
        ))
        .id();

    // Manually set the parent relationship so the system can resolve it.
    app.world_mut()
        .entity_mut(generator)
        .add_child(boundary);

    app.update(); // setup_gravity_well_boundary_visual runs on Added<GravityWellBoundary>

    let has_mesh = app.world().entity(boundary).get::<Mesh2d>().is_some();
    assert!(
        has_mesh,
        "GravityWellBoundary entity should have Mesh2d after visual setup"
    );
}

#[test]
fn gravity_well_boundary_gets_mesh_material_after_visual_setup() {
    // Verify the rendering system attaches MeshMaterial2d to GravityWellBoundary entities.
    let mut app = gravity_well_boundary_app();

    let generator = app
        .world_mut()
        .spawn((
            GravityWellGenerator {
                safe_radius: 2000.0,
                pull_strength: 50.0,
                requires_projectile: true,
            },
            Transform::default(),
            GlobalTransform::default(),
        ))
        .id();

    let boundary = app
        .world_mut()
        .spawn((
            GravityWellBoundary,
            Transform::default(),
            GlobalTransform::default(),
        ))
        .id();

    app.world_mut().entity_mut(generator).add_child(boundary);

    app.update();

    let has_material = app
        .world()
        .entity(boundary)
        .get::<MeshMaterial2d<ColorMaterial>>()
        .is_some();
    assert!(
        has_material,
        "GravityWellBoundary entity should have MeshMaterial2d after visual setup"
    );
}

#[test]
fn gravity_well_boundary_material_is_semi_transparent_orange() {
    // Verify the material color is orange with low alpha (~0.4).
    let mut app = gravity_well_boundary_app();

    let generator = app
        .world_mut()
        .spawn((
            GravityWellGenerator {
                safe_radius: 2000.0,
                pull_strength: 50.0,
                requires_projectile: true,
            },
            Transform::default(),
            GlobalTransform::default(),
        ))
        .id();

    let boundary = app
        .world_mut()
        .spawn((
            GravityWellBoundary,
            Transform::default(),
            GlobalTransform::default(),
        ))
        .id();

    app.world_mut().entity_mut(generator).add_child(boundary);

    app.update();

    let mat_handle = app
        .world()
        .entity(boundary)
        .get::<MeshMaterial2d<ColorMaterial>>()
        .expect("GravityWellBoundary should have MeshMaterial2d")
        .clone();

    let materials = app.world().resource::<Assets<ColorMaterial>>();
    let mat = materials
        .get(&mat_handle.0)
        .expect("Material should exist in Assets");
    let color = mat.color.to_srgba();
    // Red channel should be high (orange)
    assert!(
        color.red > 0.9,
        "Boundary material red channel should be high (orange), got {}",
        color.red
    );
    // Alpha should be low (~0.4) — semi-transparent
    assert!(
        color.alpha < 0.6,
        "Boundary material alpha should be low (semi-transparent), got {}",
        color.alpha
    );
    assert!(
        color.alpha > 0.1,
        "Boundary material alpha should be > 0 (visible), got {}",
        color.alpha
    );
}

#[test]
fn generate_circle_outline_mesh_produces_vertices() {
    let mesh = generate_circle_outline_mesh(100.0, 5.0);
    let positions = mesh
        .attribute(Mesh::ATTRIBUTE_POSITION)
        .expect("Circle outline mesh should have positions");
    let len = match positions {
        bevy::mesh::VertexAttributeValues::Float32x3(v) => v.len(),
        _ => panic!("Expected Float32x3 positions"),
    };
    assert!(len >= 3, "Circle outline mesh should have at least 3 vertices, got {len}");
    let indices = mesh.indices().expect("Circle outline mesh should have indices");
    let index_count = match indices {
        bevy::mesh::Indices::U32(v) => v.len(),
        _ => panic!("Expected U32 indices"),
    };
    assert!(
        index_count >= 3,
        "Circle outline mesh should have at least 3 indices, got {index_count}"
    );
}

#[test]
fn generate_circle_outline_mesh_different_radii_produce_different_vertex_counts() {
    // Larger radius with same segment count — vertex positions should differ.
    let mesh_small = generate_circle_outline_mesh(50.0, 5.0);
    let mesh_large = generate_circle_outline_mesh(500.0, 5.0);

    let positions_small = mesh_small
        .attribute(Mesh::ATTRIBUTE_POSITION)
        .expect("Small circle mesh should have positions");
    let positions_large = mesh_large
        .attribute(Mesh::ATTRIBUTE_POSITION)
        .expect("Large circle mesh should have positions");

    let verts_small = match positions_small {
        bevy::mesh::VertexAttributeValues::Float32x3(v) => v,
        _ => panic!("Expected Float32x3 positions"),
    };
    let verts_large = match positions_large {
        bevy::mesh::VertexAttributeValues::Float32x3(v) => v,
        _ => panic!("Expected Float32x3 positions"),
    };

    // Same segment count means same vertex count, but max vertex extent should differ.
    let max_extent_small = verts_small
        .iter()
        .map(|v| (v[0] * v[0] + v[1] * v[1]).sqrt())
        .fold(0.0f32, f32::max);
    let max_extent_large = verts_large
        .iter()
        .map(|v| (v[0] * v[0] + v[1] * v[1]).sqrt())
        .fold(0.0f32, f32::max);

    assert!(
        max_extent_large > max_extent_small,
        "Larger radius should produce larger max vertex extent: small={max_extent_small:.1}, large={max_extent_large:.1}"
    );
}

#[test]
fn hundred_seed_validation_still_passes_after_gravity_well_boundary() {
    let config = TutorialConfig::default();
    for seed in 0..100 {
        let result = validate_tutorial_seed(seed, &config);
        assert!(
            result.is_ok(),
            "Seed {seed} should still pass validation after gravity well boundary added: {:?}",
            result.expect_err("Expected Ok")
        );
    }
}
