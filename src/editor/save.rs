use bevy::prelude::*;

use crate::blueprint::{BlockSlot, Blueprint};
use crate::state::GameState;
use super::components::*;
use super::resources::*;

/// Handles S (save), R (reset), and Escape (back to menu).
pub fn editor_save_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut next_state: ResMut<NextState<GameState>>,
    mut commands: Commands,
    block_query: Query<(Entity, &Transform, &EditorBlock)>,
    falling_query: Query<Entity, With<FallingBlock>>,
    prod_rect_query: Query<Entity, With<EditorProductionRect>>,
    mut build_state: ResMut<EditorBuildState>,
    mut slot_state: ResMut<EditorSlotState>,
    mut production: ResMut<EditorProductionState>,
) {
    // ── Escape: back to menu ──
    if keyboard.just_pressed(KeyCode::Escape) {
        next_state.set(GameState::Menu);
        return;
    }

    // ── R: clear all blocks ──
    if keyboard.just_pressed(KeyCode::KeyR) {
        for (entity, _, _) in &block_query {
            commands.entity(entity).despawn();
        }
        for entity in &falling_query {
            commands.entity(entity).despawn();
        }
        for entity in &prod_rect_query {
            commands.entity(entity).despawn();
        }
        build_state.block_count = 0;
        build_state.status_msg = String::new();
        production.is_producing = false;
        production.current_height = 0.0;
        slot_state.locked_width = None;
        return;
    }

    // ── S: save to custom_level.json ──
    if keyboard.just_pressed(KeyCode::KeyS) {
        let mut placed: Vec<(&Transform, &EditorBlock)> = block_query
            .iter()
            .map(|(_, t, b)| (t, b))
            .collect();

        // Sort bottom-to-top for a natural layer order in the JSON.
        placed.sort_by(|a, b| {
            a.0.translation
                .y
                .partial_cmp(&b.0.translation.y)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        let slots: Vec<BlockSlot> = placed
            .iter()
            .map(|(t, b)| BlockSlot {
                width: b.width,
                height: b.height,
                x: t.translation.x,
                y: t.translation.y,
            })
            .collect();

        let blueprint = Blueprint {
            slots,
            level_number: 7,
        };

        match serde_json::to_string_pretty(&blueprint) {
            Ok(json) => match std::fs::write("custom_level.json", json.as_bytes()) {
                Ok(_) => {
                    build_state.status_msg = "Saved → custom_level.json".to_string();
                }
                Err(e) => {
                    build_state.status_msg = format!("Error writing file: {e}");
                }
            },
            Err(e) => {
                build_state.status_msg = format!("Serialization error: {e}");
            }
        }
    }
}
