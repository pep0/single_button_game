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
    windows: Query<&Window>,
) {
    // Reset camera position
    if let Ok(mut cam_t) = camera_query.single_mut() {
        cam_t.translation.y = 0.0;
    }

    let (w, h) = windows.single()
        .map(|win| (win.width(), win.height()))
        .unwrap_or((512.0, 768.0));
    let scale = (w / 512.0).max(h / 768.0);
    let size = Vec2::new(512.0 * scale, 768.0 * scale);

    commands.spawn((
        MenuText,
        Sprite {
            image: asset_server.load("images/title_screen.png"),
            custom_size: Some(size),
            ..default()
        },
        Transform::from_xyz(0.0, 0.0, 0.0),
    ));
}

fn menu_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    touches: Res<Touches>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    if keyboard.just_pressed(KeyCode::Space) || touches.any_just_pressed() {
        next_state.set(GameState::Playing);
    }
    if keyboard.just_pressed(KeyCode::KeyE) {
        next_state.set(GameState::Editor);
    }
}
