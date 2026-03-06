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

### STORY-025: Add per-level score bar

**status:** done
**priority:** high

#### What
Add a vertical score bar on the right edge of the game screen that shows accumulated
block points versus a per-level threshold. Scoring: grey block = 0 pts, yellow = 1 pt,
green = 2 pts. Threshold = number of blocks in the level.

The bar is always visible during play. It fills from the bottom as blocks settle and
earn points. The threshold line marks the goal. When the accumulated score reaches or
exceeds the threshold the fill turns green.

Layout (world space, repositioned each frame via `ScreenShake.base_camera_y`):
- x = +234 (right edge of 512 px viewport)
- Background: 10 × 160 px dark rect `srgba(0.15, 0.15, 0.18, 0.80)` at z = 1.5
- Fill: 10 × `(ratio * 160)` px rect anchored to the bottom of the background
  - Below threshold: gold `srgb(0.85, 0.72, 0.22)`
  - At/above threshold: spring green `srgb(0.38, 0.88, 0.55)`
- Threshold line: 14 × 2 px white-ish rect `srgba(0.95, 0.95, 0.95, 0.70)` at z = 1.6,
  positioned at `bg_bottom_y + 160 * (target as f32 / target as f32)` — i.e. at the
  very top of the background since threshold == target (always full bar height)

Score is accumulated when a block settles in `check_per_block_settle` (when
`rest_secs >= 0.4`): compute `tier: u8` the same way as block colouring
(score ≥ 0.80 → 2, ≥ 0.60 → 1, else 0) and add to `level_score.accumulated`.

#### Why
Players need feedback on how well they are placing blocks mid-level. The bar makes
the scoring system visible and gives a clear goal to chase.

#### Acceptance criteria
- [ ] A vertical bar is visible on the right edge during play
- [ ] Fill grows as blocks settle; colour is gold below threshold, green at/above
- [ ] A threshold line sits at the top of the background (since threshold = num bricks)
- [ ] Bar repositions correctly when the camera moves up for tall towers
- [ ] `cargo build` compiles clean

#### Context & constraints
- New resource `LevelScoreBar { accumulated: i32, target: i32, threshold_reached: bool }`
  in `src/playing/resources.rs`
- New components `ScoreBarBg` and `ScoreBarFill` in `src/playing/components.rs`
- Spawn the two rect entities (background + fill) in `setup::setup_playing`; init the
  resource there too (`LevelScoreBar { accumulated: 0, target: num_slots as i32, .. }`)
- `check_per_block_settle` in `settle.rs` gets `mut level_score: ResMut<LevelScoreBar>`;
  add tier points when `timer.popup_shown` flips to true (same block, same condition)
- Add `update_score_bar` system to `ui.rs`; register it in `mod.rs` alongside
  `update_hearts`
- The fill rect uses a 1×1 unit mesh scaled by the system each frame — same pattern
  as ghost blocks and the slot indicator
- Do NOT touch physics, audio, or any non-playing file

#### Result
Added `LevelScoreBar { accumulated, target, threshold_reached }` resource to
`resources.rs`. Added `ScoreBarBg`, `ScoreBarFill`, `ScoreBarThreshold` components
to `components.rs`. In `setup_playing`, spawned three rect entities (bg 10×160,
fill 10×1 scaled by system, threshold line 14×2) tagged `PlayingEntity`, and init
the resource with `target = num_slots`. In `check_per_block_settle`, added tier
point accumulation (`level_score.accumulated += tier as i32`) alongside the existing
popup spawn. Added `update_score_bar` to `ui.rs`: repositions bar each frame relative
to `shake.base_camera_y`, scales fill height from ratio, shows gold below threshold
and green at/above. Registered system in `mod.rs`. Builds clean.

---

### STORY-026: Particle burst when level score threshold is reached

**status:** done
**priority:** medium

#### What
When `level_score.accumulated` reaches `level_score.target` for the first time during
play (detected inside `update_score_bar`), trigger a one-shot celebration burst of
colorful particles at the bar position.

Add `spawn_celebration_burst` to `src/playing/particles.rs`:
- 20 particles total
- Mix of gold `srgba(1.0, 0.80, 0.20, 0.90)` and spring-green
  `srgba(0.40, 0.95, 0.55, 0.90)` (alternate by particle index)
- Spray upward and outward from `(234.0, bar_fill_top_y)`
- Lifetime: 0.6–1.0 s, radius: 3–7 px

In `update_score_bar`: when threshold is first crossed (`!level_score.threshold_reached`
and `accumulated >= target`), set `threshold_reached = true` and call
`spawn_celebration_burst`. The system already queries `Commands`, `Assets<Mesh>`,
`Assets<ColorMaterial>` for the fill colour update — add them if not already present.

#### Why
Visual confirmation that the player has met the goal, matching the existing smoke
particles for block landings.

#### Acceptance criteria
- [ ] Colorful particles burst from the top of the score bar the moment the threshold
      is reached
- [ ] Burst triggers only once per level (no repeat if more blocks land afterward)
- [ ] Particles fade and disappear naturally (existing `tick_particles` system handles them)
- [ ] `cargo build` compiles clean

#### Context & constraints
- Files: `src/playing/particles.rs` (new function), `src/playing/ui.rs` (`update_score_bar`)
- `spawn_celebration_burst` signature matches `spawn_smoke_burst` for consistency
- The `threshold_reached` flag on `LevelScoreBar` (added in STORY-025) prevents re-firing
- Do NOT change `tick_particles`, smoke burst, or any other system

#### Result
Added `spawn_celebration_burst` to `particles.rs`: 20 particles alternating gold
and spring-green, fanned upward (27°–153°), lifetime 0.6–1.0 s, radius 3–7 px.
In `update_score_bar` (`ui.rs`), detect first threshold crossing
(`!threshold_reached && accumulated >= target`), set the flag, and call
`spawn_celebration_burst` at the top of the bar. Extended function params to include
`Commands`, `Assets<Mesh>`. Builds clean.

---

### STORY-027: Larger eyes proportional to block size, add eyebrows

**status:** done
**priority:** medium

#### What
Two improvements to block face generation in `src/playing/faces.rs`:

**1. Remove the hard 52 px eye-size cap**
Currently `face_unit = ph.min(pw).min(52.0).max(12.0)`. Change the upper cap from
`52.0` to `100.0` so that large blocks can have proportionally larger eyes and mouths.

**2. Add eyebrows**
Spawn two eyebrow entities per face (left and right). Each eyebrow is a flat dark
ellipse (scaled circle) above the corresponding eye:
- Width: `eye_r * 2.2`, Height: `eye_r * 0.32`
- Y position: `eye_y + eye_r * 1.55` (above the eye white)
- X position: `±eye_x` (same x as the eye centres)
- z: eye z + 0.05 (drawn in front of eyes)
- Color: dark `srgba(0.07, 0.07, 0.10, 0.92)` (same as pupils)

Eyebrow rotation (angle in radians, applied via `Quat::from_rotation_z`):
| State    | Left brow                | Right brow              |
|----------|--------------------------|-------------------------|
| falling  | `+0.30` (outer up = fear)| `-0.30`                 |
| grey (0) | `-0.18` (inner up = angry)| `+0.18`                |
| yellow(1)| `0.0` (flat = neutral)   | `0.0`                  |
| green (2)| `+0.18` (outer up = happy)| `-0.18`                |
| tilted   | `0.0` (hidden with eye whites) — set scale to ZERO |

Add `left_brow: Entity` and `right_brow: Entity` fields to `BlockFace`.
Update `update_faces` to set scale and rotation for both brow entities each frame
(same pattern as left_eye / right_eye).

#### Why
Small blocks already have small faces; large blocks should look proportionally bigger
and more expressive. Eyebrows greatly improve readability of emotional expression.

#### Acceptance criteria
- [ ] A large block (≥ 100 px in either dimension) has noticeably larger eyes than before
- [ ] Every block has two visible eyebrows above the eyes
- [ ] Brow angle changes with score tier (angry grey, neutral yellow, happy green)
- [ ] Brows tilt to fear angle while block is falling
- [ ] Brows disappear (scale zero) when block is tilted (X-eyes state)
- [ ] `cargo build` compiles clean

#### Context & constraints
- Only `src/playing/faces.rs` changes
- The `sp` closure already spawns unit-circle children — reuse it for brows
- Brows are updated in `update_faces` alongside left_eye / right_eye, same query
- Do NOT touch `input.rs`, physics, particles, or other files

#### Result
Raised `face_unit` cap from `min(52.0)` to `min(100.0)` in `spawn_face` so large
blocks get proportionally bigger eyes and mouths. Added `left_brow`/`right_brow`
Entity fields plus `brow_y`/`eye_x` geometry fields to `BlockFace`. In
`spawn_face`, spawned two dark flat-ellipse brows at `eye_y + eye_r * 1.55`.
In `update_faces`: when tilted, brows are hidden (scale zero); otherwise, brow
angle encodes expression — fear (+0.30 outer-up) while falling, angry inner-up
(-0.18) for grey, flat (0.0) for yellow, happy outer-up (+0.18) for green.
Builds clean.

---

### STORY-001: Example — Add score display to HUD

**status:** done
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
Added `ScoreText` and `ScoreTextShadow` marker components. Spawned a
"Score: X/Y" text entity with drop shadow in top-right corner during
`setup_playing`. `update_score_text` system reads `LevelScoreBar` and
repositions text each frame. Resets on new level. Builds clean.

---

### STORY-031: Per-block color variation within score tier

**status:** done
**priority:** medium

#### What
Each block currently renders with a flat color determined solely by its score tier
(green / yellow / grey). Add per-block hue/saturation/brightness variation so every
block looks individually distinct while still reading as its tier.

The block's "mouth mask" (the crescent mesh that gives the impression of a smile or
frown) must also be tinted with the same randomised color so face and body stay
visually coherent.

#### Why
All blocks of the same tier look identical, making the tower look repetitive.
Per-block variation adds visual richness without breaking the semantic color coding.

#### Acceptance criteria
- [ ] Each spawned block is assigned a unique color at spawn time (not per-level)
- [ ] The color stays within the block's tier palette:
  - Green tier: hue roughly 100-160°, saturation 45-75%, lightness 40-65%
  - Yellow tier: hue roughly 40-65°, saturation 60-85%, lightness 45-65%
  - Grey tier: desaturated, lightness 35-60%, slight hue drift allowed (±20°)
- [ ] The mouth mask mesh is tinted to match the block's body color
- [ ] Color is fixed at spawn — it does not change while the block falls or settles
- [ ] No visible seam between block body and mouth mask

#### Context & constraints
- Block spawning: `src/playing/spawn.rs` (or wherever `BlockSlot` entities are
  created with their `ColorMaterial`)
- Face/mouth rendering: `src/playing/faces.rs` — the mouth mask is a separate child
  mesh entity; its `ColorMaterial` must be updated at spawn alongside the body
- Score tier is available on the block component (0 = grey, 1 = yellow, 2 = green)
- Use `rand` (already a dependency) for the per-block randomisation; seed from
  Bevy's `rand` resource or a simple thread_rng
- Do not change eye or pupil colors — those stay dark

#### Result
Replaced fixed block colors with HSL-based per-block variation using
deterministic LCG hash. Each tier generates colors within its palette range.
Mouth mask now receives the actual body color to avoid seams. Border color
auto-derived by darkening fill 40%. Builds clean.

---

### STORY-032: Phone viewport reference frames in level editor canvas

**status:** done
**priority:** low

#### What
The canvas editor currently shows a single dashed outline for the 512 × 768 game
viewport. Add two additional phone-size reference outlines so designers can see how
a level will look on real devices:

- **iPhone 17 Pro Max**: 440 × 956 (largest common phone)
- **iPhone 14**: 390 × 844 (mainstream mid-size phone)

These should appear as subtle outlines only (no fill), clearly distinct from the
existing game-frame box, with a small label identifying each size.

#### Why
The game is deployed as a WASM build played on phones. Designers have no visual cue
for whether their block layouts are visible on real device screen sizes.

#### Acceptance criteria
- [ ] Two new outline rectangles are drawn in the canvas editor at the correct pixel
  dimensions (centred on the canvas origin, same as the existing 512 × 768 frame)
- [ ] Each outline has its own distinct colour (different from the existing
  `FRAME_COLOR` and from each other) at low opacity so they don't dominate
- [ ] Each outline has a small label (e.g. `"440 × 956"`) near its top-left corner
- [ ] Outlines are visible behind blocks but don't interfere with editing
- [ ] The existing 512 × 768 "game frame" outline and its label are unchanged

#### Context & constraints
- Canvas setup: `src/bin/level_editor/canvas_screen.rs`, function `setup_canvas`
- Existing frame drawn with four thin `Rectangle` meshes tagged `CanvasEntity` —
  follow the same pattern
- Outlines should be tagged `CanvasEntity` so they are despawned on screen exit
- Keep z-index below editing elements (use z < 0 like the existing frame at z = -0.1)

#### Result
Added iPhone 17 Pro Max (440x956, teal) and iPhone 14 (390x844, amber)
reference outlines to canvas editor. Each uses four thin Rectangle meshes
tagged CanvasEntity at z=-0.15, with a small label near top-left corner.
Existing 512x768 frame unchanged. Builds clean.

---

### STORY-033: Fix WASM audio autoplay policy

**status:** done
**priority:** high

#### What
Web browsers block audio until the user has interacted with the page. In the WASM
build the synthesised block sounds may be silently dropped on the first interaction
because the audio context is still suspended. Fix this so sounds play correctly from
the first tap/click onward.

#### Why
Players on mobile and desktop web hear no sound at all if the browser's autoplay
policy blocks the audio context from starting. This makes the game feel broken.

#### Acceptance criteria
- [ ] The first block-drop or collision sound plays correctly after the first user
  interaction (tap or key press) — no silent first interaction
- [ ] Works in Chrome and Safari on iOS (the most restrictive browsers)
- [ ] No audible glitch or pop on the first sound
- [ ] Desktop build is unaffected

#### Context & constraints
- Audio system: `src/playing/audio.rs` — sounds are synthesised PCM WAV blobs,
  played via Bevy's `AudioPlayer`
- Bevy's `WinitPlugin` can resume the audio context on the first `winit` event;
  check if `ResumeAudio` or a custom JS interop via `wasm-bindgen` is needed
- The `index.html` may need a one-time event listener that calls
  `AudioContext.resume()` on the first `pointerdown` or `keydown` event, bridged
  to Bevy if Bevy doesn't handle it automatically
- `cfg\!(target_arch = "wasm32")` can be used to gate WASM-only code
- Do not add new audio files; the synthesised sounds must continue working

#### Result
JS patch in index.html wraps AudioContext constructor, tracks all instances,
and resumes suspended ones on pointerdown/touchstart/keydown/mousedown.
Rust-side WasmAudioPlugin (src/wasm_audio.rs) provides backup via js_sys::eval
on first Bevy-detected input. Added web-sys AudioContext/AudioContextState
features, wasm-bindgen, js-sys deps. Desktop unaffected. Builds clean.

---

### STORY-034: Ensure all text fits the screen on small viewports

**status:** done
**priority:** medium

#### What
Several text elements in the game are positioned or sized with fixed world-space
values designed for a 512 × 768 viewport. On smaller phones (390 × 844 and below)
or in landscape orientation some text overflows or is clipped. Audit all text and
make it fit.

#### Why
The WASM build is played on phones. Unreadable or missing text degrades the
experience significantly on small screens.

#### Acceptance criteria
- [ ] **In-game HUD**: level name + block counter text does not overflow the screen
  width on a 390 px wide viewport; truncate or wrap if needed
- [ ] **Level-complete overlay**: 52 pt font may need to scale down on narrow screens
  so it fits within the viewport width
- [ ] **Failed screen**: all text lines visible and readable at 390 × 844 and
  375 × 667; long prompt text is wrapped
- [ ] **Stats screen**: all text fits within the screen width with the 460 px wrap
  constraint adjusted if the viewport is narrower than 460 px
- [ ] **Level editor** (Sequence and Canvas screens): header, hints, HUD line, and
  status text remain readable and on-screen at common desktop resolutions
- [ ] Text is never clipped by the screen edge (a small inset margin is acceptable)

#### Context & constraints
- In-game text: `src/playing/ui.rs`, `src/playing/setup.rs`, `src/failed.rs`,
  `src/stats.rs`
- Level editor text: `src/bin/level_editor/sequence_screen.rs`,
  `src/bin/level_editor/canvas_screen.rs`
- Query the primary `Window` for `width()` / `height()` (pattern already used in
  `update_hearts` and `update_score_bar` after STORY responsive-HUD work)
- `TextBounds::new_horizontal(width)` can be used to wrap text; combine with
  `TextLayout` for alignment
- Keep font sizes readable — prefer clamping position/wrapping width over shrinking
  fonts below ~14 pt

#### Result
Audited all text across HUD, failed screen, stats screen, level-complete
overlay, and both editor screens. Added viewport-aware font scaling (min ~14pt),
TextBounds wrapping based on actual window width, and 16px inset margins.
Stats screen replaced static WRAP_WIDTH with runtime value. Builds clean.
