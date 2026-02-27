mod audio;
mod components;
mod resources;
mod setup;
mod input;
mod settle;
mod ui;

pub use components::FrozenTowerBlock;
pub use resources::*;

use bevy::prelude::*;
use crate::state::GameState;

pub struct PlayingPlugin;

impl Plugin for PlayingPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<audio::BlockLanded>()
            .add_systems(OnEnter(GameState::Playing), (setup::setup_playing, audio::setup_audio, setup::setup_block_svgs))
            .add_systems(
                Update,
                (
                    input::slot_oscillation,
                    input::production_input,
                    settle::check_per_block_settle,
                    settle::check_settle,
                    settle::check_failure,
                    settle::detect_landings,
                    audio::play_landing_audio,
                    audio::play_collision_audio,
                )
                    .run_if(in_state(GameState::Playing)),
            )
            .add_systems(
                Update,
                (
                    ui::camera_follow,
                    ui::update_hud,
                    ui::update_ghost_highlights,
                    ui::animate_score_popups,
                    ui::animate_level_complete,
                    ui::update_evaluating_indicator,
                    ui::update_hearts,
                )
                    .run_if(in_state(GameState::Playing)),
            )
            .add_systems(OnExit(GameState::Playing), setup::cleanup_playing);
    }
}
