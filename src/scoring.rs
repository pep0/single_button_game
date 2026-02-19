use bevy::prelude::*;

use crate::blueprint::Blueprint;
use crate::constants::*;
use crate::playing::ProducedDimensions;
use crate::state::{cleanup, cleanup_shared_resources, GameState, LevelSequence, Score};

pub struct ScoringPlugin;

impl Plugin for ScoringPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::Scoring), setup_scoring)
            .add_systems(Update, scoring_input.run_if(in_state(GameState::Scoring)))
            .add_systems(
                OnExit(GameState::Scoring),
                (cleanup::<ScoringEntity>, cleanup_shared_resources),
            );
    }
}

#[derive(Component)]
struct ScoringEntity;

fn setup_scoring(
    mut commands: Commands,
    blueprint: Res<Blueprint>,
    produced: Res<ProducedDimensions>,
    mut score: ResMut<Score>,
    sequence: Res<LevelSequence>,
) {
    let mut shape_scores = Vec::new();
    let mut total = 0.0;

    for (i, slot) in blueprint.slots.iter().enumerate() {
        if i >= produced.widths.len() {
            break;
        }
        let pw = produced.widths[i];
        let ph = produced.heights[i];
        let w_ratio = (pw / slot.width).min(slot.width / pw);
        let h_ratio = (ph / slot.height).min(slot.height / ph);
        let s = w_ratio * h_ratio;
        shape_scores.push(s);
        total += s;
    }

    let avg = if shape_scores.is_empty() {
        0.0
    } else {
        total / shape_scores.len() as f32
    };

    let round_percent = avg * 100.0;
    score.total_score += avg;
    score.rounds_played += 1;

    // Title
    commands.spawn((
        ScoringEntity,
        Text2d::new(format!("Level {} Complete!", blueprint.level_number)),
        TextFont {
            font_size: 36.0,
            ..default()
        },
        TextColor(SLOT_COLOR),
        Transform::from_xyz(0.0, 200.0, 1.0),
    ));

    commands.spawn((
        ScoringEntity,
        Text2d::new("Structure Stable!"),
        TextFont {
            font_size: 24.0,
            ..default()
        },
        TextColor(Color::srgb(0.5, 0.9, 0.5)),
        Transform::from_xyz(0.0, 155.0, 1.0),
    ));

    // Per-block breakdown
    for (i, s) in shape_scores.iter().enumerate() {
        commands.spawn((
            ScoringEntity,
            Text2d::new(format!("Block {}: {:.0}%", i + 1, s * 100.0)),
            TextFont {
                font_size: 18.0,
                ..default()
            },
            TextColor(TEXT_COLOR),
            Transform::from_xyz(0.0, 100.0 - i as f32 * 30.0, 1.0),
        ));
    }

    let overall_y = 100.0 - shape_scores.len() as f32 * 30.0 - 25.0;
    commands.spawn((
        ScoringEntity,
        Text2d::new(format!("Round Score: {:.0}%", round_percent)),
        TextFont {
            font_size: 28.0,
            ..default()
        },
        TextColor(TOWER_BLOCK_COLOR),
        Transform::from_xyz(0.0, overall_y, 1.0),
    ));

    if score.rounds_played > 0 {
        commands.spawn((
            ScoringEntity,
            Text2d::new(format!(
                "Average Score: {:.0}%",
                score.total_score / score.rounds_played as f32 * 100.0
            )),
            TextFont {
                font_size: 22.0,
                ..default()
            },
            TextColor(Color::srgba(0.7, 0.7, 0.7, 0.9)),
            Transform::from_xyz(0.0, overall_y - 40.0, 1.0),
        ));
    }

    let is_last = score.round + 1 >= sequence.entries.len();
    let prompt = if is_last {
        "Press SPACE to see final results"
    } else {
        "Press SPACE for next level"
    };
    commands.spawn((
        ScoringEntity,
        Text2d::new(prompt),
        TextFont {
            font_size: 22.0,
            ..default()
        },
        TextColor(TEXT_COLOR),
        Transform::from_xyz(0.0, -220.0, 1.0),
    ));
}

fn scoring_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut next_state: ResMut<NextState<GameState>>,
    mut score: ResMut<Score>,
    sequence: Res<LevelSequence>,
) {
    if keyboard.just_pressed(KeyCode::Space) {
        let next = score.round + 1;
        score.round = next;
        if next >= sequence.entries.len() {
            next_state.set(GameState::Stats);
        } else {
            next_state.set(GameState::Playing);
        }
    }
}
