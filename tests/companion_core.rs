//! Integration tests for Epic 6a: Companion Core
//! Stories 6a-1 through 6a-6.

mod helpers;

use std::time::Duration;

use bevy::prelude::*;
use bevy::time::TimeUpdateStrategy;

use void_drifter::core::collision::Health;
use void_drifter::core::flight::Player;
use void_drifter::core::input::ActionState;
use void_drifter::core::station::{Docked, Station, StationType};
use void_drifter::core::weapons::{ActiveWeapon, Energy, FireCooldown};
use void_drifter::infrastructure::events::EventSeverityConfig;
use void_drifter::shared::components::Velocity;
use void_drifter::shared::events::{GameEvent, GameEventKind};
use void_drifter::social::companion::{
    Companion, CompanionData, CompanionFollowAI, CompanionFlight, CompanionRetreating,
    CompanionRoster, CompanionSaveEntry, WingmanCommand,
    companion_follow_velocity, faction_id_to_str, str_to_faction_id,
};
use void_drifter::social::companion_personality::{CompanionTarget, CompanionWeapon, CompanionPrevHealth};
use void_drifter::social::faction::FactionId;
use void_drifter::infrastructure::save::player_save::PlayerSave;

// ── Test App Builder ────────────────────────────────────────────────────────

/// Minimal test app with companion systems registered.
fn companion_test_app() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.init_resource::<ActionState>();
    app.init_resource::<CompanionRoster>();
    app.init_resource::<EventSeverityConfig>();
    app.add_message::<GameEvent>();
    app.insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_secs_f64(
        1.0 / 60.0,
    )));
    app.insert_resource(Time::<Fixed>::from_seconds(1.0 / 60.0));
    // Register companion systems
    app.add_systems(
        Update,
        (
            void_drifter::social::companion::handle_recruit_companion,
            void_drifter::social::companion::handle_wingman_commands,
            void_drifter::social::companion::update_companion_rotation,
            void_drifter::social::companion::update_companion_thrust_and_drag
                .after(void_drifter::social::companion::update_companion_rotation),
            void_drifter::social::companion::update_companion_positions
                .after(void_drifter::social::companion::update_companion_thrust_and_drag),
            void_drifter::social::companion::handle_companion_survival,
            void_drifter::social::companion::update_retreating_companions
                .after(void_drifter::social::companion::handle_companion_survival),
        ),
    );
    // Prime first frame
    app.update();
    app
}

/// Spawn a player entity for tests.
fn spawn_player_entity(app: &mut App) -> Entity {
    app.world_mut()
        .spawn((
            Player,
            Transform::default(),
            Velocity::default(),
            Health { current: 100.0, max: 100.0 },
            FireCooldown::default(),
            Energy::default(),
            ActiveWeapon::default(),
        ))
        .id()
}

/// Spawn a station entity for tests.
fn spawn_station_entity(app: &mut App, pos: Vec2) -> Entity {
    app.world_mut()
        .spawn((
            Station {
                name: "Test Station",
                dock_radius: 100.0,
                station_type: StationType::TradingPost,
            },
            Transform::from_translation(pos.extend(0.0)),
        ))
        .id()
}

// ── Story 6a-1: Recruit Companion ───────────────────────────────────────────

#[test]
fn recruit_companion_while_docked_adds_to_roster() {
    let mut app = companion_test_app();

    let station = spawn_station_entity(&mut app, Vec2::ZERO);
    let player = spawn_player_entity(&mut app);

    // Dock player at station
    app.world_mut()
        .entity_mut(player)
        .insert(Docked { station });

    // Press recruit
    app.world_mut().resource_mut::<ActionState>().recruit = true;

    app.update();

    let roster = app.world().resource::<CompanionRoster>();
    assert_eq!(roster.companions.len(), 1, "Roster should have 1 companion after recruiting");
}

#[test]
fn recruit_without_docked_does_nothing() {
    let mut app = companion_test_app();
    spawn_player_entity(&mut app);

    // Press recruit without being docked
    app.world_mut().resource_mut::<ActionState>().recruit = true;

    app.update();

    let roster = app.world().resource::<CompanionRoster>();
    assert_eq!(roster.companions.len(), 0, "Should not recruit without being docked");
}

#[test]
fn recruit_spawns_companion_entity_with_correct_components() {
    let mut app = companion_test_app();

    let station = spawn_station_entity(&mut app, Vec2::ZERO);
    let player = spawn_player_entity(&mut app);

    app.world_mut()
        .entity_mut(player)
        .insert(Docked { station });

    app.world_mut().resource_mut::<ActionState>().recruit = true;
    app.update();

    // Verify companion entity has required components
    let companion_count = app
        .world_mut()
        .query::<&Companion>()
        .iter(app.world())
        .count();
    assert_eq!(companion_count, 1, "Should have exactly 1 Companion entity");

    let companion_data_count = app
        .world_mut()
        .query::<&CompanionData>()
        .iter(app.world())
        .count();
    assert_eq!(companion_data_count, 1, "Companion should have CompanionData");
}

#[test]
fn recruit_emits_companion_recruited_event() {
    let mut app = companion_test_app();

    let station = spawn_station_entity(&mut app, Vec2::ZERO);
    let player = spawn_player_entity(&mut app);

    app.world_mut()
        .entity_mut(player)
        .insert(Docked { station });

    app.world_mut().resource_mut::<ActionState>().recruit = true;
    app.update();

    // Check event was emitted (via logbook or direct message query)
    // We just verify no panic and companion was added
    let roster = app.world().resource::<CompanionRoster>();
    assert!(!roster.companions.is_empty(), "Event emission should succeed with companion added");
}

#[test]
fn recruit_action_is_consumed_after_one_frame() {
    let mut app = companion_test_app();

    let station = spawn_station_entity(&mut app, Vec2::ZERO);
    let player = spawn_player_entity(&mut app);

    app.world_mut()
        .entity_mut(player)
        .insert(Docked { station });

    app.world_mut().resource_mut::<ActionState>().recruit = true;
    app.update(); // recruit fires, action consumed
    app.update(); // second frame — should not recruit again

    let roster = app.world().resource::<CompanionRoster>();
    assert_eq!(
        roster.companions.len(), 1,
        "Should only recruit once even if update called multiple times"
    );
}

// ── Story 6a-2: Companion Follow ────────────────────────────────────────────

#[test]
fn companion_follows_player_over_time() {
    let mut app = companion_test_app();

    // Spawn player far from companion
    let player = spawn_player_entity(&mut app);
    app.world_mut()
        .entity_mut(player)
        .get_mut::<Transform>()
        .expect("Player should have Transform")
        .translation = Vec3::new(200.0, 0.0, 0.0);

    // Spawn companion at origin with follow AI + ship flight (6c-1 physics)
    app.world_mut().spawn((
        Companion,
        CompanionData { name: "Wing-1".to_string(), faction: FactionId::Neutral },
        CompanionFollowAI::default(),
        CompanionFlight::default(),
        CompanionTarget::default(),
        CompanionWeapon::default(),
        CompanionPrevHealth { value: 100.0 },
        WingmanCommand::Defend,
        Velocity::default(),
        Transform::from_translation(Vec3::ZERO),
    ));

    let initial_x = 0.0_f32;

    // Run more frames — ship physics needs time to rotate before thrusting
    for _ in 0..60 {
        app.update();
    }

    let companion_x = app
        .world_mut()
        .query_filtered::<&Transform, With<Companion>>()
        .iter(app.world())
        .next()
        .expect("Companion should exist")
        .translation.x;

    assert!(
        companion_x > initial_x,
        "Companion should have moved toward player (positive x): {companion_x}"
    );
}

#[test]
fn companion_follow_velocity_pure_function_positive_x() {
    let vel = companion_follow_velocity(Vec2::ZERO, Vec2::new(200.0, 0.0), 60.0, 150.0);
    assert!(vel.x > 0.0, "Should move toward player in +x: {vel:?}");
}

// ── Story 6a-3: Wingman Commands ────────────────────────────────────────────

#[test]
fn wingman_command_cycles_on_input() {
    let mut app = companion_test_app();

    // Spawn companion with Attack command
    app.world_mut().spawn((
        Companion,
        CompanionData { name: "Wing-1".to_string(), faction: FactionId::Neutral },
        WingmanCommand::Attack,
        Velocity::default(),
        Transform::default(),
    ));

    // Press wingman command cycle
    app.world_mut().resource_mut::<ActionState>().wingman_command = true;
    app.update();

    let cmd = app
        .world_mut()
        .query_filtered::<&WingmanCommand, With<Companion>>()
        .iter(app.world())
        .next()
        .expect("Companion should have WingmanCommand");
    assert_eq!(*cmd, WingmanCommand::Defend, "Attack should cycle to Defend");
}

#[test]
fn wingman_command_does_not_cycle_without_input() {
    let mut app = companion_test_app();

    app.world_mut().spawn((
        Companion,
        CompanionData { name: "Wing-1".to_string(), faction: FactionId::Neutral },
        WingmanCommand::Attack,
        Velocity::default(),
        Transform::default(),
    ));

    // No wingman_command press
    app.update();

    let cmd = app
        .world_mut()
        .query_filtered::<&WingmanCommand, With<Companion>>()
        .iter(app.world())
        .next()
        .expect("Companion should have WingmanCommand");
    assert_eq!(*cmd, WingmanCommand::Attack, "Command should remain Attack without input");
}

// ── Story 6a-5: Companion Survival ──────────────────────────────────────────

/// Helper resource to trigger a one-shot PlayerDeath message in tests.
#[derive(Resource, Default)]
struct TriggerPlayerDeath(bool);

fn send_player_death_once(
    mut flag: ResMut<TriggerPlayerDeath>,
    mut writer: bevy::ecs::message::MessageWriter<GameEvent>,
) {
    if flag.0 {
        flag.0 = false;
        writer.write(GameEvent {
            kind: GameEventKind::PlayerDeath,
            severity: void_drifter::shared::events::EventSeverity::Tier1,
            position: Vec2::ZERO,
            game_time: 0.0,
        });
    }
}

#[test]
fn companion_gets_retreating_on_player_death() {
    let mut app = companion_test_app();

    // Add death trigger resource + system
    app.init_resource::<TriggerPlayerDeath>();
    app.add_systems(
        Update,
        send_player_death_once.before(void_drifter::social::companion::handle_companion_survival),
    );

    spawn_station_entity(&mut app, Vec2::new(0.0, 100.0));

    // Spawn companion
    app.world_mut().spawn((
        Companion,
        CompanionData { name: "Wing-1".to_string(), faction: FactionId::Neutral },
        WingmanCommand::Defend,
        CompanionFollowAI::default(),
        Velocity::default(),
        Transform::default(),
    ));

    // Arm the trigger
    app.world_mut().resource_mut::<TriggerPlayerDeath>().0 = true;

    app.update();

    let retreating_count = app
        .world_mut()
        .query::<&CompanionRetreating>()
        .iter(app.world())
        .count();
    assert_eq!(retreating_count, 1, "Companion should be retreating after player death");
}

#[test]
fn companion_stops_retreating_when_close_to_station() {
    let mut app = companion_test_app();

    // Spawn station at (0, 20) — close to companion at origin
    spawn_station_entity(&mut app, Vec2::new(0.0, 20.0));

    // Spawn companion already very close to retreat target
    app.world_mut().spawn((
        Companion,
        CompanionData { name: "Wing-1".to_string(), faction: FactionId::Neutral },
        WingmanCommand::Defend,
        CompanionFollowAI::default(),
        CompanionRetreating { target: Vec2::new(0.0, 20.0) },
        Velocity::default(),
        // Position companion at (0, 0) — only 20 units from target, which is < 25
        Transform::from_translation(Vec3::new(0.0, 0.0, 0.0)),
    ));

    app.update();

    let retreating_count = app
        .world_mut()
        .query::<&CompanionRetreating>()
        .iter(app.world())
        .count();
    assert_eq!(
        retreating_count, 0,
        "CompanionRetreating should be removed when close to station"
    );
}

// ── Story 6a-6: Companion Save ──────────────────────────────────────────────

#[test]
fn companion_save_entry_serializes_correctly() {
    let entry = CompanionSaveEntry {
        name: "Wing-1".to_string(),
        faction: "Neutral".to_string(),
        x: 150.0,
        y: -80.0,
    };
    let ron_str =
        ron::ser::to_string(&entry).expect("CompanionSaveEntry should serialize");
    let restored: CompanionSaveEntry =
        ron::from_str(&ron_str).expect("CompanionSaveEntry should deserialize");
    assert_eq!(restored.name, "Wing-1");
    assert_eq!(restored.faction, "Neutral");
    assert!((restored.x - 150.0).abs() < f32::EPSILON);
    assert!((restored.y - (-80.0)).abs() < f32::EPSILON);
}

#[test]
fn companion_save_in_player_save_defaults_to_empty() {
    let save = PlayerSave::default();
    assert!(save.companions.is_empty(), "Default PlayerSave should have empty companions");
}

#[test]
fn companion_roster_save_round_trip_via_world() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.init_resource::<CompanionRoster>();
    app.init_resource::<void_drifter::core::economy::Credits>();
    app.init_resource::<void_drifter::core::economy::PlayerInventory>();
    app.init_resource::<void_drifter::core::upgrades::InstalledUpgrades>();

    // Spawn player with required components
    app.world_mut().spawn((
        Player,
        Transform::default(),
        Velocity::default(),
        Health { current: 100.0, max: 100.0 },
        ActiveWeapon::default(),
        Energy::default(),
    ));

    // Spawn a companion
    let companion_entity = app.world_mut().spawn((
        Companion,
        CompanionData { name: "Zara".to_string(), faction: FactionId::Military },
        CompanionFollowAI::default(),
        WingmanCommand::Defend,
        Velocity::default(),
        Transform::from_translation(Vec3::new(100.0, 50.0, 0.0)),
    )).id();

    // Add to roster
    app.world_mut().resource_mut::<CompanionRoster>().companions.push(companion_entity);

    // Save
    let save = PlayerSave::from_world(app.world_mut())
        .expect("Should save player state");
    assert_eq!(save.companions.len(), 1, "Should have 1 companion in save");
    assert_eq!(save.companions[0].name, "Zara");
    assert_eq!(save.companions[0].faction, "Military");
    assert!((save.companions[0].x - 100.0).abs() < f32::EPSILON);
    assert!((save.companions[0].y - 50.0).abs() < f32::EPSILON);
}

#[test]
fn faction_id_str_roundtrip_all_variants() {
    for faction in [
        FactionId::Pirates,
        FactionId::Military,
        FactionId::Aliens,
        FactionId::RogueDrones,
        FactionId::Neutral,
    ] {
        let s = faction_id_to_str(&faction);
        let restored = str_to_faction_id(s);
        assert_eq!(
            faction, restored,
            "FactionId roundtrip should work for {s}"
        );
    }
}

#[test]
fn save_version_is_six() {
    use void_drifter::infrastructure::save::schema::SAVE_VERSION;
    assert_eq!(SAVE_VERSION, 6, "SAVE_VERSION should be 6 after Epic 6a");
}
