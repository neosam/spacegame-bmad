use std::collections::HashMap;

use bevy::prelude::*;
use serde::Deserialize;

use crate::core::flight::Player;
use crate::core::spawning::{Asteroid, ScoutDrone};

// ── Types ────────────────────────────────────────────────────────────────

/// Classification of entities for minimap display.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BlipType {
    Asteroid,
    ScoutDrone,
    Station,
    Player,
}

/// Minimap configuration loaded from `assets/config/minimap.ron`.
#[derive(Resource, Deserialize, Clone, Debug)]
pub struct MinimapConfig {
    /// World-unit radius within which entities appear as blips.
    pub scanner_range: f32,
    /// Screen-pixel radius of the minimap circle.
    pub minimap_radius: f32,
    /// Screen-pixel size of each blip dot.
    pub blip_size: f32,
    /// Margin from the right edge of the screen.
    pub margin_right: f32,
    /// Margin from the top edge of the screen.
    pub margin_top: f32,
    /// RGBA colors per blip type (0.0–1.0).
    pub color_player: [f32; 4],
    pub color_asteroid: [f32; 4],
    pub color_drone: [f32; 4],
    /// Minimap background RGBA.
    pub color_background: [f32; 4],
    /// Station blip RGBA (future use).
    pub color_station: [f32; 4],
    /// Border ring RGBA.
    pub color_border: [f32; 4],
    /// Width of the border ring in pixels.
    pub border_width: f32,
}

impl Default for MinimapConfig {
    fn default() -> Self {
        Self {
            scanner_range: 2000.0,
            minimap_radius: 80.0,
            blip_size: 4.0,
            margin_right: 20.0,
            margin_top: 20.0,
            color_player: [1.0, 1.0, 1.0, 1.0],
            color_asteroid: [0.5, 0.5, 0.5, 0.8],
            color_drone: [0.9, 0.2, 0.2, 0.9],
            color_station: [0.2, 0.8, 0.2, 1.0],
            color_background: [0.0, 0.0, 0.1, 0.6],
            color_border: [0.3, 0.5, 0.7, 0.8],
            border_width: 2.0,
        }
    }
}

impl MinimapConfig {
    /// Load config from RON string.
    pub fn from_ron(ron_str: &str) -> Result<Self, ron::error::SpannedError> {
        ron::from_str(ron_str)
    }
}

// ── Pure functions ───────────────────────────────────────────────────────

/// Convert a world-space offset (entity - player) to minimap-local coordinates.
///
/// Returns `(x, y)` in minimap pixel space, where `(0, 0)` is the center.
/// Result is clamped to `minimap_radius` so blips at the edge of scanner range
/// sit on the minimap boundary.
///
/// **Y-axis flip:** World Y-up is converted to UI Y-down.
pub fn world_to_minimap(offset: Vec2, scanner_range: f32, minimap_radius: f32) -> Vec2 {
    if scanner_range <= 0.0 {
        return Vec2::ZERO;
    }
    let scaled = offset / scanner_range * minimap_radius;
    let len = scaled.length();
    let clamped = if len > minimap_radius {
        scaled / len * minimap_radius
    } else {
        scaled
    };
    // Flip Y: world Y-up → UI Y-down
    Vec2::new(clamped.x, -clamped.y)
}

/// Check whether a squared distance falls within scanner range.
pub fn is_in_scanner_range(distance_sq: f32, scanner_range: f32) -> bool {
    distance_sq <= scanner_range * scanner_range
}

/// Return the `Color` for a given blip type.
pub fn blip_color(blip_type: BlipType, config: &MinimapConfig) -> Color {
    let c = match blip_type {
        BlipType::Player => config.color_player,
        BlipType::Asteroid => config.color_asteroid,
        BlipType::ScoutDrone => config.color_drone,
        BlipType::Station => config.color_station,
    };
    Color::srgba(c[0], c[1], c[2], c[3])
}

// ── Marker components ───────────────────────────────────────────────────

/// Marks the minimap root UI container.
#[derive(Component)]
pub struct MinimapRoot;

/// Marks a blip UI node and tracks which world entity it represents.
#[derive(Component)]
pub struct MinimapBlip {
    pub source_entity: Entity,
}

/// Marks the player-center dot on the minimap.
#[derive(Component)]
pub struct MinimapPlayerDot;

// ── Resources ───────────────────────────────────────────────────────────

/// Tracks the mapping from world entity → minimap blip entity.
#[derive(Resource, Default)]
pub struct MinimapState {
    pub blips: HashMap<Entity, Entity>,
}

// ── Systems ─────────────────────────────────────────────────────────────

/// Spawn the minimap UI: background circle, border, and player center dot.
pub fn setup_minimap(mut commands: Commands, config: Res<MinimapConfig>) {
    let diameter = config.minimap_radius * 2.0;
    let total_diameter = diameter + config.border_width * 2.0;

    // Border ring (outer circle)
    let border_id = commands
        .spawn((
            Node {
                width: Val::Px(total_diameter),
                height: Val::Px(total_diameter),
                position_type: PositionType::Absolute,
                right: Val::Px(config.margin_right),
                top: Val::Px(config.margin_top),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                border_radius: BorderRadius::all(Val::Percent(50.0)),
                ..default()
            },
            BackgroundColor(Color::srgba(
                config.color_border[0],
                config.color_border[1],
                config.color_border[2],
                config.color_border[3],
            )),
        ))
        .id();

    // Minimap background (inner circle, child of border)
    let root_id = commands
        .spawn((
            MinimapRoot,
            Node {
                width: Val::Px(diameter),
                height: Val::Px(diameter),
                overflow: Overflow::clip(),
                border_radius: BorderRadius::all(Val::Percent(50.0)),
                ..default()
            },
            BackgroundColor(Color::srgba(
                config.color_background[0],
                config.color_background[1],
                config.color_background[2],
                config.color_background[3],
            )),
        ))
        .id();

    commands.entity(border_id).add_child(root_id);

    // Player center dot
    let player_dot = commands
        .spawn((
            MinimapPlayerDot,
            Node {
                width: Val::Px(config.blip_size + 2.0),
                height: Val::Px(config.blip_size + 2.0),
                position_type: PositionType::Absolute,
                left: Val::Px(config.minimap_radius - (config.blip_size + 2.0) / 2.0),
                top: Val::Px(config.minimap_radius - (config.blip_size + 2.0) / 2.0),
                border_radius: BorderRadius::all(Val::Percent(50.0)),
                ..default()
            },
            BackgroundColor(Color::srgba(
                config.color_player[0],
                config.color_player[1],
                config.color_player[2],
                config.color_player[3],
            )),
        ))
        .id();

    commands.entity(root_id).add_child(player_dot);
}

/// Update minimap blips every frame.
///
/// For each entity (Asteroid/ScoutDrone) in scanner range of the player,
/// creates or updates a blip UI node. Removes blips for entities that
/// left range or were despawned.
#[allow(clippy::type_complexity)]
pub fn update_minimap_blips(
    mut commands: Commands,
    player_query: Query<&Transform, With<Player>>,
    entity_query: Query<(Entity, &Transform, Has<Asteroid>), Or<(With<Asteroid>, With<ScoutDrone>)>>,
    config: Res<MinimapConfig>,
    mut state: ResMut<MinimapState>,
    minimap_root: Query<Entity, With<MinimapRoot>>,
    mut blip_nodes: Query<&mut Node, With<MinimapBlip>>,
) {
    let Ok(player_tf) = player_query.single() else {
        return;
    };
    let Ok(root_entity) = minimap_root.single() else {
        return;
    };

    let player_pos = player_tf.translation.truncate();
    let half_blip = config.blip_size / 2.0;

    // Collect entities currently in range
    let mut in_range = HashMap::<Entity, (Vec2, BlipType)>::new();
    for (entity, transform, is_asteroid) in entity_query.iter() {
        let blip_type = if is_asteroid {
            BlipType::Asteroid
        } else {
            BlipType::ScoutDrone
        };

        let entity_pos = transform.translation.truncate();
        let offset = entity_pos - player_pos;
        let dist_sq = offset.length_squared();

        if is_in_scanner_range(dist_sq, config.scanner_range) {
            in_range.insert(entity, (offset, blip_type));
        }
    }

    // Remove blips for entities no longer in range or despawned
    let stale: Vec<Entity> = state
        .blips
        .keys()
        .filter(|e| !in_range.contains_key(e))
        .copied()
        .collect();
    for entity in stale {
        if let Some(blip_entity) = state.blips.remove(&entity) {
            commands.entity(blip_entity).despawn();
        }
    }

    // Update existing blips and create new ones
    for (entity, (offset, blip_type)) in &in_range {
        let minimap_pos =
            world_to_minimap(*offset, config.scanner_range, config.minimap_radius);
        let left = config.minimap_radius + minimap_pos.x - half_blip;
        let top = config.minimap_radius + minimap_pos.y - half_blip;

        if let Some(blip_entity) = state.blips.get(entity) {
            // Update existing blip position
            if let Ok(mut node) = blip_nodes.get_mut(*blip_entity) {
                node.left = Val::Px(left);
                node.top = Val::Px(top);
            }
        } else {
            // Spawn new blip
            let color = blip_color(*blip_type, &config);
            let blip_id = commands
                .spawn((
                    MinimapBlip {
                        source_entity: *entity,
                    },
                    Node {
                        width: Val::Px(config.blip_size),
                        height: Val::Px(config.blip_size),
                        position_type: PositionType::Absolute,
                        left: Val::Px(left),
                        top: Val::Px(top),
                        border_radius: BorderRadius::all(Val::Percent(50.0)),
                        ..default()
                    },
                    BackgroundColor(color),
                ))
                .id();

            commands.entity(root_entity).add_child(blip_id);
            state.blips.insert(*entity, blip_id);
        }
    }
}

// ── Unit tests ──────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn minimap_config_default_has_valid_values() {
        let config = MinimapConfig::default();
        assert!(config.scanner_range > 0.0);
        assert!(config.minimap_radius > 0.0);
        assert!(config.blip_size > 0.0);
        assert!(config.margin_right >= 0.0);
        assert!(config.margin_top >= 0.0);
        assert!(config.border_width >= 0.0);
    }

    #[test]
    fn minimap_config_from_ron() {
        let ron_str = r#"(
            scanner_range: 3000.0,
            minimap_radius: 100.0,
            blip_size: 6.0,
            margin_right: 30.0,
            margin_top: 30.0,
            color_player: (1.0, 1.0, 1.0, 1.0),
            color_asteroid: (0.4, 0.4, 0.4, 0.7),
            color_drone: (0.8, 0.1, 0.1, 0.9),
            color_station: (0.1, 0.7, 0.1, 1.0),
            color_background: (0.0, 0.0, 0.15, 0.7),
            color_border: (0.2, 0.4, 0.6, 0.8),
            border_width: 3.0,
        )"#;
        let config = MinimapConfig::from_ron(ron_str).expect("Should parse MinimapConfig RON");
        assert!((config.scanner_range - 3000.0).abs() < f32::EPSILON);
        assert!((config.minimap_radius - 100.0).abs() < f32::EPSILON);
        assert!((config.blip_size - 6.0).abs() < f32::EPSILON);
    }

    #[test]
    fn minimap_config_from_ron_invalid() {
        let result = MinimapConfig::from_ron("not valid ron");
        assert!(result.is_err(), "Invalid RON should return error");
    }

    #[test]
    fn world_to_minimap_center_returns_zero() {
        let result = world_to_minimap(Vec2::ZERO, 1000.0, 80.0);
        assert!((result.x).abs() < f32::EPSILON);
        assert!((result.y).abs() < f32::EPSILON);
    }

    #[test]
    fn world_to_minimap_scales_correctly() {
        // Entity at half scanner range to the right → half minimap radius to the right
        let result = world_to_minimap(Vec2::new(500.0, 0.0), 1000.0, 80.0);
        assert!((result.x - 40.0).abs() < 0.01, "x should be 40.0, got {}", result.x);
        assert!((result.y).abs() < 0.01, "y should be 0.0, got {}", result.y);
    }

    #[test]
    fn world_to_minimap_flips_y_axis() {
        // World Y-up → UI Y-down
        let result = world_to_minimap(Vec2::new(0.0, 500.0), 1000.0, 80.0);
        assert!((result.x).abs() < 0.01);
        assert!((result.y - (-40.0)).abs() < 0.01, "y should be -40.0 (flipped), got {}", result.y);
    }

    #[test]
    fn world_to_minimap_clamps_beyond_range() {
        // Entity at 2× scanner range → clamped to minimap radius
        let result = world_to_minimap(Vec2::new(2000.0, 0.0), 1000.0, 80.0);
        let len = result.length();
        assert!(
            (len - 80.0).abs() < 0.01,
            "Blip beyond range should be clamped to minimap_radius, got {len}"
        );
    }

    #[test]
    fn world_to_minimap_zero_scanner_range_returns_zero() {
        let result = world_to_minimap(Vec2::new(100.0, 200.0), 0.0, 80.0);
        assert!((result.x).abs() < f32::EPSILON);
        assert!((result.y).abs() < f32::EPSILON);
    }

    #[test]
    fn is_in_scanner_range_inside() {
        assert!(is_in_scanner_range(500.0 * 500.0, 1000.0));
    }

    #[test]
    fn is_in_scanner_range_outside() {
        assert!(!is_in_scanner_range(1500.0 * 1500.0, 1000.0));
    }

    #[test]
    fn is_in_scanner_range_exact_boundary() {
        assert!(is_in_scanner_range(1000.0 * 1000.0, 1000.0));
    }

    #[test]
    fn blip_color_returns_expected_colors() {
        let config = MinimapConfig::default();
        let player_color = blip_color(BlipType::Player, &config);
        let asteroid_color = blip_color(BlipType::Asteroid, &config);
        let drone_color = blip_color(BlipType::ScoutDrone, &config);

        // Player should be white
        assert_eq!(player_color, Color::srgba(1.0, 1.0, 1.0, 1.0));
        // Asteroid should be gray
        assert_eq!(asteroid_color, Color::srgba(0.5, 0.5, 0.5, 0.8));
        // Drone should be red
        assert_eq!(drone_color, Color::srgba(0.9, 0.2, 0.2, 0.9));
    }
}
