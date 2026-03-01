#![deny(clippy::unwrap_used)]

pub mod core;
pub mod dev;
pub mod game_states;
pub mod infrastructure;
pub mod rendering;
pub mod shared;
pub mod social;
pub mod world;

use self::core::CorePlugin;
use self::dev::DevPlugin;
use self::infrastructure::InfrastructurePlugin;
use self::rendering::RenderingPlugin;
use self::social::SocialPlugin;
use self::world::WorldPlugin;

/// Returns all game plugins. Entry point for both main.rs and tests.
pub fn game_plugins() -> (CorePlugin, RenderingPlugin, DevPlugin, WorldPlugin, InfrastructurePlugin, SocialPlugin) {
    (CorePlugin, RenderingPlugin, DevPlugin, WorldPlugin, InfrastructurePlugin, SocialPlugin)
}
