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

### STORY-015: Increase face variety — eye size, spacing, and wide-block scaling

**status:** done
**priority:** medium

#### What
Make block faces feel more distinct and lively by adding more random variation and
by tying eye geometry to the block's physical dimensions:

1. **Wide-block eye spacing:** Eyes should be placed further apart on wider blocks.
   Currently `eye_x` is a fixed fraction of `face_unit = ph.min(pw).min(52).max(12)`.
   Instead, let `face_unit` (or a separate `span` variable) scale with `pw` so that
   on a very wide block the eyes sit near the left and right thirds, not bunched in
   the centre.

2. **Random eye size:** `eye_r` should vary per block in roughly a ±25 % band (e.g.
   `face_unit * (0.09 + prand(seed+20) * 0.05)` instead of a fixed coefficient).
   `pupil_r` should remain proportional to `eye_r`.

3. **Random mouth width:** `mouth_w` should have a ±20 % random factor so settled
   smiles / frowns differ noticeably between blocks.

4. **Random vertical offset for eyes:** Add a small random vertical jitter
   (`± face_unit * 0.05`) to the eye Y position so high/low-browed faces appear.

All changes stay within `src/playing/faces.rs` `spawn_face`.

#### Why
Currently most faces look very similar because the geometry is almost deterministic.
More variation gives each block a unique personality and makes the tower visually
interesting.

#### Acceptance criteria
- [ ] On a wide block (width > 100 px) the eyes are noticeably further apart than
      on a narrow block of the same height
- [ ] Two blocks of the same size placed back-to-back have visibly different eye
      sizes (one pair clearly larger than the other roughly 1 in 3 times)
- [ ] No regression in existing expressions (panic, grin, flat) — they still read
      correctly after the geometry changes
- [ ] `cargo build` compiles clean

#### Context & constraints
- Only `src/playing/faces.rs` `spawn_face` needs to change
- `pw` and `ph` (block width/height) are already passed in — use them directly
- Keep the crossed-eyes variant (20 % chance) working after the refactor
- The `prand` helper is already in the file; use new seed offsets (20+) to avoid
  collisions with existing randomness
- Do NOT touch `update_faces`, `production_input`, or any other file

#### Result
Eye spacing now scales with `pw` (24–32 % of block width from centre) so wide
blocks have naturally wide-set eyes. Eye radius varies ±~25 % per block via
`prand(seed+20)` (range: `face_unit * 0.09–0.14`). Mouth width varies ±~20 %
via `prand(seed+21)`. Eye Y position gets a small random vertical jitter of ±5 %
of `face_unit` via `prand(seed+22)` for high/low-brow variety. Crossed-eyes
variant continues to work (uses the new variable `eye_r`). No changes to
`update_faces` or any other file. Builds clean.

---

### STORY-016: Show a smile on accurate blocks

**status:** done
**priority:** medium

#### What
Blocks that score well (green tier, score ≥ 0.80) should display a visible curved
smile rather than just a wide flat line. Implement a proper arc/curve smile using
one of these approaches (pick the simplest that compiles):

- **Option A (recommended):** Spawn an extra thin ellipse (white) positioned
  slightly above the dark mouth ellipse so only the bottom half of the dark shape
  peeks out, creating a "D on its side" crescent smile — achieved purely with
  overlapping circles and no custom mesh.
- **Option B:** Use a `RegularPolygon` or a custom `Mesh` built with
  `Mesh::new(PrimitiveTopology::TriangleList, …)` to draw a thin arc sector.

For yellow tier (0.60–0.79) the existing flat small-smile shape is acceptable; no
change needed there.

Grey tier (< 0.60) should get a subtle downward arc frown using the same crescent
trick: place a white ellipse slightly below the dark mouth ellipse.

#### Why
The current "smile" is just a flat wide ellipse — it reads as a blank expression.
A real curved mouth is the single most impactful change to make faces feel expressive.

#### Acceptance criteria
- [ ] Green-tier blocks display a clear curved upward smile after settling
- [ ] Grey-tier blocks display a visible downward frown arc after settling
- [ ] Panic expression (falling) is unchanged — still shows the O-mouth
- [ ] The smile/frown appears and disappears correctly when expression transitions
      (e.g. block settles from panic to green)
- [ ] `cargo build` compiles clean

#### Context & constraints
- Relevant file: `src/playing/faces.rs`
- `BlockFace` currently stores 5 entity IDs; add a 6th (`mouth_mask: Entity`) for
  the masking white ellipse. Store it as `Option<Entity>` since yellow tier doesn't
  need it
- `update_faces` must also move/scale the mask entity each frame to stay aligned
  with the mouth
- `spawn_face` already has `commands`, `meshes`, `materials`, `parent` — use them
- Do NOT change scoring, physics, or any other file

#### Result
Added `mouth_y: f32` and `mouth_mask: Option<Entity>` to `BlockFace`. For green
and grey tiers `spawn_face` spawns a 6th unit-circle child coloured with the
matching block fill (`BLOCK_GREEN` / `BLOCK_GREY`, z+0.02 above the dark mouth).
`update_faces` slides the mask each frame: for green it shifts up `mouth_sy * 0.75`
exposing a bottom-arc smile; for grey it shifts down the same amount exposing a
top-arc frown. During panic or for yellow tier the mask is scaled to zero. No
changes to `input.rs` or any other file. Builds clean.

---

### STORY-017: Cross out eyes when a block is tilted too far

**status:** done
**priority:** low

#### What
When a settled block is rotated more than ~25° from horizontal, replace the normal
pupils with ✕-style crossed-out eyes (two short diagonal lines forming an X in
each eye socket).

Implementation approach:
- In `update_faces`, read the block entity's `Transform` rotation and extract the
  angle: `transform.rotation.to_euler(EulerRot::XYZ).2.abs()` (z-angle in radians).
- If `angle > 0.436` rad (≈ 25°), override the pupil scale to draw an X shape.
  Since pupils are single circles, approximate the X by making each pupil a very
  flat, wide ellipse rotated ±45° (`Transform` rotation on the pupil entity).
  Two overlapping flat ellipses at +45° and -45° read as an ×.
- Below the threshold, restore pupils to their normal round/small shape.
- The block's own `Transform` is not currently in the `block_query` — add it
  (`&Transform` to the query tuple in `update_faces`).

#### Why
Tilted blocks look comically broken; giving them crossed-out eyes makes the
physical state legible and funny without extra UI.

#### Acceptance criteria
- [ ] A block settled at > 25° tilt displays X-shaped pupils (two overlapping flat
      ellipses rotated ±45° inside each eye socket)
- [ ] A block settled near-horizontal (< 25°) shows its normal pupils
- [ ] Expression transitions correctly when a tilted block eventually comes to rest
      flat (if physics allows it)
- [ ] `cargo build` compiles clean

#### Context & constraints
- Relevant file: `src/playing/faces.rs` — only `update_faces` needs changes
- The query currently is:
  `Query<(&BlockFace, &BlockSettleTimer, &LinearVelocity)>`
  Add `&Transform` to it
- Do NOT add new components, new files, or touch anything outside `faces.rs`
- Threshold 25° (0.436 rad) is a suggested starting point; adjust if it feels wrong
  during testing

#### Result
Added `left_pupil2` / `right_pupil2` entity fields to `BlockFace` — a second dark
circle per eye, spawned at the same local position as the first pupil but hidden
(`scale = Vec3::ZERO`) by default. `update_faces` now also reads `&Transform` from
the block entity; two queries are kept disjoint via `With<BlockFace>` /
`Without<BlockFace>` filters to avoid mutable aliasing on `Transform`. When the
block's z-rotation exceeds 0.436 rad (25°) and it isn't falling, both pupils per
eye are set to a flat `1.6 × 0.28 × eye_r` bar: the first at +45°, the second at
-45°, forming an ✕. Outside the threshold, pupils revert to round (`Quat::IDENTITY`)
and `pupil2` entities are zeroed. Builds clean.

---

### STORY-018: Shorten post-level wait or allow Space to skip it

**status:** done
**priority:** medium

#### What
After a level ends the game waits for a fixed delay before transitioning to the
stats/next-level screen. This wait currently feels too long. Do one or both of:

1. **Reduce the default delay** to ≤ 1.5 s (find the constant/timer and lower it).
2. **Allow Space (or any input) to skip the remaining wait** — if the player presses
   Space during the post-level delay, transition immediately.

#### Why
Players who are confident they want to move on are forced to wait. Cutting the
delay or adding a skip makes the pacing feel responsive.

#### Acceptance criteria
- [ ] The post-level pause is noticeably shorter (≤ 1.5 s) OR pressing Space
      skips it immediately
- [ ] The game still transitions correctly to the stats screen (no crash, no
      skipped state)
- [ ] If a skip key is added, it should only work after the level is definitively
      complete (not during the final block's settle animation)
- [ ] `cargo build` compiles clean

#### Context & constraints
- The post-level delay timer is likely in `src/playing/ui.rs` (`animate_level_complete`)
  or `src/playing/settle.rs` (`check_settle`) — check both
- The `GameState` transition that ends the level is in `src/playing/settle.rs`
  (`check_settle`) or triggered by the overlay animation completing
- Do NOT change the level-complete overlay animation speed (the fade-in/out) —
  only the hold/wait duration
- Do NOT touch physics, scoring, or the level editor

#### Result
Reduced `HOLD` from 1.5 s to 0.8 s in `animate_level_complete` (`src/playing/ui.rs`),
bringing the total overlay time from 2.2 s to 1.5 s (fade-in 0.3 + hold 0.8 +
fade-out 0.4). Added `keyboard: Res<ButtonInput<KeyCode>>` param; if Space is
pressed while the overlay is showing, `level_complete_timer` is immediately set to
`TOTAL`, causing the state transition on the same frame. Skip only activates after
`showing_level_complete` is set (i.e. all blocks settled), so it can't fire during
the settle animation. Builds clean.

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
