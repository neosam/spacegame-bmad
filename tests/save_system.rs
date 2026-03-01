mod helpers;

use bevy::prelude::*;
use helpers::{spawn_player, test_app};
use std::fs;

use void_drifter::core::input::ActionState;
use void_drifter::infrastructure::logbook::Logbook;
use void_drifter::infrastructure::save::{SaveConfig, SaveState, save_game, load_game};
use void_drifter::infrastructure::save::player_save::PlayerSave;
use void_drifter::infrastructure::save::world_save::WorldSave;
use void_drifter::infrastructure::save::schema::SAVE_VERSION;
use void_drifter::shared::events::GameEventKind;
use void_drifter::world::{ExploredChunks, ChunkCoord, BiomeType, WorldConfig};

fn cleanup_save_dir(dir: &str) {
    let _ = fs::remove_dir_all(dir);
}

#[test]
fn save_then_load_restores_player_position() {
    let save_dir = "target/test_saves/pos_test/";
    cleanup_save_dir(save_dir);
    fs::create_dir_all(save_dir).expect("Should create dir");

    // Manually write player save file
    let player_save = PlayerSave {
        schema_version: SAVE_VERSION,
        position: (500.0, -300.0),
        rotation: 0.5,
        velocity: (10.0, -5.0),
        health_current: 80.0,
        health_max: 100.0,
        active_weapon: "Spread".to_string(),
        energy_current: 75.0,
        energy_max: 100.0,
    };
    let ron_str = player_save.to_ron().expect("Should serialize");
    fs::write(format!("{save_dir}player.ron"), &ron_str).expect("Should write");

    let world_save = WorldSave {
        schema_version: SAVE_VERSION,
        seed: 42,
        explored_chunks: vec![],
    };
    let world_ron = world_save.to_ron().expect("Should serialize");
    fs::write(format!("{save_dir}world.ron"), &world_ron).expect("Should write");

    // Create app with load_game
    let mut app = test_app();
    app.insert_resource(SaveConfig { save_dir: save_dir.to_string() });
    app.init_resource::<SaveState>();
    let player = spawn_player(&mut app);

    // Run load_game
    app.add_systems(Update, load_game);
    app.update();

    // Verify player position restored
    let transform = app
        .world()
        .entity(player)
        .get::<Transform>()
        .expect("Player should have Transform");
    assert!(
        (transform.translation.x - 500.0).abs() < 0.01,
        "Player X should be restored to 500.0, got {}",
        transform.translation.x
    );
    assert!(
        (transform.translation.y - (-300.0)).abs() < 0.01,
        "Player Y should be restored to -300.0, got {}",
        transform.translation.y
    );

    // Verify save_state
    let save_state = app.world().resource::<SaveState>();
    assert!(save_state.loaded_from_save, "Should be marked as loaded from save");

    cleanup_save_dir(save_dir);
}

#[test]
fn save_then_load_restores_explored_chunks() {
    let save_dir = "target/test_saves/chunks_test/";
    cleanup_save_dir(save_dir);
    fs::create_dir_all(save_dir).expect("Should create dir");

    // Write world save with explored chunks
    let world_save = WorldSave {
        schema_version: SAVE_VERSION,
        seed: 42,
        explored_chunks: vec![
            ((10, 20), "DeepSpace".to_string()),
            ((11, 20), "AsteroidField".to_string()),
            ((-10, 22), "WreckField".to_string()),
        ],
    };
    let world_ron = world_save.to_ron().expect("Should serialize");
    fs::write(format!("{save_dir}world.ron"), &world_ron).expect("Should write");

    // Create player save too (needed for load)
    let player_save = PlayerSave {
        schema_version: SAVE_VERSION,
        position: (0.0, 0.0),
        rotation: 0.0,
        velocity: (0.0, 0.0),
        health_current: 100.0,
        health_max: 100.0,
        active_weapon: "Laser".to_string(),
        energy_current: 100.0,
        energy_max: 100.0,
    };
    fs::write(format!("{save_dir}player.ron"), player_save.to_ron().expect("Should serialize"))
        .expect("Should write");

    // Create app
    let mut app = test_app();
    app.insert_resource(SaveConfig { save_dir: save_dir.to_string() });
    app.init_resource::<SaveState>();
    let _player = spawn_player(&mut app);

    // Run load_game
    app.add_systems(Update, load_game);
    app.update();

    // Verify explored chunks restored
    let explored = app.world().resource::<ExploredChunks>();
    assert!(
        explored.chunks.contains_key(&ChunkCoord { x: 10, y: 20 }),
        "Should contain chunk (10,20)"
    );
    assert_eq!(
        explored.chunks.get(&ChunkCoord { x: 10, y: 20 }),
        Some(&BiomeType::DeepSpace)
    );
    assert_eq!(
        explored.chunks.get(&ChunkCoord { x: 11, y: 20 }),
        Some(&BiomeType::AsteroidField)
    );
    assert_eq!(
        explored.chunks.get(&ChunkCoord { x: -10, y: 22 }),
        Some(&BiomeType::WreckField)
    );

    cleanup_save_dir(save_dir);
}

#[test]
fn load_missing_files_starts_fresh() {
    let save_dir = "target/test_saves/missing_test/";
    cleanup_save_dir(save_dir);

    let mut app = test_app();
    app.insert_resource(SaveConfig { save_dir: save_dir.to_string() });
    app.init_resource::<SaveState>();
    let player = spawn_player(&mut app);

    // Load with no save files — should not panic and player stays at origin
    app.add_systems(Update, load_game);
    app.update();

    let transform = app
        .world()
        .entity(player)
        .get::<Transform>()
        .expect("Player should have Transform");
    assert!(
        transform.translation.x.abs() < 0.01,
        "Player should be at origin X, got {}",
        transform.translation.x
    );
    assert!(
        transform.translation.y.abs() < 0.01,
        "Player should be at origin Y, got {}",
        transform.translation.y
    );

    // loaded_from_save should remain false
    let save_state = app.world().resource::<SaveState>();
    assert!(!save_state.loaded_from_save, "Should not be marked as loaded from save");
}

#[test]
fn load_corrupt_file_starts_fresh() {
    let save_dir = "target/test_saves/corrupt_test/";
    cleanup_save_dir(save_dir);
    fs::create_dir_all(save_dir).expect("Should create dir");

    // Write garbage data to save files
    fs::write(format!("{save_dir}player.ron"), "{{{{garbage not valid ron }}}}")
        .expect("Should write garbage");
    fs::write(format!("{save_dir}world.ron"), "also not valid")
        .expect("Should write garbage");

    let mut app = test_app();
    app.insert_resource(SaveConfig { save_dir: save_dir.to_string() });
    app.init_resource::<SaveState>();
    let player = spawn_player(&mut app);

    // Load with corrupt files — should not panic, player stays at defaults
    app.add_systems(Update, load_game);
    app.update();

    let transform = app
        .world()
        .entity(player)
        .get::<Transform>()
        .expect("Player should have Transform");
    assert!(
        transform.translation.x.abs() < 0.01,
        "Player should be at origin X after corrupt load, got {}",
        transform.translation.x
    );

    cleanup_save_dir(save_dir);
}

#[test]
fn load_restores_world_seed() {
    let save_dir = "target/test_saves/seed_test/";
    cleanup_save_dir(save_dir);
    fs::create_dir_all(save_dir).expect("Should create dir");

    // Write world save with a specific seed
    let world_save = WorldSave {
        schema_version: SAVE_VERSION,
        seed: 9999,
        explored_chunks: vec![],
    };
    fs::write(format!("{save_dir}world.ron"), world_save.to_ron().expect("Should serialize"))
        .expect("Should write");

    // Write minimal player save
    let player_save = PlayerSave {
        schema_version: SAVE_VERSION,
        position: (0.0, 0.0),
        rotation: 0.0,
        velocity: (0.0, 0.0),
        health_current: 100.0,
        health_max: 100.0,
        active_weapon: "Laser".to_string(),
        energy_current: 100.0,
        energy_max: 100.0,
    };
    fs::write(format!("{save_dir}player.ron"), player_save.to_ron().expect("Should serialize"))
        .expect("Should write");

    let mut app = test_app();
    app.insert_resource(SaveConfig { save_dir: save_dir.to_string() });
    app.init_resource::<SaveState>();
    let _player = spawn_player(&mut app);

    // Verify seed is NOT 9999 before load
    let seed_before = app.world().resource::<WorldConfig>().seed;
    assert_ne!(seed_before, 9999, "Seed should not be 9999 before load");

    // Run load_game
    app.add_systems(Update, load_game);
    app.update();

    // Verify seed restored
    let world_config = app.world().resource::<WorldConfig>();
    assert_eq!(world_config.seed, 9999, "Seed should be restored to 9999 from save");

    cleanup_save_dir(save_dir);
}

#[test]
fn save_game_creates_files() {
    let save_dir = "target/test_saves/save_creates/";
    cleanup_save_dir(save_dir);

    let mut app = test_app();
    app.insert_resource(SaveConfig { save_dir: save_dir.to_string() });
    app.init_resource::<SaveState>();
    app.add_systems(Update, save_game);
    let _player = spawn_player(&mut app);

    // Trigger save
    app.world_mut().resource_mut::<ActionState>().save = true;
    app.update();

    // Verify files exist
    assert!(
        fs::metadata(format!("{save_dir}player.ron")).is_ok(),
        "player.ron should exist after save"
    );
    assert!(
        fs::metadata(format!("{save_dir}world.ron")).is_ok(),
        "world.ron should exist after save"
    );

    // Verify RON is valid
    let player_ron = fs::read_to_string(format!("{save_dir}player.ron"))
        .expect("Should read player.ron");
    let player = PlayerSave::from_ron(&player_ron).expect("Should parse player.ron");
    assert_eq!(player.schema_version, SAVE_VERSION);

    let world_ron = fs::read_to_string(format!("{save_dir}world.ron"))
        .expect("Should read world.ron");
    let world = WorldSave::from_ron(&world_ron).expect("Should parse world.ron");
    assert_eq!(world.schema_version, SAVE_VERSION);

    cleanup_save_dir(save_dir);
}

#[test]
fn save_game_emits_game_saved_event() {
    let save_dir = "target/test_saves/event_test/";
    cleanup_save_dir(save_dir);

    let mut app = test_app();
    app.insert_resource(SaveConfig { save_dir: save_dir.to_string() });
    app.init_resource::<SaveState>();
    app.add_systems(Update, save_game);
    let _player = spawn_player(&mut app);

    // Trigger save
    app.world_mut().resource_mut::<ActionState>().save = true;
    app.update(); // save_game emits GameSaved event
    app.update(); // record_game_events processes message into logbook

    // Check logbook for GameSaved event
    let logbook = app.world().resource::<Logbook>();
    let has_game_saved = logbook.entries().iter().any(|entry| {
        matches!(entry.kind, GameEventKind::GameSaved)
    });
    assert!(has_game_saved, "Logbook should contain GameSaved event after save");

    cleanup_save_dir(save_dir);
}
