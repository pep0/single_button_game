mod blueprint;
mod constants;
mod editor;
mod failed;
mod menu;
mod playing;
mod scoring;
mod state;
mod stats;

use avian2d::prelude::*;
use bevy::{prelude::*, window::WindowResolution};
use constants::*;
use state::{GameState, LevelSequence, Score};

fn main() {
    #[allow(unused_mut)]
    let mut window = Window {
        title: "Tower Stacker".into(),
        resolution: WindowResolution::new(512, 768),
        ..default()
    };
    #[cfg(target_arch = "wasm32")]
    {
        window.canvas = Some("#bevy".to_string());
        window.fit_canvas_to_parent = true;
    }

    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(window),
            ..default()
        }))
        .add_plugins(PhysicsPlugins::default().with_length_unit(100.0))
        .insert_resource(ClearColor(BG_COLOR))
        .insert_resource(Gravity(Vec2::new(0.0, -GRAVITY_SCALE)))
        .init_state::<GameState>()
        .init_resource::<Score>()
        .init_resource::<LevelSequence>()
        .add_systems(Startup, (setup_camera, load_level_sequence))
        .add_plugins((
            menu::MenuPlugin,
            playing::PlayingPlugin,
            scoring::ScoringPlugin,
            failed::FailedPlugin,
            stats::StatsPlugin,
            editor::EditorPlugin,
        ))
        .run();
}

fn setup_camera(mut commands: Commands) {
    commands.spawn(Camera2d);
}

fn load_level_sequence(mut commands: Commands) {
    commands.insert_resource(LevelSequence {
        entries: blueprint::load_sequence(),
    });
}
