use single_button_game::blueprint::{BlockSlot, Blueprint};

pub fn save_blueprint(
    slots: &[BlockSlot],
    name: Option<&str>,
    path: &str,
) -> Result<(), String> {
    let blueprint = Blueprint {
        slots: slots.to_vec(),
        level_name: name.map(|s| s.to_string()),
    };

    let full_path = std::path::Path::new("levels").join(path);

    // Ensure parent directories exist
    if let Some(parent) = full_path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| format!("Cannot create directory {}: {e}", parent.display()))?;
    }

    let json = serde_json::to_string_pretty(&blueprint)
        .map_err(|e| format!("Serialization error: {e}"))?;

    std::fs::write(&full_path, json.as_bytes())
        .map_err(|e| format!("Cannot write {}: {e}", full_path.display()))?;

    Ok(())
}

pub fn save_sequence(entries: &[String]) -> Result<(), String> {
    let json = serde_json::to_string_pretty(entries)
        .map_err(|e| format!("Serialization error: {e}"))?;

    std::fs::create_dir_all("levels")
        .map_err(|e| format!("Cannot create levels/: {e}"))?;

    std::fs::write("levels/sequence.json", json.as_bytes())
        .map_err(|e| format!("Cannot write levels/sequence.json: {e}"))?;

    Ok(())
}

pub fn load_adjacent_blueprints(
    entries: &[String],
    current_index: usize,
) -> (Vec<BlockSlot>, Vec<BlockSlot>) {
    let load = |path: &str| -> Vec<BlockSlot> {
        let full = format!("levels/{}", path);
        match std::fs::read_to_string(&full) {
            Ok(s) => match serde_json::from_str::<Blueprint>(&s) {
                Ok(bp) => bp.slots,
                Err(_) => Vec::new(),
            },
            Err(_) => Vec::new(),
        }
    };

    let prev = if current_index > 0 {
        entries
            .get(current_index - 1)
            .map(|p| load(p))
            .unwrap_or_default()
    } else {
        Vec::new()
    };

    let next = entries
        .get(current_index + 1)
        .map(|p| load(p))
        .unwrap_or_default();

    (prev, next)
}

pub fn load_sequence() -> Vec<String> {
    match std::fs::read_to_string("levels/sequence.json") {
        Ok(s) => serde_json::from_str(&s).unwrap_or_default(),
        Err(_) => Vec::new(),
    }
}

/// Try to read the level name from a blueprint file.
pub fn load_level_name(path: &str) -> Option<String> {
    let full = format!("levels/{}", path);
    let s = std::fs::read_to_string(&full).ok()?;
    let bp: Blueprint = serde_json::from_str(&s).ok()?;
    bp.level_name
}
