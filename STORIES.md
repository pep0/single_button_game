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

**status:** pending
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
