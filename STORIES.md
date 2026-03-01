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

### STORY-014: Block faces — eyes and mouth that react to score and physics

**status:** done
**priority:** medium

#### What
Each placed block gets a procedurally generated face (two eyes + a mouth) drawn
as small `Mesh2d` shapes (circles/ellipses via rectangles). The face expression
is determined by the block's score and changes when the block is falling.

**Expression rules:**
| Situation | Eyes | Mouth |
|---|---|---|
| Score ≥ 0.80 (green) | Normal round, wide apart | Big arc smile |
| Score 0.60–0.79 (yellow) | Slightly squinted | Flat or small smile |
| Score < 0.60 (grey) | Half-closed / droopy | Downward arc frown |
| Falling / not yet settled | Wide open (panic) | Open oval "O" mouth |
| Crossed eyes variant | One eye shifted inward | Any mouth |

**Random variation:** Eye spacing, pupil offset, and mouth curve are seeded
randomly per block so no two blocks look identical.

**Extra ideas to consider implementing:**
- Eyebrows: two thin rectangles above the eyes, angle changes with expression
- Sweat drop: a small teardrop shape on the side for the grey/bad-score block
- Stars/spirals above a block that lands very badly (score < 0.40)
- Pupils track the direction of gravity / velocity while falling

#### Why
Personality makes the blocks feel alive and gives the player emotional feedback
about how well they're doing — without adding any extra text.

#### Acceptance criteria
- [ ] Every placed block has two eyes and a mouth spawned as `Mesh2d` children
- [ ] Expression matches the score tier (green/yellow/grey)
- [ ] Face changes to "panic" expression while the block is still falling
      (detect via `RigidBody` linear velocity or a `BlockSettleTimer`)
- [ ] Random variation: no two blocks have identical faces
- [ ] Faces scale correctly with the block (use relative child transforms)
- [ ] `cargo build` compiles clean

#### Context & constraints
- Relevant file: `src/playing/input.rs` — block entity is spawned in `production_input`
- `BlockSettleTimer` in `src/playing/components.rs` tracks settling; use it to
  detect "still falling" state
- Keep face meshes as children of the block entity so they move/rotate with it
- Use `Mesh2d` + `ColorMaterial` (same pattern as existing border/fill children)
- Face z-offset: `0.2` above the fill rectangle (which is at z=0.0 relative to block)
- Do NOT modify physics, scoring, or any other system

#### Result
Created `src/playing/faces.rs` with `BlockFace` component, `spawn_face` helper,
and `update_faces` system. Each spawned block gets 5 child unit-circle entities
(left/right eye white, left/right pupil dark, mouth dark) stored by entity ID
in `BlockFace`. `update_faces` runs every frame and sets scales to match the
current expression:

- **Falling / panic** (`rest_secs < 0.35` or `speed > 60`): wide-open eyes,
  tall O-shaped mouth.
- **Green (≥ 0.80)**: normal round eyes, wide flat grin.
- **Yellow (0.60–0.79)**: slightly squinted eyes, smaller smile.
- **Grey (< 0.60)**: droopy (very flattened) eyes, narrow flat mouth.

~20 % of blocks get crossed eyes (inner pupil offset). Eye spacing has slight
random variation per block. All face entities tagged `PlayingEntity` so they
clean up with the rest. Registered `faces::update_faces` in the UI update
batch. Builds clean.

---

### STORY-013: Particle effects — smoke on block landing, dust on impact

**status:** done
**priority:** medium

#### What
Add simple particle effects triggered by game events. Start with **landing smoke**
when a block settles, then consider the extras below.

**Effects to implement:**

1. **Landing smoke** (required): When a block's `BlockSettleTimer` completes
   (block is considered settled), burst 6–10 small grey/white circles outward
   from the block's bottom edge. Each particle: random direction (mostly
   sideways/downward), random size (4–10 px), fades out over 0.4–0.8 s,
   drifts with slight upward float. Use `Mesh2d` circles.

2. **Impact flash** (optional): A brief white flash ring that expands then fades
   at the moment of first collision (use `CollisionEventsEnabled` which is
   already on block entities).

3. **Score sparkles** (optional): On a green (≥ 0.80) score, emit 4–6 small
   yellow/gold dots that float upward and fade, similar to the score popup.

4. **Dust on bad landing** (optional): On grey (< 0.60) score, a few dark
   particles fall downward.

**Extra ideas:**
- Trailing particles while the block is falling (subtle, low opacity)
- Ground puff when the first block lands on the ground

#### Why
Juice. Particle feedback makes impacts feel satisfying and communicates game
events viscerally without UI.

#### Acceptance criteria
- [ ] Landing smoke burst spawns when a block settles (after `BlockSettleTimer`)
- [ ] Particles fade out and despawn automatically (no memory leak)
- [ ] Effect does not interfere with physics or game logic
- [ ] `cargo build` compiles clean
- [ ] No significant frame-rate drop (keep total live particle count < 60)

#### Context & constraints
- Particle entities should have `PlayingEntity` so they're cleaned up on level end
- Each particle: `Mesh2d` circle + `ColorMaterial`, `Transform`, and a custom
  `Particle { velocity: Vec2, lifetime: f32, age: f32 }` component
- Add a `tick_particles` system in `src/playing/` that moves particles each frame
  and despawns those whose `age >= lifetime`
- Settle detection: look at `src/playing/settle.rs` — the per-block settle check
  fires an event or transitions state; hook into that moment
- Do NOT use any external particle crate — keep it in pure Bevy `Mesh2d`

#### Result
Created `src/playing/particles.rs` with `Particle` component, `prand` helper,
`spawn_smoke_burst`, and `tick_particles`. Each settled block emits 8 grey
circle particles from its bottom edge: spread across the block width, velocity
fans outward with slight upward float, fades alpha to 0 over 0.45–0.9 s.
Smoke triggers in `check_per_block_settle` at 0.4 s of rest (same as score
popup). All particles tagged `PlayingEntity` so they're cleaned up on level
exit. Registered `particles::tick_particles` in the UI update batch. Builds clean.

---

### STORY-012: Switch block texture to PNG with transparent background and correct corner clamping

**status:** done
**priority:** high

#### What
The textured blocks currently load `assets/blocks/green_block.jpg`. Two problems:

1. **No transparency support.** JPEG has no alpha channel, so the block renders
   with a solid rectangle behind the image instead of showing whatever is
   underneath (e.g. the ghost outlines). Switch to `assets/images/single_block.png`,
   which is a PNG and can carry a transparent background.

2. **Corners must not scale.** With 9-slice scaling the `max_corner_scale` field
   caps how much corners can shrink when the block is smaller than 2 × border.
   Verify the `SLICE_BORDER` constant matches the actual corner region size in
   `single_block.png` and that `max_corner_scale: 1.0` is set, so corners are
   always rendered at 1:1 pixel size.

#### Why
A JPEG block texture leaves an ugly opaque rectangle over ghost outlines and other
visual layers. The PNG with transparency gives a clean look. Incorrect corner scaling
makes the decorative border art (rounded corners, rivets, etc.) stretch or squash.

#### Acceptance criteria
- [ ] Block texture path changed from `"blocks/green_block.jpg"` to
      `"images/single_block.png"` in `src/playing/input.rs` (both the
      production-rect spawn and the placed-block child sprite)
- [ ] `SLICE_BORDER` value matches the actual corner region in `single_block.png`
      (measure the image; update the constant if needed)
- [ ] `max_corner_scale: 1.0` is set on both `TextureSlicer` instances so
      corners never scale up or down
- [ ] The `"jpeg"` Bevy feature added in the previous story can be removed from
      `Cargo.toml` if no other asset uses JPEG — remove it to keep the binary lean
- [ ] `cargo build` compiles clean
- [ ] In-game blocks show the PNG texture with transparent areas visible through
      the block edges, and corners look crisp at any block size

#### Context & constraints
- Relevant file: `src/playing/input.rs` — two `asset_server.load(…)` calls and
  the `SLICE_BORDER` constant at the top of the file
- `Cargo.toml` — `bevy` features list; remove `"jpeg"` if unused
- Asset: `assets/images/single_block.png` — inspect the image to confirm the
  corner region size before setting `SLICE_BORDER`
- `max_corner_scale: 1.0` means "never scale corners beyond their natural
  pixel size"; this is already in the code but double-check it is present on
  both spawns
- Do NOT change game logic, physics, scoring, or any other visual system

#### Result
Switched both `asset_server.load(…)` calls in `src/playing/input.rs` from
`"blocks/green_block.jpg"` to `"images/single_block.png"`. Updated `SLICE_BORDER`
from 34.0 to 40.0 to match the corner art extent in the new 566×150 RGBA PNG
(rounded corners + hatching extend ~40 px from each edge). Removed `"jpeg"` from
Bevy's feature list in `Cargo.toml` since no asset requires it anymore.
`max_corner_scale: 1.0` was already set on both `TextureSlicer` instances.
Builds clean.

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
