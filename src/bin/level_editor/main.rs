mod canvas_screen;
mod drag;
mod file_io;
mod hit_test;
mod sequence_screen;
mod state;

use bevy::{prelude::*, window::WindowResolution};
use state::{CanvasState, EditorScreen, SequenceEditorState};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Tower Stacker – Level Editor".into(),
                resolution: WindowResolution::new(1100u32, 750u32),
                ..default()
            }),
            ..default()
        }))
        .insert_resource(ClearColor(Color::srgb(0.08, 0.08, 0.12)))
        .init_state::<EditorScreen>()
        .init_resource::<SequenceEditorState>()
        .init_resource::<CanvasState>()
        .add_systems(Startup, spawn_camera)
        .add_plugins(sequence_screen::SequenceScreenPlugin)
        .add_plugins(canvas_screen::CanvasScreenPlugin)
        .run();
}

fn spawn_camera(mut commands: Commands) {
    commands.spawn(Camera2d);
}

