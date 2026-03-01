mod helpers;

use bevy::prelude::*;
use helpers::{spawn_player, test_app};
use std::fs;

use void_drifter::core::collision::Collider;
use void_drifter::core::input::ActionState;
use void_drifter::infrastructure::logbook::Logbook;
use void_drifter::infrastructure::save::{SaveConfig, SaveState, save_game, load_game};
use void_drifter::infrastructure::save::player_save::PlayerSave;
use void_drifter::infrastructure::save::world_save::WorldSave;
use void_drifter::infrastructure::save::schema::SAVE_VERSION;
use void_drifter::shared::events::GameEventKind;
use void_drifter::infrastructure::save::delta::{ChunkDelta, SeedIndex, WorldDeltas};
use void_drifter::world::{ActiveChunks, BiomeConfig, ChunkEntity, ChunkEntityIndex, ChunkLoadState, ExploredChunks, ChunkCoord, BiomeType, PendingChunks, WorldConfig};

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
        chunk_deltas: vec![],
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
        chunk_deltas: vec![],
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
        chunk_deltas: vec![],
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

#[test]
fn v1_save_loads_with_empty_deltas() {
    let save_dir = "target/test_saves/v1_migration/";
    cleanup_save_dir(save_dir);
    fs::create_dir_all(save_dir).expect("Should create dir");

    // Write a v1 world save (no chunk_deltas field)
    let v1_ron = r#"(
        schema_version: 1,
        seed: 42,
        explored_chunks: [
            ((0, 0), "DeepSpace"),
            ((1, 0), "AsteroidField"),
        ],
    )"#;
    fs::write(format!("{save_dir}world.ron"), v1_ron).expect("Should write");

    // Write minimal player save (v2)
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

    app.add_systems(Update, load_game);
    app.update();

    // Verify explored chunks loaded correctly
    let explored = app.world().resource::<ExploredChunks>();
    assert_eq!(explored.chunks.len(), 2, "Should have 2 explored chunks from v1 save");

    // Verify WorldDeltas is empty (v1 had no deltas)
    let deltas = app.world().resource::<WorldDeltas>();
    assert!(deltas.deltas.is_empty(), "WorldDeltas should be empty after v1 migration");

    cleanup_save_dir(save_dir);
}

#[test]
fn save_preserves_deltas_across_sessions() {
    let save_dir = "target/test_saves/delta_persist/";
    cleanup_save_dir(save_dir);
    fs::create_dir_all(save_dir).expect("Should create dir");

    // Write a v2 world save with deltas
    let world_save = WorldSave {
        schema_version: SAVE_VERSION,
        seed: 42,
        explored_chunks: vec![((0, 0), "DeepSpace".to_string())],
        chunk_deltas: vec![ChunkDelta {
            coord: ChunkCoord { x: 0, y: 0 },
            destroyed: vec![1, 3, 7],
        }],
    };
    let world_ron = world_save.to_ron().expect("Should serialize");
    fs::write(format!("{save_dir}world.ron"), &world_ron).expect("Should write");

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

    app.add_systems(Update, load_game);
    app.update();

    // Verify deltas were restored
    let deltas = app.world().resource::<WorldDeltas>();
    assert_eq!(deltas.deltas.len(), 1, "Should have 1 chunk delta");
    let chunk_delta = deltas
        .deltas
        .get(&ChunkCoord { x: 0, y: 0 })
        .expect("Should have delta for (0,0)");
    assert_eq!(chunk_delta.destroyed, vec![1, 3, 7]);

    cleanup_save_dir(save_dir);
}

#[test]
fn empty_deltas_same_as_no_deltas() {
    let save_dir = "target/test_saves/empty_deltas/";
    cleanup_save_dir(save_dir);
    fs::create_dir_all(save_dir).expect("Should create dir");

    // Save with empty WorldDeltas
    let world_save = WorldSave {
        schema_version: SAVE_VERSION,
        seed: 42,
        explored_chunks: vec![((0, 0), "DeepSpace".to_string())],
        chunk_deltas: vec![],
    };
    let world_ron = world_save.to_ron().expect("Should serialize");
    fs::write(format!("{save_dir}world.ron"), &world_ron).expect("Should write");

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

    app.add_systems(Update, load_game);
    app.update();

    // WorldDeltas should be empty
    let deltas = app.world().resource::<WorldDeltas>();
    assert!(deltas.deltas.is_empty(), "Empty deltas should result in empty WorldDeltas");

    // Explored chunks should load normally
    let explored = app.world().resource::<ExploredChunks>();
    assert_eq!(explored.chunks.len(), 1);

    cleanup_save_dir(save_dir);
}

#[test]
fn destroy_entity_then_save_load_stays_destroyed() {
    let save_dir = "target/test_saves/delta_destroy/";
    cleanup_save_dir(save_dir);

    // First app: load chunks, record entity count, simulate destruction via delta
    let mut app = test_app();
    spawn_player(&mut app);
    helpers::run_until_loaded(&mut app);

    // Count entities in chunk (0,0)
    let mut query = app
        .world_mut()
        .query_filtered::<(&ChunkEntity, &SeedIndex), With<void_drifter::core::collision::Collider>>();
    let entities_in_chunk_0: Vec<(ChunkCoord, u32)> = query
        .iter(app.world())
        .filter(|(ce, _)| ce.coord == ChunkCoord { x: 0, y: 0 })
        .map(|(ce, si)| (ce.coord, si.0))
        .collect();

    if entities_in_chunk_0.is_empty() {
        // If chunk (0,0) has no entities (possible with some seeds), skip test
        cleanup_save_dir(save_dir);
        return;
    }

    // Manually add a delta for the first entity in chunk (0,0)
    let destroyed_index = entities_in_chunk_0[0].1;
    let original_count = entities_in_chunk_0.len();
    {
        let mut deltas = app.world_mut().resource_mut::<WorldDeltas>();
        let delta = deltas
            .deltas
            .entry(ChunkCoord { x: 0, y: 0 })
            .or_insert_with(|| ChunkDelta {
                coord: ChunkCoord { x: 0, y: 0 },
                destroyed: Vec::new(),
            });
        delta.destroyed.push(destroyed_index);
    }

    // Save the game
    app.insert_resource(SaveConfig {
        save_dir: save_dir.to_string(),
    });
    app.init_resource::<SaveState>();
    app.world_mut().resource_mut::<ActionState>().save = true;
    app.add_systems(Update, save_game);
    app.update();

    // Verify save files exist
    assert!(
        fs::metadata(format!("{save_dir}world.ron")).is_ok(),
        "world.ron should exist after save"
    );

    // Second app: build from scratch so load_game runs at Startup before any chunks load
    use std::time::Duration;
    use bevy::time::TimeUpdateStrategy;

    let mut app2 = App::new();
    app2.add_plugins(MinimalPlugins);
    app2.init_resource::<ActionState>();
    app2.insert_resource(void_drifter::core::flight::FlightConfig::default());
    app2.insert_resource(void_drifter::core::weapons::WeaponConfig::default());
    app2.init_resource::<void_drifter::core::collision::DamageQueue>();
    app2.init_resource::<void_drifter::core::collision::DestroyedPositions>();
    app2.init_resource::<void_drifter::core::collision::LaserHitPositions>();
    app2.insert_resource(WorldConfig::default());
    app2.insert_resource(BiomeConfig::default());
    app2.init_resource::<ActiveChunks>();
    app2.init_resource::<ExploredChunks>();
    app2.init_resource::<ChunkEntityIndex>();
    app2.init_resource::<PendingChunks>();
    app2.init_resource::<ChunkLoadState>();
    app2.init_resource::<WorldDeltas>();
    app2.add_message::<void_drifter::shared::events::GameEvent>();
    app2.insert_resource(void_drifter::infrastructure::events::EventSeverityConfig::default());
    app2.init_resource::<void_drifter::infrastructure::logbook::Logbook>();
    app2.insert_resource(SaveConfig { save_dir: save_dir.to_string() });
    app2.init_resource::<SaveState>();

    // Add load_game to Startup so it runs BEFORE update_chunks
    app2.add_systems(Startup, load_game);
    app2.add_systems(FixedUpdate, void_drifter::world::update_chunks);
    app2.insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_secs_f64(1.0 / 60.0)));
    app2.insert_resource(Time::<Fixed>::from_seconds(1.0 / 60.0));

    // Spawn player with required components for load_game
    app2.world_mut().spawn((
        void_drifter::core::flight::Player,
        void_drifter::shared::components::Velocity::default(),
        void_drifter::core::collision::Health { current: 100.0, max: 100.0 },
        Collider { radius: 12.0 },
        void_drifter::core::weapons::FireCooldown::default(),
        void_drifter::core::weapons::Energy::default(),
        void_drifter::core::weapons::ActiveWeapon::default(),
        Transform::default(),
    ));

    // First update: Startup runs load_game (restores deltas), then FixedUpdate loads chunks
    app2.update();
    // Run enough frames for all chunks to load
    let wc = app2.world().resource::<WorldConfig>().clone();
    let total_chunks = (2 * wc.load_radius + 1).pow(2) as usize;
    let frames = total_chunks.div_ceil(wc.max_chunks_per_frame);
    for _ in 0..frames {
        app2.update();
    }

    // Count entities in chunk (0,0) in the new app
    let entities_after: Vec<u32> = {
        let mut query2 = app2
            .world_mut()
            .query_filtered::<(&ChunkEntity, &SeedIndex), With<Collider>>();
        query2
            .iter(app2.world())
            .filter(|(ce, _)| ce.coord == ChunkCoord { x: 0, y: 0 })
            .map(|(_, si)| si.0)
            .collect()
    };

    assert_eq!(
        entities_after.len(),
        original_count - 1,
        "Should have one fewer entity in chunk (0,0) after loading with delta"
    );
    assert!(
        !entities_after.contains(&destroyed_index),
        "Destroyed entity index {destroyed_index} should not be present after reload"
    );

    cleanup_save_dir(save_dir);
}

#[test]
fn golden_fixture_v1_loads_successfully() {
    let fixture_ron = include_str!("fixtures/saves/test_world_v1.ron");
    let world_save = WorldSave::from_ron(fixture_ron)
        .expect("Golden v1 fixture should load with auto-migration");

    assert_eq!(world_save.schema_version, SAVE_VERSION);
    assert_eq!(world_save.seed, 42);
    assert_eq!(world_save.explored_chunks.len(), 3);
    assert!(world_save.chunk_deltas.is_empty());
}

/// End-to-end: damage kills entity → track_destroyed_entities records delta →
/// save → fresh app → load → chunk loads without destroyed entity.
#[test]
fn e2e_damage_track_save_load_filters_entity() {
    use void_drifter::core::collision::DamageQueue;

    let save_dir = "target/test_saves/e2e_track/";
    cleanup_save_dir(save_dir);

    // First app: load chunks, find an entity, kill it via DamageQueue
    let mut app = test_app();
    spawn_player(&mut app);
    helpers::run_until_loaded(&mut app);

    // Find first chunk entity in chunk (0,0) with SeedIndex
    let target = {
        let mut query = app
            .world_mut()
            .query_filtered::<(Entity, &ChunkEntity, &SeedIndex), With<Collider>>();
        query
            .iter(app.world())
            .find(|(_, ce, _)| ce.coord == ChunkCoord { x: 0, y: 0 })
            .map(|(e, ce, si)| (e, ce.coord, si.0))
    };

    let Some((target_entity, target_coord, target_seed_idx)) = target else {
        // Chunk (0,0) has no entities with this seed — skip
        cleanup_save_dir(save_dir);
        return;
    };

    // Count original entities in that chunk
    let original_count = {
        let mut query = app
            .world_mut()
            .query_filtered::<&ChunkEntity, With<Collider>>();
        query
            .iter(app.world())
            .filter(|ce| ce.coord == target_coord)
            .count()
    };

    // Kill the entity by setting health to 0 via DamageQueue
    {
        let mut dq = app.world_mut().resource_mut::<DamageQueue>();
        dq.entries.push((target_entity, 9999.0));
    }

    // Run frame: apply_damage → track_destroyed_entities → despawn_destroyed
    app.update();

    // Verify track_destroyed_entities recorded the delta
    let deltas = app.world().resource::<WorldDeltas>();
    let chunk_delta = deltas
        .deltas
        .get(&target_coord)
        .expect("WorldDeltas should have delta for chunk after entity destruction");
    assert!(
        chunk_delta.destroyed.contains(&target_seed_idx),
        "Destroyed seed index {} should be in chunk delta",
        target_seed_idx
    );

    // Save the game
    app.insert_resource(SaveConfig {
        save_dir: save_dir.to_string(),
    });
    app.init_resource::<SaveState>();
    app.world_mut().resource_mut::<ActionState>().save = true;
    app.add_systems(Update, save_game);
    app.update();

    // Second app: load from save, verify destroyed entity is filtered out
    use std::time::Duration;
    use bevy::time::TimeUpdateStrategy;

    let mut app2 = App::new();
    app2.add_plugins(MinimalPlugins);
    app2.init_resource::<ActionState>();
    app2.insert_resource(void_drifter::core::flight::FlightConfig::default());
    app2.insert_resource(void_drifter::core::weapons::WeaponConfig::default());
    app2.init_resource::<void_drifter::core::collision::DamageQueue>();
    app2.init_resource::<void_drifter::core::collision::DestroyedPositions>();
    app2.init_resource::<void_drifter::core::collision::LaserHitPositions>();
    app2.insert_resource(WorldConfig::default());
    app2.insert_resource(BiomeConfig::default());
    app2.init_resource::<ActiveChunks>();
    app2.init_resource::<ExploredChunks>();
    app2.init_resource::<ChunkEntityIndex>();
    app2.init_resource::<PendingChunks>();
    app2.init_resource::<ChunkLoadState>();
    app2.init_resource::<WorldDeltas>();
    app2.add_message::<void_drifter::shared::events::GameEvent>();
    app2.insert_resource(void_drifter::infrastructure::events::EventSeverityConfig::default());
    app2.init_resource::<void_drifter::infrastructure::logbook::Logbook>();
    app2.insert_resource(SaveConfig { save_dir: save_dir.to_string() });
    app2.init_resource::<SaveState>();

    app2.add_systems(Startup, load_game);
    app2.add_systems(FixedUpdate, void_drifter::world::update_chunks);
    app2.insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_secs_f64(1.0 / 60.0)));
    app2.insert_resource(Time::<Fixed>::from_seconds(1.0 / 60.0));

    app2.world_mut().spawn((
        void_drifter::core::flight::Player,
        void_drifter::shared::components::Velocity::default(),
        void_drifter::core::collision::Health { current: 100.0, max: 100.0 },
        Collider { radius: 12.0 },
        void_drifter::core::weapons::FireCooldown::default(),
        void_drifter::core::weapons::Energy::default(),
        void_drifter::core::weapons::ActiveWeapon::default(),
        Transform::default(),
    ));

    // Load + run enough frames
    app2.update();
    let wc = app2.world().resource::<WorldConfig>().clone();
    let total_chunks = (2 * wc.load_radius + 1).pow(2) as usize;
    let frames = total_chunks.div_ceil(wc.max_chunks_per_frame);
    for _ in 0..frames {
        app2.update();
    }

    // Count entities in target chunk — should be one fewer
    let entities_after: Vec<u32> = {
        let mut query2 = app2
            .world_mut()
            .query_filtered::<(&ChunkEntity, &SeedIndex), With<Collider>>();
        query2
            .iter(app2.world())
            .filter(|(ce, _)| ce.coord == target_coord)
            .map(|(_, si)| si.0)
            .collect()
    };

    assert_eq!(
        entities_after.len(),
        original_count - 1,
        "Should have one fewer entity after E2E damage → track → save → load"
    );
    assert!(
        !entities_after.contains(&target_seed_idx),
        "Destroyed entity seed index {} should not be present after reload",
        target_seed_idx
    );

    cleanup_save_dir(save_dir);
}
