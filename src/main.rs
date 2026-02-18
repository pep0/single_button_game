mod blueprint;
mod constants;
mod editor;
mod failed;
mod menu;
mod playing;
mod scoring;
mod state;

use avian2d::prelude::*;
use bevy::{prelude::*, window::WindowResolution};
use constants::*;
use state::{GameState, Score};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Tower Stacker".into(),
                resolution: WindowResolution::new(800, 600),
                ..default()
            }),
            ..default()
        }))
        .add_plugins(PhysicsPlugins::default().with_length_unit(100.0))
        .insert_resource(ClearColor(BG_COLOR))
        .insert_resource(Gravity(Vec2::new(0.0, -GRAVITY_SCALE)))
        .init_state::<GameState>()
        .init_resource::<Score>()
        .add_systems(Startup, setup_camera)
        .add_plugins((
            menu::MenuPlugin,
            playing::PlayingPlugin,
            scoring::ScoringPlugin,
            failed::FailedPlugin,
            editor::EditorPlugin,
        ))
        .run();
}

fn setup_camera(mut commands: Commands) {
    commands.spawn(Camera2d);
}
