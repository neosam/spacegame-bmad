pub mod background;
pub mod effects;
pub mod minimap;
pub mod vector_art;
pub mod world_map;

use bevy::prelude::*;

use crate::core::camera::camera_follow_player;
use crate::core::collision::{Collider, Health};
use crate::core::economy::Credits;
use crate::core::flight::Player;
use crate::core::upgrades::InstalledUpgrades;
use crate::shared::components::{MaterialType, NeedsMaterialDropVisual, NeedsShipUpgradeVisual};
use crate::core::spawning::{
    NeedsAsteroidVisual, NeedsDroneVisual, NeedsFighterVisual, NeedsHeavyCruiserVisual,
    NeedsSniperVisual, NeedsTraderVisual, SpawningConfig,
};
use crate::core::station::{Docked, NeedsStationVisual, Station, StationType};
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
    generate_asteroid_mesh, generate_circle_outline_mesh, generate_drone_mesh,
    generate_fighter_mesh, generate_heavy_cruiser_mesh, generate_laser_mesh,
    generate_material_drop_mesh, generate_player_mesh, generate_projectile_mesh,
    generate_sniper_mesh, generate_trader_mesh, generate_tutorial_generator_mesh,
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

        // Credits HUD startup + update
        app.add_systems(Startup, spawn_credits_hud);
        app.add_systems(Update, update_credits_hud);

        // Material drop assets + visual attach
        app.init_resource::<MaterialDropAssets>();
        app.add_systems(Startup, setup_material_drop_assets);
        app.add_systems(Update, attach_material_drop_visual);

        // Startup systems
        app.add_systems(
            Startup,
            (
                setup_player,
                setup_laser_assets,
                setup_projectile_assets,
                setup_asteroid_assets,
                setup_drone_assets,
                setup_fighter_assets,
                setup_heavy_cruiser_assets,
                setup_sniper_assets,
                setup_trader_assets,
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
                render_fighters,
                render_heavy_cruisers,
                render_snipers,
                render_traders,
                render_stations,
                render_tutorial_stations,
                render_tutorial_wrecks,
                render_tutorial_generators,
                setup_gravity_well_boundary_visual,
                update_tutorial_station_visual,
                trigger_damage_flash,
            ),
        );

        // Ship upgrade visual: update player color based on hull upgrade tier
        app.add_systems(Update, update_ship_upgrade_visual);

        // Station Shop UI: spawn on dock, despawn on undock
        app.add_systems(
            Update,
            (spawn_station_ui, despawn_station_ui).chain(),
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

/// Cached mesh and material handles for each open-world Station type.
#[derive(Resource)]
struct StationTypeAssets {
    trading_mesh: Handle<Mesh>,
    trading_mat: Handle<ColorMaterial>,
    repair_mesh: Handle<Mesh>,
    repair_mat: Handle<ColorMaterial>,
    black_market_mesh: Handle<Mesh>,
    black_market_mat: Handle<ColorMaterial>,
}

/// Initialize per-type Station visual assets once at startup.
/// TradingPost: green 40px, RepairStation: blue 35px, BlackMarket: purple 30px.
fn setup_station_assets(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let trading_mesh = meshes.add(generate_tutorial_station_mesh(40.0));
    let trading_mat = materials.add(ColorMaterial::from(Color::srgb(0.0, 1.0, 0.533)));
    let repair_mesh = meshes.add(generate_tutorial_station_mesh(35.0));
    let repair_mat = materials.add(ColorMaterial::from(Color::srgb(0.267, 0.533, 1.0)));
    let black_market_mesh = meshes.add(generate_tutorial_station_mesh(30.0));
    let black_market_mat = materials.add(ColorMaterial::from(Color::srgb(0.667, 0.267, 1.0)));
    commands.insert_resource(StationTypeAssets {
        trading_mesh, trading_mat,
        repair_mesh, repair_mat,
        black_market_mesh, black_market_mat,
    });
}

/// Attaches cached mesh and material to newly spawned Station entities based on their StationType.
fn render_stations(
    mut commands: Commands,
    assets: Res<StationTypeAssets>,
    query: Query<(Entity, &Station), With<NeedsStationVisual>>,
) {
    for (entity, station) in query.iter() {
        let (mesh, mat) = match station.station_type {
            StationType::TradingPost => (assets.trading_mesh.clone(), assets.trading_mat.clone()),
            StationType::RepairStation => (assets.repair_mesh.clone(), assets.repair_mat.clone()),
            StationType::BlackMarket => (assets.black_market_mesh.clone(), assets.black_market_mat.clone()),
        };
        commands
            .entity(entity)
            .insert((Mesh2d(mesh), MeshMaterial2d(mat)))
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

// ── Epic 4: Combat Enemy Rendering ──────────────────────────────────────

/// Cached mesh and material handles for Fighter enemies.
#[derive(Resource)]
struct FighterAssets {
    mesh: Handle<Mesh>,
    material: Handle<ColorMaterial>,
}

fn setup_fighter_assets(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let mesh = meshes.add(generate_fighter_mesh(12.0));
    // Orange — aggressive, fast attacker
    let material = materials.add(ColorMaterial::from(Color::srgb(1.0, 0.4, 0.1)));
    commands.insert_resource(FighterAssets { mesh, material });
}

fn render_fighters(
    mut commands: Commands,
    assets: Res<FighterAssets>,
    query: Query<Entity, With<NeedsFighterVisual>>,
) {
    for entity in query.iter() {
        commands
            .entity(entity)
            .insert((
                Mesh2d(assets.mesh.clone()),
                MeshMaterial2d(assets.material.clone()),
            ))
            .remove::<NeedsFighterVisual>();
    }
}

/// Cached mesh and material handles for Heavy Cruiser enemies.
#[derive(Resource)]
struct HeavyCruiserAssets {
    mesh: Handle<Mesh>,
    material: Handle<ColorMaterial>,
}

fn setup_heavy_cruiser_assets(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let mesh = meshes.add(generate_heavy_cruiser_mesh(22.0));
    // Steel blue-grey — heavy, imposing
    let material = materials.add(ColorMaterial::from(Color::srgb(0.4, 0.5, 0.7)));
    commands.insert_resource(HeavyCruiserAssets { mesh, material });
}

fn render_heavy_cruisers(
    mut commands: Commands,
    assets: Res<HeavyCruiserAssets>,
    query: Query<Entity, With<NeedsHeavyCruiserVisual>>,
) {
    for entity in query.iter() {
        commands
            .entity(entity)
            .insert((
                Mesh2d(assets.mesh.clone()),
                MeshMaterial2d(assets.material.clone()),
            ))
            .remove::<NeedsHeavyCruiserVisual>();
    }
}

/// Cached mesh and material handles for Sniper enemies.
#[derive(Resource)]
struct SniperAssets {
    mesh: Handle<Mesh>,
    material: Handle<ColorMaterial>,
}

fn setup_sniper_assets(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let mesh = meshes.add(generate_sniper_mesh(10.0));
    // Purple — precise, long-range
    let material = materials.add(ColorMaterial::from(Color::srgb(0.7, 0.1, 0.9)));
    commands.insert_resource(SniperAssets { mesh, material });
}

fn render_snipers(
    mut commands: Commands,
    assets: Res<SniperAssets>,
    query: Query<Entity, With<NeedsSniperVisual>>,
) {
    for entity in query.iter() {
        commands
            .entity(entity)
            .insert((
                Mesh2d(assets.mesh.clone()),
                MeshMaterial2d(assets.material.clone()),
            ))
            .remove::<NeedsSniperVisual>();
    }
}

/// Cached mesh and material handles for Trader Ship entities.
#[derive(Resource)]
struct TraderAssets {
    mesh: Handle<Mesh>,
    material: Handle<ColorMaterial>,
}

fn setup_trader_assets(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let mesh = meshes.add(generate_trader_mesh(14.0));
    // Light yellow — neutral, commercial
    let material = materials.add(ColorMaterial::from(Color::srgb(0.9, 0.9, 0.5)));
    commands.insert_resource(TraderAssets { mesh, material });
}

fn render_traders(
    mut commands: Commands,
    assets: Res<TraderAssets>,
    query: Query<Entity, With<NeedsTraderVisual>>,
) {
    for entity in query.iter() {
        commands
            .entity(entity)
            .insert((
                Mesh2d(assets.mesh.clone()),
                MeshMaterial2d(assets.material.clone()),
            ))
            .remove::<NeedsTraderVisual>();
    }
}

// ── Station Shop UI ──────────────────────────────────────────────────────

/// Marker component placed on the root UI entity of the station shop panel.
/// Used for despawning and test queries.
#[derive(Component, Debug)]
pub struct StationUiRoot;

/// Spawns a station shop UI panel when the player gains a `Docked` component.
///
/// Reacts to `Added<Docked>` — fires on the frame the player docks.
/// Looks up the station name from the `Docked.station` entity reference.
pub fn spawn_station_ui(
    player_query: Query<&Docked, (Added<Docked>, With<Player>)>,
    station_query: Query<&Station>,
    mut commands: Commands,
) {
    let Ok(docked) = player_query.single() else {
        return;
    };

    // Look up station name and type in a single query; fall back gracefully if missing
    let (station_name, station_type_label) = station_query
        .get(docked.station)
        .map(|s| (s.name, s.station_type.display_name()))
        .unwrap_or(("Unknown Station", "Unknown Type"));

    // Root panel — full-width strip anchored to the bottom of the screen
    let root = commands
        .spawn((
            StationUiRoot,
            Node {
                width: Val::Percent(100.0),
                height: Val::Px(180.0),
                position_type: PositionType::Absolute,
                bottom: Val::Px(0.0),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                padding: UiRect::all(Val::Px(12.0)),
                row_gap: Val::Px(6.0),
                ..default()
            },
            BackgroundColor(Color::srgba(0.05, 0.05, 0.15, 0.88)),
            GlobalZIndex(50),
        ))
        .id();

    // Station name title
    let title = commands
        .spawn((
            Text(station_name.to_string()),
            TextFont {
                font_size: 22.0,
                ..default()
            },
            TextColor(Color::WHITE),
        ))
        .id();

    // Station type subtitle
    let type_label = commands
        .spawn((
            Text(station_type_label.to_string()),
            TextFont {
                font_size: 16.0,
                ..default()
            },
            TextColor(Color::srgba(0.7, 0.7, 0.7, 1.0)),
        ))
        .id();

    // Shop row (placeholder)
    let shop_row = commands
        .spawn((
            Text("Shop  —  not yet available".to_string()),
            TextFont {
                font_size: 16.0,
                ..default()
            },
            TextColor(Color::srgba(0.7, 0.7, 0.7, 1.0)),
        ))
        .id();

    // Repair row (placeholder)
    let repair_row = commands
        .spawn((
            Text("Repair  —  not yet available".to_string()),
            TextFont {
                font_size: 16.0,
                ..default()
            },
            TextColor(Color::srgba(0.7, 0.7, 0.7, 1.0)),
        ))
        .id();

    // Hint
    let hint = commands
        .spawn((
            Text("Press E to undock".to_string()),
            TextFont {
                font_size: 13.0,
                ..default()
            },
            TextColor(Color::srgba(0.5, 0.5, 0.5, 1.0)),
        ))
        .id();

    commands
        .entity(root)
        .add_children(&[title, type_label, shop_row, repair_row, hint]);
}

/// Despawns all `StationUiRoot` entities when the player's `Docked` component is removed.
pub fn despawn_station_ui(
    mut removed: RemovedComponents<Docked>,
    query: Query<Entity, With<StationUiRoot>>,
    mut commands: Commands,
) {
    if removed.read().next().is_some() {
        for entity in query.iter() {
            commands.entity(entity).despawn();
        }
    }
}

// ── Credits HUD ─────────────────────────────────────────────────────────

/// Marker for the Credits HUD root entity.
#[derive(Component, Debug)]
pub struct CreditsHudRoot;

/// Marker for the Credits HUD text entity (child of `CreditsHudRoot`).
#[derive(Component, Debug)]
pub struct CreditsHudText;

/// Spawns a top-left Credits HUD at startup.
pub fn spawn_credits_hud(mut commands: Commands) {
    let root = commands
        .spawn((
            CreditsHudRoot,
            Node {
                position_type: PositionType::Absolute,
                top: Val::Px(8.0),
                left: Val::Px(8.0),
                ..default()
            },
            GlobalZIndex(10),
        ))
        .id();

    let text = commands
        .spawn((
            CreditsHudText,
            Text("Credits: 0".to_string()),
            TextFont {
                font_size: 18.0,
                ..default()
            },
            TextColor(Color::WHITE),
        ))
        .id();

    commands.entity(root).add_child(text);
}

/// Updates the Credits HUD text each frame to reflect the current balance.
pub fn update_credits_hud(
    credits: Res<Credits>,
    mut text_query: Query<&mut Text, With<CreditsHudText>>,
) {
    for mut text in text_query.iter_mut() {
        *text = Text(format!("Credits: {}", credits.balance));
    }
}

// ── Material Drop Rendering ────────────────────────────────────────────────

/// Cached mesh and material handles for each material drop type.
#[derive(Resource, Default)]
pub struct MaterialDropAssets {
    pub common_scrap_mesh: Handle<Mesh>,
    pub common_scrap_material: Handle<ColorMaterial>,
    pub rare_alloy_mesh: Handle<Mesh>,
    pub rare_alloy_material: Handle<ColorMaterial>,
    pub energy_core_mesh: Handle<Mesh>,
    pub energy_core_material: Handle<ColorMaterial>,
}

/// Creates mesh and material handles for all three material drop types at startup.
pub fn setup_material_drop_assets(
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut assets: ResMut<MaterialDropAssets>,
) {
    let drop_mesh = generate_material_drop_mesh(8.0);

    assets.common_scrap_mesh = meshes.add(drop_mesh.clone());
    assets.common_scrap_material = materials.add(ColorMaterial::from(Color::srgb(0.5, 0.5, 0.5)));

    assets.rare_alloy_mesh = meshes.add(drop_mesh.clone());
    assets.rare_alloy_material = materials.add(ColorMaterial::from(Color::srgb(0.27, 0.53, 1.0)));

    assets.energy_core_mesh = meshes.add(drop_mesh);
    assets.energy_core_material = materials.add(ColorMaterial::from(Color::srgb(1.0, 0.84, 0.0)));
}

/// Attaches the appropriate mesh and material to newly spawned material drop entities.
/// Removes the NeedsMaterialDropVisual marker once visuals are attached.
pub fn attach_material_drop_visual(
    mut commands: Commands,
    query: Query<(Entity, &MaterialType), With<NeedsMaterialDropVisual>>,
    assets: Res<MaterialDropAssets>,
) {
    for (entity, material_type) in query.iter() {
        let (mesh, mat) = match material_type {
            MaterialType::CommonScrap => (&assets.common_scrap_mesh, &assets.common_scrap_material),
            MaterialType::RareAlloy => (&assets.rare_alloy_mesh, &assets.rare_alloy_material),
            MaterialType::EnergyCore => (&assets.energy_core_mesh, &assets.energy_core_material),
        };
        commands
            .entity(entity)
            .insert((Mesh2d(mesh.clone()), MeshMaterial2d(mat.clone())))
            .remove::<NeedsMaterialDropVisual>();
    }
}

// ── Epic 5: Ship Upgrade Visual ──────────────────────────────────────────

/// Updates the player ship's mesh material color based on the HullStrength upgrade tier.
/// Triggered by the NeedsShipUpgradeVisual marker (set by core when InstalledUpgrades changes).
///
/// Color tiers:
/// - Tier 0: Standard gold/yellow
/// - Tier 1–2: Blue tone
/// - Tier 3–4: Gold tone (brighter)
/// - Tier 5: Silver/chrome
fn update_ship_upgrade_visual(
    mut commands: Commands,
    mut materials: ResMut<Assets<ColorMaterial>>,
    query: Query<(Entity, &MeshMaterial2d<ColorMaterial>), (With<Player>, With<NeedsShipUpgradeVisual>)>,
    installed: Res<InstalledUpgrades>,
) {
    use crate::core::upgrades::ShipSystem;
    let hull_tier = installed.ship_tier(ShipSystem::HullStrength);
    let color = match hull_tier {
        0 => Color::srgb(1.0, 0.85, 0.2),   // standard gold/yellow
        1 | 2 => Color::srgb(0.3, 0.6, 1.0), // blue tone
        3 | 4 => Color::srgb(1.0, 0.75, 0.1), // bright gold
        _ => Color::srgb(0.85, 0.92, 1.0),   // silver/chrome (tier 5+)
    };
    for (entity, mat_handle) in query.iter() {
        if let Some(material) = materials.get_mut(&mat_handle.0) {
            material.color = color;
        }
        commands.entity(entity).remove::<NeedsShipUpgradeVisual>();
    }
}
