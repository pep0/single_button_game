use crate::constants::*;

pub fn load_blueprint(path: &str) -> Result<Blueprint, String> {
    let full_path = format!("levels/{}", path);
    let contents = std::fs::read_to_string(&full_path)
        .map_err(|e| format!("Cannot read {full_path}: {e}"))?;
    serde_json::from_str(&contents).map_err(|e| format!("Parse error in {full_path}: {e}"))
}

pub fn load_sequence() -> Vec<String> {
    match std::fs::read_to_string("levels/sequence.json") {
        Ok(contents) => match serde_json::from_str::<Vec<String>>(&contents) {
            Ok(seq) => seq,
            Err(e) => {
                bevy::log::warn!("Failed to parse levels/sequence.json: {e}; using default sequence");
                default_sequence()
            }
        },
        Err(e) => {
            bevy::log::warn!("Failed to read levels/sequence.json: {e}; using default sequence");
            default_sequence()
        }
    }
}

fn default_sequence() -> Vec<String> {
    vec![
        "standard/01_simple_stack.json".to_string(),
        "standard/02_two_posts.json".to_string(),
        "standard/03_bridge.json".to_string(),
        "standard/04_pyramid.json".to_string(),
        "standard/05_balance.json".to_string(),
        "standard/06_arch.json".to_string(),
    ]
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct BlockSlot {
    pub width: f32,
    pub height: f32,
    pub x: f32,
    pub y: f32,
}

#[derive(bevy::prelude::Resource, Clone, serde::Serialize, serde::Deserialize)]
pub struct Blueprint {
    pub slots: Vec<BlockSlot>,
    pub level_number: usize,
}

fn build_blueprint(round: usize) -> Blueprint {
    let level_number = round % 6;

    let slots = match level_number {
        0 => {
            // Level 1 - "Simple Stack" (3 blocks at x=0, tapering up)
            let mut slots = Vec::new();
            let mut y = GROUND_Y + GROUND_HALF_HEIGHT;
            let specs = [(100.0, 50.0), (90.0, 40.0), (80.0, 45.0)];
            for (w, h) in specs {
                slots.push(BlockSlot {
                    width: w,
                    height: h,
                    x: 0.0,
                    y: y + h / 2.0,
                });
                y += h;
            }
            slots
        }
        1 => {
            // Level 2 - "Two Posts" (2 equal columns side by side)
            let mut slots = Vec::new();
            let mut y_left = GROUND_Y + GROUND_HALF_HEIGHT;
            let mut y_right = GROUND_Y + GROUND_HALF_HEIGHT;
            // Left column
            for h in [60.0, 50.0] {
                slots.push(BlockSlot {
                    width: 60.0,
                    height: h,
                    x: -80.0,
                    y: y_left + h / 2.0,
                });
                y_left += h;
            }
            // Right column
            for h in [60.0, 50.0] {
                slots.push(BlockSlot {
                    width: 60.0,
                    height: h,
                    x: 80.0,
                    y: y_right + h / 2.0,
                });
                y_right += h;
            }
            slots
        }
        2 => {
            // Level 3 - "Bridge" (2 posts + wide plank on top)
            let mut slots = Vec::new();
            let post_h = 70.0;
            // Left post
            slots.push(BlockSlot {
                width: 50.0,
                height: post_h,
                x: -100.0,
                y: GROUND_Y + GROUND_HALF_HEIGHT + post_h / 2.0,
            });
            // Right post
            slots.push(BlockSlot {
                width: 50.0,
                height: post_h,
                x: 100.0,
                y: GROUND_Y + GROUND_HALF_HEIGHT + post_h / 2.0,
            });
            // Plank
            let plank_h = 25.0;
            slots.push(BlockSlot {
                width: 250.0,
                height: plank_h,
                x: 0.0,
                y: GROUND_Y + GROUND_HALF_HEIGHT + post_h + plank_h / 2.0,
            });
            slots
        }
        3 => {
            // Level 4 - "Pyramid" (wide base, medium middle, narrow top)
            let mut slots = Vec::new();
            let mut y = GROUND_Y + GROUND_HALF_HEIGHT;
            let specs = [(160.0, 40.0), (100.0, 40.0), (50.0, 35.0)];
            for (w, h) in specs {
                slots.push(BlockSlot {
                    width: w,
                    height: h,
                    x: 0.0,
                    y: y + h / 2.0,
                });
                y += h;
            }
            slots
        }
        4 => {
            // Level 5 - "Balance" (wide base, narrow pillar, wide top)
            let mut slots = Vec::new();
            let mut y = GROUND_Y + GROUND_HALF_HEIGHT;
            let specs = [(140.0, 30.0), (30.0, 70.0), (130.0, 25.0)];
            for (w, h) in specs {
                slots.push(BlockSlot {
                    width: w,
                    height: h,
                    x: 0.0,
                    y: y + h / 2.0,
                });
                y += h;
            }
            slots
        }
        _ => {
            // Level 6 - "Arch" (2 angled posts + keystone)
            let mut slots = Vec::new();
            let post_h = 80.0;
            // Left post (slightly inward)
            slots.push(BlockSlot {
                width: 45.0,
                height: post_h,
                x: -70.0,
                y: GROUND_Y + GROUND_HALF_HEIGHT + post_h / 2.0,
            });
            // Right post
            slots.push(BlockSlot {
                width: 45.0,
                height: post_h,
                x: 70.0,
                y: GROUND_Y + GROUND_HALF_HEIGHT + post_h / 2.0,
            });
            // Keystone
            let key_h = 30.0;
            slots.push(BlockSlot {
                width: 180.0,
                height: key_h,
                x: 0.0,
                y: GROUND_Y + GROUND_HALF_HEIGHT + post_h + key_h / 2.0,
            });
            // Top block
            let top_h = 25.0;
            slots.push(BlockSlot {
                width: 80.0,
                height: top_h,
                x: 0.0,
                y: GROUND_Y + GROUND_HALF_HEIGHT + post_h + key_h + top_h / 2.0,
            });
            slots
        }
    };

    Blueprint {
        slots,
        level_number: round + 1,
    }
}
