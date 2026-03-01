use bevy::prelude::*;
use single_button_game::blueprint::BlockSlot;

// ── Top-level screen state ────────────────────────────────────────────────────

#[derive(States, Debug, Clone, PartialEq, Eq, Hash, Default)]
pub enum EditorScreen {
    #[default]
    Sequence,
    Canvas,
}

// ── Sequence screen ───────────────────────────────────────────────────────────

pub enum SeqInput {
    AddPath { buf: String },
}

#[derive(Resource)]
pub struct SequenceEditorState {
    pub entries: Vec<String>,
    pub blueprints: Vec<Option<String>>, // cached level names (None = not loaded yet)
    pub cursor: usize,
    pub grabbed: Option<usize>,
    pub text_input: Option<SeqInput>,
    pub status_msg: String,
    pub status_timer: f32,
}

impl Default for SequenceEditorState {
    fn default() -> Self {
        Self {
            entries: Vec::new(),
            blueprints: Vec::new(),
            cursor: 0,
            grabbed: None,
            text_input: None,
            status_msg: String::new(),
            status_timer: 0.0,
        }
    }
}

// ── Canvas screen ─────────────────────────────────────────────────────────────

pub enum CanvasInput {
    FilenamePrompt { buf: String },
    LevelName { buf: String },
}

#[derive(Resource)]
pub struct CanvasState {
    pub filepath: Option<String>,
    pub level_name: Option<String>,
    pub slots: Vec<BlockSlot>,
    pub sequence_index: Option<usize>,
    pub selected_block: Option<usize>,
    pub dirty: bool,
    pub sync_needed: bool,
    pub snap_grid: f32,
    pub grid_needs_rebuild: bool,
    pub text_input: Option<CanvasInput>,
    pub showing_unsaved_warning: bool,
    pub status_msg: String,
    pub status_timer: f32,
}

impl Default for CanvasState {
    fn default() -> Self {
        Self {
            filepath: None,
            level_name: None,
            slots: Vec::new(),
            sequence_index: None,
            selected_block: None,
            dirty: false,
            sync_needed: false,
            snap_grid: 0.0,
            grid_needs_rebuild: true,
            text_input: None,
            showing_unsaved_warning: false,
            status_msg: String::new(),
            status_timer: 0.0,
        }
    }
}
