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
