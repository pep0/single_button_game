# Stories

Stories are tasks for Claude Code to execute. Each story should be self-contained enough that an agent can work on it autonomously.

**Agent instructions:** When asked to work on a story, read the full entry, check `status`, and update it as you progress. Mark `status: done` and fill `result` when finished.

---

## Template

```
### STORY-000: Short imperative title

**status:** pending
**priority:** medium   <!-- low | medium | high -->

#### What
A clear description of what needs to be done. Be specific — assume the agent hasn't
seen recent conversation context.

#### Why
Why this matters. What problem it solves or what value it adds.

#### Acceptance criteria
- [ ] Criterion one: observable, testable outcome
- [ ] Criterion two
- [ ] Criterion three

#### Context & constraints
- Relevant files: `src/foo.rs`, `src/bar.rs`
- Do NOT touch: `src/baz.rs` (unrelated system)
- Any known edge cases, API quirks, or architectural decisions to respect

#### Result
<!-- Agent fills this in when done -->
```

---

## Stories

<!-- Add new stories below this line -->

### STORY-004: Replace level_number with level_name; use sequence counter for display

**status:** done
**priority:** medium

#### What
Remove the `level_number` field from `Blueprint` and every level JSON. Rename the
existing `name: Option<String>` field to `level_name: Option<String>`. Give each of
the 10 levels in the standard sequence an interesting name. In-game, display the
level name alongside a counter derived from `score.round + 1` (the position in the
sequence), not from anything stored in the file.

#### Why
`level_number` is redundant — the game already has `score.round` as a sequential
counter. Storing it in the file creates drift (several files already have
`level_number: 1` for levels that are not level 1). Names are more evocative and
give the player a sense of place.

#### Acceptance criteria
- [ ] `Blueprint.level_number` field is gone from `src/blueprint.rs`
- [ ] `Blueprint.name` is renamed to `Blueprint.level_name` (stays `Option<String>`,
      keeps `#[serde(default, skip_serializing_if = "Option::is_none")]`)
- [ ] All 10 files in `levels/standard/` have the `"level_number"` key removed and a
      `"level_name"` key added with an interesting name (see suggestions below)
- [ ] HUD during play shows e.g. `"Level 3 — Crossing    Block: 1/3"` using
      `score.round + 1` for the number and `blueprint.level_name` for the name;
      fall back to just `"Level 3"` when `level_name` is `None`
- [ ] "Level X Complete!" banner uses the same counter, optionally appending the name
- [ ] "Failed on Level X" screen uses the same counter
- [ ] Binary level editor: `save_blueprint` no longer takes or stores `level_number`
- [ ] Binary level editor: `load_level_name` reads `blueprint.level_name`
- [ ] `CanvasState.level_number` field removed from `src/bin/level_editor/state.rs`
- [ ] `src/editor/save.rs` no longer sets `level_number` when constructing `Blueprint`
- [ ] Game compiles and the HUD displays correctly for a test run

#### Suggested level names (agent may improve these)
The sequence order in `levels/sequence.json`:

| File | Slots | Suggested name |
|---|---|---|
| `standard/1_single_block.json` | 1 | "Drop Zone" |
| `standard/2_two_blocks.json` | 2 | "Odd Couple" |
| `standard/01_simple_stack.json` | 3 | "The Column" |
| `standard/02_two_posts.json` | 4 | "The Gate" |
| `standard/03_bridge.json` | 3 | "Crossing" |
| `standard/04_pyramid.json` | 3 | "Ziggurat" |
| `standard/05_balance.json` | 3 | "Needle" |
| `standard/06_arch.json` | 4 | "The Arch" |
| `standard/tower.json` | 2 | "Watchtower" |
| `standard/carrier.json` | 6 | "Flagship" |

#### Context & constraints
- `src/blueprint.rs` — `Blueprint` struct; `level_number` is on line ~81, `name` on ~83
- `src/playing/ui.rs` — HUD update (`blueprint.level_number`) and level-complete
  banner both reference `blueprint.level_number`; `score.round` is already in scope
- `src/playing/setup.rs` — initial HUD text at line ~130 references `blueprint.level_number`
- `src/failed.rs` — "Failed on Level {}" at line ~44
- `src/editor/save.rs` — sets `level_number: 7` in two places (lines ~137, ~186);
  also sets `name: None` — update `name` → `level_name`
- `src/bin/level_editor/file_io.rs` — `save_blueprint` takes `level_number: usize`;
  `load_level_name` reads `bp.name` — update to `bp.level_name`
- `src/bin/level_editor/state.rs` — `CanvasState.level_number: usize` and its default
- `src/bin/level_editor/canvas_screen.rs` — passes `canvas.level_number` to
  `save_blueprint`; `canvas.name` should be renamed to `canvas.level_name`
  where it maps to the Blueprint field
- `src/bin/level_editor/sequence_screen.rs` — references `bp.level_number` when
  loading a level into `CanvasState`
- Custom levels in `levels/custom/` and scratch files (`asdkfj.json`, etc.) do NOT
  need `level_name` set — `None` is fine
- Do NOT change the sequence.json file or the file loading order

#### Result
Removed `level_number` from `Blueprint`; renamed `name` → `level_name` (Option<String>,
same serde attrs). Updated all 10 standard level JSONs: removed `"level_number"`,
added `"level_name"` with evocative names (Drop Zone, Odd Couple, The Column, The Gate,
Crossing, Ziggurat, Needle, The Arch, Watchtower, Flagship). HUD now shows
`"Level N — Name    Block: x/y"` using `score.round + 1` as the counter; falls back
to `"Level N    Block: x/y"` when no name. Level-complete banner and failed screen
use the same counter. Dropped unused `blueprint` param from `setup_failed`. Binary
level editor updated throughout (`file_io`, `state`, `canvas_screen`, `sequence_screen`).
Builds clean, no warnings.

---

### STORY-003: Align floor position between level editor and game

**status:** done
**priority:** medium

#### What
The game's ground rectangle is misaligned relative to where the binary level editor
treats the floor surface. This causes ghost (preview) blocks in the game to visually
overlap or pixel-fight with the floor, and makes the two tools inconsistent.

Fix: shift the game's floor mesh + collider **down** by `GROUND_HALF_HEIGHT` so its
top edge sits at `GROUND_Y - GROUND_HALF_HEIGHT` instead of `GROUND_Y`. This creates
a clear visual gap between the floor rect and the ghost blocks whose bottoms sit at
`GROUND_Y`. Adjust the physics collider by the same amount so blocks still land
correctly on the floor surface.

Also fix the in-game editor (`src/editor/`) whose floor is currently centered at
`GROUND_Y` (top edge `GROUND_Y + GROUND_HALF_HEIGHT`), which is 10 px too high
relative to where physics blocks actually land.

#### Why
Currently:
- Binary level editor: floor is a 4 px line centered at `GROUND_Y`; blueprint slots
  are drawn so block bottoms sit at ~`GROUND_Y`.
- Game: ghost blocks are spawned with bottom at `GROUND_Y`, but the floor rect's top
  edge is also exactly at `GROUND_Y` → they share the same edge and look like they
  overlap.
- In-game editor: floor rect top is at `GROUND_Y + GROUND_HALF_HEIGHT` (10 px higher
  than in the game), so the two environments look different.

Consistent rule: `GROUND_Y` is the **surface** the blocks rest on. The visual floor
rect should sit **below** `GROUND_Y`, never touching it.

#### Acceptance criteria
- [ ] In the game, the floor rect's **top edge** is at `GROUND_Y - GROUND_HALF_HEIGHT`
      (i.e. `Transform::from_xyz(0.0, GROUND_Y - GROUND_HALF_HEIGHT * 2.0, 0.0)`)
- [ ] The physics collider moves with the mesh — blocks physically land at
      `GROUND_Y - GROUND_HALF_HEIGHT` and the ghost blocks are shifted down by the
      same amount (or the ghost block positions are updated) so they still rest on
      the collider surface without floating or sinking
- [ ] The in-game editor (`src/editor/setup.rs`) floor is drawn at the same
      position as the game floor (center `GROUND_Y - GROUND_HALF_HEIGHT * 2.0`
      or whichever offset makes its top align with `GROUND_Y`)
- [ ] No ghost blocks visually overlap the floor rectangle
- [ ] The binary level editor (`src/bin/level_editor/canvas_screen.rs`) floor line
      stays at `GROUND_Y` (it is already approximately correct as a thin reference line)
- [ ] Game compiles and blocks stack without sinking into the floor

#### Context & constraints
- `GROUND_Y = -200.0`, `GROUND_HALF_HEIGHT = 10.0` — defined in `src/constants.rs`
- Game floor spawn: `src/playing/setup.rs` line ~93 —
  `Transform::from_xyz(0.0, GROUND_Y - GROUND_HALF_HEIGHT, 0.0)` (current)
- Ghost blocks: `src/playing/setup.rs` line ~106 —
  `Transform::from_xyz(slot.x, slot.y, 0.1)` — `slot.y` is the block center
- In-game editor floor: `src/editor/setup.rs` line ~30 —
  `Transform::from_xyz(0.0, GROUND_Y, 0.0)` (current — wrong, off by GROUND_HALF_HEIGHT)
- In-game editor landing y: `src/editor/input.rs` line ~194 —
  `let mut landing_y = GROUND_Y + GROUND_HALF_HEIGHT + half_h;`
- If ghost block slot positions are shifted, the blueprint files on disk do NOT need to
  change — apply the offset only at render time, not to stored data
- Avian physics: collider position is driven by `Transform`, so moving the mesh also
  moves the collider automatically — no separate collider offset needed

#### Result
Root cause: in-game editor floor was centered at `GROUND_Y` (top at `GROUND_Y+10`),
10 px higher than the game floor (top at `GROUND_Y`). Binary-editor blueprint ghosts
touched the game floor top at the same pixel, rendering as visual overlap.

Fix: moved BOTH the game floor and in-game editor floor down by one more
`GROUND_HALF_HEIGHT` — new center `GROUND_Y - GROUND_HALF_HEIGHT * 2.0`, top at
`GROUND_Y - GROUND_HALF_HEIGHT`. Ghost blocks from binary-editor blueprints now have
a clean 10 px gap above the visual floor. Updated the in-game editor's `landing_y`,
slot oscillation fold base, and default `slot_y` to the new surface
(`GROUND_Y - GROUND_HALF_HEIGHT`) so blocks placed in the editor still land correctly.

Files changed: `src/playing/setup.rs`, `src/editor/setup.rs`, `src/editor/resources.rs`,
`src/editor/input.rs` (2 lines). No blueprint data changed.

---

### STORY-002: Replace SVG blocks with plain rectangles

**status:** done
**priority:** medium

#### What
Remove the `bevy_svg` dependency and replace the SVG block visuals with plain Bevy
rectangles (mesh + `ColorMaterial`). Each placed block should render as a filled
rectangle with a visible border. The score-based color coding must be preserved:
green (fit ≥ 0.80), yellow (fit ≥ 0.60), grey (fit < 0.60).

#### Why
The SVG blocks add a heavyweight dependency and asset pipeline complexity. Plain
rectangles are simpler, faster to render, and easier to style going forward.

#### Acceptance criteria
- [ ] `bevy_svg` is removed from `Cargo.toml` and all `use bevy_svg::…` imports are gone
- [ ] `SvgPlugin` is removed from the app builder in `src/main.rs`
- [ ] `BlockSvgAssets` resource and `setup_block_svgs` system are deleted
- [ ] Each placed block is rendered as a filled rectangle matching the block's physics size
- [ ] Border/outline is visible on each block (e.g. a slightly larger dark rectangle behind the fill)
- [ ] Color is green / yellow / grey based on the same score thresholds as before (≥0.80 / ≥0.60 / below)
- [ ] Game compiles and blocks appear correctly during play

#### Context & constraints
- SVG spawning logic is in `src/playing/input.rs` — the child `Svg2d` entity and the
  `svg_handle` selection should be replaced with a colored `Mesh2d` + `MeshMaterial2d`
- `BlockSvgAssets` resource is defined in `src/playing/resources.rs` — delete it
- `setup_block_svgs` is in `src/playing/setup.rs` — delete it and remove it from the
  system schedule in `src/playing/mod.rs`
- For the border: spawn two rectangle children — one slightly larger dark rectangle
  (z = -0.1 relative to parent) and one fill rectangle on top, both centered on the
  block origin; OR use a single mesh with a border shader if already available
- The existing `meshes` and `materials` params are already present in the
  `handle_input` system — reuse them
- SVG assets in `assets/blocks/` can be left on disk or deleted; do not break any
  other asset references

#### Result
Removed `bevy_svg` from `Cargo.toml`, `SvgPlugin` from `src/main.rs`, `BlockSvgAssets`
resource and `setup_block_svgs` system from `src/playing/`. In `src/playing/input.rs`,
replaced the single `Svg2d` child with two `Mesh2d` children per block: a dark border
rect at z=-0.1 sized `(pw + 6) × (ph + 6)` and a fill rect at z=0 sized `pw × ph`.
Fill color is green/yellow/grey based on the same score thresholds. Builds clean.

---

### STORY-001: Example — Add score display to HUD

**status:** pending
**priority:** low

#### What
Display the current score in the top-right corner of the game HUD during play.

#### Why
Players have no feedback on how well they are doing mid-run. A live score gives them
a reason to keep optimizing.

#### Acceptance criteria
- [ ] Score is visible during gameplay in the top-right corner
- [ ] Score updates in real time as blocks are cleared
- [ ] Text is readable against both light and dark backgrounds (use a drop shadow or outline)
- [ ] Score resets to 0 on new game

#### Context & constraints
- Score logic lives in `src/stats.rs`
- HUD/UI spawning is in `src/main.rs`
- Use the existing Bevy UI node system — do not add a new UI library
- Font assets are in `assets/`

#### Result
<!-- Agent fills this in when done -->
