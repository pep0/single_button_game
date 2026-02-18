use bevy::prelude::*;

use crate::blueprint::Blueprint;
use crate::constants::*;
use crate::state::{cleanup, cleanup_shared_resources, GameState, Score};

pub struct FailedPlugin;

impl Plugin for FailedPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::Failed), setup_failed)
            .add_systems(Update, failed_input.run_if(in_state(GameState::Failed)))
            .add_systems(
                OnExit(GameState::Failed),
                (cleanup::<FailedEntity>, cleanup_shared_resources),
            );
    }
}

#[derive(Component)]
struct FailedEntity;

fn setup_failed(mut commands: Commands, score: Res<Score>, blueprint: Res<Blueprint>) {
    commands.spawn((
        FailedEntity,
        Text2d::new("Structure Collapsed!"),
        TextFont {
            font_size: 40.0,
            ..default()
        },
        TextColor(FAIL_COLOR),
        Transform::from_xyz(0.0, 80.0, 1.0),
    ));

    commands.spawn((
        FailedEntity,
        Text2d::new(format!("Failed on Level {}", blueprint.level_number)),
        TextFont {
            font_size: 24.0,
            ..default()
        },
        TextColor(TEXT_COLOR),
        Transform::from_xyz(0.0, 20.0, 1.0),
    ));

    if score.rounds_played > 0 {
        commands.spawn((
            FailedEntity,
            Text2d::new(format!(
                "Final Average: {:.0}%  ({} rounds completed)",
                score.total_score / score.rounds_played as f32 * 100.0,
                score.rounds_played
            )),
            TextFont {
                font_size: 20.0,
                ..default()
            },
            TextColor(Color::srgba(0.7, 0.7, 0.7, 0.9)),
            Transform::from_xyz(0.0, -30.0, 1.0),
        ));
    }

    commands.spawn((
        FailedEntity,
        Text2d::new("Press SPACE to try again"),
        TextFont {
            font_size: 22.0,
            ..default()
        },
        TextColor(TEXT_COLOR),
        Transform::from_xyz(0.0, -100.0, 1.0),
    ));
}

fn failed_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut next_state: ResMut<NextState<GameState>>,
    mut score: ResMut<Score>,
) {
    if keyboard.just_pressed(KeyCode::Space) {
        score.round = 0;
        score.total_score = 0.0;
        score.rounds_played = 0;
        next_state.set(GameState::Menu);
    }
}
