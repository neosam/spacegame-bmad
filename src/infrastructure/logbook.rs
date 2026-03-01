use std::collections::VecDeque;

use bevy::prelude::*;

use crate::shared::events::{EventSeverity, GameEventKind};

/// A single recorded game event in the logbook.
#[derive(Clone, Debug)]
pub struct LogbookEntry {
    pub kind: GameEventKind,
    /// Human-readable display label, computed once on creation.
    /// For entries restored from save: holds the saved label string.
    pub kind_label: String,
    pub severity: EventSeverity,
    pub game_time: f64,
    pub position: Vec2,
}

/// Stores a capped history of game events for the logbook UI and telemetry.
/// Uses VecDeque for O(1) front-removal when at capacity.
#[derive(Resource, Debug)]
pub struct Logbook {
    entries: VecDeque<LogbookEntry>,
    pub max_entries: usize,
}

impl Default for Logbook {
    fn default() -> Self {
        Self {
            entries: VecDeque::new(),
            max_entries: 500,
        }
    }
}

impl Logbook {
    /// Appends an entry, dropping the oldest if at capacity (O(1)).
    pub fn push(&mut self, entry: LogbookEntry) {
        if self.entries.len() >= self.max_entries {
            self.entries.pop_front();
        }
        self.entries.push_back(entry);
    }

    /// Returns an iterator over entries matching the given severity.
    pub fn entries_by_severity(
        &self,
        severity: EventSeverity,
    ) -> impl Iterator<Item = &LogbookEntry> {
        self.entries.iter().filter(move |e| e.severity == severity)
    }

    /// Returns the last `count` entries (or fewer if not enough).
    pub fn recent_entries(&self, count: usize) -> Vec<&LogbookEntry> {
        let start = self.entries.len().saturating_sub(count);
        self.entries.iter().skip(start).collect()
    }

    /// Returns all entries.
    pub fn entries(&self) -> &VecDeque<LogbookEntry> {
        &self.entries
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_entry(kind: GameEventKind, severity: EventSeverity) -> LogbookEntry {
        use crate::shared::events::event_kind_label;
        let label = event_kind_label(&kind);
        LogbookEntry {
            kind,
            kind_label: label,
            severity,
            game_time: 0.0,
            position: Vec2::ZERO,
        }
    }

    #[test]
    fn logbook_push_appends_entry() {
        let mut logbook = Logbook::default();
        logbook.push(make_entry(
            GameEventKind::PlayerRespawned,
            EventSeverity::Tier2,
        ));
        assert_eq!(logbook.entries().len(), 1);
    }

    #[test]
    fn logbook_capacity_drops_oldest() {
        let mut logbook = Logbook {
            entries: VecDeque::new(),
            max_entries: 3,
        };
        logbook.push(make_entry(
            GameEventKind::PlayerRespawned,
            EventSeverity::Tier2,
        ));
        logbook.push(make_entry(
            GameEventKind::PlayerDeath,
            EventSeverity::Tier1,
        ));
        logbook.push(make_entry(
            GameEventKind::EnemyDestroyed { entity_type: "asteroid" },
            EventSeverity::Tier3,
        ));
        // At capacity — push one more, oldest (PlayerRespawned) should be dropped
        logbook.push(make_entry(
            GameEventKind::PlayerRespawned,
            EventSeverity::Tier2,
        ));
        assert_eq!(logbook.entries().len(), 3);
        // First entry should now be PlayerDeath (the second one pushed originally)
        assert_eq!(logbook.entries()[0].severity, EventSeverity::Tier1);
    }

    #[test]
    fn logbook_entries_by_severity_filters() {
        let mut logbook = Logbook::default();
        logbook.push(make_entry(
            GameEventKind::PlayerDeath,
            EventSeverity::Tier1,
        ));
        logbook.push(make_entry(
            GameEventKind::PlayerRespawned,
            EventSeverity::Tier2,
        ));
        logbook.push(make_entry(
            GameEventKind::EnemyDestroyed { entity_type: "drone" },
            EventSeverity::Tier3,
        ));
        logbook.push(make_entry(
            GameEventKind::PlayerDeath,
            EventSeverity::Tier1,
        ));

        let tier1: Vec<_> = logbook.entries_by_severity(EventSeverity::Tier1).collect();
        assert_eq!(tier1.len(), 2);
        let tier2: Vec<_> = logbook.entries_by_severity(EventSeverity::Tier2).collect();
        assert_eq!(tier2.len(), 1);
    }

    #[test]
    fn logbook_recent_entries_returns_last_n() {
        let mut logbook = Logbook::default();
        for i in 0..5 {
            logbook.push(LogbookEntry {
                kind: GameEventKind::PlayerRespawned,
                kind_label: "Player respawned".to_string(),
                severity: EventSeverity::Tier2,
                game_time: i as f64,
                position: Vec2::ZERO,
            });
        }
        let recent = logbook.recent_entries(3);
        assert_eq!(recent.len(), 3);
        assert!((recent[0].game_time - 2.0).abs() < f64::EPSILON);
        assert!((recent[1].game_time - 3.0).abs() < f64::EPSILON);
        assert!((recent[2].game_time - 4.0).abs() < f64::EPSILON);
    }

    #[test]
    fn logbook_recent_entries_returns_all_when_count_exceeds_length() {
        let mut logbook = Logbook::default();
        logbook.push(make_entry(
            GameEventKind::PlayerRespawned,
            EventSeverity::Tier2,
        ));
        let recent = logbook.recent_entries(100);
        assert_eq!(recent.len(), 1);
    }
}
