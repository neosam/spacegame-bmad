use std::collections::HashMap;

use bevy::ecs::message::MessageWriter;
use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::core::economy::{Credits, PlayerInventory};
use crate::infrastructure::events::EventSeverityConfig;
use crate::shared::components::MaterialType;
use crate::shared::events::{GameEvent, GameEventKind};

// ── Ship Systems (8) ─────────────────────────────────────────────────────

/// The 8 upgradable ship systems.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ShipSystem {
    Thrust,
    MaxSpeed,
    Rotation,
    EnergyCapacity,
    EnergyRegen,
    ScannerRange,
    HullStrength,
    CargoCapacity,
}

// ── Weapon Systems (5) ───────────────────────────────────────────────────

/// The 5 upgradable weapon attributes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum WeaponSystem {
    LaserDamage,
    LaserFireRate,
    SpreadDamage,
    SpreadFireRate,
    EnergyEfficiency,
}

// ── Acquisition Method ───────────────────────────────────────────────────

/// How a recipe can be acquired.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AcquisitionMethod {
    /// Can only be crafted using materials.
    CraftOnly,
    /// Can only be purchased with credits at a station.
    BuyOnly,
    /// Can be crafted or purchased.
    CraftOrBuy,
}

// ── Crafting Recipe ──────────────────────────────────────────────────────

/// Describes a single crafting recipe for a ship or weapon upgrade.
#[derive(Debug, Clone)]
pub struct CraftingRecipe {
    pub ship_system: Option<ShipSystem>,
    pub weapon_system: Option<WeaponSystem>,
    /// Tier 1–5.
    pub tier: u8,
    pub cost_common_scrap: u32,
    pub cost_rare_alloy: u32,
    pub cost_energy_core: u32,
    pub credit_cost: u32,
    pub acquisition: AcquisitionMethod,
    pub display_name: &'static str,
}

// ── Installed Upgrades ───────────────────────────────────────────────────

/// Tracks which tier (0 = none) is installed for each ship and weapon system.
#[derive(Resource, Default, Debug, Clone)]
pub struct InstalledUpgrades {
    pub ship: HashMap<ShipSystem, u8>,
    pub weapon: HashMap<WeaponSystem, u8>,
}

impl InstalledUpgrades {
    /// Returns the installed tier for the given ship system (0 = none).
    pub fn ship_tier(&self, system: ShipSystem) -> u8 {
        self.ship.get(&system).copied().unwrap_or(0)
    }

    /// Returns the installed tier for the given weapon system (0 = none).
    pub fn weapon_tier(&self, system: WeaponSystem) -> u8 {
        self.weapon.get(&system).copied().unwrap_or(0)
    }
}

// ── Discovered Recipes ───────────────────────────────────────────────────

/// Tracks which recipes the player has discovered.
/// Tier-1 recipes are available by default.
#[derive(Resource, Debug, Clone)]
pub struct DiscoveredRecipes {
    pub recipes: Vec<CraftingRecipe>,
}

impl Default for DiscoveredRecipes {
    fn default() -> Self {
        Self {
            recipes: default_tier1_recipes(),
        }
    }
}

// ── Crafting Request ─────────────────────────────────────────────────────

/// Set by UI or test code to request crafting of a recipe at the given index.
#[derive(Resource, Default)]
pub struct CraftingRequest {
    pub recipe_index: Option<usize>,
}

// ── Station UI State ─────────────────────────────────────────────────────

/// Tracks the cursor index in the crafting recipe list shown in the station UI.
/// Navigation systems (nav_up/nav_down) modify `selected_recipe_index`.
/// The craft system reads it to dispatch a `CraftingRequest`.
#[derive(Resource, Default, Debug)]
pub struct StationUiState {
    pub selected_recipe_index: usize,
}

// ── Pending Craft Events (B0002-safe buffer) ─────────────────────────────

/// Buffer for craft events to be emitted separately (avoids MessageReader + MessageWriter conflict).
#[derive(Resource, Default)]
pub struct PendingCraftEvents {
    pub events: Vec<(String, u8, Vec2, f64)>,
}

// ── Base Stats ───────────────────────────────────────────────────────────

/// The unmodified base values for all stats that upgrades can affect.
/// Initialized from FlightConfig + WeaponConfig at startup.
#[derive(Resource)]
pub struct BaseStats {
    pub thrust_power: f32,
    pub max_speed: f32,
    pub rotation_speed: f32,
    pub energy_regen_rate: f32,
    pub spread_energy_cost: f32,
    pub laser_damage: f32,
    pub laser_fire_rate: f32,
    pub spread_damage: f32,
    pub spread_fire_rate: f32,
    pub health_max: f32,
    pub energy_max: f32,
}

// ── Pure Functions ───────────────────────────────────────────────────────

/// Returns the stat multiplier for a given upgrade tier.
/// Tier 0 = 1.0 (no upgrade); Tier 5 = 1.5 (+50%).
pub fn compute_upgrade_multiplier(tier: u8) -> f32 {
    1.0 + (tier as f32) * 0.1
}

/// Returns true if the player can craft the given recipe right now.
/// Checks material counts and credit balance.
pub fn can_craft(recipe: &CraftingRecipe, inventory: &PlayerInventory, credits: &Credits) -> bool {
    let scrap = inventory
        .items
        .get(&MaterialType::CommonScrap)
        .copied()
        .unwrap_or(0);
    let alloy = inventory
        .items
        .get(&MaterialType::RareAlloy)
        .copied()
        .unwrap_or(0);
    let core = inventory
        .items
        .get(&MaterialType::EnergyCore)
        .copied()
        .unwrap_or(0);
    scrap >= recipe.cost_common_scrap
        && alloy >= recipe.cost_rare_alloy
        && core >= recipe.cost_energy_core
        && credits.balance >= recipe.credit_cost
}

/// Returns all Tier-1 recipes that are available by default.
pub fn default_tier1_recipes() -> Vec<CraftingRecipe> {
    vec![
        // Ship system upgrades
        CraftingRecipe {
            ship_system: Some(ShipSystem::Thrust),
            weapon_system: None,
            tier: 1,
            cost_common_scrap: 3,
            cost_rare_alloy: 0,
            cost_energy_core: 0,
            credit_cost: 20,
            acquisition: AcquisitionMethod::CraftOnly,
            display_name: "Thrust I",
        },
        CraftingRecipe {
            ship_system: Some(ShipSystem::MaxSpeed),
            weapon_system: None,
            tier: 1,
            cost_common_scrap: 3,
            cost_rare_alloy: 0,
            cost_energy_core: 0,
            credit_cost: 20,
            acquisition: AcquisitionMethod::CraftOnly,
            display_name: "Max Speed I",
        },
        CraftingRecipe {
            ship_system: Some(ShipSystem::Rotation),
            weapon_system: None,
            tier: 1,
            cost_common_scrap: 2,
            cost_rare_alloy: 1,
            cost_energy_core: 0,
            credit_cost: 25,
            acquisition: AcquisitionMethod::CraftOnly,
            display_name: "Rotation I",
        },
        CraftingRecipe {
            ship_system: Some(ShipSystem::EnergyCapacity),
            weapon_system: None,
            tier: 1,
            cost_common_scrap: 0,
            cost_rare_alloy: 2,
            cost_energy_core: 1,
            credit_cost: 30,
            acquisition: AcquisitionMethod::CraftOnly,
            display_name: "Energy Cap I",
        },
        CraftingRecipe {
            ship_system: Some(ShipSystem::EnergyRegen),
            weapon_system: None,
            tier: 1,
            cost_common_scrap: 1,
            cost_rare_alloy: 1,
            cost_energy_core: 1,
            credit_cost: 35,
            acquisition: AcquisitionMethod::CraftOnly,
            display_name: "Energy Regen I",
        },
        CraftingRecipe {
            ship_system: Some(ShipSystem::ScannerRange),
            weapon_system: None,
            tier: 1,
            cost_common_scrap: 2,
            cost_rare_alloy: 0,
            cost_energy_core: 0,
            credit_cost: 15,
            acquisition: AcquisitionMethod::CraftOrBuy,
            display_name: "Scanner I",
        },
        CraftingRecipe {
            ship_system: Some(ShipSystem::HullStrength),
            weapon_system: None,
            tier: 1,
            cost_common_scrap: 4,
            cost_rare_alloy: 1,
            cost_energy_core: 0,
            credit_cost: 40,
            acquisition: AcquisitionMethod::CraftOnly,
            display_name: "Hull I",
        },
        CraftingRecipe {
            ship_system: Some(ShipSystem::CargoCapacity),
            weapon_system: None,
            tier: 1,
            cost_common_scrap: 0,
            cost_rare_alloy: 0,
            cost_energy_core: 0,
            credit_cost: 15,
            acquisition: AcquisitionMethod::BuyOnly,
            display_name: "Cargo I",
        },
        // Weapon upgrades
        CraftingRecipe {
            ship_system: None,
            weapon_system: Some(WeaponSystem::LaserDamage),
            tier: 1,
            cost_common_scrap: 2,
            cost_rare_alloy: 1,
            cost_energy_core: 0,
            credit_cost: 30,
            acquisition: AcquisitionMethod::CraftOnly,
            display_name: "Laser Dmg I",
        },
        CraftingRecipe {
            ship_system: None,
            weapon_system: Some(WeaponSystem::LaserFireRate),
            tier: 1,
            cost_common_scrap: 1,
            cost_rare_alloy: 1,
            cost_energy_core: 0,
            credit_cost: 25,
            acquisition: AcquisitionMethod::CraftOnly,
            display_name: "Laser Rate I",
        },
        CraftingRecipe {
            ship_system: None,
            weapon_system: Some(WeaponSystem::SpreadDamage),
            tier: 1,
            cost_common_scrap: 2,
            cost_rare_alloy: 1,
            cost_energy_core: 0,
            credit_cost: 30,
            acquisition: AcquisitionMethod::CraftOnly,
            display_name: "Spread Dmg I",
        },
        CraftingRecipe {
            ship_system: None,
            weapon_system: Some(WeaponSystem::SpreadFireRate),
            tier: 1,
            cost_common_scrap: 1,
            cost_rare_alloy: 1,
            cost_energy_core: 0,
            credit_cost: 25,
            acquisition: AcquisitionMethod::CraftOnly,
            display_name: "Spread Rate I",
        },
        CraftingRecipe {
            ship_system: None,
            weapon_system: Some(WeaponSystem::EnergyEfficiency),
            tier: 1,
            cost_common_scrap: 0,
            cost_rare_alloy: 2,
            cost_energy_core: 1,
            credit_cost: 40,
            acquisition: AcquisitionMethod::CraftOnly,
            display_name: "Energy Eff I",
        },
    ]
}

// ── Systems ──────────────────────────────────────────────────────────────

/// Initializes BaseStats from the current FlightConfig and WeaponConfig at startup.
pub fn init_base_stats(
    mut commands: Commands,
    flight_config: Res<crate::core::flight::FlightConfig>,
    weapon_config: Res<crate::core::weapons::WeaponConfig>,
) {
    commands.insert_resource(BaseStats {
        thrust_power: flight_config.thrust_power,
        max_speed: flight_config.max_speed,
        rotation_speed: flight_config.rotation_speed,
        energy_regen_rate: weapon_config.energy_regen_rate,
        spread_energy_cost: weapon_config.spread_energy_cost,
        laser_damage: weapon_config.laser_damage,
        laser_fire_rate: weapon_config.laser_fire_rate,
        spread_damage: weapon_config.spread_damage,
        spread_fire_rate: weapon_config.spread_fire_rate,
        health_max: 100.0,
        energy_max: weapon_config.energy_max,
    });
}

/// Processes a pending CraftingRequest.
/// The player must be docked; materials and credits are deducted; upgrade tier is recorded.
pub fn process_crafting_request(
    mut request: ResMut<CraftingRequest>,
    player_query: Query<(), With<crate::core::station::Docked>>,
    mut inventory: ResMut<PlayerInventory>,
    mut credits: ResMut<Credits>,
    mut installed: ResMut<InstalledUpgrades>,
    recipes: Res<DiscoveredRecipes>,
    mut pending: ResMut<PendingCraftEvents>,
    time: Res<Time>,
) {
    let Some(idx) = request.recipe_index.take() else {
        return;
    };
    // Player must be docked
    if player_query.is_empty() {
        return;
    }
    let Some(recipe) = recipes.recipes.get(idx) else {
        return;
    };
    if !can_craft(recipe, &inventory, &credits) {
        return;
    }

    // Deduct materials
    if recipe.cost_common_scrap > 0 {
        *inventory
            .items
            .entry(MaterialType::CommonScrap)
            .or_insert(0) -= recipe.cost_common_scrap;
    }
    if recipe.cost_rare_alloy > 0 {
        *inventory.items.entry(MaterialType::RareAlloy).or_insert(0) -= recipe.cost_rare_alloy;
    }
    if recipe.cost_energy_core > 0 {
        *inventory
            .items
            .entry(MaterialType::EnergyCore)
            .or_insert(0) -= recipe.cost_energy_core;
    }
    credits.balance = credits.balance.saturating_sub(recipe.credit_cost);

    // Install upgrade
    if let Some(sys) = recipe.ship_system {
        installed.ship.insert(sys, recipe.tier);
    }
    if let Some(sys) = recipe.weapon_system {
        installed.weapon.insert(sys, recipe.tier);
    }

    // Queue event
    pending.events.push((
        recipe.display_name.to_string(),
        recipe.tier,
        Vec2::ZERO,
        time.elapsed_secs_f64(),
    ));
}

/// Applies upgrade multipliers to FlightConfig, WeaponConfig, and player entity stats.
/// Only runs when InstalledUpgrades has changed.
pub fn apply_upgrade_effects(
    installed: Res<InstalledUpgrades>,
    base: Res<BaseStats>,
    mut flight_config: ResMut<crate::core::flight::FlightConfig>,
    mut weapon_config: ResMut<crate::core::weapons::WeaponConfig>,
    mut player_query: Query<
        (
            &mut crate::core::collision::Health,
            &mut crate::core::weapons::Energy,
        ),
        With<crate::core::flight::Player>,
    >,
) {
    if !installed.is_changed() {
        return;
    }

    // Flight stats
    flight_config.thrust_power =
        base.thrust_power * compute_upgrade_multiplier(installed.ship_tier(ShipSystem::Thrust));
    flight_config.max_speed =
        base.max_speed * compute_upgrade_multiplier(installed.ship_tier(ShipSystem::MaxSpeed));
    flight_config.rotation_speed = base.rotation_speed
        * compute_upgrade_multiplier(installed.ship_tier(ShipSystem::Rotation));

    // Weapon stats
    weapon_config.energy_regen_rate = base.energy_regen_rate
        * compute_upgrade_multiplier(installed.ship_tier(ShipSystem::EnergyRegen));
    let eff_tier = installed.weapon_tier(WeaponSystem::EnergyEfficiency);
    weapon_config.spread_energy_cost =
        base.spread_energy_cost / compute_upgrade_multiplier(eff_tier);
    weapon_config.laser_damage = base.laser_damage
        * compute_upgrade_multiplier(installed.weapon_tier(WeaponSystem::LaserDamage));
    weapon_config.laser_fire_rate = base.laser_fire_rate
        * compute_upgrade_multiplier(installed.weapon_tier(WeaponSystem::LaserFireRate));
    weapon_config.spread_damage = base.spread_damage
        * compute_upgrade_multiplier(installed.weapon_tier(WeaponSystem::SpreadDamage));
    weapon_config.spread_fire_rate = base.spread_fire_rate
        * compute_upgrade_multiplier(installed.weapon_tier(WeaponSystem::SpreadFireRate));

    // Player entity stats (health + energy capacity)
    for (mut health, mut energy) in player_query.iter_mut() {
        let hull_mult =
            compute_upgrade_multiplier(installed.ship_tier(ShipSystem::HullStrength));
        let new_max = base.health_max * hull_mult;
        if (health.max - new_max).abs() > f32::EPSILON {
            let ratio = health.current / health.max;
            health.max = new_max;
            health.current = (ratio * new_max).min(new_max);
        }
        energy.max_capacity =
            base.energy_max * compute_upgrade_multiplier(installed.ship_tier(ShipSystem::EnergyCapacity));
    }
}

/// Drains PendingCraftEvents and emits UpgradeCrafted GameEvents.
pub fn emit_craft_events(
    mut pending: ResMut<PendingCraftEvents>,
    mut game_events: MessageWriter<GameEvent>,
    severity_config: Res<EventSeverityConfig>,
) {
    for (system_name, tier, position, game_time) in pending.events.drain(..) {
        let kind = GameEventKind::UpgradeCrafted {
            system_name: system_name.clone(),
            tier,
        };
        game_events.write(GameEvent {
            severity: severity_config.severity_for(&kind),
            kind,
            position,
            game_time,
        });
    }
}

/// Navigates the station UI recipe list up/down when the player is docked.
/// Wraps around at both ends.
pub fn navigate_station_ui(
    action_state: Res<crate::core::input::ActionState>,
    player_query: Query<(), With<crate::core::station::Docked>>,
    recipes: Res<DiscoveredRecipes>,
    mut ui_state: ResMut<StationUiState>,
) {
    if player_query.is_empty() {
        return;
    }
    let count = recipes.recipes.len();
    if count == 0 {
        return;
    }
    if action_state.nav_up {
        if ui_state.selected_recipe_index == 0 {
            ui_state.selected_recipe_index = count - 1;
        } else {
            ui_state.selected_recipe_index -= 1;
        }
    }
    if action_state.nav_down {
        ui_state.selected_recipe_index = (ui_state.selected_recipe_index + 1) % count;
    }
}

/// Submits a CraftingRequest for the currently selected recipe when the player presses craft.
pub fn handle_craft_input(
    action_state: Res<crate::core::input::ActionState>,
    player_query: Query<(), With<crate::core::station::Docked>>,
    ui_state: Res<StationUiState>,
    mut request: ResMut<CraftingRequest>,
) {
    if player_query.is_empty() {
        return;
    }
    if action_state.craft {
        request.recipe_index = Some(ui_state.selected_recipe_index);
    }
}

/// Marks the player with NeedsShipUpgradeVisual when InstalledUpgrades changes,
/// so the rendering layer can update ship appearance.
pub fn mark_player_needs_upgrade_visual(
    installed: Res<InstalledUpgrades>,
    mut commands: Commands,
    player_query: Query<Entity, With<crate::core::flight::Player>>,
) {
    if !installed.is_changed() {
        return;
    }
    for entity in player_query.iter() {
        commands
            .entity(entity)
            .insert(crate::shared::components::NeedsShipUpgradeVisual);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compute_upgrade_multiplier_tier0_is_one() {
        assert!((compute_upgrade_multiplier(0) - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn compute_upgrade_multiplier_tier5_is_one_point_five() {
        assert!((compute_upgrade_multiplier(5) - 1.5).abs() < f32::EPSILON);
    }

    #[test]
    fn installed_upgrades_default_ship_tier_is_zero() {
        let installed = InstalledUpgrades::default();
        assert_eq!(installed.ship_tier(ShipSystem::Thrust), 0);
    }

    #[test]
    fn installed_upgrades_default_weapon_tier_is_zero() {
        let installed = InstalledUpgrades::default();
        assert_eq!(installed.weapon_tier(WeaponSystem::LaserDamage), 0);
    }

    #[test]
    fn discovered_recipes_default_has_tier1_recipes() {
        let recipes = DiscoveredRecipes::default();
        assert!(!recipes.recipes.is_empty(), "Should have default tier-1 recipes");
        assert!(
            recipes.recipes.iter().all(|r| r.tier == 1),
            "All default recipes should be tier 1"
        );
    }

    #[test]
    fn default_tier1_recipes_count() {
        let recipes = default_tier1_recipes();
        assert_eq!(recipes.len(), 13, "Should have 13 tier-1 recipes (8 ship + 5 weapon)");
    }
}
