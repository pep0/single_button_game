# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Commands

```bash
cargo run          # Run the game
cargo build        # Debug build
cargo build --release  # Release build
cargo check        # Fast type-check without linking
```

There are no tests. Container builds use `Containerfile` (not `Dockerfile`).

## Architecture

**Tower Stacker** is a Bevy 0.18 + Avian2D physics game. The player uses Space bar to size and drop rectangular blocks onto blueprint target positions.

### Game State Machine

`GameState` (in `src/state.rs`) drives everything via Bevy's state system:

```
Menu → Playing → Scoring → Playing (next round)
              ↘ Failed → Playing (retry)
Editor ↔ Playing  (test-play mode: EditorTestPlay resource present)
```

Each state has a corresponding plugin or module that registers `OnEnter`/`OnExit`/`Update` systems.

### Core Data Flow

- **`Blueprint`** (resource, `src/blueprint.rs`) — describes the target structure: a `Vec<BlockSlot>`, each with `x`, `y`, `width`, `height`. Built procedurally by `build_blueprint(round)` cycling through 6 hardcoded levels, or loaded from a JSON file in the editor.
- **`ProducedDimensions`** (resource, `src/playing/resources.rs`) — records the actual widths/heights the player produced, used by Scoring to compute accuracy.
- **`Score`** (resource, `src/state.rs`) — accumulates round scores across levels.

### Playing Module (`src/playing/`)

- **`input.rs`** — `slot_oscillation`: animates slot width (sine wave); `production_input`: Space-press locks width and grows the block, Space-release drops it as an Avian2D `RigidBody::Dynamic`.
- **`settle.rs`** — `check_settle`: after all blocks placed, waits 1.5 s then requires 0.5 s of continuous rest (velocity < 2.0 or `Sleeping`) before declaring pass; `check_failure`: detects blocks fallen below `FAIL_Y_THRESHOLD` or tilted > 15°.
- **`setup.rs`** — spawns ground, ghost outlines, HUD, and initializes resources from the current Blueprint.
- **`ui.rs`** — camera follow, HUD text, ghost block highlighting.

### Editor Module (`src/editor/`)

- **`input.rs`** — Arrow Left/Right to move slot; Space to grow/drop block; Arrow Down for instant-place using `last_block_height`; all input blocked while filename modal is active.
- **`save.rs`** — S opens filename input modal (typed inline, Enter to confirm), serializes `EditorBlock` entities to `Blueprint` JSON; P launches test-play by inserting the Blueprint resource and `EditorTestPlay` marker, then switching to `GameState::Playing`; R clears all blocks; Escape exits to menu.
- **`fall.rs`** — animates `FallingBlock` entities falling to their computed `target_y` (no physics engine in the editor).
- **`setup.rs`** / **`ui.rs`** / **`resources.rs`** / **`components.rs`** — editor-specific state, HUD, and entity markers.

Test-play mode: when `EditorTestPlay` resource exists during `GameState::Playing`, settle/failure checks return to `GameState::Editor` instead of `Scoring`/`Failed`. Escape during test-play also returns to the editor.

### Constants (`src/constants.rs`)

All colors, layout values (`GROUND_Y`, `SPAWN_HEIGHT_ABOVE`), slot oscillation parameters, physics tuning, and editor-specific constants live here.

### Cleanup Pattern

`state::cleanup::<T>` despawns all entities with component `T`. Each module tags its entities with a state-scoped marker (e.g., `PlayingEntity`, `EditorEntity`, `ScoringEntity`) and calls `cleanup` on `OnExit`.

### Custom Level JSON Format

Blueprints serialize to/from JSON (`serde_json`). Example structure:
```json
{ "slots": [{ "width": 100.0, "height": 50.0, "x": 0.0, "y": -190.0 }], "level_number": 7 }
```
Files like `custom_level.json` and `42.json` in the repo root are editor-saved levels.
