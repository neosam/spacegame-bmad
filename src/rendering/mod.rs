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
use crate::infrastructure::logbook::Logbook;
use crate::shared::components::{MaterialType, NeedsMaterialDropVisual, NeedsShipUpgradeVisual};
use crate::shared::events::EventSeverity;
use crate::core::spawning::{
    NeedsAsteroidVisual, NeedsBossVisual, NeedsDroneVisual, NeedsFighterVisual,
    NeedsHeavyCruiserVisual, NeedsSniperVisual, NeedsTraderVisual, SpawningConfig,
};
use crate::core::station::{Docked, NeedsStationVisual, Station, StationType};
use crate::core::wormhole::{NeedsWormholeVisual, Wormhole};
use crate::core::tutorial::{generate_tutorial_zone, GravityWellBoundary, GravityWellGenerator, TutorialConfig, TutorialStation, TutorialWreck};
use crate::social::companion::{CompanionData, NeedsCompanionVisual};
use crate::social::companion_personality::{BarkDisplay, PlayerOpinions, format_opinion_score};
use crate::social::enemy_ai::{AttackWarning, BossRetreatBark};
use crate::social::faction::FactionId;
use crate::core::weapons::{
    ActiveWeapon, Energy, FireCooldown, NeedsEnemyProjectileVisual,
    NeedsLaserVisual, NeedsProjectileVisual, WeaponConfig,
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
    generate_asteroid_mesh, generate_boss_mesh, generate_circle_outline_mesh,
    generate_companion_mesh, generate_drone_mesh, generate_fighter_mesh,
    generate_heavy_cruiser_mesh, generate_laser_mesh, generate_material_drop_mesh,
    generate_player_mesh, generate_projectile_mesh, generate_sniper_mesh, generate_trader_mesh,
    generate_tutorial_generator_mesh, generate_tutorial_station_mesh, generate_tutorial_wreck_mesh,
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

        // Vitals HUD (Health + Energy bars)
        app.add_systems(Startup, spawn_vitals_hud);
        app.add_systems(Update, update_vitals_hud);

        // Bark HUD (companion one-liners)
        app.add_systems(Startup, spawn_bark_hud);
        app.add_systems(Update, update_bark_hud);
        // Coords HUD (above minimap, top-right)
        app.add_systems(Startup, spawn_coords_hud);
        app.add_systems(Update, update_coords_hud);
        // Story 7-2: Boss attack warning visual (pulsing color)
        app.add_systems(Update, update_boss_warning_visual);
        // Story 7-5: Boss retreat HUD
        app.add_systems(Startup, spawn_boss_retreat_hud);
        app.add_systems(Update, update_boss_retreat_hud);

        // Epic 8: Logbook UI
        app.init_resource::<LogbookUiOpen>();
        app.init_resource::<LogbookMilestones>();
        app.add_systems(Update, (
            toggle_logbook_ui,
            update_logbook_milestones,
            (spawn_logbook_ui, despawn_logbook_ui).chain(),
        ));

        // Material drop assets + visual attach
        app.init_resource::<MaterialDropAssets>();
        app.add_systems(Startup, setup_material_drop_assets);
        app.add_systems(Update, attach_material_drop_visual);

        // Story 9-1: Wormhole assets + visual attach
        app.add_systems(Startup, setup_wormhole_assets);
        app.add_systems(Update, attach_wormhole_visual);

        // Startup systems
        app.add_systems(
            Startup,
            (
                setup_player,
                setup_laser_assets,
                setup_projectile_assets,
                setup_enemy_projectile_assets,
                setup_asteroid_assets,
                setup_drone_assets,
                setup_fighter_assets,
                setup_heavy_cruiser_assets,
                setup_sniper_assets,
                setup_boss_assets,
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
                render_enemy_projectiles,
                render_asteroids,
                render_drones,
                render_fighters,
                render_heavy_cruisers,
                render_snipers,
                render_bosses,
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

        // Story 6a-4: Companion visuals — setup assets and attach on spawn
        app.add_systems(Startup, setup_companion_assets);
        app.add_systems(Update, render_companions);

        // Station Shop UI: spawn on dock, update while docked, despawn on undock
        app.add_systems(
            Update,
            (spawn_station_ui, update_station_ui, despawn_station_ui).chain(),
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

/// Cached mesh and material handles for enemy projectiles (red).
#[derive(Resource)]
struct EnemyProjectileAssets {
    mesh: Handle<Mesh>,
    material: Handle<ColorMaterial>,
}

fn setup_enemy_projectile_assets(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let mesh = meshes.add(generate_projectile_mesh(4.0));
    let material = materials.add(ColorMaterial::from(Color::srgb(1.0, 0.1, 0.1)));
    commands.insert_resource(EnemyProjectileAssets { mesh, material });
}

fn render_enemy_projectiles(
    mut commands: Commands,
    assets: Res<EnemyProjectileAssets>,
    query: Query<Entity, With<NeedsEnemyProjectileVisual>>,
) {
    for entity in query.iter() {
        commands
            .entity(entity)
            .insert((
                Mesh2d(assets.mesh.clone()),
                MeshMaterial2d(assets.material.clone()),
            ))
            .remove::<NeedsEnemyProjectileVisual>();
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

// ── Story 7-1: Boss Visual ───────────────────────────────────────────────

/// Cached mesh and material handles for Boss enemy entities.
#[derive(Resource)]
struct BossAssets {
    mesh: Handle<Mesh>,
    material: Handle<ColorMaterial>,
}

fn setup_boss_assets(
    mut commands: Commands,
    config: Res<SpawningConfig>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let mesh = meshes.add(generate_boss_mesh(config.boss_collider_radius));
    // Dark red — dangerous, imposing threat
    let material = materials.add(ColorMaterial::from(Color::srgb(0.8, 0.1, 0.1)));
    commands.insert_resource(BossAssets { mesh, material });
}

fn render_bosses(
    mut commands: Commands,
    assets: Res<BossAssets>,
    query: Query<Entity, With<NeedsBossVisual>>,
) {
    for entity in query.iter() {
        commands
            .entity(entity)
            .insert((
                Mesh2d(assets.mesh.clone()),
                MeshMaterial2d(assets.material.clone()),
            ))
            .remove::<NeedsBossVisual>();
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

/// Builds and spawns the station UI panel with full recipe list.
///
/// Called both on initial dock and when the UI needs a refresh (e.g. after crafting).
fn build_station_ui(
    station_name: &str,
    station_type_label: &str,
    credits: &crate::core::economy::Credits,
    inventory: &crate::core::economy::PlayerInventory,
    recipes: &crate::core::upgrades::DiscoveredRecipes,
    installed: &crate::core::upgrades::InstalledUpgrades,
    ui_state: &crate::core::upgrades::StationUiState,
    commands: &mut Commands,
) {
    use crate::shared::components::MaterialType;
    use crate::core::upgrades::{AcquisitionMethod, can_craft};

    let scrap = inventory
        .items
        .get(&MaterialType::CommonScrap)
        .copied()
        .unwrap_or(0);
    let alloy = inventory
        .items
        .get(&MaterialType::RareAlloy)
        .copied()
        .unwrap_or(0);
    let core = inventory
        .items
        .get(&MaterialType::EnergyCore)
        .copied()
        .unwrap_or(0);

    // Root panel — full-width strip anchored to the bottom of the screen
    let root = commands
        .spawn((
            StationUiRoot,
            Node {
                width: Val::Percent(100.0),
                height: Val::Px(220.0),
                position_type: PositionType::Absolute,
                bottom: Val::Px(0.0),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::FlexStart,
                justify_content: JustifyContent::FlexStart,
                padding: UiRect::all(Val::Px(12.0)),
                row_gap: Val::Px(4.0),
                ..default()
            },
            BackgroundColor(Color::srgba(0.05, 0.05, 0.15, 0.88)),
            GlobalZIndex(50),
        ))
        .id();

    // Header row: station name and type
    let header_text = format!("{station_name}  [{station_type_label}]");
    let header = commands
        .spawn((
            Text(header_text),
            TextFont {
                font_size: 20.0,
                ..default()
            },
            TextColor(Color::WHITE),
        ))
        .id();

    // Resource row: credits and inventory
    let resources_text = format!(
        "Credits: {}  |  Scrap: {}  Alloy: {}  Core: {}",
        credits.balance, scrap, alloy, core
    );
    let resources = commands
        .spawn((
            Text(resources_text),
            TextFont {
                font_size: 15.0,
                ..default()
            },
            TextColor(Color::srgba(0.8, 0.9, 0.8, 1.0)),
        ))
        .id();

    // Separator
    let separator = commands
        .spawn((
            Text("─────────────────────────────────────────────".to_string()),
            TextFont {
                font_size: 13.0,
                ..default()
            },
            TextColor(Color::srgba(0.4, 0.4, 0.5, 1.0)),
        ))
        .id();

    // Recipe rows — show up to 5, centered around selected index
    let total = recipes.recipes.len();
    let selected = ui_state.selected_recipe_index.min(total.saturating_sub(1));
    let max_visible = 5usize;
    let start = if total <= max_visible {
        0
    } else {
        let half = max_visible / 2;
        if selected < half {
            0
        } else if selected + (max_visible - half) > total {
            total - max_visible
        } else {
            selected - half
        }
    };
    let end = (start + max_visible).min(total);

    let mut children: Vec<Entity> = vec![header, resources, separator];

    for idx in start..end {
        let recipe = &recipes.recipes[idx];
        let is_selected = idx == selected;
        let craftable = can_craft(recipe, inventory, credits);
        let already_installed = recipe.ship_system
            .map(|s| installed.ship_tier(s) >= recipe.tier)
            .or_else(|| recipe.weapon_system.map(|s| installed.weapon_tier(s) >= recipe.tier))
            .unwrap_or(false);

        let method_label = match recipe.acquisition {
            AcquisitionMethod::CraftOnly => "[CRAFT]",
            AcquisitionMethod::BuyOnly => "[BUY]",
            AcquisitionMethod::CraftOrBuy => "[CRAFT/BUY]",
        };

        let cursor = if is_selected { "►" } else { " " };
        let installed_tag = if already_installed { " ✓" } else { "" };
        let row_text = format!(
            "{}  {:<14} {:>2}s {:>2}a {:>2}c  {:>3}cr  {}{}",
            cursor,
            recipe.display_name,
            recipe.cost_common_scrap,
            recipe.cost_rare_alloy,
            recipe.cost_energy_core,
            recipe.credit_cost,
            method_label,
            installed_tag,
        );

        let row_color = if already_installed {
            Color::srgba(0.5, 0.8, 0.5, 1.0) // green: already owned
        } else if craftable {
            Color::srgba(0.9, 0.9, 0.9, 1.0) // white: affordable
        } else {
            Color::srgba(0.45, 0.45, 0.45, 1.0) // grey: cannot afford
        };

        let row = commands
            .spawn((
                Text(row_text),
                TextFont {
                    font_size: 15.0,
                    ..default()
                },
                TextColor(row_color),
            ))
            .id();
        children.push(row);
    }

    // Bottom separator
    let sep2 = commands
        .spawn((
            Text("─────────────────────────────────────────────".to_string()),
            TextFont {
                font_size: 13.0,
                ..default()
            },
            TextColor(Color::srgba(0.4, 0.4, 0.5, 1.0)),
        ))
        .id();
    children.push(sep2);

    // Hint row
    let hint = commands
        .spawn((
            Text("R/T = navigate  |  F = craft/buy  |  E = undock".to_string()),
            TextFont {
                font_size: 13.0,
                ..default()
            },
            TextColor(Color::srgba(0.5, 0.5, 0.5, 1.0)),
        ))
        .id();
    children.push(hint);

    commands.entity(root).add_children(&children);
}

/// Spawns a station shop UI panel when the player gains a `Docked` component.
///
/// Reacts to `Added<Docked>` — fires on the frame the player docks.
/// Looks up the station name from the `Docked.station` entity reference.
pub fn spawn_station_ui(
    player_query: Query<&Docked, (Added<Docked>, With<Player>)>,
    station_query: Query<&Station>,
    credits: Res<crate::core::economy::Credits>,
    inventory: Res<crate::core::economy::PlayerInventory>,
    recipes: Res<crate::core::upgrades::DiscoveredRecipes>,
    installed: Res<crate::core::upgrades::InstalledUpgrades>,
    ui_state: Res<crate::core::upgrades::StationUiState>,
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

    build_station_ui(
        station_name,
        station_type_label,
        &credits,
        &inventory,
        &recipes,
        &installed,
        &ui_state,
        &mut commands,
    );
}

/// Updates the station UI every frame while docked.
///
/// Despawns the existing StationUiRoot and respawns a fresh one reflecting
/// current credits, inventory, and recipe selection. This is the simplest
/// approach for MVP — cheap enough for a UI that only runs while docked.
pub fn update_station_ui(
    player_query: Query<&Docked, With<Player>>,
    station_query: Query<&Station>,
    credits: Res<crate::core::economy::Credits>,
    inventory: Res<crate::core::economy::PlayerInventory>,
    recipes: Res<crate::core::upgrades::DiscoveredRecipes>,
    installed: Res<crate::core::upgrades::InstalledUpgrades>,
    ui_state: Res<crate::core::upgrades::StationUiState>,
    existing_ui: Query<Entity, With<StationUiRoot>>,
    mut commands: Commands,
) {
    // Only refresh when at least one of the relevant resources changed or selection moved
    if !credits.is_changed()
        && !inventory.is_changed()
        && !recipes.is_changed()
        && !installed.is_changed()
        && !ui_state.is_changed()
    {
        return;
    }

    let Ok(docked) = player_query.single() else {
        return;
    };

    // Despawn old UI
    for entity in existing_ui.iter() {
        commands.entity(entity).despawn();
    }

    let (station_name, station_type_label) = station_query
        .get(docked.station)
        .map(|s| (s.name, s.station_type.display_name()))
        .unwrap_or(("Unknown Station", "Unknown Type"));

    build_station_ui(
        station_name,
        station_type_label,
        &credits,
        &inventory,
        &recipes,
        &installed,
        &ui_state,
        &mut commands,
    );
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

// ── Vitals HUD (Health + Energy) ─────────────────────────────────────────

const VITALS_BAR_WIDTH: f32 = 120.0;
const VITALS_BAR_HEIGHT: f32 = 10.0;

/// Marker for the health bar fill node.
#[derive(Component, Debug)]
pub struct HealthBarFill;

/// Marker for the energy bar fill node.
#[derive(Component, Debug)]
pub struct EnergyBarFill;

/// Marker for the health bar text.
#[derive(Component, Debug)]
pub struct HealthBarText;

/// Marker for the energy bar text.
#[derive(Component, Debug)]
pub struct EnergyBarText;

/// Spawns the bottom-left Health + Energy HUD at startup.
pub fn spawn_vitals_hud(mut commands: Commands) {
    let root = commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                bottom: Val::Px(8.0),
                left: Val::Px(8.0),
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(4.0),
                ..default()
            },
            GlobalZIndex(10),
        ))
        .id();

    // ── Health row ──
    let hp_row = commands
        .spawn(Node {
            flex_direction: FlexDirection::Row,
            align_items: AlignItems::Center,
            column_gap: Val::Px(4.0),
            ..default()
        })
        .id();
    let hp_label = commands
        .spawn((
            Text("HP".to_string()),
            TextFont { font_size: 12.0, ..default() },
            TextColor(Color::srgba(0.8, 0.8, 0.8, 1.0)),
            Node { width: Val::Px(20.0), ..default() },
        ))
        .id();
    let hp_bg = commands
        .spawn((
            Node {
                width: Val::Px(VITALS_BAR_WIDTH),
                height: Val::Px(VITALS_BAR_HEIGHT),
                overflow: Overflow::clip(),
                border_radius: BorderRadius::all(Val::Px(2.0)),
                ..default()
            },
            BackgroundColor(Color::srgba(0.1, 0.1, 0.1, 0.8)),
        ))
        .id();
    let hp_fill = commands
        .spawn((
            HealthBarFill,
            Node {
                width: Val::Px(VITALS_BAR_WIDTH),
                height: Val::Percent(100.0),
                ..default()
            },
            BackgroundColor(Color::srgb(0.85, 0.15, 0.15)),
        ))
        .id();
    commands.entity(hp_bg).add_child(hp_fill);
    let hp_text = commands
        .spawn((
            HealthBarText,
            Text("?/?".to_string()),
            TextFont { font_size: 11.0, ..default() },
            TextColor(Color::srgba(0.9, 0.9, 0.9, 1.0)),
        ))
        .id();
    commands.entity(hp_row).add_children(&[hp_label, hp_bg, hp_text]);

    // ── Energy row ──
    let en_row = commands
        .spawn(Node {
            flex_direction: FlexDirection::Row,
            align_items: AlignItems::Center,
            column_gap: Val::Px(4.0),
            ..default()
        })
        .id();
    let en_label = commands
        .spawn((
            Text("EN".to_string()),
            TextFont { font_size: 12.0, ..default() },
            TextColor(Color::srgba(0.8, 0.8, 0.8, 1.0)),
            Node { width: Val::Px(20.0), ..default() },
        ))
        .id();
    let en_bg = commands
        .spawn((
            Node {
                width: Val::Px(VITALS_BAR_WIDTH),
                height: Val::Px(VITALS_BAR_HEIGHT),
                overflow: Overflow::clip(),
                border_radius: BorderRadius::all(Val::Px(2.0)),
                ..default()
            },
            BackgroundColor(Color::srgba(0.1, 0.1, 0.1, 0.8)),
        ))
        .id();
    let en_fill = commands
        .spawn((
            EnergyBarFill,
            Node {
                width: Val::Px(VITALS_BAR_WIDTH),
                height: Val::Percent(100.0),
                ..default()
            },
            BackgroundColor(Color::srgb(0.1, 0.6, 1.0)),
        ))
        .id();
    commands.entity(en_bg).add_child(en_fill);
    let en_text = commands
        .spawn((
            EnergyBarText,
            Text("?/?".to_string()),
            TextFont { font_size: 11.0, ..default() },
            TextColor(Color::srgba(0.9, 0.9, 0.9, 1.0)),
        ))
        .id();
    commands.entity(en_row).add_children(&[en_label, en_bg, en_text]);

    commands.entity(root).add_children(&[hp_row, en_row]);
}

/// Updates health and energy bars every frame from the player entity.
pub fn update_vitals_hud(
    player_query: Query<(&Health, &Energy), With<crate::core::flight::Player>>,
    mut hp_fill: Query<&mut Node, (With<HealthBarFill>, Without<EnergyBarFill>)>,
    mut en_fill: Query<&mut Node, (With<EnergyBarFill>, Without<HealthBarFill>)>,
    mut hp_text: Query<&mut Text, (With<HealthBarText>, Without<EnergyBarText>)>,
    mut en_text: Query<&mut Text, (With<EnergyBarText>, Without<HealthBarText>)>,
) {
    let Ok((health, energy)) = player_query.single() else {
        return;
    };

    let hp_ratio = if health.max > 0.0 { (health.current / health.max).clamp(0.0, 1.0) } else { 0.0 };
    let en_ratio = if energy.max_capacity > 0.0 { (energy.current / energy.max_capacity).clamp(0.0, 1.0) } else { 0.0 };

    if let Ok(mut node) = hp_fill.single_mut() {
        node.width = Val::Px(VITALS_BAR_WIDTH * hp_ratio);
    }
    if let Ok(mut node) = en_fill.single_mut() {
        node.width = Val::Px(VITALS_BAR_WIDTH * en_ratio);
    }
    if let Ok(mut text) = hp_text.single_mut() {
        *text = Text(format!("{}/{}", health.current as i32, health.max as i32));
    }
    if let Ok(mut text) = en_text.single_mut() {
        *text = Text(format!("{}/{}", energy.current as i32, energy.max_capacity as i32));
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

// ── Story 6a-4: Companion Visuals ─────────────────────────────────────────

/// Cached mesh and per-faction color material handles for companion ships.
#[derive(Resource)]
struct CompanionAssets {
    mesh: Handle<Mesh>,
    material_neutral: Handle<ColorMaterial>,
    material_pirates: Handle<ColorMaterial>,
    material_military: Handle<ColorMaterial>,
    material_aliens: Handle<ColorMaterial>,
    material_rogue_drones: Handle<ColorMaterial>,
}

/// Initialize companion visual assets once at startup.
fn setup_companion_assets(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let mesh = meshes.add(generate_companion_mesh());
    commands.insert_resource(CompanionAssets {
        mesh,
        // Neutral: cyan
        material_neutral: materials.add(ColorMaterial::from(Color::srgb(0.3, 0.9, 0.9))),
        // Pirates: red-orange
        material_pirates: materials.add(ColorMaterial::from(Color::srgb(0.9, 0.3, 0.1))),
        // Military: blue-green
        material_military: materials.add(ColorMaterial::from(Color::srgb(0.2, 0.7, 0.4))),
        // Aliens: purple
        material_aliens: materials.add(ColorMaterial::from(Color::srgb(0.7, 0.2, 0.9))),
        // Rogue Drones: orange-yellow
        material_rogue_drones: materials.add(ColorMaterial::from(Color::srgb(0.9, 0.65, 0.1))),
    });
}

/// Attaches companion mesh and faction-specific material to newly spawned companion entities.
fn render_companions(
    mut commands: Commands,
    assets: Res<CompanionAssets>,
    query: Query<(Entity, &CompanionData), With<NeedsCompanionVisual>>,
) {
    for (entity, companion_data) in query.iter() {
        let material = match companion_data.faction {
            FactionId::Neutral => assets.material_neutral.clone(),
            FactionId::Pirates => assets.material_pirates.clone(),
            FactionId::Military => assets.material_military.clone(),
            FactionId::Aliens => assets.material_aliens.clone(),
            FactionId::RogueDrones => assets.material_rogue_drones.clone(),
        };
        commands
            .entity(entity)
            .insert((
                Mesh2d(assets.mesh.clone()),
                MeshMaterial2d(material),
            ))
            .remove::<NeedsCompanionVisual>();
    }
}

// ── Bark HUD (Epic 6b-1) ─────────────────────────────────────────────────

/// Marker for the bark HUD text node.
#[derive(Component, Debug)]
pub struct BarkHudMarker;

/// Spawns the bark HUD: a centered text node at the top of the screen.
pub fn spawn_bark_hud(mut commands: Commands) {
    commands.spawn((
        BarkHudMarker,
        Text::new(""),
        TextFont {
            font_size: 16.0,
            ..default()
        },
        TextColor(Color::srgba(1.0, 0.95, 0.7, 1.0)),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(8.0),
            left: Val::Percent(50.0),
            ..default()
        },
        Visibility::Hidden,
    ));
}

/// Updates bark HUD visibility and text from the `BarkDisplay` resource.
/// If `PlayerOpinions` is available, appends the opinion score: `Wing-1 (+12): "Target down!"`
pub fn update_bark_hud(
    bark_display: Res<BarkDisplay>,
    opinions: Option<Res<PlayerOpinions>>,
    mut query: Query<(&mut Text, &mut Visibility), With<BarkHudMarker>>,
) {
    let Ok((mut text, mut visibility)) = query.single_mut() else {
        return;
    };
    match &bark_display.current {
        Some((name, bark)) => {
            // Look up opinion score by matching companion name via entity — we display
            // the first opinion score available as a proxy for the speaking companion.
            let score_str = opinions
                .as_ref()
                .and_then(|o| o.scores.values().next().copied())
                .map(format_opinion_score)
                .map(|s| format!(" {s}"))
                .unwrap_or_default();
            text.0 = format!("{name}{score_str}: \"{bark}\"");
            *visibility = Visibility::Visible;
        }
        None => {
            *visibility = Visibility::Hidden;
        }
    }
}

// ── Coords HUD ───────────────────────────────────────────────────────────

#[derive(Component)]
struct CoordsHudMarker;

pub fn spawn_coords_hud(mut commands: Commands) {
    commands.spawn((
        CoordsHudMarker,
        Text::new(""),
        TextFont { font_size: 12.0, ..default() },
        TextColor(Color::srgba(0.7, 0.9, 1.0, 0.85)),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(8.0),
            right: Val::Px(20.0),
            ..default()
        },
    ));
}

pub fn update_coords_hud(
    player_query: Query<&Transform, With<Player>>,
    mut query: Query<&mut Text, With<CoordsHudMarker>>,
) {
    let Ok(transform) = player_query.single() else { return };
    let Ok(mut text) = query.single_mut() else { return };
    let x = transform.translation.x;
    let y = transform.translation.y;
    let chunk_x = (x / 1000.0).floor() as i32;
    let chunk_y = (y / 1000.0).floor() as i32;
    text.0 = format!("Chunk ({chunk_x}, {chunk_y})\n({x:.0}, {y:.0})");
}

// ── Story 7-2: Boss Attack Warning Visual ────────────────────────────────

/// Pulses boss mesh color while `AttackWarning` is active.
/// Changes to bright warning orange (pulsing). Resets to dark red when no warning.
/// Uses the shared BossAssets material — acceptable for typical single-boss-warning scenarios.
pub fn update_boss_warning_visual(
    boss_assets: Res<BossAssets>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    warning_query: Query<&AttackWarning, With<crate::core::spawning::BossEnemy>>,
    time: Res<Time>,
) {
    let Some(mat) = materials.get_mut(&boss_assets.material) else {
        return;
    };
    // Check if any boss has an active warning
    let mut active_warning: Option<f32> = None;
    for warning in warning_query.iter() {
        active_warning = Some(warning.timer);
        break; // Use first active warning
    }
    if let Some(timer) = active_warning {
        // Pulse: brightness oscillates with period based on timer
        use std::f32::consts::TAU;
        let brightness = (timer * TAU).sin().abs();
        // Pulse between dark red (0.8, 0.1, 0.1) and bright orange (1.0, 0.6, 0.0)
        mat.color = Color::srgb(
            0.8 + 0.2 * brightness,
            0.1 + 0.5 * brightness,
            0.1 * (1.0 - brightness),
        );
    } else {
        // Reset to original dark red — no active warnings
        mat.color = Color::srgb(0.8, 0.1, 0.1);
    }
    let _ = time; // used implicitly via timer tick in update_boss_telegraphing
}

// ── Epic 8: Logbook UI ───────────────────────────────────────────────────

/// Whether the logbook overlay is currently open.
#[derive(Resource, Default, Debug)]
pub struct LogbookUiOpen(pub bool);

/// Milestone flags tracked for chapter headings.
#[derive(Resource, Default, Debug)]
pub struct LogbookMilestones {
    pub tutorial_complete: bool,
    pub first_boss_destroyed: bool,
    pub first_companion_recruited: bool,
    pub first_station_docked: bool,
}

/// Marker for the logbook UI root entity.
#[derive(Component, Debug)]
pub struct LogbookUiRoot;

/// Marker for the logbook UI text content node (rebuilt on each refresh).
#[derive(Component, Debug)]
pub struct LogbookContentNode;

/// Toggles the logbook on L key press.
pub fn toggle_logbook_ui(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut logbook_open: ResMut<LogbookUiOpen>,
) {
    if keyboard.just_pressed(KeyCode::KeyL) {
        logbook_open.0 = !logbook_open.0;
    }
}

/// Reads GameEvents to track milestones (Story 8-3).
pub fn update_logbook_milestones(
    mut milestones: ResMut<LogbookMilestones>,
    logbook: Res<Logbook>,
) {
    use crate::shared::events::GameEventKind;
    for entry in logbook.entries().iter() {
        match &entry.kind {
            GameEventKind::TutorialComplete => milestones.tutorial_complete = true,
            GameEventKind::BossDestroyed { .. } => milestones.first_boss_destroyed = true,
            GameEventKind::CompanionRecruited { .. } => milestones.first_companion_recruited = true,
            GameEventKind::StationDocked => milestones.first_station_docked = true,
            _ => {}
        }
    }
}

/// Returns a chapter heading label for a milestone event, if applicable.
fn milestone_heading_for(entry_idx: usize, logbook: &Logbook) -> Option<&'static str> {
    use crate::shared::events::GameEventKind;
    let entries: Vec<_> = logbook.entries().iter().collect();
    let Some(entry) = entries.get(entry_idx) else {
        return None;
    };
    match &entry.kind {
        GameEventKind::TutorialComplete => Some("— Tutorial abgeschlossen —"),
        GameEventKind::BossDestroyed { .. } => {
            // Only on the first BossDestroyed entry
            let is_first = entries[..entry_idx]
                .iter()
                .all(|e| !matches!(&e.kind, GameEventKind::BossDestroyed { .. }));
            if is_first { Some("— Erster Boss besiegt —") } else { None }
        }
        GameEventKind::CompanionRecruited { .. } => {
            let is_first = entries[..entry_idx]
                .iter()
                .all(|e| !matches!(&e.kind, GameEventKind::CompanionRecruited { .. }));
            if is_first { Some("— Erster Begleiter rekrutiert —") } else { None }
        }
        GameEventKind::StationDocked => {
            let is_first = entries[..entry_idx]
                .iter()
                .all(|e| !matches!(&e.kind, GameEventKind::StationDocked));
            if is_first { Some("— Erste Station angedockt —") } else { None }
        }
        _ => None,
    }
}

/// Spawns the logbook UI overlay when `LogbookUiOpen` becomes true.
pub fn spawn_logbook_ui(
    logbook_open: Res<LogbookUiOpen>,
    logbook: Res<Logbook>,
    existing: Query<Entity, With<LogbookUiRoot>>,
    mut commands: Commands,
) {
    if !logbook_open.is_changed() && !logbook.is_changed() {
        return;
    }
    // Despawn existing UI before rebuilding
    for entity in existing.iter() {
        commands.entity(entity).despawn();
    }
    if !logbook_open.0 {
        return;
    }

    let header_text = "LOGBUCH  [L] Schließen".to_string();

    // Show last 30 entries (Logbook already contains only Tier1+Tier2)
    let all_entries: Vec<_> = logbook.entries().iter().enumerate().collect();
    let visible: Vec<_> = all_entries
        .iter()
        .rev()
        .take(30)
        .collect::<Vec<_>>()
        .into_iter()
        .rev()
        .cloned()
        .collect();

    // Root panel — centered modal
    let root = commands
        .spawn((
            LogbookUiRoot,
            Node {
                width: Val::Percent(80.0),
                height: Val::Percent(70.0),
                position_type: PositionType::Absolute,
                top: Val::Percent(15.0),
                left: Val::Percent(10.0),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::FlexStart,
                padding: UiRect::all(Val::Px(16.0)),
                row_gap: Val::Px(4.0),
                overflow: Overflow::clip(),
                ..default()
            },
            BackgroundColor(Color::srgba(0.04, 0.04, 0.12, 0.93)),
            GlobalZIndex(100),
        ))
        .id();

    // Header
    let header = commands
        .spawn((
            LogbookContentNode,
            Text(header_text),
            TextFont { font_size: 17.0, ..default() },
            TextColor(Color::srgb(0.5, 0.8, 1.0)),
        ))
        .id();
    commands.entity(root).add_child(header);

    // Separator
    let sep = commands
        .spawn((
            LogbookContentNode,
            Text("─────────────────────────────────────────────────────".to_string()),
            TextFont { font_size: 12.0, ..default() },
            TextColor(Color::srgba(0.3, 0.3, 0.5, 1.0)),
        ))
        .id();
    commands.entity(root).add_child(sep);

    if visible.is_empty() {
        let empty = commands
            .spawn((
                LogbookContentNode,
                Text("(Keine Einträge)".to_string()),
                TextFont { font_size: 14.0, ..default() },
                TextColor(Color::srgba(0.5, 0.5, 0.5, 1.0)),
            ))
            .id();
        commands.entity(root).add_child(empty);
    } else {
        for (orig_idx, entry) in &visible {
            // Chapter heading if applicable (Story 8-3)
            if let Some(heading) = milestone_heading_for(*orig_idx, &logbook) {
                let heading_node = commands
                    .spawn((
                        LogbookContentNode,
                        Text(heading.to_string()),
                        TextFont { font_size: 13.0, ..default() },
                        TextColor(Color::srgb(1.0, 0.85, 0.3)),
                    ))
                    .id();
                commands.entity(root).add_child(heading_node);
            }

            let (color, severity_icon) = match entry.severity {
                EventSeverity::Tier1 => (Color::srgb(1.0, 0.95, 0.95), "★★★"),
                EventSeverity::Tier2 => (Color::srgb(1.0, 0.9, 0.6), "★★ "),
                EventSeverity::Tier3 => (Color::srgba(0.65, 0.65, 0.65, 1.0), "★  "),
            };
            let line = format!("[{:.1}s] {} {}", entry.game_time, severity_icon, entry.kind_label);
            let row = commands
                .spawn((
                    LogbookContentNode,
                    Text(line),
                    TextFont { font_size: 13.0, ..default() },
                    TextColor(color),
                ))
                .id();
            commands.entity(root).add_child(row);
        }
    }
}

/// Despawns logbook UI when closed.
pub fn despawn_logbook_ui(
    logbook_open: Res<LogbookUiOpen>,
    existing: Query<Entity, With<LogbookUiRoot>>,
    mut commands: Commands,
) {
    if logbook_open.is_changed() && !logbook_open.0 {
        for entity in existing.iter() {
            commands.entity(entity).despawn();
        }
    }
}

// ── Story 7-5: Boss Retreat HUD ──────────────────────────────────────────

/// Marker for the boss retreat HUD text node.
#[derive(Component, Debug)]
pub struct BossRetreatHudMarker;

/// Spawns the boss retreat HUD: centered warning text shown briefly when boss flees.
pub fn spawn_boss_retreat_hud(mut commands: Commands) {
    commands.spawn((
        BossRetreatHudMarker,
        Text::new(""),
        TextFont {
            font_size: 22.0,
            ..default()
        },
        TextColor(Color::srgb(1.0, 0.3, 0.1)),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(40.0),
            left: Val::Percent(35.0),
            ..default()
        },
        Visibility::Hidden,
    ));
}

/// Shows "BOSS RETREATING" for the duration of BossRetreatBark.timer.
pub fn update_boss_retreat_hud(
    bark: Res<BossRetreatBark>,
    mut query: Query<(&mut Text, &mut Visibility), With<BossRetreatHudMarker>>,
) {
    let Ok((mut text, mut visibility)) = query.single_mut() else {
        return;
    };
    if bark.timer > 0.0 {
        text.0 = "BOSS RETREATING".to_string();
        *visibility = Visibility::Visible;
    } else {
        *visibility = Visibility::Hidden;
    }
}

// ── Story 9-1: Wormhole Visuals ──────────────────────────────────────────

/// Cached mesh and material handles for wormhole entities.
#[derive(Resource)]
pub struct WormholeAssets {
    pub mesh: Handle<Mesh>,
    pub material_active: Handle<ColorMaterial>,
    pub material_cleared: Handle<ColorMaterial>,
}

/// Initialize wormhole visual assets once at startup.
fn setup_wormhole_assets(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let mesh = meshes.add(Circle::new(40.0));
    let material_active = materials.add(ColorMaterial::from(Color::srgba(0.0, 0.8, 1.0, 0.7)));
    let material_cleared = materials.add(ColorMaterial::from(Color::srgba(0.3, 0.3, 0.3, 0.4)));
    commands.insert_resource(WormholeAssets {
        mesh,
        material_active,
        material_cleared,
    });
}

/// Attaches cached mesh and material to newly spawned wormhole entities.
fn attach_wormhole_visual(
    query: Query<(Entity, &Wormhole), With<NeedsWormholeVisual>>,
    mut commands: Commands,
    assets: Res<WormholeAssets>,
) {
    for (entity, wormhole) in query.iter() {
        let material = if wormhole.cleared {
            assets.material_cleared.clone()
        } else {
            assets.material_active.clone()
        };
        commands
            .entity(entity)
            .insert((
                Mesh2d(assets.mesh.clone()),
                MeshMaterial2d(material),
            ))
            .remove::<NeedsWormholeVisual>();
    }
}
