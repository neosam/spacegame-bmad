#![deny(clippy::unwrap_used)]

mod helpers;

use std::time::Duration;
use bevy::prelude::*;
use bevy::time::TimeUpdateStrategy;
use void_drifter::core::flight::Player;
use void_drifter::core::tutorial::{
    advance_phase_on_wreck_shot, apply_gravity_well, generate_tutorial_zone,
    validate_tutorial_seed, GravityWellGenerator, TutorialConfig, TutorialPhase, TutorialStation,
    TutorialWreck, TutorialZone, WeaponsLocked, WreckShotState,
};
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
