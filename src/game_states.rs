use bevy::prelude::*;

/// Top-level game state. Sprint 0: only `Playing` is active.
/// Loading, Menu, Paused will be added in later stories.
#[derive(States, Default, Clone, Eq, PartialEq, Debug, Hash)]
pub enum GameState {
    #[default]
    Playing,
}

/// Sub-states within Playing. Sprint 0: only `Flying` is active.
/// Docked, InWormhole, InTutorial added in later epics.
#[derive(SubStates, Default, Clone, Eq, PartialEq, Debug, Hash)]
#[source(GameState = GameState::Playing)]
pub enum PlayingSubState {
    #[default]
    Flying,
    /// Player is inside a wormhole arena. Open-world systems are suspended.
    InWormhole,
}
