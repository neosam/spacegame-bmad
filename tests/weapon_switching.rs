mod helpers;

use helpers::{spawn_player, test_app};
use void_drifter::core::input::ActionState;
use void_drifter::core::tutorial::SpreadUnlocked;
use void_drifter::core::weapons::{
    ActiveWeapon, Energy, FireCooldown, LaserPulse, SpreadProjectile,
};

/// Spawn a player with SpreadUnlocked for weapon-switching tests.
fn spawn_player_with_spread(app: &mut bevy::prelude::App) -> bevy::prelude::Entity {
    let entity = spawn_player(app);
    app.world_mut().entity_mut(entity).insert(SpreadUnlocked);
    entity
}

#[test]
fn switch_to_spread_then_fire_spawns_spread_projectiles() {
    let mut app = test_app();
    let entity = spawn_player_with_spread(&mut app);

    // Verify starts as Laser
    let weapon = app
        .world()
        .entity(entity)
        .get::<ActiveWeapon>()
        .expect("Should have ActiveWeapon");
    assert_eq!(*weapon, ActiveWeapon::Laser);

    // Switch weapon
    app.world_mut().resource_mut::<ActionState>().switch_weapon = true;
    app.update();

    // Verify switched to Spread
    let weapon = app
        .world()
        .entity(entity)
        .get::<ActiveWeapon>()
        .expect("Should have ActiveWeapon");
    assert_eq!(
        *weapon,
        ActiveWeapon::Spread,
        "Should have switched to Spread"
    );

    // Fire (reset switch_weapon first — no read_input in test to auto-reset)
    {
        let mut action = app.world_mut().resource_mut::<ActionState>();
        action.switch_weapon = false;
        action.fire = true;
    }
    app.update();

    let spread_count = app
        .world_mut()
        .query::<&SpreadProjectile>()
        .iter(app.world())
        .count();
    let laser_count = app
        .world_mut()
        .query::<&LaserPulse>()
        .iter(app.world())
        .count();

    assert!(
        spread_count > 0,
        "Should spawn SpreadProjectiles after switching to Spread"
    );
    assert_eq!(
        laser_count, 0,
        "Should NOT spawn LaserPulse when ActiveWeapon is Spread"
    );
}

#[test]
fn switch_back_to_laser_then_fire_spawns_laser_pulse() {
    let mut app = test_app();
    let entity = spawn_player_with_spread(&mut app);

    // Switch to Spread
    app.world_mut().resource_mut::<ActionState>().switch_weapon = true;
    app.update();

    let weapon = app
        .world()
        .entity(entity)
        .get::<ActiveWeapon>()
        .expect("Should have ActiveWeapon");
    assert_eq!(*weapon, ActiveWeapon::Spread);

    // Switch back to Laser
    app.world_mut().resource_mut::<ActionState>().switch_weapon = true;
    app.update();

    let weapon = app
        .world()
        .entity(entity)
        .get::<ActiveWeapon>()
        .expect("Should have ActiveWeapon");
    assert_eq!(
        *weapon,
        ActiveWeapon::Laser,
        "Should have switched back to Laser"
    );

    // Fire (reset switch_weapon first — no read_input in test to auto-reset)
    {
        let mut action = app.world_mut().resource_mut::<ActionState>();
        action.switch_weapon = false;
        action.fire = true;
    }
    app.update();

    let laser_count = app
        .world_mut()
        .query::<&LaserPulse>()
        .iter(app.world())
        .count();
    let spread_count = app
        .world_mut()
        .query::<&SpreadProjectile>()
        .iter(app.world())
        .count();

    assert!(
        laser_count > 0,
        "Should spawn LaserPulse after switching back to Laser"
    );
    assert_eq!(
        spread_count, 0,
        "Should NOT spawn SpreadProjectile when ActiveWeapon is Laser"
    );
}

#[test]
fn fire_cooldown_persists_across_weapon_switch() {
    let mut app = test_app();
    let entity = spawn_player_with_spread(&mut app);

    // Fire laser to set cooldown
    app.world_mut().resource_mut::<ActionState>().fire = true;
    app.update();

    let cooldown_after_fire = app
        .world()
        .entity(entity)
        .get::<FireCooldown>()
        .expect("Should have FireCooldown")
        .timer;
    assert!(
        cooldown_after_fire > 0.0,
        "Cooldown should be set after firing"
    );

    // Switch weapon
    app.world_mut().resource_mut::<ActionState>().fire = false;
    app.world_mut().resource_mut::<ActionState>().switch_weapon = true;
    app.update();

    let cooldown_after_switch = app
        .world()
        .entity(entity)
        .get::<FireCooldown>()
        .expect("Should have FireCooldown")
        .timer;

    // Cooldown should still be positive (just decremented by one frame)
    // It was set to 1/fire_rate = 0.25s, minus 1/60s ≈ 0.233s
    assert!(
        cooldown_after_switch > 0.0,
        "Fire cooldown should persist across weapon switch, got {}",
        cooldown_after_switch
    );
}

#[test]
fn energy_persists_across_weapon_switch() {
    let mut app = test_app();
    let entity = spawn_player_with_spread(&mut app);

    // Drain some energy
    app.world_mut()
        .entity_mut(entity)
        .get_mut::<Energy>()
        .expect("Player should have Energy")
        .current = 50.0;

    // Switch weapon
    app.world_mut().resource_mut::<ActionState>().switch_weapon = true;
    app.update();

    let energy = app
        .world()
        .entity(entity)
        .get::<Energy>()
        .expect("Should have Energy")
        .current;

    // Energy should be >= 50 (may have regenerated slightly, but should NOT have been reset)
    assert!(
        energy >= 50.0 && energy < 55.0,
        "Energy should persist across weapon switch (expect ~50 + small regen), got {}",
        energy
    );
}

#[test]
fn multiple_rapid_switches_settle_on_correct_weapon() {
    let mut app = test_app();
    let entity = spawn_player_with_spread(&mut app);

    // Switch 3 times: Laser → Spread → Laser → Spread
    for _ in 0..3 {
        app.world_mut().resource_mut::<ActionState>().switch_weapon = true;
        app.update();
    }

    // 3 switches from Laser: L→S→L→S = Spread
    let weapon = app
        .world()
        .entity(entity)
        .get::<ActiveWeapon>()
        .expect("Should have ActiveWeapon");
    assert_eq!(
        *weapon,
        ActiveWeapon::Spread,
        "After 3 switches from Laser, should be Spread (odd number of toggles)"
    );

    // Switch once more → back to Laser
    app.world_mut().resource_mut::<ActionState>().switch_weapon = true;
    app.update();

    let weapon = app
        .world()
        .entity(entity)
        .get::<ActiveWeapon>()
        .expect("Should have ActiveWeapon");
    assert_eq!(
        *weapon,
        ActiveWeapon::Laser,
        "After 4 switches from Laser, should be back to Laser (even number of toggles)"
    );
}
