use bevy::prelude::*;
use serde::{Serialize, Deserialize};

use crate::core::collision::Health;
use crate::core::economy::{Credits, PlayerInventory};
use crate::core::flight::Player;
use crate::core::upgrades::{InstalledUpgrades, ShipSystem, WeaponSystem};
use crate::core::weapons::{ActiveWeapon, Energy};
use crate::shared::components::{MaterialType, Velocity};
use crate::social::companion::{
    Companion, CompanionData, CompanionFollowAI, CompanionRoster, CompanionSaveEntry,
    NeedsCompanionVisual, WingmanCommand, str_to_faction_id,
};

use super::schema::{check_version, SaveError, SAVE_VERSION};

/// Serializable snapshot of player state.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct PlayerSave {
    pub schema_version: u32,
    pub position: (f32, f32),
    pub rotation: f32,
    pub velocity: (f32, f32),
    pub health_current: f32,
    pub health_max: f32,
    pub active_weapon: String,
    pub energy_current: f32,
    pub energy_max: f32,
    /// Player's credit balance. Defaults to 0 for backward compatibility with v1/v2/v3 saves.
    #[serde(default)]
    pub credits: u32,
    /// Inventory: CommonScrap count. Defaults to 0 for backward compatibility.
    #[serde(default)]
    pub inventory_common_scrap: u32,
    /// Inventory: RareAlloy count. Defaults to 0 for backward compatibility.
    #[serde(default)]
    pub inventory_rare_alloy: u32,
    /// Inventory: EnergyCore count. Defaults to 0 for backward compatibility.
    #[serde(default)]
    pub inventory_energy_core: u32,
    /// Upgrade tiers — default 0 for backward compatibility with v1–v4 saves.
    #[serde(default)]
    pub upgrade_ship_thrust: u8,
    #[serde(default)]
    pub upgrade_ship_max_speed: u8,
    #[serde(default)]
    pub upgrade_ship_rotation: u8,
    #[serde(default)]
    pub upgrade_ship_energy_capacity: u8,
    #[serde(default)]
    pub upgrade_ship_energy_regen: u8,
    #[serde(default)]
    pub upgrade_ship_scanner_range: u8,
    #[serde(default)]
    pub upgrade_ship_hull_strength: u8,
    #[serde(default)]
    pub upgrade_ship_cargo_capacity: u8,
    #[serde(default)]
    pub upgrade_weapon_laser_damage: u8,
    #[serde(default)]
    pub upgrade_weapon_laser_fire_rate: u8,
    #[serde(default)]
    pub upgrade_weapon_spread_damage: u8,
    #[serde(default)]
    pub upgrade_weapon_spread_fire_rate: u8,
    #[serde(default)]
    pub upgrade_weapon_energy_efficiency: u8,
    /// Companion roster — saved companions with their position and faction.
    /// Defaults to empty for backward compatibility with v1–v5 saves.
    #[serde(default)]
    pub companions: Vec<CompanionSaveEntry>,
}

impl Default for PlayerSave {
    fn default() -> Self {
        Self {
            schema_version: SAVE_VERSION,
            position: (0.0, 0.0),
            rotation: 0.0,
            velocity: (0.0, 0.0),
            health_current: 100.0,
            health_max: 100.0,
            active_weapon: "Laser".to_string(),
            energy_current: 100.0,
            energy_max: 100.0,
            credits: 0,
            inventory_common_scrap: 0,
            inventory_rare_alloy: 0,
            inventory_energy_core: 0,
            upgrade_ship_thrust: 0,
            upgrade_ship_max_speed: 0,
            upgrade_ship_rotation: 0,
            upgrade_ship_energy_capacity: 0,
            upgrade_ship_energy_regen: 0,
            upgrade_ship_scanner_range: 0,
            upgrade_ship_hull_strength: 0,
            upgrade_ship_cargo_capacity: 0,
            upgrade_weapon_laser_damage: 0,
            upgrade_weapon_laser_fire_rate: 0,
            upgrade_weapon_spread_damage: 0,
            upgrade_weapon_spread_fire_rate: 0,
            upgrade_weapon_energy_efficiency: 0,
            companions: Vec::new(),
        }
    }
}

impl PlayerSave {
    /// Builds a PlayerSave from individual component references.
    /// Single source of truth for component-to-save conversion.
    pub fn from_components(
        transform: &Transform,
        velocity: &Velocity,
        health: &Health,
        active_weapon: &ActiveWeapon,
        energy: &Energy,
    ) -> Self {
        let weapon_str = match active_weapon {
            ActiveWeapon::Laser => "Laser",
            ActiveWeapon::Spread => "Spread",
        };
        PlayerSave {
            schema_version: SAVE_VERSION,
            position: (transform.translation.x, transform.translation.y),
            rotation: transform.rotation.to_euler(EulerRot::XYZ).2,
            velocity: (velocity.0.x, velocity.0.y),
            health_current: health.current,
            health_max: health.max,
            active_weapon: weapon_str.to_string(),
            energy_current: energy.current,
            energy_max: energy.max_capacity,
            credits: 0,
            inventory_common_scrap: 0,
            inventory_rare_alloy: 0,
            inventory_energy_core: 0,
            upgrade_ship_thrust: 0,
            upgrade_ship_max_speed: 0,
            upgrade_ship_rotation: 0,
            upgrade_ship_energy_capacity: 0,
            upgrade_ship_energy_regen: 0,
            upgrade_ship_scanner_range: 0,
            upgrade_ship_hull_strength: 0,
            upgrade_ship_cargo_capacity: 0,
            upgrade_weapon_laser_damage: 0,
            upgrade_weapon_laser_fire_rate: 0,
            upgrade_weapon_spread_damage: 0,
            upgrade_weapon_spread_fire_rate: 0,
            upgrade_weapon_energy_efficiency: 0,
            companions: Vec::new(),
        }
    }

    /// Applies saved state to individual component references.
    /// Single source of truth for save-to-component conversion.
    pub fn apply_to_components(
        &self,
        transform: &mut Transform,
        velocity: &mut Velocity,
        health: &mut Health,
        active_weapon: &mut ActiveWeapon,
        energy: &mut Energy,
    ) {
        transform.translation.x = self.position.0;
        transform.translation.y = self.position.1;
        transform.rotation = Quat::from_rotation_z(self.rotation);
        velocity.0 = Vec2::new(self.velocity.0, self.velocity.1);
        health.current = self.health_current;
        health.max = self.health_max;
        *active_weapon = match self.active_weapon.as_str() {
            "Spread" => ActiveWeapon::Spread,
            _ => ActiveWeapon::Laser,
        };
        energy.current = self.energy_current;
        energy.max_capacity = self.energy_max;
    }

    /// Extracts player state from the ECS world.
    /// Returns `None` if no player entity with required components exists.
    pub fn from_world(world: &mut World) -> Option<Self> {
        let mut query = world.query_filtered::<(
            &Transform,
            &Velocity,
            &Health,
            &ActiveWeapon,
            &Energy,
        ), With<Player>>();

        let (transform, velocity, health, active_weapon, energy) =
            query.iter(world).next()?;

        let mut save = Self::from_components(transform, velocity, health, active_weapon, energy);
        save.credits = world.get_resource::<Credits>().map(|c| c.balance).unwrap_or(0);
        if let Some(inv) = world.get_resource::<PlayerInventory>() {
            save.inventory_common_scrap = inv.items.get(&MaterialType::CommonScrap).copied().unwrap_or(0);
            save.inventory_rare_alloy = inv.items.get(&MaterialType::RareAlloy).copied().unwrap_or(0);
            save.inventory_energy_core = inv.items.get(&MaterialType::EnergyCore).copied().unwrap_or(0);
        }
        if let Some(installed) = world.get_resource::<InstalledUpgrades>() {
            save.upgrade_ship_thrust = installed.ship_tier(ShipSystem::Thrust);
            save.upgrade_ship_max_speed = installed.ship_tier(ShipSystem::MaxSpeed);
            save.upgrade_ship_rotation = installed.ship_tier(ShipSystem::Rotation);
            save.upgrade_ship_energy_capacity = installed.ship_tier(ShipSystem::EnergyCapacity);
            save.upgrade_ship_energy_regen = installed.ship_tier(ShipSystem::EnergyRegen);
            save.upgrade_ship_scanner_range = installed.ship_tier(ShipSystem::ScannerRange);
            save.upgrade_ship_hull_strength = installed.ship_tier(ShipSystem::HullStrength);
            save.upgrade_ship_cargo_capacity = installed.ship_tier(ShipSystem::CargoCapacity);
            save.upgrade_weapon_laser_damage = installed.weapon_tier(WeaponSystem::LaserDamage);
            save.upgrade_weapon_laser_fire_rate = installed.weapon_tier(WeaponSystem::LaserFireRate);
            save.upgrade_weapon_spread_damage = installed.weapon_tier(WeaponSystem::SpreadDamage);
            save.upgrade_weapon_spread_fire_rate = installed.weapon_tier(WeaponSystem::SpreadFireRate);
            save.upgrade_weapon_energy_efficiency = installed.weapon_tier(WeaponSystem::EnergyEfficiency);
        }
        // Save companion roster (Story 6a-6)
        let mut companion_query = world.query_filtered::<(&CompanionData, &Transform), With<Companion>>();
        save.companions = companion_query
            .iter(world)
            .map(|(data, transform)| CompanionSaveEntry::from_components(data, transform))
            .collect();
        Some(save)
    }

    /// Applies saved state to the player entity in the world.
    pub fn apply_to_world(&self, world: &mut World) {
        let mut query = world.query_filtered::<(
            &mut Transform,
            &mut Velocity,
            &mut Health,
            &mut ActiveWeapon,
            &mut Energy,
        ), With<Player>>();

        let Some((mut transform, mut velocity, mut health, mut active_weapon, mut energy)) =
            query.iter_mut(world).next()
        else {
            warn!("No player entity found to apply save data");
            return;
        };

        self.apply_to_components(
            &mut transform, &mut velocity, &mut health, &mut active_weapon, &mut energy,
        );

        // Restore credits resource
        if let Some(mut credits) = world.get_resource_mut::<Credits>() {
            credits.balance = self.credits;
        }
        // Restore inventory resource
        if let Some(mut inv) = world.get_resource_mut::<PlayerInventory>() {
            inv.items.clear();
            if self.inventory_common_scrap > 0 {
                inv.items.insert(MaterialType::CommonScrap, self.inventory_common_scrap);
            }
            if self.inventory_rare_alloy > 0 {
                inv.items.insert(MaterialType::RareAlloy, self.inventory_rare_alloy);
            }
            if self.inventory_energy_core > 0 {
                inv.items.insert(MaterialType::EnergyCore, self.inventory_energy_core);
            }
        }
        // Restore upgrade tiers
        if let Some(mut installed) = world.get_resource_mut::<InstalledUpgrades>() {
            installed.ship.clear();
            installed.weapon.clear();
            if self.upgrade_ship_thrust > 0 { installed.ship.insert(ShipSystem::Thrust, self.upgrade_ship_thrust); }
            if self.upgrade_ship_max_speed > 0 { installed.ship.insert(ShipSystem::MaxSpeed, self.upgrade_ship_max_speed); }
            if self.upgrade_ship_rotation > 0 { installed.ship.insert(ShipSystem::Rotation, self.upgrade_ship_rotation); }
            if self.upgrade_ship_energy_capacity > 0 { installed.ship.insert(ShipSystem::EnergyCapacity, self.upgrade_ship_energy_capacity); }
            if self.upgrade_ship_energy_regen > 0 { installed.ship.insert(ShipSystem::EnergyRegen, self.upgrade_ship_energy_regen); }
            if self.upgrade_ship_scanner_range > 0 { installed.ship.insert(ShipSystem::ScannerRange, self.upgrade_ship_scanner_range); }
            if self.upgrade_ship_hull_strength > 0 { installed.ship.insert(ShipSystem::HullStrength, self.upgrade_ship_hull_strength); }
            if self.upgrade_ship_cargo_capacity > 0 { installed.ship.insert(ShipSystem::CargoCapacity, self.upgrade_ship_cargo_capacity); }
            if self.upgrade_weapon_laser_damage > 0 { installed.weapon.insert(WeaponSystem::LaserDamage, self.upgrade_weapon_laser_damage); }
            if self.upgrade_weapon_laser_fire_rate > 0 { installed.weapon.insert(WeaponSystem::LaserFireRate, self.upgrade_weapon_laser_fire_rate); }
            if self.upgrade_weapon_spread_damage > 0 { installed.weapon.insert(WeaponSystem::SpreadDamage, self.upgrade_weapon_spread_damage); }
            if self.upgrade_weapon_spread_fire_rate > 0 { installed.weapon.insert(WeaponSystem::SpreadFireRate, self.upgrade_weapon_spread_fire_rate); }
            if self.upgrade_weapon_energy_efficiency > 0 { installed.weapon.insert(WeaponSystem::EnergyEfficiency, self.upgrade_weapon_energy_efficiency); }
        }
        // Restore companion roster (Story 6a-6)
        if let Some(mut roster) = world.get_resource_mut::<CompanionRoster>() {
            roster.companions.clear();
        }
        for entry in &self.companions {
            let pos = Vec2::new(entry.x, entry.y);
            let faction = str_to_faction_id(&entry.faction);
            let entity = world.spawn((
                Companion,
                CompanionData {
                    name: entry.name.clone(),
                    faction,
                },
                CompanionFollowAI::default(),
                WingmanCommand::Defend,
                NeedsCompanionVisual,
                Velocity::default(),
                Transform::from_translation(pos.extend(0.0)),
            )).id();
            if let Some(mut roster) = world.get_resource_mut::<CompanionRoster>() {
                roster.companions.push(entity);
            }
        }
    }

    /// Serializes to pretty-printed RON.
    pub fn to_ron(&self) -> Result<String, SaveError> {
        let pretty = ron::ser::PrettyConfig::new()
            .depth_limit(4)
            .separate_tuple_members(true);
        ron::ser::to_string_pretty(self, pretty)
            .map_err(|e| SaveError::ParseError(format!("{e}")))
    }

    /// Deserializes from RON with version check.
    pub fn from_ron(ron_str: &str) -> Result<Self, SaveError> {
        check_version(ron_str)?;
        ron::from_str(ron_str).map_err(|e| SaveError::ParseError(format!("{e}")))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_player_save() -> PlayerSave {
        PlayerSave {
            schema_version: SAVE_VERSION,
            position: (100.0, 200.0),
            rotation: 1.5,
            velocity: (10.0, -5.0),
            health_current: 80.0,
            health_max: 100.0,
            active_weapon: "Laser".to_string(),
            energy_current: 75.0,
            energy_max: 100.0,
            credits: 0,
            inventory_common_scrap: 0,
            inventory_rare_alloy: 0,
            inventory_energy_core: 0,
            upgrade_ship_thrust: 0,
            upgrade_ship_max_speed: 0,
            upgrade_ship_rotation: 0,
            upgrade_ship_energy_capacity: 0,
            upgrade_ship_energy_regen: 0,
            upgrade_ship_scanner_range: 0,
            upgrade_ship_hull_strength: 0,
            upgrade_ship_cargo_capacity: 0,
            upgrade_weapon_laser_damage: 0,
            upgrade_weapon_laser_fire_rate: 0,
            upgrade_weapon_spread_damage: 0,
            upgrade_weapon_spread_fire_rate: 0,
            upgrade_weapon_energy_efficiency: 0,
            companions: Vec::new(),
        }
    }

    #[test]
    fn player_save_roundtrip() {
        let original = sample_player_save();
        let ron_str = original.to_ron().expect("Should serialize");
        let restored = PlayerSave::from_ron(&ron_str).expect("Should deserialize");

        assert_eq!(restored.schema_version, original.schema_version);
        assert!((restored.position.0 - original.position.0).abs() < f32::EPSILON);
        assert!((restored.position.1 - original.position.1).abs() < f32::EPSILON);
        assert!((restored.rotation - original.rotation).abs() < 0.001);
        assert!((restored.velocity.0 - original.velocity.0).abs() < f32::EPSILON);
        assert!((restored.velocity.1 - original.velocity.1).abs() < f32::EPSILON);
        assert!((restored.health_current - original.health_current).abs() < f32::EPSILON);
        assert!((restored.health_max - original.health_max).abs() < f32::EPSILON);
        assert_eq!(restored.active_weapon, original.active_weapon);
        assert!((restored.energy_current - original.energy_current).abs() < f32::EPSILON);
        assert!((restored.energy_max - original.energy_max).abs() < f32::EPSILON);
    }

    #[test]
    fn player_save_from_ron_corrupt_returns_error() {
        let result = PlayerSave::from_ron("not valid ron data {{{");
        assert!(result.is_err());
    }

    #[test]
    fn player_save_from_world_extracts_components() {
        use crate::core::collision::Health;

        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.world_mut().spawn((
            Player,
            Transform::from_translation(Vec3::new(50.0, 75.0, 0.0)),
            Velocity(Vec2::new(3.0, -4.0)),
            Health { current: 60.0, max: 100.0 },
            ActiveWeapon::Spread,
            Energy { current: 40.0, max_capacity: 100.0 },
        ));

        let save = PlayerSave::from_world(app.world_mut()).expect("Should extract player");
        assert!((save.position.0 - 50.0).abs() < f32::EPSILON);
        assert!((save.position.1 - 75.0).abs() < f32::EPSILON);
        assert!((save.velocity.0 - 3.0).abs() < f32::EPSILON);
        assert!((save.velocity.1 - (-4.0)).abs() < f32::EPSILON);
        assert!((save.health_current - 60.0).abs() < f32::EPSILON);
        assert!((save.health_max - 100.0).abs() < f32::EPSILON);
        assert_eq!(save.active_weapon, "Spread");
        assert!((save.energy_current - 40.0).abs() < f32::EPSILON);
        assert!((save.energy_max - 100.0).abs() < f32::EPSILON);
    }
}
