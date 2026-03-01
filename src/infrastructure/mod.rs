pub mod audio;
pub mod events;
pub mod logbook;
pub mod save;

use bevy::prelude::*;

use crate::core::CoreSet;
use crate::shared::events::GameEvent;
use self::audio::AudioInfrastructurePlugin;
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
        // Run in Update (not FixedUpdate) so all FixedUpdate event emitters have
        // already completed before we drain the message buffer. This guarantees
        // that Tier1/2 events emitted in CoreSet::Events or after it are captured.
        app.add_systems(Update, record_game_events);
        app.add_plugins(save::SavePlugin);
        app.add_plugins(AudioInfrastructurePlugin);
    }
}
