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

### STORY-011: Wrap stats screen text so nothing clips outside the window

**status:** done
**priority:** medium

#### What
All `Text2d` entities spawned in `setup_stats` (`src/stats.rs`) should be
constrained to the window width so long strings wrap onto a second line instead
of being clipped at the left/right edges. The window is 512 px wide, so the safe
wrappable width is **460 px** (25 px margin each side).

Lines that can overflow at their current font sizes:
- `"Levels Completed: {rounds_played}"` — font 26 (reported clipping)
- `"Average Accuracy: {avg:.0}%"` — font 30
- `"NEW BEST!  (previous: {prev_best:.0}%)"` — font 24
- `"Press SPACE to return to menu"` — font 22
- `"All Levels Complete!"` — font 40 (shortest string but largest font)

#### Why
On the 512×768 portrait window some stats strings extend beyond the visible
area, cutting off information the player needs to see (number of completed
levels, accuracy score, high-score comparison).

#### Acceptance criteria
- [ ] Every text entity in `setup_stats` has a `TextBounds` width of 460 px so
      long strings wrap rather than overflow
- [ ] Wrapped text is centre-aligned (consistent with the rest of the screen)
- [ ] Vertical positions may need small adjustments if wrapping causes line-height
      overlap between adjacent text entities — fix if needed
- [ ] `cargo build` compiles clean
- [ ] No other screen (menu, playing HUD, failed, editor) is touched

#### Context & constraints
- Only `src/stats.rs` `setup_stats` needs to change
- In Bevy 0.18, add two extra components to each `commands.spawn(…)` bundle:
  - `TextLayout::new(JustifyText::Center, LineBreak::WordBoundary)` — enables
    word-wrap and centres each line
  - `TextBounds::from(Vec2::new(460.0, f32::INFINITY))` — sets the wrap width
    (infinite height so text is never clipped vertically)
  - Verify the exact API against `bevy::text` before implementing; field names
    may differ slightly between patch releases
- The `"Best: {prev:.0}%"` branch (short string, font 20) is unlikely to clip
  but should receive the same treatment for consistency
- Do NOT change font sizes, colours, or transform positions unless vertical
  overlap forces it

#### Result
Added `use bevy::text::TextBounds;` (not in prelude) and two shared constants
`WRAP_WIDTH = 460.0` and `WRAP_LAYOUT = TextLayout { justify: Justify::Center,
linebreak: LineBreak::WordBoundary }`. All six `commands.spawn(…)` bundles in
`setup_stats` now include `TextBounds::new_horizontal(WRAP_WIDTH)` and
`WRAP_LAYOUT`. No font sizes, colours, or positions were changed. Builds clean.

---

### STORY-010: Remove tower mode and all related code

**status:** done
**priority:** medium

#### What
Delete the "tower mode" feature entirely. Tower mode was an experimental variant
where completed blocks persisted across levels as a growing frozen stack, and new
levels spawned their blueprint above the existing stack. Remove every trace of it:
the feature toggle resource, the frozen-block conversion logic, the blueprint Y-offset
calculation, and the menu key binding.

#### Why
Tower mode is unused and adds complexity to the setup, cleanup, and input systems.
Removing it simplifies the codebase and eliminates the `FrozenTowerBlock` component
that is re-exported from the playing module into the menu.

#### Acceptance criteria
- [ ] `TowerModeActive` resource is deleted from `src/state.rs`
- [ ] `FrozenTowerBlock` component is deleted from `src/playing/components.rs`
- [ ] `pub use components::FrozenTowerBlock;` re-export removed from `src/playing/mod.rs`
- [ ] `setup_playing` no longer has `tower_mode` or `frozen_query` params; the
      blueprint Y-offset block (the `if tower_mode.is_some()` section) is deleted
- [ ] `cleanup_playing` no longer has `tower_mode` or `tower_block_query` params;
      the `FrozenTowerBlock` conversion block is deleted
- [ ] `production_input` in `src/playing/input.rs` no longer has `tower_mode` param;
      the `if tower_mode.is_none()` guard is removed and `PlayingEntity` is always
      inserted on the spawned block entity
- [ ] `src/menu.rs` no longer imports `FrozenTowerBlock` or `TowerModeActive`; the
      `frozen_query` param and its despawn loop are removed; the `T` key handler
      (`KeyCode::KeyT` → insert `TowerModeActive`) is removed
- [ ] `cargo build` compiles clean with no unused-import warnings

#### Context & constraints
- `src/state.rs` — `TowerModeActive` is a marker resource (`#[derive(Resource)] pub struct TowerModeActive;`)
- `src/playing/components.rs` — `FrozenTowerBlock { height: f32 }` (~line 38)
- `src/playing/mod.rs` — `pub use components::FrozenTowerBlock;` (~line 9)
- `src/playing/setup.rs`:
  - `setup_playing` params: remove `tower_mode: Option<Res<TowerModeActive>>` and
    `frozen_query: Query<(&Transform, &FrozenTowerBlock)>`; delete the
    `if tower_mode.is_some() { … }` blueprint-offset block (~lines 57-87)
  - `cleanup_playing` params: remove `tower_mode: Option<Res<TowerModeActive>>` and
    `tower_block_query: Query<(Entity, &TowerBlockDims), With<TowerBlock>>`; delete
    the `if tower_mode.is_some() { … }` conversion block (~lines 183-194)
- `src/playing/input.rs` — remove `use crate::state::TowerModeActive`, the
  `tower_mode` param from `production_input`, and the `if tower_mode.is_none()`
  guard (~line 143); the block entity must always receive `PlayingEntity`
- `src/menu.rs` — remove `use crate::playing::FrozenTowerBlock`,
  `TowerModeActive` from the state import, the `frozen_query` param and its
  `for entity in &frozen_query { commands.entity(entity).despawn(); }` loop,
  `commands.remove_resource::<TowerModeActive>()`, and the entire
  `if keyboard.just_pressed(KeyCode::KeyT)` block in `menu_input`
- `TowerBlock`, `TowerBlockDims`, and `BlockSettleTimer` are **not** tower-mode
  specific — they are general playing-state components used by settle, audio, and
  UI systems. Do NOT remove them.
- Do NOT touch the level editor or any other game state.

#### Result
Deleted `TowerModeActive` from `src/state.rs` and `FrozenTowerBlock` from
`src/playing/components.rs`. Removed the `pub use components::FrozenTowerBlock`
re-export from `src/playing/mod.rs`. Stripped `tower_mode` and `frozen_query`
params plus the blueprint Y-offset block from `setup_playing`; stripped
`tower_mode` and `tower_block_query` params plus the frozen-block conversion
block from `cleanup_playing`. Removed `TowerModeActive` import and `tower_mode`
param from `production_input` — `PlayingEntity` is now always inserted on the
block entity. Cleaned up `src/menu.rs`: removed `FrozenTowerBlock`/
`TowerModeActive` imports, `frozen_query` param and despawn loop, the
`remove_resource` call, and the `T` key handler. Also dropped the now-needless
`mut` on `blueprint` in `setup_playing`. Builds clean, no warnings.

---

### STORY-009: Fix hearts and post-level text clipping outside the visible window

**status:** done
**priority:** high

#### What
Two related visibility bugs introduced when the window was resized to 512×768 (portrait):

1. **Hearts (lives display) are off-screen.** They are positioned at
   `x = -360 + i * 22` (i = 0, 1, 2), giving x values of -360, -338, -316.
   The window is 512 px wide, so the visible x range is **-256 to +256**. All
   three hearts are outside the left edge and never visible.

2. **Post-level score popups and the "Level X Complete!" overlay can appear
   outside the visible area.** Score popups are spawned at the placed block's
   world position and float upward. When the camera has scrolled up to follow a
   tall tower, popups spawned near the top of the structure can start at or
   above the top edge of the viewport and immediately float further off-screen.
   The level-complete overlay is updated to `cam_y + 40.0` each frame, which
   keeps it vertically centered on camera, but its vertical position may still
   sit above the visible area when `cam_y` is large.

#### Why
Players cannot see how many lives they have left, and they miss the per-block
accuracy scores and the level-complete confirmation that reward good play.

#### Acceptance criteria
- [ ] All three hearts are fully visible in the top-left corner of the window
      during play (must remain within the -256..+256 x range)
- [ ] Hearts continue to follow the camera vertically (already done via
      `update_hearts`)
- [ ] Per-block score popups (e.g. "87%") are visible on screen when they
      appear — clamp or offset their spawn y so they start within the camera's
      current view, regardless of how high the camera has scrolled
- [ ] The "Level X Complete!" overlay text is fully on screen for its entire
      fade-in / hold / fade-out animation
- [ ] `cargo build` compiles clean, no regression in game flow

#### Context & constraints
- **Hearts position:** `src/playing/ui.rs` `update_hearts` (and the initial
  spawn in `src/playing/setup.rs` ~line 160).
  Current: `transform.translation.x = -360.0 + heart.0 as f32 * 22.0;`
  Fix: shift x right so all hearts land within the visible range. A good
  anchor is near the top-left: x ≈ -230 for the first heart, stepping +22 per
  heart → -230, -208, -186. Update both the spawn transform and the per-frame
  `update_hearts` assignment.
- **Score popup spawn position:** `src/playing/settle.rs` `check_per_block_settle`.
  Popups are spawned at `transform.translation.y + ph / 2.0 + 10.0` in world
  space. To keep them on-screen, clamp the spawn y to at most
  `camera_y + VIEW_HALF_HEIGHT - 40.0` (where `VIEW_HALF_HEIGHT = 384.0` for
  the 768 px tall window). This requires reading the camera transform in
  `check_per_block_settle` (add `camera_query: Query<&Transform, With<Camera2d>>`
  as a param) or passing the clamped y through a resource.
- **Level-complete overlay:** `src/playing/ui.rs` `animate_score_popups` spawns
  it at `cam_y + 40.0`. Verify this stays within the upper half of the screen;
  adjust the offset if needed so the text doesn't clip at the top.
- Window: 512×768 → half extents **256 × 384**. `GROUND_Y = -200`.
- Do NOT change any physics, blueprint, or game-logic code — visual/UI only.
- Do NOT touch the level editor screens.

#### Result
Hearts x anchor shifted from -360 to -230 in both the initial spawn
(`src/playing/setup.rs`) and the per-frame `update_hearts` (`src/playing/ui.rs`),
placing all three hearts at x = -230, -208, -186 — within the ±256 visible range.
Score popups now clamp their spawn y to `cam_y + 304` (80 px from the top edge)
so they always start within the viewport and have room to float without clipping.
Level-complete overlay was already camera-relative and within bounds — no change
needed. Builds clean.

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
