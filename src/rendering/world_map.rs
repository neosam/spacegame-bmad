use std::collections::HashMap;

use bevy::prelude::*;
use serde::Deserialize;

use crate::core::flight::Player;
use crate::core::station::DiscoveredStations;
use crate::world::{ChunkCoord, ExploredChunks};
use crate::world::chunk::world_to_chunk;

// ── Config ──────────────────────────────────────────────────────────────

/// World map overlay configuration loaded from `assets/config/world_map.ron`.
#[derive(Resource, Deserialize, Clone, Debug)]
pub struct WorldMapConfig {
    /// Pixel size of each chunk tile on the map.
    pub tile_size: f32,
    /// Pixel size of the player position marker.
    pub player_marker_size: f32,
    /// Map container width in pixels.
    pub map_width: f32,
    /// Map container height in pixels.
    pub map_height: f32,
    /// Deep Space biome tile color RGBA.
    pub color_deep_space: [f32; 4],
    /// Asteroid Field biome tile color RGBA.
    pub color_asteroid_field: [f32; 4],
    /// Wreck Field biome tile color RGBA.
    pub color_wreck_field: [f32; 4],
    /// Player marker color RGBA.
    pub color_player: [f32; 4],
    /// Overlay background color RGBA.
    pub color_background: [f32; 4],
    /// Station marker color RGBA.
    pub color_station: [f32; 4],
}

impl Default for WorldMapConfig {
    fn default() -> Self {
        Self {
            tile_size: 12.0,
            player_marker_size: 8.0,
            map_width: 800.0,
            map_height: 600.0,
            color_deep_space: [0.08, 0.08, 0.2, 0.9],
            color_asteroid_field: [0.45, 0.45, 0.45, 0.9],
            color_wreck_field: [0.6, 0.4, 0.15, 0.9],
            color_player: [1.0, 1.0, 1.0, 1.0],
            color_background: [0.0, 0.0, 0.0, 0.75],
            color_station: [0.2, 0.9, 0.2, 1.0],
        }
    }
}

impl WorldMapConfig {
    /// Load config from RON string with graceful fallback.
    pub fn from_ron(ron_str: &str) -> Result<Self, ron::error::SpannedError> {
        ron::from_str(ron_str)
    }
}

// ── Pure functions ──────────────────────────────────────────────────────

use crate::world::BiomeType;

/// Convert a chunk coordinate to map-space pixel position relative to the map container.
///
/// The map is centered on `player_chunk`. Result is `(left, top)` in pixels
/// from the map container's top-left corner.
///
/// **Y-axis flip:** World Y-up → UI Y-down.
pub fn chunk_to_map_position(
    chunk: ChunkCoord,
    player_chunk: ChunkCoord,
    map_center: Vec2,
    tile_size: f32,
) -> Vec2 {
    let dx = (chunk.x - player_chunk.x) as f32;
    let dy = (chunk.y - player_chunk.y) as f32;
    Vec2::new(
        map_center.x + dx * tile_size - tile_size / 2.0,
        map_center.y - dy * tile_size - tile_size / 2.0, // Y-flip
    )
}

/// Return the `Color` for a given biome type based on config.
pub fn biome_map_color(biome: BiomeType, config: &WorldMapConfig) -> Color {
    let arr = match biome {
        BiomeType::DeepSpace => config.color_deep_space,
        BiomeType::AsteroidField => config.color_asteroid_field,
        BiomeType::WreckField => config.color_wreck_field,
    };
    Color::srgba(arr[0], arr[1], arr[2], arr[3])
}

/// Convert a world-space position to map-space pixel position (sub-chunk precision).
///
/// Returns `(left, top)` in pixels from the map container's top-left corner.
/// **Y-axis flip:** World Y-up → UI Y-down.
pub fn world_to_map_position(
    world_pos: Vec2,
    player_world_pos: Vec2,
    chunk_size: f32,
    map_center: Vec2,
    tile_size: f32,
) -> Vec2 {
    let offset = world_pos - player_world_pos;
    let map_offset = offset / chunk_size * tile_size;
    Vec2::new(
        map_center.x + map_offset.x,
        map_center.y - map_offset.y, // Y-flip
    )
}

/// Check whether a chunk tile would be visible within the map container.
pub fn is_tile_visible(
    chunk: ChunkCoord,
    player_chunk: ChunkCoord,
    map_width: f32,
    map_height: f32,
    tile_size: f32,
) -> bool {
    let map_center = Vec2::new(map_width / 2.0, map_height / 2.0);
    let pos = chunk_to_map_position(chunk, player_chunk, map_center, tile_size);
    pos.x > -tile_size
        && pos.x < map_width
        && pos.y > -tile_size
        && pos.y < map_height
}

// ── Marker components ───────────────────────────────────────────────────

/// Marks the world map root UI overlay.
#[derive(Component)]
pub struct WorldMapRoot;

/// Marks a chunk tile on the world map.
#[derive(Component)]
pub struct WorldMapTile;

/// Marks the player position marker on the world map.
#[derive(Component)]
pub struct WorldMapPlayerMarker;

/// Marks a station marker on the world map, storing the station's world position
/// so it can be repositioned when the player moves to a new chunk.
#[derive(Component)]
pub struct WorldMapStationMarker {
    pub world_pos: Vec2,
}

// ── Resources ───────────────────────────────────────────────────────────

/// Whether the world map overlay is currently open.
#[derive(Resource, Default)]
pub struct WorldMapOpen(pub bool);

/// Tracks which chunks have been rendered on the currently open map.
#[derive(Resource, Default)]
pub struct WorldMapState {
    pub rendered_chunks: HashMap<ChunkCoord, Entity>,
    pub map_container: Option<Entity>,
    pub center_chunk: ChunkCoord,
}

// ── Systems ─────────────────────────────────────────────────────────────

/// Toggle the world map overlay on M key press.
#[allow(clippy::too_many_arguments)]
pub fn toggle_world_map(
    mut commands: Commands,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut map_open: ResMut<WorldMapOpen>,
    mut map_state: ResMut<WorldMapState>,
    config: Res<WorldMapConfig>,
    explored: Res<ExploredChunks>,
    player_query: Query<&Transform, With<Player>>,
    world_config: Res<crate::world::WorldConfig>,
    root_query: Query<Entity, With<WorldMapRoot>>,
    discovered_stations: Option<Res<DiscoveredStations>>,
) {
    if !keyboard.just_pressed(KeyCode::KeyM) {
        return;
    }

    if map_open.0 {
        // Close the map
        for entity in root_query.iter() {
            commands.entity(entity).despawn();
        }
        map_state.rendered_chunks.clear();
        map_state.map_container = None;
        map_open.0 = false;
    } else {
        // Open the map — use persistent discovered positions (survive chunk unloads)
        map_open.0 = true;
        open_world_map(
            &mut commands,
            &config,
            &explored,
            &player_query,
            world_config.chunk_size,
            &mut map_state,
            discovered_stations.as_ref().map(|d| d.positions.as_slice()).unwrap_or(&[]),
        );
    }
}

/// Spawn the world map UI hierarchy.
fn open_world_map(
    commands: &mut Commands,
    config: &WorldMapConfig,
    explored: &ExploredChunks,
    player_query: &Query<&Transform, With<Player>>,
    chunk_size: f32,
    map_state: &mut WorldMapState,
    station_positions: &[Vec2],
) {
    let (player_world_pos, player_chunk) = if let Ok(tf) = player_query.single() {
        let wp = tf.translation.truncate();
        (wp, world_to_chunk(wp, chunk_size))
    } else {
        (Vec2::ZERO, ChunkCoord { x: 0, y: 0 })
    };

    let map_center = Vec2::new(config.map_width / 2.0, config.map_height / 2.0);

    // Root overlay (fullscreen semi-transparent)
    let root = commands
        .spawn((
            WorldMapRoot,
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                position_type: PositionType::Absolute,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            GlobalZIndex(100),
            BackgroundColor(Color::srgba(
                config.color_background[0],
                config.color_background[1],
                config.color_background[2],
                config.color_background[3],
            )),
        ))
        .id();

    // Map container (fixed size, centered, clipped)
    let container = commands
        .spawn((
            Node {
                width: Val::Px(config.map_width),
                height: Val::Px(config.map_height),
                overflow: Overflow::clip(),
                ..default()
            },
            BackgroundColor(Color::srgba(0.02, 0.02, 0.06, 0.95)),
        ))
        .id();

    commands.entity(root).add_child(container);
    map_state.map_container = Some(container);
    map_state.center_chunk = player_chunk;

    // Spawn tiles for explored chunks
    for (&coord, &biome) in &explored.chunks {
        if is_tile_visible(coord, player_chunk, config.map_width, config.map_height, config.tile_size) {
            let pos = chunk_to_map_position(coord, player_chunk, map_center, config.tile_size);
            let color = biome_map_color(biome, config);

            let tile = commands
                .spawn((
                    WorldMapTile,
                    Node {
                        width: Val::Px(config.tile_size),
                        height: Val::Px(config.tile_size),
                        position_type: PositionType::Absolute,
                        left: Val::Px(pos.x),
                        top: Val::Px(pos.y),
                        ..default()
                    },
                    BackgroundColor(color),
                ))
                .id();

            commands.entity(container).add_child(tile);
            map_state.rendered_chunks.insert(coord, tile);
        }
    }

    // Player marker at center
    let marker = commands
        .spawn((
            WorldMapPlayerMarker,
            Node {
                width: Val::Px(config.player_marker_size),
                height: Val::Px(config.player_marker_size),
                position_type: PositionType::Absolute,
                left: Val::Px(map_center.x - config.player_marker_size / 2.0),
                top: Val::Px(map_center.y - config.player_marker_size / 2.0),
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

    commands.entity(container).add_child(marker);

    // Station markers — bright green squares, always visible
    let station_color = Color::srgba(
        config.color_station[0],
        config.color_station[1],
        config.color_station[2],
        config.color_station[3],
    );
    let station_size = config.player_marker_size + 2.0;
    for &station_world_pos in station_positions {
        let pos = world_to_map_position(
            station_world_pos,
            player_world_pos,
            chunk_size,
            map_center,
            config.tile_size,
        );
        let station_marker = commands
            .spawn((
                WorldMapStationMarker { world_pos: station_world_pos },
                Node {
                    width: Val::Px(station_size),
                    height: Val::Px(station_size),
                    position_type: PositionType::Absolute,
                    left: Val::Px(pos.x - station_size / 2.0),
                    top: Val::Px(pos.y - station_size / 2.0),
                    ..default()
                },
                BackgroundColor(station_color),
            ))
            .id();
        commands.entity(container).add_child(station_marker);
    }
}

/// Update the world map while it is open: reposition tiles when player moves,
/// add newly explored tiles, remove tiles that scrolled out of view.
#[allow(clippy::too_many_arguments)]
pub fn update_world_map(
    mut commands: Commands,
    map_open: Res<WorldMapOpen>,
    config: Res<WorldMapConfig>,
    explored: Res<ExploredChunks>,
    mut map_state: ResMut<WorldMapState>,
    player_query: Query<&Transform, With<Player>>,
    world_config: Res<crate::world::WorldConfig>,
    mut tile_query: Query<&mut Node, (With<WorldMapTile>, Without<WorldMapStationMarker>)>,
    mut station_marker_query: Query<(&WorldMapStationMarker, &mut Node), Without<WorldMapTile>>,
) {
    if !map_open.0 {
        return;
    }

    let Some(container) = map_state.map_container else {
        return;
    };

    let (player_world_pos, player_chunk) = if let Ok(tf) = player_query.single() {
        let wp = tf.translation.truncate();
        (wp, world_to_chunk(wp, world_config.chunk_size))
    } else {
        (Vec2::ZERO, ChunkCoord { x: 0, y: 0 })
    };

    let map_center = Vec2::new(config.map_width / 2.0, config.map_height / 2.0);
    let center_changed = player_chunk != map_state.center_chunk;

    if center_changed {
        // Reposition existing tiles and remove those now out of view
        let mut to_remove = Vec::new();
        for (&coord, &tile_entity) in &map_state.rendered_chunks {
            if is_tile_visible(coord, player_chunk, config.map_width, config.map_height, config.tile_size) {
                let pos = chunk_to_map_position(coord, player_chunk, map_center, config.tile_size);
                if let Ok(mut node) = tile_query.get_mut(tile_entity) {
                    node.left = Val::Px(pos.x);
                    node.top = Val::Px(pos.y);
                }
            } else {
                to_remove.push(coord);
                commands.entity(tile_entity).despawn();
            }
        }
        for coord in to_remove {
            map_state.rendered_chunks.remove(&coord);
        }
        map_state.center_chunk = player_chunk;

        // Reposition station markers
        let station_size = config.player_marker_size + 2.0;
        for (marker, mut node) in station_marker_query.iter_mut() {
            let pos = world_to_map_position(
                marker.world_pos,
                player_world_pos,
                world_config.chunk_size,
                map_center,
                config.tile_size,
            );
            node.left = Val::Px(pos.x - station_size / 2.0);
            node.top = Val::Px(pos.y - station_size / 2.0);
        }
    }

    // Add tiles for newly explored chunks (or chunks that scrolled into view)
    for (&coord, &biome) in &explored.chunks {
        if map_state.rendered_chunks.contains_key(&coord) {
            continue;
        }
        if !is_tile_visible(coord, player_chunk, config.map_width, config.map_height, config.tile_size) {
            continue;
        }

        let pos = chunk_to_map_position(coord, player_chunk, map_center, config.tile_size);
        let color = biome_map_color(biome, &config);

        let tile = commands
            .spawn((
                WorldMapTile,
                Node {
                    width: Val::Px(config.tile_size),
                    height: Val::Px(config.tile_size),
                    position_type: PositionType::Absolute,
                    left: Val::Px(pos.x),
                    top: Val::Px(pos.y),
                    ..default()
                },
                BackgroundColor(color),
            ))
            .id();

        commands.entity(container).add_child(tile);
        map_state.rendered_chunks.insert(coord, tile);
    }
}

// ── Unit tests ──────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn world_map_config_default_has_valid_values() {
        let config = WorldMapConfig::default();
        assert!(config.tile_size > 0.0);
        assert!(config.player_marker_size > 0.0);
        assert!(config.map_width > 0.0);
        assert!(config.map_height > 0.0);
    }

    #[test]
    fn world_map_config_from_ron() {
        let ron_str = r#"(
            tile_size: 16.0,
            player_marker_size: 10.0,
            map_width: 1000.0,
            map_height: 800.0,
            color_deep_space: (0.1, 0.1, 0.25, 1.0),
            color_asteroid_field: (0.5, 0.5, 0.5, 1.0),
            color_wreck_field: (0.7, 0.5, 0.2, 1.0),
            color_player: (1.0, 1.0, 1.0, 1.0),
            color_background: (0.0, 0.0, 0.0, 0.8),
            color_station: (0.2, 0.9, 0.2, 1.0),
        )"#;
        let config = WorldMapConfig::from_ron(ron_str).expect("Should parse WorldMapConfig RON");
        assert!((config.tile_size - 16.0).abs() < f32::EPSILON);
        assert!((config.map_width - 1000.0).abs() < f32::EPSILON);
    }

    #[test]
    fn world_to_map_position_player_at_origin() {
        // Station directly to the right of the player
        let pos = world_to_map_position(
            Vec2::new(500.0, 0.0),
            Vec2::ZERO,
            1000.0, // chunk_size
            Vec2::new(400.0, 300.0),
            12.0,   // tile_size
        );
        // offset = (500, 0), map_offset = (500/1000*12, 0) = (6, 0)
        assert!((pos.x - 406.0).abs() < 0.01, "x should be 406, got {}", pos.x);
        assert!((pos.y - 300.0).abs() < 0.01, "y should be 300, got {}", pos.y);
    }

    #[test]
    fn world_to_map_position_y_flipped() {
        // Station north of player → appears above center (lower y in UI)
        let pos = world_to_map_position(
            Vec2::new(0.0, 500.0),
            Vec2::ZERO,
            1000.0,
            Vec2::new(400.0, 300.0),
            12.0,
        );
        assert!((pos.x - 400.0).abs() < 0.01, "x should be 400, got {}", pos.x);
        assert!((pos.y - 294.0).abs() < 0.01, "y should be 294 (flipped), got {}", pos.y);
    }

    #[test]
    fn world_map_config_from_ron_invalid() {
        let result = WorldMapConfig::from_ron("not valid ron");
        assert!(result.is_err(), "Invalid RON should return error");
    }

    #[test]
    fn chunk_to_map_position_at_player_returns_center() {
        let player = ChunkCoord { x: 5, y: 5 };
        let center = Vec2::new(400.0, 300.0);
        let pos = chunk_to_map_position(player, player, center, 12.0);
        assert!(
            (pos.x - (400.0 - 6.0)).abs() < f32::EPSILON,
            "x should be center - half_tile, got {}",
            pos.x
        );
        assert!(
            (pos.y - (300.0 - 6.0)).abs() < f32::EPSILON,
            "y should be center - half_tile, got {}",
            pos.y
        );
    }

    #[test]
    fn chunk_to_map_position_east_of_player() {
        let player = ChunkCoord { x: 0, y: 0 };
        let east = ChunkCoord { x: 1, y: 0 };
        let center = Vec2::new(400.0, 300.0);
        let tile_size = 12.0;
        let pos = chunk_to_map_position(east, player, center, tile_size);
        // dx = 1, so left = center.x + 1*12 - 6 = 406
        assert!(
            (pos.x - 406.0).abs() < f32::EPSILON,
            "East chunk should be right of center, got {}",
            pos.x
        );
    }

    #[test]
    fn chunk_to_map_position_north_of_player_y_flip() {
        let player = ChunkCoord { x: 0, y: 0 };
        let north = ChunkCoord { x: 0, y: 1 };
        let center = Vec2::new(400.0, 300.0);
        let tile_size = 12.0;
        let pos = chunk_to_map_position(north, player, center, tile_size);
        // dy = 1, y-flip: top = center.y - 1*12 - 6 = 282
        assert!(
            (pos.y - 282.0).abs() < f32::EPSILON,
            "North chunk should be ABOVE center (lower y in UI), got {}",
            pos.y
        );
    }

    #[test]
    fn biome_map_color_returns_expected_colors() {
        let config = WorldMapConfig::default();
        let deep = biome_map_color(BiomeType::DeepSpace, &config);
        let asteroid = biome_map_color(BiomeType::AsteroidField, &config);
        let wreck = biome_map_color(BiomeType::WreckField, &config);

        assert_eq!(deep, Color::srgba(0.08, 0.08, 0.2, 0.9));
        assert_eq!(asteroid, Color::srgba(0.45, 0.45, 0.45, 0.9));
        assert_eq!(wreck, Color::srgba(0.6, 0.4, 0.15, 0.9));
    }

    #[test]
    fn is_tile_visible_in_range() {
        let player = ChunkCoord { x: 0, y: 0 };
        let nearby = ChunkCoord { x: 1, y: 1 };
        assert!(is_tile_visible(nearby, player, 800.0, 600.0, 12.0));
    }

    #[test]
    fn is_tile_visible_far_away() {
        let player = ChunkCoord { x: 0, y: 0 };
        let far = ChunkCoord { x: 1000, y: 1000 };
        assert!(!is_tile_visible(far, player, 800.0, 600.0, 12.0));
    }

    #[test]
    fn is_tile_visible_large_offset_culled() {
        let player = ChunkCoord { x: 0, y: 0 };
        // With tile_size=12, map_width=800: max visible dx ~ 800/12/2 ~ 33
        let barely_out = ChunkCoord { x: 40, y: 0 };
        assert!(!is_tile_visible(barely_out, player, 800.0, 600.0, 12.0));
    }
}
