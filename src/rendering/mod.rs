pub mod vector_art;

use bevy::prelude::*;

use crate::core::flight::Player;
use crate::core::weapons::{
    ActiveWeapon, Energy, FireCooldown, NeedsLaserVisual, NeedsProjectileVisual, WeaponConfig,
};
use crate::shared::components::Velocity;

use self::vector_art::{generate_laser_mesh, generate_player_mesh, generate_projectile_mesh};

pub struct RenderingPlugin;

impl Plugin for RenderingPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Startup,
            (setup_player, setup_laser_assets, setup_projectile_assets),
        );
        // Render visuals after core systems spawn entities
        app.add_systems(Update, (render_laser_pulses, render_spread_projectiles));
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

/// Spawn the player entity with lyon-generated mesh, warm bright color.
fn setup_player(
    mut commands: Commands,
    config: Res<WeaponConfig>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let mesh = generate_player_mesh(1);
    let mesh_handle = meshes.add(mesh);
    let material_handle = materials.add(ColorMaterial::from(Color::srgb(1.0, 0.85, 0.2)));

    commands.spawn((
        Player,
        Velocity::default(),
        FireCooldown::default(),
        Energy {
            current: config.energy_max,
            max_capacity: config.energy_max,
        },
        ActiveWeapon::default(),
        Mesh2d(mesh_handle),
        MeshMaterial2d(material_handle),
        Transform::default(),
    ));
}
