use bevy::prelude::*;

use crate::constants::*;
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
) {
    // Reset camera position
    if let Ok(mut cam_t) = camera_query.single_mut() {
        cam_t.translation.y = 0.0;
    }

    commands.spawn((
        MenuText,
        Text2d::new("TOWER STACKER"),
        TextFont {
            font_size: 40.0,
            ..default()
        },
        TextColor(SLOT_COLOR),
        Transform::from_xyz(0.0, 60.0, 0.0),
    ));

    commands.spawn((
        MenuText,
        Text2d::new("Press SPACE to Start"),
        TextFont {
            font_size: 24.0,
            ..default()
        },
        TextColor(TEXT_COLOR),
        Transform::from_xyz(0.0, -20.0, 0.0),
    ));

    commands.spawn((
        MenuText,
        Text2d::new("Hold SPACE to extrude blocks\nMatch the blueprint to build!"),
        TextFont {
            font_size: 16.0,
            ..default()
        },
        TextColor(Color::srgba(0.7, 0.7, 0.7, 0.8)),
        Transform::from_xyz(0.0, -80.0, 0.0),
    ));
}

fn menu_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    if keyboard.just_pressed(KeyCode::Space) {
        next_state.set(GameState::Playing);
    }
}
