use bevy::prelude::*;
use serde::{Serialize, Deserialize};

use crate::core::collision::Health;
use crate::core::economy::Credits;
use crate::core::flight::Player;
use crate::core::weapons::{ActiveWeapon, Energy};
use crate::shared::components::Velocity;

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
    /// Player's credit balance. Defaults to 0 for backward compatibility with v1/v2 saves.
    #[serde(default)]
    pub credits: u32,
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
