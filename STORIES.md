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

### STORY-019: Hide eye whites when block is tilted (X-eyes show bare)

**status:** done
**priority:** low

#### What
When a block's tilt exceeds 25° (the threshold that already switches pupils to
X-marks), the white eye-socket circles should also be hidden so only the raw dark
✕ bars are visible. Right now the white backing remains, making the X-eyes look
like white circles with an X drawn on them instead of bare crossed-out eyes.

The fix is entirely in `update_faces` in `src/playing/faces.rs`: in the `tilted`
branch, scale `left_eye` and `right_eye` to `Vec3::ZERO`; restore them to their
normal `(eye_sx, eye_sy, 1.0)` scale in the non-tilted branch.

#### Why
Without whites the X-eyes read as proper cartoon "KO" eyes rather than normal eyes
with an overlay. The visual effect is much more legible and funny.

#### Acceptance criteria
- [ ] While a block's tilt > 25°, the white eye circles are invisible (scale 0)
      and only the dark ✕ bars show
- [ ] When tilt drops back below 25°, the white circles reappear normally
- [ ] No change to the panic, grin, or flat expressions on non-tilted blocks
- [ ] `cargo build` compiles clean

#### Context & constraints
- Only `update_faces` in `src/playing/faces.rs` needs to change
- The `tilted` boolean and the eye entity references (`face.left_eye`,
  `face.right_eye`) are already in scope — just move/add scale assignments
- Do NOT touch `spawn_face` or any other file

#### Result
Moved the eye-white scale assignment into the `tilted` / `else` branches in
`update_faces`. When `tilted`, `left_eye` and `right_eye` are scaled to
`Vec3::ZERO`; the ✕ bars are the only visible feature. In the `else` branch they
are restored to `(eye_sx, eye_sy, 1.0)` as before. No other changes. Builds clean.

---

### STORY-020: Idle animations — blinking and looking around

**status:** done
**priority:** medium

#### What
Settled blocks should occasionally blink and glance sideways/up/down to feel alive.
Both animations only play when the block is **not** falling (i.e. `!falling`).

**Blinking:**
- Each block blinks on an independent random schedule (~every 2–5 s).
- A blink lasts ~0.12 s: eye height briefly scales to nearly 0 (e.g. `eye_sy * 0.08`)
  then opens back up. Just the eyes — pupils and mouth are unaffected.

**Looking around:**
- Each block occasionally glances in a random direction (~every 3–7 s), holds the
  look for ~0.4–0.8 s, then returns to centre.
- Implement by offsetting all four pupil entities (`left_pupil`, `right_pupil`,
  `left_pupil2`, `right_pupil2`) by a small translation delta in x and y.
  Max offset: ±`eye_r * 0.45` in each axis, clamped so pupils stay inside the
  white eye circle.

**State to add to `BlockFace`:**
```
blink_timer:    f32,   // counts down; blink triggers at 0
blink_age:      f32,   // how long the current blink has been open (0 = not blinking)
look_timer:     f32,   // counts down; new glance direction chosen at 0
look_duration:  f32,   // how long to hold the current glance
look_age:       f32,   // how long we've been looking in the current direction
look_dx:        f32,   // current pupil x offset
look_dy:        f32,   // current pupil y offset
```

Initialise all to 0 in `spawn_face`. Use `prand` with high seed offsets (30+)
seeded with `seed + frame_count` or similar to vary timing between blocks.

**Advancing timers** requires `Res<Time>` in `update_faces`. Add it.

#### Why
Static faces feel like stickers. Blinking and glancing make the blocks feel like
living characters reacting to their situation.

#### Acceptance criteria
- [ ] Settled blocks visibly blink (eye height briefly collapses) at irregular
      intervals, roughly every 2–5 s
- [ ] Settled blocks occasionally shift their pupils sideways or up/down and then
      return to centre
- [ ] Neither animation plays while the block is falling (panic expression stays
      still)
- [ ] Blocks blink and look independently of each other (not all in sync)
- [ ] `cargo build` compiles clean

#### Context & constraints
- Only `src/playing/faces.rs` needs to change (`BlockFace` struct + `update_faces`)
- `update_faces` already has access to `time: Res<Time>` — add it to the param list
- Use `prand` for initial timer values; for ongoing variation seed with something
  that changes per block (e.g. `seed.wrapping_add(30 + blink_count * 7)`)
- The look offset must be applied to the pupil entity **translation** (x/y), not
  scale — store the base pupil positions (`left_pupil_x`, `left_pupil_y`, etc.)
  in `BlockFace` at spawn time so offsets can be applied relative to them
- Do NOT animate during panic (`falling == true`) or while the tilt X-eyes are
  showing (`tilted == true`)
- Do NOT touch any file other than `src/playing/faces.rs`

#### Result
Added 14 new fields to `BlockFace`: `seed`, base pupil positions (`left/right_pupil_x/y`),
blink state (`blink_timer`, `blink_age`, `blink_count`), and look-around state
(`look_timer`, `look_duration`, `look_age`, `look_dx`, `look_dy`, `look_count`).
`spawn_face` initialises timers with staggered values via `prand(seed+30/31)` so
blocks don't blink or glance in sync. `update_faces` gained `time: Res<Time>` and
`mut block_query`. Each frame when `!falling && !tilted`: blink timer counts down,
triggers a 0.12 s squint (`eye_sy * 0.08`), then restarts with a new `prand`
interval (2–5 s); look timer counts down, picks a random `±eye_r * 0.45` offset,
holds for 0.4–0.8 s, returns to centre, waits 3–7 s, repeats. Offset is applied
to pupil entity `translation.x/y` each frame relative to stored base positions.
All animations reset when block falls or tilts. Builds clean.

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
