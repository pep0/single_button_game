mod components;
mod resources;
mod setup;
mod input;
mod settle;
mod ui;

pub use resources::*;

use bevy::prelude::*;
use crate::state::GameState;

pub struct PlayingPlugin;

impl Plugin for PlayingPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::Playing), setup::setup_playing)
            .add_systems(
                Update,
                (
                    input::slot_oscillation,
                    input::production_input,
                    settle::check_settle,
                    settle::check_failure,
                    ui::camera_follow,
                    ui::update_hud,
                    ui::update_ghost_highlights,
                    ui::animate_score_popups,
                )
                    .run_if(in_state(GameState::Playing)),
            )
            .add_systems(OnExit(GameState::Playing), setup::cleanup_playing);
    }
}
