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

### STORY-021: Use per-tier darker border colour on blocks

**status:** done
**priority:** medium

#### What
Block borders are currently a fixed near-black colour (`srgb(0.10, 0.10, 0.15)`)
regardless of block score. Replace this with a per-tier border that is a darkened
version of the block fill colour:

| Tier | Fill | Border (≈ 60 % of fill) |
|---|---|---|
| Green (≥ 0.80) | `srgb(0.38, 0.72, 0.45)` | `srgb(0.22, 0.43, 0.27)` |
| Yellow (0.60–0.79) | `srgb(0.82, 0.70, 0.30)` | `srgb(0.49, 0.42, 0.18)` |
| Grey (< 0.60) | `srgb(0.48, 0.46, 0.52)` | `srgb(0.29, 0.28, 0.31)` |

#### Why
A near-black border looks harsh and unrelated to the block colour. A darker tint
of the same hue makes the block feel like a single cohesive shape.

#### Acceptance criteria
- [ ] Each score tier's border visually matches the hue of its fill (darker shade)
- [ ] The fixed `BLOCK_BORDER` constant is replaced by per-tier derived colours
- [ ] `cargo build` compiles clean

#### Context & constraints
- Only `src/playing/input.rs` needs to change
- `fill_color` and `score_tier` are already computed before the border child is
  spawned — derive `border_color` from the same tier logic and use it in the
  `ColorMaterial` for the border rectangle child
- `BLOCK_BORDER` constant can be removed; the three new border colours can be
  inline constants or derived inline
- Do NOT touch physics, faces, particles, or any other file

#### Result
Removed `BLOCK_BORDER` constant from `src/playing/input.rs` and added three
per-tier border constants (`BORDER_GREEN`, `BORDER_YELLOW`, `BORDER_GREY`) at
≈ 60 % brightness of their matching fill hue. Added `border_color` selection
alongside the existing `fill_color` selection; the border rectangle child now
uses `border_color` instead of the fixed dark constant. Builds clean.

---

### STORY-022: Show the visible game-frame boundary in the level editor

**status:** done
**priority:** medium

#### What
Add a thin rectangle outline to the editor canvas showing the exact bounds of the
player's visible game window. This helps designers know which blocks will be visible
at level start without guessing.

The game window is **512 × 768 px** with the camera starting at world origin
`(0, 0)`, so the visible rectangle is:
- x: **−256 to +256**
- y: **−384 to +384**

Draw this as a thin (2 px) unfilled rectangle outline in a neutral colour (e.g.
`srgba(0.9, 0.9, 0.9, 0.35)`) at z = −0.1 (behind blocks). Label it "game frame"
with small text near the top edge.

Since the editor camera can pan, the outline should always stay at these fixed
world coordinates (it is spawned once in `setup_canvas` and never moved).

#### Why
Designers have no reference for what the player sees at level start. The outline
prevents placing key blocks above the visible area or too close to the edges.

#### Acceptance criteria
- [ ] A rectangle outline is visible in the editor at `±256 × ±384` world coords
- [ ] The outline does not move when the editor camera pans
- [ ] A small "game frame" label sits near the top edge of the outline
- [ ] Blocks and other editor elements draw on top of the outline (z ordering)
- [ ] `cargo build` compiles clean

#### Context & constraints
- Only `src/bin/level_editor/canvas_screen.rs` → `setup_canvas` needs to change
- Draw the outline as four thin `Rectangle` meshes (top, bottom, left, right edges)
  or a single hollow rectangle — four thin rects is simplest with existing helpers
- Tag all new entities `CanvasEntity` so they are cleaned up on exit
- Do NOT change game code, other editor files, or the sequence screen

#### Result
Added `FRAME_HW=256`, `FRAME_HH=384`, `FRAME_COLOR` constants to
`canvas_screen.rs`. In `setup_canvas`, spawned four thin (2 px) `Rectangle`
meshes as the top/bottom/left/right edges of the 512×768 game frame at z=−0.1,
plus a small "game frame" text label at the top-left corner. All tagged
`CanvasEntity` for automatic cleanup. Builds clean.

---

### STORY-023: Fix floor position in level editor to match the game

**status:** done
**priority:** medium

#### What
The editor draws a ground line at `GROUND_Y = −200`. In the game, the ground
physics body is positioned at `Transform::from_xyz(0.0, GROUND_Y − GROUND_HALF_HEIGHT * 2.0, 0.0)`
with half-height `GROUND_HALF_HEIGHT = 10`, so the **top surface of the game
ground** is at `y = (GROUND_Y − GROUND_HALF_HEIGHT * 2.0) + GROUND_HALF_HEIGHT`
= `GROUND_Y − GROUND_HALF_HEIGHT` = **−210**.

The editor line at −200 is 10 units above where blocks actually land, causing
placed blocks to appear to float or clip into the ground when previewed in-game.

Fix: move the editor ground line from `GROUND_Y` to `GROUND_Y − GROUND_HALF_HEIGHT`
(i.e. `−210`). Import `GROUND_HALF_HEIGHT` from `single_button_game::constants`
alongside `GROUND_Y`.

#### Why
Accurate floor position lets designers place blocks so their bottom edge rests
exactly on the ground surface, matching what they see in play.

#### Acceptance criteria
- [ ] The ground line in the editor canvas is drawn at y = −210 (not −200)
- [ ] `cargo build` compiles clean
- [ ] No other visual or behavioural changes

#### Context & constraints
- Only `src/bin/level_editor/canvas_screen.rs` → `setup_canvas` needs to change
  (the ground `Transform::from_xyz(0.0, GROUND_Y, 0.0)` line)
- `GROUND_HALF_HEIGHT` is already exported from `single_button_game::constants`;
  update the import at the top of the file to include it
- Do NOT touch game code or the sequence screen

#### Result
Added `GROUND_HALF_HEIGHT` to the constants import in `canvas_screen.rs`.
Changed the ground line spawn from `GROUND_Y` (−200) to
`GROUND_Y − GROUND_HALF_HEIGHT` (−210), matching the game's physics ground
top surface. Builds clean.

---

### STORY-024: Remove adjacent-level preview overlays from the level editor

**status:** pending
**priority:** low

#### What
The editor canvas shows ghost blocks for the previous and next levels (orange and
blue transparent overlays). These were designed for "tower mode" — a feature that
has since been removed. The overlays are now meaningless and add visual clutter.

Remove:
1. The `prev_slots` / `next_slots` fields from `CanvasState`
   (`src/bin/level_editor/state.rs`)
2. All code that populates these fields (likely in the sequence screen or
   wherever `CanvasState` is initialised)
3. The overlay-spawning loops in `setup_canvas`
   (`src/bin/level_editor/canvas_screen.rs`)
4. The `OverlayBlock` component and `PREV_COLOR` / `NEXT_COLOR` / `OVERLAY_GAP`
   constants (if they are no longer used)
5. The bounding-box calculations (`current_bottom`, `current_top`,
   `prev_y_offset`, `next_y_offset`) that existed solely to position the overlays

#### Why
The overlays reference removed tower-mode behaviour and create confusion about
which blocks belong to the level being edited.

#### Acceptance criteria
- [ ] No orange or blue ghost blocks appear in the editor canvas
- [ ] `CanvasState` no longer has `prev_slots` / `next_slots` fields
- [ ] `cargo build` compiles clean with no unused-variable warnings
- [ ] The rest of the editor (block placement, save, sequence screen) is unaffected

#### Context & constraints
- Files to change: `src/bin/level_editor/state.rs`,
  `src/bin/level_editor/canvas_screen.rs`
- Also check `src/bin/level_editor/sequence_screen.rs` and `main.rs` for any
  code that writes to `prev_slots` / `next_slots` — remove those too
- Do NOT touch the game code or `src/playing/`

#### Result
<!-- Agent fills this in when done -->

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
