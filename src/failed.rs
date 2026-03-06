use bevy::prelude::*;
use bevy::text::TextBounds;

use crate::constants::*;
use crate::state::{cleanup, cleanup_shared_resources, FailureReason, GameState, Score};

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

fn setup_failed(
    mut commands: Commands,
    mut score: ResMut<Score>,
    reason: Option<Res<FailureReason>>,
    windows: Query<&Window>,
) {
    score.lives = score.lives.saturating_sub(1);

    // Compute a safe text wrap width with 16 px margin on each side.
    let viewport_w = windows.single().map(|w| w.width()).unwrap_or(512.0);
    let wrap_w = (viewport_w - 32.0).max(200.0);
    // Scale font sizes proportionally for narrow viewports; keep minimum readable.
    let scale = (viewport_w / 512.0).min(1.0);
    let wrap_layout = TextLayout::new(Justify::Center, LineBreak::WordBoundary);

    commands.spawn((
        FailedEntity,
        Text2d::new("Structure Collapsed!"),
        TextFont {
            font_size: (40.0 * scale).max(24.0),
            ..default()
        },
        TextColor(FAIL_COLOR),
        TextBounds::new_horizontal(wrap_w),
        wrap_layout,
        Transform::from_xyz(0.0, 80.0, 1.0),
    ));

    commands.spawn((
        FailedEntity,
        Text2d::new(format!("Failed on Level {}", score.round + 1)),
        TextFont {
            font_size: (24.0 * scale).max(16.0),
            ..default()
        },
        TextColor(TEXT_COLOR),
        TextBounds::new_horizontal(wrap_w),
        wrap_layout,
        Transform::from_xyz(0.0, 20.0, 1.0),
    ));

    if let Some(r) = reason {
        if !r.message.is_empty() {
            commands.spawn((
                FailedEntity,
                Text2d::new(r.message.clone()),
                TextFont {
                    font_size: (18.0 * scale).max(14.0),
                    ..default()
                },
                TextColor(Color::srgba(0.95, 0.55, 0.3, 0.95)),
                TextBounds::new_horizontal(wrap_w),
                wrap_layout,
                Transform::from_xyz(0.0, -15.0, 1.0),
            ));
        }
    }

    commands.spawn((
        FailedEntity,
        Text2d::new(format!("Lives remaining: {}", score.lives)),
        TextFont {
            font_size: (20.0 * scale).max(14.0),
            ..default()
        },
        TextColor(Color::srgba(0.85, 0.42, 0.40, 0.95)),
        TextBounds::new_horizontal(wrap_w),
        wrap_layout,
        Transform::from_xyz(0.0, -55.0, 1.0),
    ));

    let prompt = if score.lives > 0 {
        "Press SPACE or tap to retry"
    } else {
        "Game Over!  Press SPACE or tap to return to menu"
    };
    commands.spawn((
        FailedEntity,
        Text2d::new(prompt),
        TextFont {
            font_size: (22.0 * scale).max(14.0),
            ..default()
        },
        TextColor(TEXT_COLOR),
        TextBounds::new_horizontal(wrap_w),
        wrap_layout,
        Transform::from_xyz(0.0, -120.0, 1.0),
    ));
}

fn failed_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    touches: Res<Touches>,
    mut next_state: ResMut<NextState<GameState>>,
    mut score: ResMut<Score>,
) {
    if keyboard.just_pressed(KeyCode::Space) || touches.any_just_pressed() {
        if score.lives > 0 {
            // Retry same level — keep score.round unchanged
            next_state.set(GameState::Playing);
        } else {
            *score = Score::default();
            next_state.set(GameState::Menu);
        }
    }
}
