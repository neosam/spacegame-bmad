pub mod delta;
pub mod migration;
pub mod schema;
pub mod player_save;
pub mod world_save;

use std::path::Path;

use bevy::ecs::message::MessageWriter;
use bevy::prelude::*;

use crate::core::collision::Health;
use crate::core::flight::Player;
use crate::core::input::ActionState;
use crate::core::weapons::{ActiveWeapon, Energy};
use crate::infrastructure::events::EventSeverityConfig;
use crate::shared::components::Velocity;
use crate::shared::events::{GameEvent, GameEventKind};
use crate::world::{ExploredChunks, WorldConfig};

use self::delta::WorldDeltas;
use self::player_save::PlayerSave;
use self::world_save::WorldSave;

/// Configuration for the save system.
#[derive(Resource, Clone, Debug)]
pub struct SaveConfig {
    pub save_dir: String,
}

impl Default for SaveConfig {
    fn default() -> Self {
        Self {
            save_dir: "saves/".to_string(),
        }
    }
}

/// Runtime state tracking for saves.
#[derive(Resource, Default, Clone, Debug)]
pub struct SaveState {
    pub last_save_time: Option<f64>,
    pub loaded_from_save: bool,
}

/// System that saves the game when `ActionState.save` is true.
#[allow(clippy::too_many_arguments)]
pub fn save_game(
    action_state: Res<ActionState>,
    config: Res<SaveConfig>,
    player_query: Query<
        (&Transform, &Velocity, &Health, &ActiveWeapon, &Energy),
        With<Player>,
    >,
    world_config: Res<WorldConfig>,
    explored_chunks: Res<ExploredChunks>,
    world_deltas: Res<WorldDeltas>,
    mut game_events: MessageWriter<GameEvent>,
    time: Res<Time>,
    severity_config: Res<EventSeverityConfig>,
    mut save_state: ResMut<SaveState>,
) {
    if !action_state.save {
        return;
    }

    // Create save directory if it doesn't exist
    if let Err(e) = std::fs::create_dir_all(&config.save_dir) {
        warn!("Failed to create save directory '{}': {e}", config.save_dir);
        return;
    }

    // Build PlayerSave from query
    let player_save = {
        let Some((transform, velocity, health, active_weapon, energy)) =
            player_query.iter().next()
        else {
            warn!("No player entity found for save");
            return;
        };

        PlayerSave::from_components(transform, velocity, health, active_weapon, energy)
    };

    // Build WorldSave
    let world_save = WorldSave::from_resources(world_config.seed, &explored_chunks, &world_deltas);

    // Serialize and write
    let player_ron = match player_save.to_ron() {
        Ok(s) => s,
        Err(e) => {
            warn!("Failed to serialize player save: {e}");
            return;
        }
    };
    let world_ron = match world_save.to_ron() {
        Ok(s) => s,
        Err(e) => {
            warn!("Failed to serialize world save: {e}");
            return;
        }
    };

    let player_path = Path::new(&config.save_dir).join("player.ron");
    let world_path = Path::new(&config.save_dir).join("world.ron");

    if let Err(e) = std::fs::write(&player_path, &player_ron) {
        warn!("Failed to write {}: {e}", player_path.display());
        return;
    }
    if let Err(e) = std::fs::write(&world_path, &world_ron) {
        warn!("Failed to write {}: {e}", world_path.display());
        return;
    }

    save_state.last_save_time = Some(time.elapsed_secs_f64());

    // Emit GameSaved event
    let position = Vec2::new(player_save.position.0, player_save.position.1);
    let kind = GameEventKind::GameSaved;
    game_events.write(GameEvent {
        severity: severity_config.severity_for(&kind),
        kind,
        position,
        game_time: time.elapsed_secs_f64(),
    });
}

/// System that loads the game at startup if save files exist.
pub fn load_game(
    config: Res<SaveConfig>,
    mut player_query: Query<
        (&mut Transform, &mut Velocity, &mut Health, &mut ActiveWeapon, &mut Energy),
        With<Player>,
    >,
    mut explored_chunks: ResMut<ExploredChunks>,
    mut world_deltas: ResMut<WorldDeltas>,
    mut save_state: ResMut<SaveState>,
    mut world_config: ResMut<WorldConfig>,
) {
    let player_path = Path::new(&config.save_dir).join("player.ron");
    let world_path = Path::new(&config.save_dir).join("world.ron");

    // Try loading player save
    match std::fs::read_to_string(&player_path) {
        Ok(contents) => {
            match PlayerSave::from_ron(&contents) {
                Ok(player_save) => {
                    if let Some((mut transform, mut velocity, mut health, mut active_weapon, mut energy)) =
                        player_query.iter_mut().next()
                    {
                        player_save.apply_to_components(
                            &mut transform, &mut velocity, &mut health,
                            &mut active_weapon, &mut energy,
                        );
                        save_state.loaded_from_save = true;
                    }
                }
                Err(e) => {
                    warn!("Corrupt player save file '{}': {e}. Starting fresh.", player_path.display());
                }
            }
        }
        Err(_) => {
            // No save file — start fresh (not an error)
        }
    }

    // Try loading world save
    match std::fs::read_to_string(&world_path) {
        Ok(contents) => {
            match WorldSave::from_ron(&contents) {
                Ok(world_save) => {
                    world_config.seed = world_save.seed;
                    world_save.apply_to_world_resources(&mut explored_chunks, &mut world_deltas);
                }
                Err(e) => {
                    warn!("Corrupt world save file '{}': {e}. Starting fresh.", world_path.display());
                }
            }
        }
        Err(_) => {
            // No save file — start fresh
        }
    }
}

/// Plugin that registers save/load systems and resources.
pub struct SavePlugin;

impl Plugin for SavePlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(SaveConfig::default());
        app.init_resource::<SaveState>();
        app.init_resource::<WorldDeltas>();
        app.add_systems(Startup, load_game);
        app.add_systems(
            FixedUpdate,
            delta::track_destroyed_entities
                .after(crate::core::collision::apply_damage)
                .before(crate::core::collision::despawn_destroyed),
        );
        app.add_systems(
            FixedUpdate,
            save_game.in_set(crate::core::CoreSet::Events),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn save_config_default_has_save_dir() {
        let config = SaveConfig::default();
        assert_eq!(config.save_dir, "saves/");
    }

    #[test]
    fn save_state_default_values() {
        let state = SaveState::default();
        assert!(state.last_save_time.is_none());
        assert!(!state.loaded_from_save);
    }
}
