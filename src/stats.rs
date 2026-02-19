use bevy::prelude::*;

use crate::constants::*;
use crate::state::{cleanup, GameState, Score};

pub struct StatsPlugin;

impl Plugin for StatsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::Stats), setup_stats)
            .add_systems(Update, stats_input.run_if(in_state(GameState::Stats)))
            .add_systems(OnExit(GameState::Stats), cleanup::<StatsEntity>);
    }
}

#[derive(Component)]
struct StatsEntity;

fn setup_stats(mut commands: Commands, score: Res<Score>) {
    commands.spawn((
        StatsEntity,
        Text2d::new("All Levels Complete!"),
        TextFont {
            font_size: 40.0,
            ..default()
        },
        TextColor(SLOT_COLOR),
        Transform::from_xyz(0.0, 150.0, 1.0),
    ));

    commands.spawn((
        StatsEntity,
        Text2d::new(format!("Levels Completed: {}", score.rounds_played)),
        TextFont {
            font_size: 26.0,
            ..default()
        },
        TextColor(TEXT_COLOR),
        Transform::from_xyz(0.0, 80.0, 1.0),
    ));

    let avg = if score.rounds_played > 0 {
        score.total_score / score.rounds_played as f32 * 100.0
    } else {
        0.0
    };

    commands.spawn((
        StatsEntity,
        Text2d::new(format!("Average Accuracy: {avg:.0}%")),
        TextFont {
            font_size: 30.0,
            ..default()
        },
        TextColor(TOWER_BLOCK_COLOR),
        Transform::from_xyz(0.0, 20.0, 1.0),
    ));

    commands.spawn((
        StatsEntity,
        Text2d::new("Press SPACE to return to menu"),
        TextFont {
            font_size: 22.0,
            ..default()
        },
        TextColor(TEXT_COLOR),
        Transform::from_xyz(0.0, -100.0, 1.0),
    ));
}

fn stats_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut next_state: ResMut<NextState<GameState>>,
    mut score: ResMut<Score>,
) {
    if keyboard.just_pressed(KeyCode::Space) {
        *score = Score::default();
        next_state.set(GameState::Menu);
    }
}
