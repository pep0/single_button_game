mod components;
mod fall;
mod input;
mod resources;
mod save;
mod setup;
mod ui;

use bevy::prelude::*;
use crate::state::GameState;

pub struct EditorPlugin;

impl Plugin for EditorPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::Editor), setup::setup_editor)
            .add_systems(
                Update,
                (
                    input::editor_slot_oscillation,
                    input::editor_production_input,
                    fall::editor_fall_system,
                    save::editor_save_input,
                    ui::editor_camera_follow,
                    ui::editor_update_hud,
                )
                    .run_if(in_state(GameState::Editor)),
            )
            .add_systems(OnExit(GameState::Editor), setup::cleanup_editor);
    }
}
