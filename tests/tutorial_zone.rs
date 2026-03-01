#![deny(clippy::unwrap_used)]

mod helpers;

use std::time::Duration;
use bevy::prelude::*;
use bevy::time::TimeUpdateStrategy;
use void_drifter::core::flight::Player;
use void_drifter::core::tutorial::{
    generate_tutorial_zone, validate_tutorial_seed, GravityWellGenerator, TutorialConfig,
    TutorialPhase, TutorialStation, TutorialZone, WeaponsLocked,
};
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
