#![deny(clippy::unwrap_used)]

mod helpers;

use bevy::prelude::*;
use void_drifter::core::economy::{Credits, PlayerInventory};
use void_drifter::core::flight::{FlightConfig, Player};
use void_drifter::core::station::Docked;
use void_drifter::core::upgrades::{
    can_craft, compute_upgrade_multiplier, default_tier1_recipes, AcquisitionMethod,
    CraftingRecipe, CraftingRequest, DiscoveredRecipes, InstalledUpgrades, PendingCraftEvents,
    ShipSystem, WeaponSystem, BaseStats,
};
use void_drifter::core::weapons::WeaponConfig;
use void_drifter::shared::components::MaterialType;
use void_drifter::infrastructure::events::EventSeverityConfig;
use void_drifter::infrastructure::save::player_save::PlayerSave;
use void_drifter::infrastructure::save::schema::SAVE_VERSION;

// ── Pure function tests ───────────────────────────────────────────────────

#[test]
fn can_craft_with_sufficient_materials() {
    let recipe = CraftingRecipe {
        ship_system: Some(ShipSystem::Thrust),
        weapon_system: None,
        tier: 1,
        cost_common_scrap: 3,
        cost_rare_alloy: 1,
        cost_energy_core: 0,
        credit_cost: 20,
        acquisition: AcquisitionMethod::CraftOnly,
        display_name: "Thrust I",
    };
    let mut inventory = PlayerInventory::default();
    inventory.items.insert(MaterialType::CommonScrap, 5);
    inventory.items.insert(MaterialType::RareAlloy, 2);
    let credits = Credits { balance: 50 };

    assert!(
        can_craft(&recipe, &inventory, &credits),
        "Should be craftable with sufficient materials and credits"
    );
}

#[test]
fn can_craft_fails_insufficient_scrap() {
    let recipe = CraftingRecipe {
        ship_system: Some(ShipSystem::Thrust),
        weapon_system: None,
        tier: 1,
        cost_common_scrap: 3,
        cost_rare_alloy: 0,
        cost_energy_core: 0,
        credit_cost: 20,
        acquisition: AcquisitionMethod::CraftOnly,
        display_name: "Thrust I",
    };
    let mut inventory = PlayerInventory::default();
    inventory.items.insert(MaterialType::CommonScrap, 2); // only 2, need 3
    let credits = Credits { balance: 50 };

    assert!(
        !can_craft(&recipe, &inventory, &credits),
        "Should fail when insufficient CommonScrap"
    );
}

#[test]
fn can_craft_fails_insufficient_credits() {
    let recipe = CraftingRecipe {
        ship_system: Some(ShipSystem::Thrust),
        weapon_system: None,
        tier: 1,
        cost_common_scrap: 3,
        cost_rare_alloy: 0,
        cost_energy_core: 0,
        credit_cost: 20,
        acquisition: AcquisitionMethod::CraftOnly,
        display_name: "Thrust I",
    };
    let mut inventory = PlayerInventory::default();
    inventory.items.insert(MaterialType::CommonScrap, 5);
    let credits = Credits { balance: 10 }; // only 10, need 20

    assert!(
        !can_craft(&recipe, &inventory, &credits),
        "Should fail when insufficient credits"
    );
}

#[test]
fn upgrade_multiplier_scales_correctly() {
    assert!(
        (compute_upgrade_multiplier(0) - 1.0).abs() < f32::EPSILON,
        "Tier 0 multiplier should be 1.0"
    );
    assert!(
        (compute_upgrade_multiplier(1) - 1.1).abs() < 0.001,
        "Tier 1 multiplier should be 1.1"
    );
    assert!(
        (compute_upgrade_multiplier(3) - 1.3).abs() < 0.001,
        "Tier 3 multiplier should be 1.3"
    );
    assert!(
        (compute_upgrade_multiplier(5) - 1.5).abs() < 0.001,
        "Tier 5 multiplier should be 1.5"
    );
}

#[test]
fn discover_recipe_adds_to_discovered() {
    let mut discovered = DiscoveredRecipes::default();
    let initial_count = discovered.recipes.len();

    // Add a new tier-2 recipe
    let new_recipe = CraftingRecipe {
        ship_system: Some(ShipSystem::Thrust),
        weapon_system: None,
        tier: 2,
        cost_common_scrap: 5,
        cost_rare_alloy: 2,
        cost_energy_core: 0,
        credit_cost: 50,
        acquisition: AcquisitionMethod::CraftOnly,
        display_name: "Thrust II",
    };
    discovered.recipes.push(new_recipe);

    assert_eq!(
        discovered.recipes.len(),
        initial_count + 1,
        "Adding a recipe should increase discovered recipes count"
    );
    assert_eq!(
        discovered.recipes.last().expect("Should have last recipe").tier,
        2,
        "Last recipe should be tier 2"
    );
}

#[test]
fn upgrade_save_load_roundtrip() {
    let save = PlayerSave {
        schema_version: SAVE_VERSION,
        position: (0.0, 0.0),
        rotation: 0.0,
        velocity: (0.0, 0.0),
        health_current: 100.0,
        health_max: 100.0,
        active_weapon: "Laser".to_string(),
        energy_current: 100.0,
        energy_max: 100.0,
        credits: 500,
        inventory_common_scrap: 0,
        inventory_rare_alloy: 0,
        inventory_energy_core: 0,
        upgrade_ship_thrust: 2,
        upgrade_ship_max_speed: 1,
        upgrade_ship_rotation: 0,
        upgrade_ship_energy_capacity: 3,
        upgrade_ship_energy_regen: 0,
        upgrade_ship_scanner_range: 1,
        upgrade_ship_hull_strength: 0,
        upgrade_ship_cargo_capacity: 0,
        upgrade_weapon_laser_damage: 1,
        upgrade_weapon_laser_fire_rate: 0,
        upgrade_weapon_spread_damage: 2,
        upgrade_weapon_spread_fire_rate: 0,
        upgrade_weapon_energy_efficiency: 1,
        companions: Vec::new(),
        logbook_entries: Vec::new(),
        cleared_wormholes: Vec::new(),
            tutorial_complete: false,
    };

    let ron_str = save.to_ron().expect("Should serialize to RON");
    let restored = PlayerSave::from_ron(&ron_str).expect("Should deserialize from RON");

    assert_eq!(
        restored.upgrade_ship_thrust, 2,
        "Thrust upgrade tier should round-trip"
    );
    assert_eq!(
        restored.upgrade_ship_max_speed, 1,
        "MaxSpeed upgrade tier should round-trip"
    );
    assert_eq!(
        restored.upgrade_ship_energy_capacity, 3,
        "EnergyCapacity upgrade tier should round-trip"
    );
    assert_eq!(
        restored.upgrade_weapon_laser_damage, 1,
        "LaserDamage weapon upgrade should round-trip"
    );
    assert_eq!(
        restored.upgrade_weapon_spread_damage, 2,
        "SpreadDamage weapon upgrade should round-trip"
    );
    assert_eq!(
        restored.upgrade_weapon_energy_efficiency, 1,
        "EnergyEfficiency weapon upgrade should round-trip"
    );
}

// ── App-level tests ───────────────────────────────────────────────────────

/// Creates a minimal app with the upgrade-system resources and systems registered.
fn upgrade_test_app() -> App {
    use void_drifter::core::upgrades::{apply_upgrade_effects, process_crafting_request, init_base_stats};

    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.insert_resource(FlightConfig::default());
    app.insert_resource(WeaponConfig::default());
    app.insert_resource(EventSeverityConfig::default());
    app.init_resource::<Credits>();
    app.init_resource::<PlayerInventory>();
    app.init_resource::<InstalledUpgrades>();
    app.init_resource::<DiscoveredRecipes>();
    app.init_resource::<CraftingRequest>();
    app.init_resource::<PendingCraftEvents>();
    app.add_message::<void_drifter::shared::events::GameEvent>();

    // init_base_stats runs at Startup
    app.add_systems(Startup, init_base_stats);
    app.add_systems(
        Update,
        (process_crafting_request, apply_upgrade_effects).chain(),
    );

    // Prime the app (first frame initializes BaseStats)
    app.update();
    app
}

#[test]
fn craft_deducts_materials() {
    let mut app = upgrade_test_app();

    // Give player inventory and credits
    {
        let mut inv = app.world_mut().resource_mut::<PlayerInventory>();
        inv.items.insert(MaterialType::CommonScrap, 10);
        inv.items.insert(MaterialType::RareAlloy, 5);
    }
    {
        let mut credits = app.world_mut().resource_mut::<Credits>();
        credits.balance = 200;
    }

    // Spawn player with Docked marker (required by process_crafting_request)
    let station_entity = app.world_mut().spawn_empty().id();
    app.world_mut().spawn((Player, Docked { station: station_entity }));

    // Request crafting of recipe index 0 (Thrust I: 3 scrap, 0 alloy, 20 credits)
    {
        let mut request = app.world_mut().resource_mut::<CraftingRequest>();
        request.recipe_index = Some(0);
    }

    app.update();

    let inv = app.world().resource::<PlayerInventory>();
    assert_eq!(
        inv.items.get(&MaterialType::CommonScrap).copied().unwrap_or(0),
        7,
        "Should have 7 CommonScrap after crafting Thrust I (cost 3)"
    );

    let credits = app.world().resource::<Credits>();
    assert_eq!(credits.balance, 180, "Should have 180 credits after spending 20");
}

#[test]
fn craft_increments_tier() {
    let mut app = upgrade_test_app();

    // Give player resources
    {
        let mut inv = app.world_mut().resource_mut::<PlayerInventory>();
        inv.items.insert(MaterialType::CommonScrap, 10);
    }
    {
        let mut credits = app.world_mut().resource_mut::<Credits>();
        credits.balance = 200;
    }

    let station_entity = app.world_mut().spawn_empty().id();
    app.world_mut().spawn((Player, Docked { station: station_entity }));

    // Craft Thrust I (index 0)
    {
        let mut request = app.world_mut().resource_mut::<CraftingRequest>();
        request.recipe_index = Some(0);
    }
    app.update();

    let installed = app.world().resource::<InstalledUpgrades>();
    assert_eq!(
        installed.ship_tier(ShipSystem::Thrust),
        1,
        "Thrust should be at tier 1 after crafting Thrust I"
    );
}

#[test]
fn apply_upgrade_effects_scales_thrust() {
    let mut app = upgrade_test_app();

    let base_thrust = app.world().resource::<FlightConfig>().thrust_power;

    // Directly set installed upgrade (bypassing crafting)
    {
        let mut installed = app.world_mut().resource_mut::<InstalledUpgrades>();
        installed.ship.insert(ShipSystem::Thrust, 2);
    }

    // Spawn player entity for apply_upgrade_effects
    app.world_mut().spawn((
        Player,
        void_drifter::core::collision::Health { current: 100.0, max: 100.0 },
        void_drifter::core::weapons::Energy { current: 100.0, max_capacity: 100.0 },
    ));

    app.update();

    let flight = app.world().resource::<FlightConfig>();
    let expected = base_thrust * compute_upgrade_multiplier(2);
    assert!(
        (flight.thrust_power - expected).abs() < 0.01,
        "Thrust power should be scaled by tier-2 multiplier (1.2x): expected {expected}, got {}",
        flight.thrust_power
    );
}

#[test]
fn can_craft_buy_only_recipe_not_blocked_by_can_craft_function() {
    // BuyOnly recipes still have can_craft evaluate based on resource costs.
    // The Cargo I recipe is BuyOnly with 0 material cost and 15 credits.
    // Finding it among defaults:
    let recipes = default_tier1_recipes();
    let cargo_recipe = recipes
        .iter()
        .find(|r| matches!(r.ship_system, Some(ShipSystem::CargoCapacity)))
        .expect("Should find CargoCapacity recipe");

    assert_eq!(
        cargo_recipe.acquisition,
        AcquisitionMethod::BuyOnly,
        "CargoCapacity should be BuyOnly"
    );

    // With enough credits and no material costs, can_craft should be true
    let inventory = PlayerInventory::default();
    let credits = Credits { balance: 100 };
    assert!(
        can_craft(cargo_recipe, &inventory, &credits),
        "BuyOnly recipe with only credit cost should pass can_craft with sufficient credits"
    );

    // With insufficient credits, should fail
    let low_credits = Credits { balance: 10 };
    assert!(
        !can_craft(cargo_recipe, &inventory, &low_credits),
        "BuyOnly recipe should fail can_craft with insufficient credits"
    );
}
