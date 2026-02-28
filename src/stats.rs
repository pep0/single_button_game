use bevy::prelude::*;
use bevy::text::TextBounds;
#[cfg(not(target_arch = "wasm32"))]
use serde::{Deserialize, Serialize};

use crate::constants::*;
use crate::state::{cleanup, GameState, Score};

#[cfg(not(target_arch = "wasm32"))]
const BEST_SCORE_PATH: &str = "best_score.json";

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

#[cfg(not(target_arch = "wasm32"))]
#[derive(Serialize, Deserialize, Default)]
struct BestScore {
    average_accuracy: f32,
}

fn load_best() -> f32 {
    #[cfg(target_arch = "wasm32")]
    {
        use web_sys::window;
        return window()
            .and_then(|w| w.local_storage().ok().flatten())
            .and_then(|s| s.get_item("tower_stacker_best").ok().flatten())
            .and_then(|v| v.parse::<f32>().ok())
            .unwrap_or(0.0);
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        std::fs::read_to_string(BEST_SCORE_PATH)
            .ok()
            .and_then(|s| serde_json::from_str::<BestScore>(&s).ok())
            .map(|b| b.average_accuracy)
            .unwrap_or(0.0)
    }
}

fn save_best(avg: f32) {
    #[cfg(target_arch = "wasm32")]
    {
        use web_sys::window;
        if let Some(storage) = window().and_then(|w| w.local_storage().ok().flatten()) {
            let _ = storage.set_item("tower_stacker_best", &avg.to_string());
        }
        return;
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let data = BestScore { average_accuracy: avg };
        if let Ok(json) = serde_json::to_string(&data) {
            let _ = std::fs::write(BEST_SCORE_PATH, json);
        }
    }
}

const WRAP_WIDTH: f32 = 460.0;
const WRAP_LAYOUT: TextLayout = TextLayout {
    justify: Justify::Center,
    linebreak: LineBreak::WordBoundary,
};

fn setup_stats(mut commands: Commands, score: Res<Score>) {
    commands.spawn((
        StatsEntity,
        Text2d::new("All Levels Complete!"),
        TextFont {
            font_size: 40.0,
            ..default()
        },
        TextColor(SLOT_COLOR),
        TextBounds::new_horizontal(WRAP_WIDTH),
        WRAP_LAYOUT,
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
        TextBounds::new_horizontal(WRAP_WIDTH),
        WRAP_LAYOUT,
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
        TextBounds::new_horizontal(WRAP_WIDTH),
        WRAP_LAYOUT,
        Transform::from_xyz(0.0, 20.0, 1.0),
    ));

    // Best score comparison
    let prev_best = load_best();
    let is_new_best = avg > prev_best + 0.01 && score.rounds_played > 0;
    if is_new_best {
        save_best(avg);
        commands.spawn((
            StatsEntity,
            Text2d::new(format!("NEW BEST!  (previous: {prev_best:.0}%)")),
            TextFont {
                font_size: 24.0,
                ..default()
            },
            TextColor(Color::srgba(0.38, 0.88, 0.55, 1.0)),
            TextBounds::new_horizontal(WRAP_WIDTH),
            WRAP_LAYOUT,
            Transform::from_xyz(0.0, -35.0, 1.0),
        ));
    } else if prev_best > 0.01 {
        commands.spawn((
            StatsEntity,
            Text2d::new(format!("Best: {prev_best:.0}%")),
            TextFont {
                font_size: 20.0,
                ..default()
            },
            TextColor(Color::srgba(0.62, 0.62, 0.65, 0.85)),
            TextBounds::new_horizontal(WRAP_WIDTH),
            WRAP_LAYOUT,
            Transform::from_xyz(0.0, -35.0, 1.0),
        ));
    }

    commands.spawn((
        StatsEntity,
        Text2d::new("Press SPACE to return to menu"),
        TextFont {
            font_size: 22.0,
            ..default()
        },
        TextColor(TEXT_COLOR),
        TextBounds::new_horizontal(WRAP_WIDTH),
        WRAP_LAYOUT,
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
