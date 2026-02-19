use bevy::prelude::*;
use single_button_game::blueprint::BlockSlot;

use crate::drag::HandlePosition;

pub const HANDLE_HALF_SIZE: f32 = 8.0;

pub enum HitResult {
    Handle {
        slot_index: usize,
        position: HandlePosition,
    },
    Block {
        slot_index: usize,
    },
    Empty,
}

/// Pure hit-test: no Bevy system, called from mouse input system.
/// `handle_positions`: for each slot_index (that has a selected block),
/// pass the 8 world-space handle positions.
pub fn find_hit(
    cursor: Vec2,
    slots: &[BlockSlot],
    selected_block: Option<usize>,
    handle_world_positions: &[(usize, HandlePosition, Vec2)],
) -> HitResult {
    // 1. Handles first (only when a block is selected)
    if selected_block.is_some() {
        for (slot_index, handle_pos, world_pos) in handle_world_positions {
            let dist = (cursor - *world_pos).abs();
            if dist.x <= HANDLE_HALF_SIZE && dist.y <= HANDLE_HALF_SIZE {
                return HitResult::Handle {
                    slot_index: *slot_index,
                    position: *handle_pos,
                };
            }
        }
    }

    // 2. Block interiors — highest slot_index wins on overlap
    let mut best: Option<usize> = None;
    for (i, slot) in slots.iter().enumerate() {
        let half_w = slot.width / 2.0;
        let half_h = slot.height / 2.0;
        if cursor.x >= slot.x - half_w
            && cursor.x <= slot.x + half_w
            && cursor.y >= slot.y - half_h
            && cursor.y <= slot.y + half_h
        {
            best = Some(i);
        }
    }
    if let Some(i) = best {
        return HitResult::Block { slot_index: i };
    }

    HitResult::Empty
}
