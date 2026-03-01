pub mod background;
pub mod effects;
pub mod minimap;
pub mod vector_art;
pub mod world_map;

use bevy::prelude::*;

use crate::core::camera::camera_follow_player;
use crate::core::collision::{Collider, Health};
use crate::core::flight::Player;
use crate::core::spawning::{NeedsAsteroidVisual, NeedsDroneVisual, SpawningConfig};
use crate::core::station::NeedsStationVisual;
use crate::core::tutorial::{generate_tutorial_zone, GravityWellBoundary, GravityWellGenerator, TutorialConfig, TutorialStation, TutorialWreck};
use crate::core::weapons::{
    ActiveWeapon, Energy, FireCooldown, NeedsLaserVisual, NeedsProjectileVisual, WeaponConfig,
};
use crate::shared::components::Velocity;
use crate::world::WorldConfig;

use self::effects::{
    apply_screen_shake, blink_invincible, remove_just_damaged_without_material,
    setup_destruction_assets, setup_flash_materials, setup_impact_flash_assets,
    spawn_destruction_effects, spawn_laser_impact_flash, trigger_damage_flash,
    trigger_screen_shake, update_damage_flash, update_destruction_effects, update_impact_flashes,
    ScreenShake,
};
use self::background::{setup_starfield, update_starfield, StarfieldConfig};
use self::minimap::{setup_minimap, update_minimap_blips, MinimapConfig, MinimapState};
use self::world_map::{toggle_world_map, update_world_map, WorldMapConfig, WorldMapOpen, WorldMapState};
use self::vector_art::{
    generate_asteroid_mesh, generate_circle_outline_mesh, generate_drone_mesh, generate_laser_mesh,
    generate_player_mesh, generate_projectile_mesh, generate_tutorial_generator_mesh,
    generate_tutorial_station_mesh, generate_tutorial_wreck_mesh,
};

pub struct RenderingPlugin;

impl Plugin for RenderingPlugin {
    fn build(&self, app: &mut App) {
        // Initialize resources
        app.init_resource::<ScreenShake>();
        app.init_resource::<StarfieldConfig>();
        app.init_resource::<MinimapState>();

        // Load MinimapConfig from RON file with graceful fallback to defaults
        let minimap_config_path = "assets/config/minimap.ron";
        let minimap_config = match std::fs::read_to_string(minimap_config_path) {
            Ok(contents) => match MinimapConfig::from_ron(&contents) {
                Ok(config) => config,
                Err(e) => {
                    warn!("Failed to parse {minimap_config_path}: {e}. Using defaults.");
                    MinimapConfig::default()
                }
            },
            Err(e) => {
                warn!("Failed to read {minimap_config_path}: {e}. Using defaults.");
                MinimapConfig::default()
            }
        };
        app.insert_resource(minimap_config);

        // Load WorldMapConfig from RON file with graceful fallback to defaults
        let world_map_config_path = "assets/config/world_map.ron";
        let world_map_config = match std::fs::read_to_string(world_map_config_path) {
            Ok(contents) => match WorldMapConfig::from_ron(&contents) {
                Ok(config) => config,
                Err(e) => {
                    warn!("Failed to parse {world_map_config_path}: {e}. Using defaults.");
                    WorldMapConfig::default()
                }
            },
            Err(e) => {
                warn!("Failed to read {world_map_config_path}: {e}. Using defaults.");
                WorldMapConfig::default()
            }
        };
        app.insert_resource(world_map_config);
        app.init_resource::<WorldMapOpen>();
        app.init_resource::<WorldMapState>();

        // Startup systems
        app.add_systems(
            Startup,
            (
                setup_player,
                setup_laser_assets,
                setup_projectile_assets,
                setup_asteroid_assets,
                setup_drone_assets,
                setup_station_assets,
                setup_tutorial_station_assets,
                setup_tutorial_wreck_assets,
                setup_tutorial_generator_assets,
                setup_flash_materials,
                setup_destruction_assets,
                setup_impact_flash_assets,
                setup_starfield,
                setup_minimap,
            ),
        );

        // Update systems: visual setup for entities
        app.add_systems(
            Update,
            (
                render_laser_pulses,
                render_spread_projectiles,
                render_asteroids,
                render_drones,
                render_stations,
                render_tutorial_stations,
                render_tutorial_wrecks,
                render_tutorial_generators,
                setup_gravity_well_boundary_visual,
                update_tutorial_station_visual,
                trigger_damage_flash,
            ),
        );

        // Update systems: visual effects + minimap
        app.add_systems(
            Update,
            (
                remove_just_damaged_without_material,
                update_damage_flash,
                trigger_screen_shake.before(spawn_destruction_effects),
                spawn_destruction_effects,
                update_destruction_effects,
                spawn_laser_impact_flash,
                update_impact_flashes,
                update_starfield,
                blink_invincible,
                update_minimap_blips,
                (toggle_world_map, update_world_map).chain(),
            ),
        );

        // PostUpdate systems: camera effects (after camera_follow_player)
        app.add_systems(PostUpdate, apply_screen_shake.after(camera_follow_player));
    }
}

/// Cached mesh and material handles for laser pulses.
#[derive(Resource)]
struct LaserAssets {
    mesh: Handle<Mesh>,
    material: Handle<ColorMaterial>,
}

/// Initialize laser visual assets once at startup.
fn setup_laser_assets(
    mut commands: Commands,
    config: Res<WeaponConfig>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let mesh = meshes.add(generate_laser_mesh(config.laser_range, config.laser_width));
    let material = materials.add(ColorMaterial::from(Color::srgb(0.4, 0.9, 1.0)));
    commands.insert_resource(LaserAssets { mesh, material });
}

/// Attaches cached mesh and material to newly spawned laser pulse entities.
fn render_laser_pulses(
    mut commands: Commands,
    laser_assets: Res<LaserAssets>,
    query: Query<Entity, With<NeedsLaserVisual>>,
) {
    for entity in query.iter() {
        commands
            .entity(entity)
            .insert((
                Mesh2d(laser_assets.mesh.clone()),
                MeshMaterial2d(laser_assets.material.clone()),
            ))
            .remove::<NeedsLaserVisual>();
    }
}

/// Cached mesh and material handles for spread projectiles.
#[derive(Resource)]
struct ProjectileAssets {
    mesh: Handle<Mesh>,
    material: Handle<ColorMaterial>,
}

/// Initialize projectile visual assets once at startup.
fn setup_projectile_assets(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let mesh = meshes.add(generate_projectile_mesh(3.0));
    let material = materials.add(ColorMaterial::from(Color::srgb(1.0, 0.4, 0.2)));
    commands.insert_resource(ProjectileAssets { mesh, material });
}

/// Attaches cached mesh and material to newly spawned spread projectile entities.
fn render_spread_projectiles(
    mut commands: Commands,
    projectile_assets: Res<ProjectileAssets>,
    query: Query<Entity, With<NeedsProjectileVisual>>,
) {
    for entity in query.iter() {
        commands
            .entity(entity)
            .insert((
                Mesh2d(projectile_assets.mesh.clone()),
                MeshMaterial2d(projectile_assets.material.clone()),
            ))
            .remove::<NeedsProjectileVisual>();
    }
}

/// Cached mesh and material handles for asteroids.
#[derive(Resource)]
struct AsteroidAssets {
    mesh: Handle<Mesh>,
    material: Handle<ColorMaterial>,
}

/// Initialize asteroid visual assets once at startup.
fn setup_asteroid_assets(
    mut commands: Commands,
    config: Res<SpawningConfig>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let mesh = meshes.add(generate_asteroid_mesh(config.asteroid_radius));
    let material = materials.add(ColorMaterial::from(Color::srgb(0.6, 0.5, 0.4)));
    commands.insert_resource(AsteroidAssets { mesh, material });
}

/// Cached mesh and material handles for Scout Drones.
#[derive(Resource)]
struct DroneAssets {
    mesh: Handle<Mesh>,
    material: Handle<ColorMaterial>,
}

/// Initialize drone visual assets once at startup.
fn setup_drone_assets(
    mut commands: Commands,
    config: Res<SpawningConfig>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let mesh = meshes.add(generate_drone_mesh(config.drone_radius));
    let material = materials.add(ColorMaterial::from(Color::srgb(0.9, 0.2, 0.2)));
    commands.insert_resource(DroneAssets { mesh, material });
}

/// Attaches cached mesh and material to newly spawned asteroid entities.
fn render_asteroids(
    mut commands: Commands,
    asteroid_assets: Res<AsteroidAssets>,
    query: Query<Entity, With<NeedsAsteroidVisual>>,
) {
    for entity in query.iter() {
        commands
            .entity(entity)
            .insert((
                Mesh2d(asteroid_assets.mesh.clone()),
                MeshMaterial2d(asteroid_assets.material.clone()),
            ))
            .remove::<NeedsAsteroidVisual>();
    }
}

/// Attaches cached mesh and material to newly spawned drone entities.
fn render_drones(
    mut commands: Commands,
    drone_assets: Res<DroneAssets>,
    query: Query<Entity, With<NeedsDroneVisual>>,
) {
    for entity in query.iter() {
        commands
            .entity(entity)
            .insert((
                Mesh2d(drone_assets.mesh.clone()),
                MeshMaterial2d(drone_assets.material.clone()),
            ))
            .remove::<NeedsDroneVisual>();
    }
}

/// Cached mesh and material handles for TutorialStation.
#[derive(Resource)]
pub struct TutorialStationAssets {
    pub mesh: Handle<Mesh>,
    pub material_defective: Handle<ColorMaterial>,
    pub material_repaired: Handle<ColorMaterial>,
}

/// Initialize TutorialStation visual assets once at startup.
fn setup_tutorial_station_assets(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let mesh = meshes.add(generate_tutorial_station_mesh(20.0));
    // Defective: dim teal; Repaired: bright teal
    let material_defective = materials.add(ColorMaterial::from(Color::srgb(0.1, 0.5, 0.5)));
    let material_repaired = materials.add(ColorMaterial::from(Color::srgb(0.2, 0.9, 0.9)));
    commands.insert_resource(TutorialStationAssets {
        mesh,
        material_defective,
        material_repaired,
    });
}

/// Attaches cached mesh and material to newly spawned TutorialStation entities.
pub fn render_tutorial_stations(
    mut commands: Commands,
    assets: Res<TutorialStationAssets>,
    query: Query<(Entity, &TutorialStation), Added<TutorialStation>>,
) {
    for (entity, station) in query.iter() {
        let material = if station.defective {
            assets.material_defective.clone()
        } else {
            assets.material_repaired.clone()
        };
        commands.entity(entity).insert((
            Mesh2d(assets.mesh.clone()),
            MeshMaterial2d(material),
        ));
    }
}

/// Updates TutorialStation material color when `defective` changes.
pub fn update_tutorial_station_visual(
    assets: Res<TutorialStationAssets>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    query: Query<(&TutorialStation, &MeshMaterial2d<ColorMaterial>), Changed<TutorialStation>>,
) {
    for (station, mat_handle) in query.iter() {
        if let Some(material) = materials.get_mut(&mat_handle.0) {
            if station.defective {
                // Dim teal — defective
                material.color = Color::srgb(0.1, 0.5, 0.5);
            } else {
                // Bright teal — repaired after docking
                material.color = Color::srgb(0.2, 0.9, 0.9);
            }
        }
        // If the material is not in the assets map (e.g. cached handle), swap it out.
        // The above direct mutation is sufficient for the same material instance.
        let _ = &assets; // suppress unused warning; assets used for color constants above
    }
}

/// Cached mesh and material handles for TutorialWreck.
#[derive(Resource)]
pub struct TutorialWreckAssets {
    pub mesh: Handle<Mesh>,
    pub material: Handle<ColorMaterial>,
}

/// Initialize TutorialWreck visual assets once at startup.
fn setup_tutorial_wreck_assets(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let mesh = meshes.add(generate_tutorial_wreck_mesh(18.0));
    // Dark grey — derelict wreck appearance
    let material = materials.add(ColorMaterial::from(Color::srgb(0.3, 0.3, 0.3)));
    commands.insert_resource(TutorialWreckAssets { mesh, material });
}

/// Attaches cached mesh and material to newly spawned TutorialWreck entities.
pub fn render_tutorial_wrecks(
    mut commands: Commands,
    assets: Res<TutorialWreckAssets>,
    query: Query<Entity, Added<TutorialWreck>>,
) {
    for entity in query.iter() {
        commands.entity(entity).insert((
            Mesh2d(assets.mesh.clone()),
            MeshMaterial2d(assets.material.clone()),
        ));
    }
}

/// Cached mesh and material handles for GravityWellGenerator.
#[derive(Resource)]
pub struct TutorialGeneratorAssets {
    pub mesh: Handle<Mesh>,
    pub material: Handle<ColorMaterial>,
}

/// Initialize GravityWellGenerator visual assets once at startup.
fn setup_tutorial_generator_assets(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let mesh = meshes.add(generate_tutorial_generator_mesh(25.0));
    // Orange — gravity well / danger indicator
    let material = materials.add(ColorMaterial::from(Color::srgb(1.0, 0.5, 0.1)));
    commands.insert_resource(TutorialGeneratorAssets { mesh, material });
}

/// Attaches cached mesh and material to newly spawned GravityWellGenerator entities.
pub fn render_tutorial_generators(
    mut commands: Commands,
    assets: Res<TutorialGeneratorAssets>,
    query: Query<Entity, Added<GravityWellGenerator>>,
) {
    for entity in query.iter() {
        commands.entity(entity).insert((
            Mesh2d(assets.mesh.clone()),
            MeshMaterial2d(assets.material.clone()),
        ));
    }
}

/// Attaches a circle-outline mesh and semi-transparent orange material to newly spawned
/// `GravityWellBoundary` child entities.  The boundary ring is centered on the parent
/// `GravityWellGenerator` and its radius equals the generator's `safe_radius`.
/// Because `GravityWellBoundary` is a child entity, it is automatically despawned
/// together with the parent `GravityWellGenerator`.
pub fn setup_gravity_well_boundary_visual(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    query: Query<(Entity, &ChildOf), Added<GravityWellBoundary>>,
    generator_query: Query<&GravityWellGenerator>,
) {
    for (entity, child_of) in query.iter() {
        let parent_entity: Entity = child_of.parent();
        let Ok(generator) = generator_query.get(parent_entity) else {
            continue;
        };
        let safe_radius = generator.safe_radius;
        // Stroke width is fixed at 8 units — thin enough to be a hint, thick enough to see.
        let stroke_width = 8.0;
        let mesh = meshes.add(generate_circle_outline_mesh(safe_radius, stroke_width));
        // Semi-transparent orange warning color.
        let material = materials.add(ColorMaterial {
            color: Color::srgba(1.0, 0.5, 0.0, 0.4),
            ..Default::default()
        });
        commands.entity(entity).insert((
            Mesh2d(mesh),
            MeshMaterial2d(material),
        ));
    }
}

/// Cached mesh and material handles for open-world Station entities.
#[derive(Resource)]
struct StationAssets {
    mesh: Handle<Mesh>,
    material: Handle<ColorMaterial>,
}

/// Initialize Station visual assets once at startup.
/// Uses a larger hexagon (radius 35) in a distinct medium-teal color
/// so open-world stations are visually different from TutorialStation.
fn setup_station_assets(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let mesh = meshes.add(generate_tutorial_station_mesh(35.0));
    // Medium-teal: distinct from tutorial station (dim/bright teal)
    let material = materials.add(ColorMaterial::from(Color::srgb(0.2, 0.7, 0.6)));
    commands.insert_resource(StationAssets { mesh, material });
}

/// Attaches cached mesh and material to newly spawned Station entities (NeedsStationVisual).
fn render_stations(
    mut commands: Commands,
    assets: Res<StationAssets>,
    query: Query<Entity, With<NeedsStationVisual>>,
) {
    for entity in query.iter() {
        commands
            .entity(entity)
            .insert((
                Mesh2d(assets.mesh.clone()),
                MeshMaterial2d(assets.material.clone()),
            ))
            .remove::<NeedsStationVisual>();
    }
}

/// Spawn the player entity with lyon-generated mesh, warm bright color.
/// Position is determined by the tutorial zone layout (player_spawn from TutorialConfig + WorldConfig).
fn setup_player(
    mut commands: Commands,
    weapon_config: Res<WeaponConfig>,
    tutorial_config: Res<TutorialConfig>,
    world_config: Res<WorldConfig>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let layout = generate_tutorial_zone(world_config.seed, &tutorial_config);
    let spawn_pos = layout.player_spawn.extend(0.0);

    let mesh = generate_player_mesh(1);
    let mesh_handle = meshes.add(mesh);
    let material_handle = materials.add(ColorMaterial::from(Color::srgb(1.0, 0.85, 0.2)));

    commands.spawn((
        Player,
        Velocity::default(),
        Health {
            current: 100.0,
            max: 100.0,
        },
        Collider { radius: 12.0 },
        FireCooldown::default(),
        Energy {
            current: weapon_config.energy_max,
            max_capacity: weapon_config.energy_max,
        },
        ActiveWeapon::default(),
        Mesh2d(mesh_handle),
        MeshMaterial2d(material_handle),
        Transform::from_translation(spawn_pos),
    ));
}
