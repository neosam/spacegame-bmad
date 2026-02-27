pub mod core;
pub mod dev;
pub mod game_states;
pub mod rendering;
pub mod shared;

use self::core::CorePlugin;
use self::dev::DevPlugin;
use self::rendering::RenderingPlugin;

/// Returns all game plugins. Entry point for both main.rs and tests.
pub fn game_plugins() -> (CorePlugin, RenderingPlugin, DevPlugin) {
    (CorePlugin, RenderingPlugin, DevPlugin)
}
