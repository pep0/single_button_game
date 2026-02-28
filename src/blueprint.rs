pub fn load_blueprint(path: &str) -> Result<Blueprint, String> {
    #[cfg(target_arch = "wasm32")]
    {
        let json = embedded_level(path)
            .ok_or_else(|| format!("No embedded level: '{path}'"))?;
        return serde_json::from_str(json).map_err(|e| format!("Parse error in {path}: {e}"));
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let full_path = format!("levels/{path}");
        let contents = std::fs::read_to_string(&full_path)
            .map_err(|e| format!("Cannot read {full_path}: {e}"))?;
        serde_json::from_str(&contents).map_err(|e| format!("Parse error in {full_path}: {e}"))
    }
}

pub fn load_sequence() -> Vec<String> {
    #[cfg(target_arch = "wasm32")]
    {
        let json = include_str!("../levels/sequence.json");
        return serde_json::from_str(json).unwrap_or_else(|_| default_sequence());
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
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
}

#[cfg(target_arch = "wasm32")]
fn embedded_level(path: &str) -> Option<&'static str> {
    match path {
        "standard/01_simple_stack.json" => Some(include_str!("../levels/standard/01_simple_stack.json")),
        "standard/02_two_posts.json"    => Some(include_str!("../levels/standard/02_two_posts.json")),
        "standard/03_bridge.json"       => Some(include_str!("../levels/standard/03_bridge.json")),
        "standard/04_pyramid.json"      => Some(include_str!("../levels/standard/04_pyramid.json")),
        "standard/05_balance.json"      => Some(include_str!("../levels/standard/05_balance.json")),
        "standard/06_arch.json"         => Some(include_str!("../levels/standard/06_arch.json")),
        "standard/tower.json"           => Some(include_str!("../levels/standard/tower.json")),
        "standard/carrier.json"         => Some(include_str!("../levels/standard/carrier.json")),
        "standard/1_single_block.json"  => Some(include_str!("../levels/standard/1_single_block.json")),
        "standard/2_two_blocks.json"    => Some(include_str!("../levels/standard/2_two_blocks.json")),
        "standard/asdkfj.json"          => Some(include_str!("../levels/standard/asdkfj.json")),
        _ => None,
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
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub level_name: Option<String>,
}
