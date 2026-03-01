use bevy::prelude::*;

// ── Types ────────────────────────────────────────────────────────────────

/// Type of open-world station, determines available services.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum StationType {
    Trading,
    Repair,
    Research,
}

// ── Components ───────────────────────────────────────────────────────────

/// Open-world station entity. Distinct from `TutorialStation` which is tutorial-specific.
#[derive(Component, Debug)]
pub struct Station {
    pub name: &'static str,
    pub dock_radius: f32,
    pub station_type: StationType,
}

/// Marker component added to player when they dock at a `Station`.
/// Removed on undocking.
#[derive(Component, Debug)]
pub struct Docked {
    /// The station entity the player is docked at.
    pub station: Entity,
}

/// Marker component added to a `Station` entity at spawn.
/// The rendering layer responds by adding `Mesh2d` + `MeshMaterial2d`
/// and then removing this marker. Core never touches rendering.
#[derive(Component, Debug)]
pub struct NeedsStationVisual;

// ── Systems ──────────────────────────────────────────────────────────────

/// Checks whether the player is within dock_radius of a Station AND pressed interact.
/// If so, inserts `Docked { station }` on the player and emits `GameEventKind::StationDocked`.
/// Consumes `interact` on successful dock so `update_undocking` (chained after) does not
/// immediately undo the dock in the same frame.
///
/// Guards:
/// - Player must NOT already have `Docked`
/// - `ActionState.interact` must be true (rising edge from E key)
/// - Distance must be <= station.dock_radius
pub fn update_docking(
    mut action_state: ResMut<crate::core::input::ActionState>,
    player_query: Query<(Entity, &Transform), (With<crate::core::flight::Player>, Without<Docked>)>,
    station_query: Query<(Entity, &Station, &Transform)>,
    mut commands: Commands,
    mut game_events: bevy::ecs::message::MessageWriter<crate::shared::events::GameEvent>,
    time: Res<Time>,
    severity_config: Res<crate::infrastructure::events::EventSeverityConfig>,
) {
    if !action_state.interact {
        return;
    }
    let Ok((player_entity, player_transform)) = player_query.single() else {
        return;
    };
    let player_pos = player_transform.translation.truncate();

    for (station_entity, station, station_transform) in station_query.iter() {
        let station_pos = station_transform.translation.truncate();
        let distance = (station_pos - player_pos).length();
        if distance <= station.dock_radius {
            commands.entity(player_entity).insert(Docked { station: station_entity });
            // Consume interact so update_undocking (chained next) does not immediately undock
            action_state.interact = false;
            let kind = crate::shared::events::GameEventKind::StationDocked;
            game_events.write(crate::shared::events::GameEvent {
                severity: severity_config.severity_for(&kind),
                kind,
                position: station_pos,
                game_time: time.elapsed_secs_f64(),
            });
            // Dock at first station found within range
            return;
        }
    }
}

/// Removes `Docked` from the player when:
/// - They move farther than `dock_radius * 1.1` from the docked station (hysteresis), OR
/// - They press interact again while already docked.
///
/// If the docked station entity no longer exists, `Docked` is also removed.
pub fn update_undocking(
    action_state: Res<crate::core::input::ActionState>,
    player_query: Query<(Entity, &Transform, &Docked), With<crate::core::flight::Player>>,
    station_query: Query<(&Station, &Transform)>,
    mut commands: Commands,
) {
    let Ok((player_entity, player_transform, docked)) = player_query.single() else {
        return;
    };

    // Undock if interact pressed again
    if action_state.interact {
        commands.entity(player_entity).remove::<Docked>();
        return;
    }

    // Undock if station entity no longer exists
    let Ok((station, station_transform)) = station_query.get(docked.station) else {
        commands.entity(player_entity).remove::<Docked>();
        return;
    };

    // Undock if player moved too far (hysteresis: 1.1x dock_radius)
    let player_pos = player_transform.translation.truncate();
    let station_pos = station_transform.translation.truncate();
    let distance = (station_pos - player_pos).length();
    if distance > station.dock_radius * 1.1 {
        commands.entity(player_entity).remove::<Docked>();
    }
}

// ── Unit Tests ───────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn station_type_variants_are_distinct() {
        assert_ne!(StationType::Trading, StationType::Repair);
        assert_ne!(StationType::Repair, StationType::Research);
        assert_ne!(StationType::Trading, StationType::Research);
    }

    #[test]
    fn station_component_fields() {
        let s = Station {
            name: "Trading Post",
            dock_radius: 120.0,
            station_type: StationType::Trading,
        };
        assert_eq!(s.name, "Trading Post");
        assert!((s.dock_radius - 120.0).abs() < f32::EPSILON);
        assert_eq!(s.station_type, StationType::Trading);
    }

    #[test]
    fn docked_component_holds_entity() {
        // Entity::from_bits is only used in tests for constructing a dummy entity id
        let fake_entity = Entity::from_bits(42);
        let docked = Docked { station: fake_entity };
        assert_eq!(docked.station, fake_entity);
    }

    #[test]
    fn needs_station_visual_is_a_component() {
        // If this compiles, NeedsStationVisual implements Component
        let _marker = NeedsStationVisual;
    }
}
