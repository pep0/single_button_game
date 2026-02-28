use bevy::ecs::message::MessageReader;
use bevy::input::keyboard::{Key, KeyboardInput};
use bevy::input::ButtonState;
use bevy::prelude::*;

use crate::blueprint::{BlockSlot, Blueprint};
use crate::state::GameState;
use super::components::*;
use super::resources::*;

/// Handles S (save), R (reset), P (test-play), and Escape (back to menu / cancel input).
pub fn editor_save_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut key_events: MessageReader<KeyboardInput>,
    mut next_state: ResMut<NextState<GameState>>,
    mut commands: Commands,
    block_query: Query<(Entity, &Transform, &EditorBlock)>,
    falling_query: Query<Entity, With<FallingBlock>>,
    prod_rect_query: Query<Entity, With<EditorProductionRect>>,
    mut build_state: ResMut<EditorBuildState>,
    mut slot_state: ResMut<EditorSlotState>,
    mut production: ResMut<EditorProductionState>,
) {
    // ── Escape: cancel filename input, or go to menu ──
    if keyboard.just_pressed(KeyCode::Escape) {
        if build_state.filename_input.is_some() {
            build_state.filename_input = None;
            // Consume the event so no other system sees it.
            return;
        }
        next_state.set(GameState::Menu);
        return;
    }

    // ── While filename input is active: capture keystrokes ──
    if build_state.filename_input.is_some() {
        for event in key_events.read() {
            if event.state != ButtonState::Pressed {
                continue;
            }
            match &event.logical_key {
                Key::Enter => {
                    let raw = build_state.filename_input.take().unwrap_or_default();
                    let filename = if raw.is_empty() {
                        "custom_level".to_string()
                    } else {
                        raw
                    };
                    let filename = if filename.ends_with(".json") {
                        filename
                    } else {
                        format!("{filename}.json")
                    };
                    save_blocks(&block_query, &filename, &mut build_state);
                }
                Key::Backspace => {
                    if let Some(ref mut buf) = build_state.filename_input {
                        buf.pop();
                    }
                }
                Key::Character(s) => {
                    if let Some(ref mut buf) = build_state.filename_input {
                        for ch in s.chars() {
                            if ch.is_alphanumeric() || ch == '-' || ch == '_' || ch == '.' {
                                buf.push(ch);
                            }
                        }
                    }
                }
                _ => {}
            }
        }
        // Block all other input while typing.
        return;
    }

    // Drain events so they don't accumulate (we're not in filename mode).
    key_events.clear();

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

    // ── S: open filename input ──
    if keyboard.just_pressed(KeyCode::KeyS) {
        build_state.filename_input = Some(String::new());
        return;
    }

    // ── P: test-play ──
    if keyboard.just_pressed(KeyCode::KeyP) {
        let mut placed: Vec<(&Transform, &EditorBlock)> = block_query
            .iter()
            .map(|(_, t, b)| (t, b))
            .collect();

        // Sort bottom-to-top.
        placed.sort_by(|a, b| {
            a.0.translation
                .y
                .partial_cmp(&b.0.translation.y)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        let snapshot_blocks: Vec<(Vec3, f32, f32)> = placed
            .iter()
            .map(|(t, b)| (t.translation, b.width, b.height))
            .collect();

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
            level_name: None,
        };

        commands.insert_resource(EditorSnapshot { blocks: snapshot_blocks });
        commands.insert_resource(blueprint);
        commands.insert_resource(EditorTestPlay);
        next_state.set(GameState::Playing);
    }
}

fn save_blocks(
    block_query: &Query<(Entity, &Transform, &EditorBlock)>,
    filename: &str,
    build_state: &mut EditorBuildState,
) {
    #[cfg(target_arch = "wasm32")]
    {
        let _ = (block_query, filename);
        build_state.status_msg = "Save not available in browser build".to_string();
        build_state.status_timer = 4.0;
        return;
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let mut placed: Vec<(&Transform, &EditorBlock)> = block_query
            .iter()
            .map(|(_, t, b)| (t, b))
            .collect();

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
            level_name: None,
        };

        let full_path = std::path::Path::new("levels/custom").join(filename);
        match serde_json::to_string_pretty(&blueprint) {
            Ok(json) => {
                if let Err(e) = std::fs::create_dir_all("levels/custom") {
                    build_state.status_msg = format!("Error creating directory: {e}");
                    return;
                }
                match std::fs::write(&full_path, json.as_bytes()) {
                    Ok(_) => {
                        build_state.status_msg = format!("Saved -> levels/custom/{filename}");
                        build_state.status_timer = 4.0;
                    }
                    Err(e) => {
                        build_state.status_msg = format!("Error writing file: {e}");
                    }
                }
            }
            Err(e) => {
                build_state.status_msg = format!("Serialization error: {e}");
            }
        }
    }
}
