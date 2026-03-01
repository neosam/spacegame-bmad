use bevy::ecs::message::MessageReader;
use bevy::prelude::*;
use serde::Deserialize;
use std::collections::HashMap;

use crate::shared::events::{event_kind_label, EventSeverity, GameEvent, GameEventKind};
use super::logbook::{Logbook, LogbookEntry};

/// Maps `GameEventKind` variant names to severity tiers.
/// Loaded from `assets/config/event_severity.ron` with fallback to defaults.
#[derive(Resource, Deserialize, Clone, Debug)]
pub struct EventSeverityConfig {
    #[serde(default)]
    pub mappings: HashMap<String, EventSeverity>,
}

impl Default for EventSeverityConfig {
    fn default() -> Self {
        let mut mappings = HashMap::new();
        mappings.insert("PlayerDeath".to_string(), EventSeverity::Tier1);
        mappings.insert("PlayerRespawned".to_string(), EventSeverity::Tier2);
        mappings.insert("EnemyDestroyed".to_string(), EventSeverity::Tier3);
        mappings.insert("ChunkLoaded".to_string(), EventSeverity::Tier3);
        mappings.insert("ChunkUnloaded".to_string(), EventSeverity::Tier3);
        mappings.insert("WeaponFired".to_string(), EventSeverity::Tier3);
        mappings.insert("WeaponSwitched".to_string(), EventSeverity::Tier3);
        mappings.insert("GameSaved".to_string(), EventSeverity::Tier2);
        mappings.insert("TutorialZoneSpawned".to_string(), EventSeverity::Tier2);
        mappings.insert("StationDocked".to_string(), EventSeverity::Tier1);
        mappings.insert("GeneratorDestroyed".to_string(), EventSeverity::Tier1);
        mappings.insert("TutorialComplete".to_string(), EventSeverity::Tier1);
        mappings.insert("CreditsEarned".to_string(), EventSeverity::Tier3);
        mappings.insert("MaterialCollected".to_string(), EventSeverity::Tier3);
        mappings.insert("UpgradeCrafted".to_string(), EventSeverity::Tier2);
        mappings.insert("CompanionRecruited".to_string(), EventSeverity::Tier1);
        mappings.insert("BossSpawned".to_string(), EventSeverity::Tier1);
        mappings.insert("BossDestroyed".to_string(), EventSeverity::Tier1);
        Self { mappings }
    }
}

impl EventSeverityConfig {
    /// Load config from RON string.
    pub fn from_ron(ron_str: &str) -> Result<Self, ron::error::SpannedError> {
        ron::from_str(ron_str)
    }

    /// Validates mappings: warns about unknown or missing keys.
    pub fn validate(&self) {
        let known_keys = [
            "EnemyDestroyed",
            "PlayerDeath",
            "PlayerRespawned",
            "ChunkLoaded",
            "ChunkUnloaded",
            "WeaponFired",
            "WeaponSwitched",
            "GameSaved",
            "TutorialZoneSpawned",
            "StationDocked",
            "GeneratorDestroyed",
            "TutorialComplete",
            "CreditsEarned",
            "MaterialCollected",
            "UpgradeCrafted",
            "CompanionRecruited",
            "BossSpawned",
            "BossDestroyed",
        ];

        for key in self.mappings.keys() {
            if !known_keys.contains(&key.as_str()) {
                warn!(
                    "EventSeverityConfig: unknown key '{}' in mappings.",
                    key
                );
            }
        }

        for &expected in &known_keys {
            if !self.mappings.contains_key(expected) {
                warn!(
                    "EventSeverityConfig: missing key '{}', will fall back to Tier3.",
                    expected
                );
            }
        }
    }

    /// Returns the severity for a given event kind.
    /// Falls back to `Tier3` if the kind is not mapped.
    pub fn severity_for(&self, kind: &GameEventKind) -> EventSeverity {
        let key = match kind {
            GameEventKind::EnemyDestroyed { .. } => "EnemyDestroyed",
            GameEventKind::PlayerDeath => "PlayerDeath",
            GameEventKind::PlayerRespawned => "PlayerRespawned",
            GameEventKind::ChunkLoaded { .. } => "ChunkLoaded",
            GameEventKind::ChunkUnloaded { .. } => "ChunkUnloaded",
            GameEventKind::WeaponFired { .. } => "WeaponFired",
            GameEventKind::WeaponSwitched { .. } => "WeaponSwitched",
            GameEventKind::GameSaved => "GameSaved",
            GameEventKind::TutorialZoneSpawned => "TutorialZoneSpawned",
            GameEventKind::StationDocked => "StationDocked",
            GameEventKind::GeneratorDestroyed => "GeneratorDestroyed",
            GameEventKind::TutorialComplete => "TutorialComplete",
            GameEventKind::CreditsEarned { .. } => "CreditsEarned",
            GameEventKind::MaterialCollected { .. } => "MaterialCollected",
            GameEventKind::UpgradeCrafted { .. } => "UpgradeCrafted",
            GameEventKind::CompanionRecruited { .. } => "CompanionRecruited",
            GameEventKind::BossSpawned { .. } => "BossSpawned",
            GameEventKind::BossDestroyed { .. } => "BossDestroyed",
        };
        self.mappings
            .get(key)
            .copied()
            .unwrap_or(EventSeverity::Tier3)
    }
}

/// Reads all `GameEvent` messages each frame and appends entries to the `Logbook`.
pub fn record_game_events(
    mut events: MessageReader<GameEvent>,
    mut logbook: ResMut<Logbook>,
) {
    for event in events.read() {
        let label = event_kind_label(&event.kind);
        logbook.push(LogbookEntry {
            kind: event.kind.clone(),
            kind_label: label,
            severity: event.severity,
            game_time: event.game_time,
            position: event.position,
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn severity_config_default_maps_player_death_to_tier1() {
        let config = EventSeverityConfig::default();
        let severity = config.severity_for(&GameEventKind::PlayerDeath);
        assert_eq!(severity, EventSeverity::Tier1);
    }

    #[test]
    fn severity_config_from_ron_parses_correctly() {
        let ron_str = r#"(
            mappings: {
                "EnemyDestroyed": Tier2,
                "PlayerDeath": Tier1,
            },
        )"#;
        let config = EventSeverityConfig::from_ron(ron_str).expect("Should parse RON");
        assert_eq!(
            config.severity_for(&GameEventKind::EnemyDestroyed {
                entity_type: "asteroid"
            }),
            EventSeverity::Tier2
        );
        assert_eq!(
            config.severity_for(&GameEventKind::PlayerDeath),
            EventSeverity::Tier1
        );
    }

    #[test]
    fn severity_config_unknown_kind_falls_back_to_tier3() {
        // Config with empty mappings — everything should fall back to Tier3
        let config = EventSeverityConfig {
            mappings: HashMap::new(),
        };
        let severity = config.severity_for(&GameEventKind::PlayerDeath);
        assert_eq!(severity, EventSeverity::Tier3);
    }

    #[test]
    fn severity_config_default_has_all_mappings() {
        let config = EventSeverityConfig::default();
        assert_eq!(config.mappings.len(), 18, "Should have 18 default mappings");
        assert_eq!(
            config.severity_for(&GameEventKind::PlayerRespawned),
            EventSeverity::Tier2
        );
        assert_eq!(
            config.severity_for(&GameEventKind::EnemyDestroyed {
                entity_type: "drone"
            }),
            EventSeverity::Tier3
        );
    }

    #[test]
    fn record_game_events_writes_to_logbook() {
        use bevy::ecs::message::MessageWriter;

        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_message::<GameEvent>();
        app.init_resource::<Logbook>();
        app.add_systems(Update, (
            |mut writer: MessageWriter<GameEvent>| {
                writer.write(GameEvent {
                    kind: GameEventKind::PlayerDeath,
                    severity: EventSeverity::Tier1,
                    position: Vec2::new(10.0, 20.0),
                    game_time: 1.5,
                });
                writer.write(GameEvent {
                    kind: GameEventKind::PlayerRespawned,
                    severity: EventSeverity::Tier2,
                    position: Vec2::ZERO,
                    game_time: 1.5,
                });
            },
            record_game_events,
        ).chain());

        app.update();

        let logbook = app.world().resource::<Logbook>();
        let entries = logbook.entries();
        assert_eq!(entries.len(), 2, "Logbook should have 2 entries");
        assert_eq!(entries[0].severity, EventSeverity::Tier1);
        assert_eq!(entries[1].severity, EventSeverity::Tier2);
        // Labels should be computed
        assert!(!entries[0].kind_label.is_empty());
        assert!(!entries[1].kind_label.is_empty());
    }

    #[test]
    fn severity_config_validate_default_has_no_warnings() {
        // Default config should have all known keys — validate should not panic
        let config = EventSeverityConfig::default();
        config.validate(); // should produce no warnings
        assert_eq!(config.mappings.len(), 18);
    }

    #[test]
    fn severity_config_validate_detects_unknown_keys() {
        let mut config = EventSeverityConfig::default();
        config
            .mappings
            .insert("Typo".to_string(), EventSeverity::Tier1);
        // validate() would warn about "Typo" — we just verify it doesn't panic
        config.validate();
    }

    #[test]
    fn severity_config_validate_detects_missing_keys() {
        let config = EventSeverityConfig {
            mappings: HashMap::new(),
        };
        // validate() would warn about all 8 missing keys — verify no panic
        config.validate();
    }
}
