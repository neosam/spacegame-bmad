use bevy::prelude::*;

// ── Types ──────────────────────────────────────────────────────────

/// Configuration for nebula background clouds.
#[derive(Resource)]
pub struct NebulaConfig {
    pub enabled: bool,
    pub base_alpha: f32,
}

impl Default for NebulaConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            base_alpha: 0.05,
        }
    }
}

/// Configuration for a single star parallax layer.
#[derive(Clone)]
pub struct StarLayerConfig {
    pub parallax_factor: f32,
    pub star_radius: f32,
    pub brightness: f32,
    pub cell_size: f32,
    pub stars_per_cell: u32,
    pub z_depth: f32,
}

/// Configuration for the entire starfield (all layers).
#[derive(Resource)]
pub struct StarfieldConfig {
    pub layers: Vec<StarLayerConfig>,
}

impl Default for StarfieldConfig {
    fn default() -> Self {
        Self {
            layers: vec![
                StarLayerConfig {
                    parallax_factor: 0.02,
                    star_radius: 0.8,
                    brightness: 0.15,
                    cell_size: 600.0,
                    stars_per_cell: 3,
                    z_depth: -12.0,
                },
                StarLayerConfig {
                    parallax_factor: 0.1,
                    star_radius: 1.2,
                    brightness: 0.25,
                    cell_size: 400.0,
                    stars_per_cell: 2,
                    z_depth: -11.0,
                },
                StarLayerConfig {
                    parallax_factor: 0.35,
                    star_radius: 1.8,
                    brightness: 0.45,
                    cell_size: 300.0,
                    stars_per_cell: 2,
                    z_depth: -10.0,
                },
                // Layer 4: very distant, very dim micro-stars for extra depth
                StarLayerConfig {
                    parallax_factor: 0.005,
                    star_radius: 0.3,
                    brightness: 0.08,
                    cell_size: 800.0,
                    stars_per_cell: 4,
                    z_depth: -13.0,
                },
            ],
        }
    }
}

/// Marker component identifying a star entity and its layer.
#[derive(Component)]
pub struct Star {
    pub layer_index: usize,
}

/// Cached mesh and material handles per layer, initialized at startup.
#[derive(Resource)]
pub struct StarAssets {
    pub meshes: Vec<Handle<Mesh>>,
    pub materials: Vec<Handle<ColorMaterial>>,
}

// ── Hash functions ─────────────────────────────────────────────────

/// Deterministic hash for star generation from grid cell coordinates.
/// Returns a pseudo-random u64 for the given cell, layer, and star index.
pub fn cell_hash(cx: i32, cy: i32, layer: usize, star_index: u32) -> u64 {
    let mut h = (cx as u64).wrapping_mul(2654435761);
    h = h.wrapping_add(cy as u64).wrapping_mul(40503);
    h = h.wrapping_add(layer as u64).wrapping_mul(12289);
    h = h.wrapping_add(star_index as u64).wrapping_mul(7919);
    h ^ (h >> 16)
}

/// Returns deterministic star positions in layer space for a given grid cell.
pub fn stars_in_cell(cx: i32, cy: i32, layer: usize, config: &StarLayerConfig) -> Vec<Vec2> {
    (0..config.stars_per_cell)
        .map(|i| {
            let h = cell_hash(cx, cy, layer, i);
            let fx = (h & 0xFFFF) as f32 / 65535.0;
            let fy = ((h >> 16) & 0xFFFF) as f32 / 65535.0;
            Vec2::new(
                cx as f32 * config.cell_size + fx * config.cell_size,
                cy as f32 * config.cell_size + fy * config.cell_size,
            )
        })
        .collect()
}

// ── Systems ────────────────────────────────────────────────────────

/// Viewport half-size assumption (half of 1280x720).
const VIEWPORT_HALF: Vec2 = Vec2::new(640.0, 360.0);

/// Spawns star entity pools and creates cached assets at startup.
pub fn setup_starfield(
    mut commands: Commands,
    config: Res<StarfieldConfig>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let mut star_meshes = Vec::new();
    let mut star_materials = Vec::new();

    for layer_config in config.layers.iter() {
        let mesh = meshes.add(Circle::new(layer_config.star_radius));
        let material = materials.add(ColorMaterial::from(Color::srgba(
            1.0,
            1.0,
            1.0,
            layer_config.brightness,
        )));
        star_meshes.push(mesh);
        star_materials.push(material);
    }

    // Spawn entity pools per layer
    for (layer_index, layer_config) in config.layers.iter().enumerate() {
        let cells_x = (1280.0_f32 / layer_config.cell_size).ceil() as u32 + 3;
        let cells_y = (720.0_f32 / layer_config.cell_size).ceil() as u32 + 3;
        let pool_size = cells_x * cells_y * layer_config.stars_per_cell;

        for _ in 0..pool_size {
            commands.spawn((
                Star { layer_index },
                Mesh2d(star_meshes[layer_index].clone()),
                MeshMaterial2d(star_materials[layer_index].clone()),
                Transform::from_xyz(0.0, 0.0, layer_config.z_depth),
                Visibility::Hidden,
            ));
        }
    }

    commands.insert_resource(StarAssets {
        meshes: star_meshes,
        materials: star_materials,
    });
}

/// Repositions star entities each frame based on camera position and parallax offset.
pub fn update_starfield(
    config: Res<StarfieldConfig>,
    camera_query: Query<&Transform, With<Camera2d>>,
    mut star_query: Query<(&Star, &mut Transform, &mut Visibility), Without<Camera2d>>,
) {
    let Ok(camera_transform) = camera_query.single() else {
        return;
    };

    let cam_pos = Vec2::new(
        camera_transform.translation.x,
        camera_transform.translation.y,
    );

    for (layer_index, layer_config) in config.layers.iter().enumerate() {
        let effective_cam = cam_pos * layer_config.parallax_factor;

        // Calculate visible cell range in layer space
        let min_cell_x =
            ((effective_cam.x - VIEWPORT_HALF.x) / layer_config.cell_size).floor() as i32 - 1;
        let max_cell_x =
            ((effective_cam.x + VIEWPORT_HALF.x) / layer_config.cell_size).ceil() as i32 + 1;
        let min_cell_y =
            ((effective_cam.y - VIEWPORT_HALF.y) / layer_config.cell_size).floor() as i32 - 1;
        let max_cell_y =
            ((effective_cam.y + VIEWPORT_HALF.y) / layer_config.cell_size).ceil() as i32 + 1;

        // Generate all visible star world positions for this layer
        let mut star_positions: Vec<Vec2> = Vec::new();
        for cx in min_cell_x..=max_cell_x {
            for cy in min_cell_y..=max_cell_y {
                for layer_pos in stars_in_cell(cx, cy, layer_index, layer_config) {
                    let world_pos = layer_pos + cam_pos * (1.0 - layer_config.parallax_factor);
                    star_positions.push(world_pos);
                }
            }
        }

        // Assign positions to pooled star entities for this layer
        let mut pos_iter = star_positions.into_iter();
        for (star, mut transform, mut visibility) in star_query.iter_mut() {
            if star.layer_index != layer_index {
                continue;
            }

            if let Some(world_pos) = pos_iter.next() {
                transform.translation.x = world_pos.x;
                transform.translation.y = world_pos.y;
                *visibility = Visibility::Visible;
            } else {
                *visibility = Visibility::Hidden;
            }
        }
    }
}

/// Spawns large semi-transparent nebula cloud entities at static world positions.
/// These act as a decorative deep background layer (z: -13.5).
pub fn setup_nebula_background(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    config: Res<NebulaConfig>,
) {
    if !config.enabled {
        return;
    }

    let base_alpha = config.base_alpha;

    // Each tuple: (position, radius, color)
    let nebulae: &[(Vec3, f32, Color)] = &[
        // 2x deep-blue/purple
        (
            Vec3::new(-2000.0, 1500.0, -13.5),
            900.0,
            Color::srgba(0.1, 0.0, 0.3, base_alpha),
        ),
        (
            Vec3::new(1200.0, -2800.0, -13.5),
            700.0,
            Color::srgba(0.1, 0.0, 0.3, base_alpha),
        ),
        // 2x orange-red
        (
            Vec3::new(3000.0, -800.0, -13.5),
            1200.0,
            Color::srgba(0.4, 0.1, 0.0, base_alpha * 0.8),
        ),
        (
            Vec3::new(-3500.0, -1200.0, -13.5),
            800.0,
            Color::srgba(0.4, 0.1, 0.0, base_alpha * 0.8),
        ),
        // 1x cyan-green
        (
            Vec3::new(500.0, 2500.0, -13.5),
            400.0,
            Color::srgba(0.0, 0.2, 0.2, base_alpha * 0.6),
        ),
    ];

    for (position, radius, color) in nebulae {
        let mesh = meshes.add(Circle::new(*radius));
        let material = materials.add(ColorMaterial::from(*color));
        commands.spawn((
            Mesh2d(mesh),
            MeshMaterial2d(material),
            Transform::from_translation(*position),
        ));
    }
}

// ── Unit tests ─────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cell_hash_is_deterministic() {
        let h1 = cell_hash(5, 10, 0, 0);
        let h2 = cell_hash(5, 10, 0, 0);
        assert_eq!(h1, h2, "Same inputs should produce same hash");
    }

    #[test]
    fn cell_hash_varies_with_inputs() {
        let h1 = cell_hash(0, 0, 0, 0);
        let h2 = cell_hash(1, 0, 0, 0);
        let h3 = cell_hash(0, 1, 0, 0);
        assert_ne!(h1, h2, "Different cell_x should produce different hash");
        assert_ne!(h1, h3, "Different cell_y should produce different hash");
    }

    #[test]
    fn cell_hash_varies_with_layer() {
        let h1 = cell_hash(5, 10, 0, 0);
        let h2 = cell_hash(5, 10, 1, 0);
        assert_ne!(h1, h2, "Different layers should produce different hash");
    }

    #[test]
    fn stars_in_cell_positions_within_bounds() {
        let config = StarLayerConfig {
            parallax_factor: 0.1,
            star_radius: 1.0,
            brightness: 0.2,
            cell_size: 400.0,
            stars_per_cell: 3,
            z_depth: -10.0,
        };
        let cx = 5;
        let cy = -3;
        let positions = stars_in_cell(cx, cy, 0, &config);

        let cell_min_x = cx as f32 * config.cell_size;
        let cell_min_y = cy as f32 * config.cell_size;
        let cell_max_x = cell_min_x + config.cell_size;
        let cell_max_y = cell_min_y + config.cell_size;

        for (i, pos) in positions.iter().enumerate() {
            assert!(
                pos.x >= cell_min_x && pos.x <= cell_max_x,
                "Star {i} x={} should be within [{cell_min_x}, {cell_max_x}]",
                pos.x
            );
            assert!(
                pos.y >= cell_min_y && pos.y <= cell_max_y,
                "Star {i} y={} should be within [{cell_min_y}, {cell_max_y}]",
                pos.y
            );
        }
    }

    #[test]
    fn stars_in_cell_count_matches_config() {
        let config = StarLayerConfig {
            parallax_factor: 0.1,
            star_radius: 1.0,
            brightness: 0.2,
            cell_size: 400.0,
            stars_per_cell: 5,
            z_depth: -10.0,
        };
        let positions = stars_in_cell(0, 0, 0, &config);
        assert_eq!(
            positions.len(),
            config.stars_per_cell as usize,
            "Should return exactly stars_per_cell positions"
        );
    }
}
