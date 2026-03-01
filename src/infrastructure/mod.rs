pub mod events;
pub mod logbook;
pub mod save;

use bevy::prelude::*;

use crate::core::CoreSet;
use crate::shared::events::GameEvent;
use self::events::{record_game_events, EventSeverityConfig};
use self::logbook::Logbook;

pub struct InfrastructurePlugin;

impl Plugin for InfrastructurePlugin {
    fn build(&self, app: &mut App) {
        // Load EventSeverityConfig from RON file with graceful fallback to defaults
        let config_path = "assets/config/event_severity.ron";
        let config = match std::fs::read_to_string(config_path) {
            Ok(contents) => match EventSeverityConfig::from_ron(&contents) {
                Ok(cfg) => cfg,
                Err(e) => {
                    warn!("Failed to parse {config_path}: {e}. Using defaults.");
                    EventSeverityConfig::default()
                }
            },
            Err(e) => {
                warn!("Failed to read {config_path}: {e}. Using defaults.");
                EventSeverityConfig::default()
            }
        };

        config.validate();
        app.insert_resource(config);
        app.init_resource::<Logbook>();
        app.add_message::<GameEvent>();
        app.add_systems(FixedUpdate, record_game_events.in_set(CoreSet::Events));
        app.add_plugins(save::SavePlugin);
    }
}
