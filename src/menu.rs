use bevy::prelude::*;

use crate::state::{cleanup, GameState};

pub struct MenuPlugin;

impl Plugin for MenuPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::Menu), setup_menu)
            .add_systems(Update, menu_input.run_if(in_state(GameState::Menu)))
            .add_systems(OnExit(GameState::Menu), cleanup::<MenuText>);
    }
}

#[derive(Component)]
struct MenuText;

fn setup_menu(
    mut commands: Commands,
    mut camera_query: Query<&mut Transform, With<Camera2d>>,
    asset_server: Res<AssetServer>,
) {
    // Reset camera position
    if let Ok(mut cam_t) = camera_query.single_mut() {
        cam_t.translation.y = 0.0;
    }

    commands.spawn((
        MenuText,
        Sprite {
            image: asset_server.load("images/title_screen.png"),
            custom_size: Some(Vec2::new(512.0, 768.0)),
            ..default()
        },
        Transform::from_xyz(0.0, 0.0, 0.0),
    ));
}

fn menu_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    if keyboard.just_pressed(KeyCode::Space) {
        next_state.set(GameState::Playing);
    }
    if keyboard.just_pressed(KeyCode::KeyE) {
        next_state.set(GameState::Editor);
    }
}
