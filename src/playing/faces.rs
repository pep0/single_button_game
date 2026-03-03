use avian2d::prelude::LinearVelocity;
use bevy::prelude::*;
use std::f32::consts::FRAC_PI_4;
use super::components::{BlockSettleTimer, PlayingEntity};

fn prand(seed: u32) -> f32 {
    let x = seed.wrapping_mul(1664525).wrapping_add(1013904223);
    let x = x.wrapping_mul(22695477).wrapping_add(1);
    ((x >> 8) & 0x00FF_FFFF) as f32 / 0x00FF_FFFFu32 as f32
}

// Fill colours duplicated from input.rs — used to tint the mouth mask.
const BLOCK_GREEN: Color = Color::srgb(0.38, 0.72, 0.45);
const BLOCK_GREY:  Color = Color::srgb(0.48, 0.46, 0.52);

#[derive(Component)]
pub struct BlockFace {
    pub score_tier: u8, // 0=grey, 1=yellow, 2=green
    seed: u32,           // stored for ongoing idle-animation randomness
    eye_r: f32,
    pupil_r: f32,
    mouth_w: f32,
    mouth_h: f32,
    mouth_y: f32,        // local-space Y of mouth centre (needed for mask offset)
    left_eye: Entity,
    right_eye: Entity,
    left_pupil: Entity,  // first bar of the X (or normal round pupil)
    right_pupil: Entity,
    left_pupil2: Entity, // second bar of the X (hidden when not tilted)
    right_pupil2: Entity,
    mouth: Entity,
    mouth_mask: Option<Entity>, // crescent mask for green smile / grey frown
    left_brow: Entity,
    right_brow: Entity,
    brow_y: f32,         // local-space Y of brow centres
    eye_x: f32,          // eye horizontal offset (for brow positioning)
    // Base pupil positions; look offsets are applied relative to these
    left_pupil_x:  f32,
    left_pupil_y:  f32,
    right_pupil_x: f32,
    right_pupil_y: f32,
    // Blink state
    blink_timer: f32, // counts down to next blink
    blink_age:   f32, // > 0 while blinking; counts up to BLINK_DUR then resets
    blink_count: u32, // incremented each blink for seed variety
    // Look-around state
    look_timer:    f32, // counts down to next glance
    look_duration: f32, // how long to hold the current glance
    look_age:      f32, // how long we've been in current glance direction
    look_dx:       f32, // current pupil x offset
    look_dy:       f32, // current pupil y offset
    look_count:    u32, // incremented each glance for seed variety
}

/// Spawns face child entities on a block and returns a `BlockFace` to insert
/// on the block entity.
pub fn spawn_face(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<ColorMaterial>,
    parent: Entity,
    pw: f32,
    ph: f32,
    score_tier: u8,
    seed: u32,
) -> BlockFace {
    let face_unit = ph.min(pw).min(100.0).max(12.0);

    // Random eye size: ±~25 % band around baseline
    let eye_r   = face_unit * (0.09 + prand(seed.wrapping_add(20)) * 0.05);
    let pupil_r = eye_r * 0.55;

    // Random mouth dimensions: ±~20 % width variation
    let mouth_w = face_unit * (0.18 + prand(seed.wrapping_add(21)) * 0.08);
    let mouth_h = face_unit * 0.22;

    // Eye spacing scales with block width so wide blocks have wider-set eyes
    let eye_x = pw * (0.24 + prand(seed.wrapping_add(10)) * 0.08);

    // Small random vertical jitter for high/low brow variety
    let eye_y_jitter = face_unit * (prand(seed.wrapping_add(22)) - 0.5) * 0.10;
    let eye_y   =  ph * 0.10 + eye_y_jitter;
    let mouth_y = -ph * 0.12;

    // 20 % chance of crossed eyes
    let crossed = prand(seed.wrapping_add(7)) < 0.20;
    let cross_x = if crossed { eye_r * 0.48 } else { 0.0 };

    // Base pupil positions (look offsets applied relative to these each frame)
    let left_pupil_x  = -eye_x + cross_x;
    let right_pupil_x =  eye_x - cross_x;
    let pupil_y       = eye_y;

    let white = Color::srgba(1.0, 1.0, 1.0, 0.95);
    let dark  = Color::srgba(0.07, 0.07, 0.10, 0.95);
    let z     = 0.2_f32; // above fill rect at z=0.0

    // Helper: spawn a unit-circle child; scale set each frame by update_faces
    let mut sp = |color: Color, x: f32, y: f32, dz: f32| -> Entity {
        let mesh = meshes.add(Circle::new(1.0));
        let mat  = materials.add(ColorMaterial::from_color(color));
        commands.spawn((
            PlayingEntity,
            ChildOf(parent),
            Mesh2d(mesh),
            MeshMaterial2d(mat),
            Transform::from_xyz(x, y, z + dz),
        )).id()
    };

    let brow_y = eye_y + eye_r * 1.55;

    let left_eye     = sp(white, -eye_x,         eye_y,   0.00);
    let right_eye    = sp(white,  eye_x,         eye_y,   0.00);
    let left_pupil   = sp(dark,  left_pupil_x,  pupil_y, 0.01);
    let right_pupil  = sp(dark,  right_pupil_x, pupil_y, 0.01);
    // Second bar of the X-eye (hidden unless block is tilted)
    let left_pupil2  = sp(dark,  left_pupil_x,  pupil_y, 0.01);
    let right_pupil2 = sp(dark,  right_pupil_x, pupil_y, 0.01);
    let mouth        = sp(dark,  0.0,            mouth_y, 0.00);
    let left_brow    = sp(dark, -eye_x,          brow_y,  0.02);
    let right_brow   = sp(dark,  eye_x,          brow_y,  0.02);

    // Crescent mask: a fill-coloured circle that slides over the dark mouth to
    // leave only a bottom arc (smile) or top arc (frown) visible.
    let mouth_mask = match score_tier {
        2 => Some(sp(BLOCK_GREEN, 0.0, mouth_y, 0.02)),
        0 => Some(sp(BLOCK_GREY,  0.0, mouth_y, 0.02)),
        _ => None,
    };

    BlockFace {
        score_tier,
        seed,
        eye_r,
        pupil_r,
        mouth_w,
        mouth_h,
        mouth_y,
        left_eye,
        right_eye,
        left_pupil,
        right_pupil,
        left_pupil2,
        right_pupil2,
        mouth,
        mouth_mask,
        left_brow,
        right_brow,
        brow_y,
        eye_x,
        left_pupil_x,
        left_pupil_y: pupil_y,
        right_pupil_x,
        right_pupil_y: pupil_y,
        // Stagger blink and look timers so blocks don't all animate in sync
        blink_timer: 2.0 + prand(seed.wrapping_add(30)) * 3.0,
        blink_age:   0.0,
        blink_count: 0,
        look_timer:    3.0 + prand(seed.wrapping_add(31)) * 4.0,
        look_duration: 0.0,
        look_age:      0.0,
        look_dx:       0.0,
        look_dy:       0.0,
        look_count:    0,
    }
}

/// Updates eye and mouth scales every frame to show falling-panic or
/// settled-score expressions, and drives blink / look-around idle animations.
pub fn update_faces(
    time: Res<Time>,
    // With<BlockFace> + Without<BlockFace> make the two Transform accesses disjoint.
    mut block_query: Query<(&mut BlockFace, &BlockSettleTimer, &LinearVelocity, &Transform), With<BlockFace>>,
    mut transforms: Query<&mut Transform, Without<BlockFace>>,
) {
    let dt = time.delta_secs();

    for (mut face, timer, vel, block_tf) in &mut block_query {
        // Panic while block is still falling / bouncing
        let falling = timer.rest_secs < 0.35 || vel.0.length() > 60.0;

        // Tilted > 25° (0.436 rad) → show X-eyes instead of normal pupils
        let tilt   = block_tf.rotation.to_euler(EulerRot::XYZ).2.abs();
        let tilted = !falling && tilt > 0.436;

        let (eye_sx, eye_sy, mouth_sx, mouth_sy) = if falling {
            // Wide-open eyes, O-shaped mouth
            (
                face.eye_r * 1.35, face.eye_r * 1.35,
                face.mouth_w * 0.65, face.mouth_h * 1.55,
            )
        } else {
            match face.score_tier {
                2 => (
                    face.eye_r, face.eye_r,
                    face.mouth_w * 2.0, face.mouth_h * 0.38, // big grin
                ),
                1 => (
                    face.eye_r, face.eye_r * 0.65,
                    face.mouth_w * 1.4, face.mouth_h * 0.30, // small smile
                ),
                _ => (
                    face.eye_r, face.eye_r * 0.42,
                    face.mouth_w * 1.0, face.mouth_h * 0.22, // flat / grumpy
                ),
            }
        };

        const BLINK_DUR: f32 = 0.12;

        // --- Idle animations (only when settled and upright) ---
        if !falling && !tilted {
            // Blink: count down, then squint for BLINK_DUR
            if face.blink_age > 0.0 {
                face.blink_age += dt;
                if face.blink_age >= BLINK_DUR {
                    face.blink_age = 0.0;
                }
            } else {
                face.blink_timer -= dt;
                if face.blink_timer <= 0.0 {
                    face.blink_age   = 0.001; // tiny positive = blinking
                    face.blink_count = face.blink_count.wrapping_add(1);
                    let bc = face.blink_count;
                    face.blink_timer = 2.0 + prand(face.seed.wrapping_add(30 + bc * 7)) * 3.0;
                }
            }

            // Look around: hold direction for look_duration, return to centre, wait
            if face.look_age < face.look_duration {
                face.look_age += dt;
            } else {
                // Look period done (or initial state) — return to centre and wait
                face.look_dx    = 0.0;
                face.look_dy    = 0.0;
                face.look_timer -= dt;
                if face.look_timer <= 0.0 {
                    face.look_count = face.look_count.wrapping_add(1);
                    let s       = face.seed.wrapping_add(40 + face.look_count * 13);
                    let max_off = face.eye_r * 0.45;
                    face.look_dx       = (prand(s)                 - 0.5) * 2.0 * max_off;
                    face.look_dy       = (prand(s.wrapping_add(1)) - 0.5) * 2.0 * max_off;
                    face.look_duration = 0.4 + prand(s.wrapping_add(2)) * 0.4;
                    face.look_age      = 0.0;
                    face.look_timer    = 3.0 + prand(s.wrapping_add(3)) * 4.0;
                }
            }
        } else {
            // Reset animations while falling or tilted so they start fresh on settle
            face.blink_age = 0.0;
            face.look_dx   = 0.0;
            face.look_dy   = 0.0;
            face.look_age  = face.look_duration; // mark current look as done
        }

        // Blink squishes the eye height to nearly zero
        let blink_eye_sy = if face.blink_age > 0.0 { eye_sy * 0.08 } else { eye_sy };

        // Copy fields needed below before immutable face borrows overlap
        let look_dx       = face.look_dx;
        let look_dy       = face.look_dy;
        let left_pupil_x  = face.left_pupil_x;
        let left_pupil_y  = face.left_pupil_y;
        let right_pupil_x = face.right_pupil_x;
        let right_pupil_y = face.right_pupil_y;
        let pupil_r       = face.pupil_r;

        // Brow dimensions
        let brow_w  = face.eye_r * 2.2;
        let brow_h  = face.eye_r * 0.32;

        if tilted {
            // Hide eye whites — only the bare ✕ bars show
            for &e in &[face.left_eye, face.right_eye] {
                if let Ok(mut t) = transforms.get_mut(e) {
                    t.scale = Vec3::ZERO;
                }
            }
            // X-eyes: two flat diagonal bars per eye socket
            let bar_sx = face.eye_r * 1.6;
            let bar_sy = face.eye_r * 0.28;
            for &e in &[face.left_pupil, face.right_pupil] {
                if let Ok(mut t) = transforms.get_mut(e) {
                    t.scale    = Vec3::new(bar_sx, bar_sy, 1.0);
                    t.rotation = Quat::from_rotation_z(FRAC_PI_4);
                }
            }
            for &e in &[face.left_pupil2, face.right_pupil2] {
                if let Ok(mut t) = transforms.get_mut(e) {
                    t.scale    = Vec3::new(bar_sx, bar_sy, 1.0);
                    t.rotation = Quat::from_rotation_z(-FRAC_PI_4);
                }
            }
            // Hide brows when block is tilted (X-eyes state)
            for &e in &[face.left_brow, face.right_brow] {
                if let Ok(mut t) = transforms.get_mut(e) {
                    t.scale = Vec3::ZERO;
                }
            }
        } else {
            for &e in &[face.left_eye, face.right_eye] {
                if let Ok(mut t) = transforms.get_mut(e) {
                    t.scale = Vec3::new(eye_sx, blink_eye_sy, 1.0);
                }
            }
            // Apply look offset to pupil positions
            if let Ok(mut t) = transforms.get_mut(face.left_pupil) {
                t.translation.x = left_pupil_x  + look_dx;
                t.translation.y = left_pupil_y  + look_dy;
                t.scale         = Vec3::new(pupil_r, pupil_r, 1.0);
                t.rotation      = Quat::IDENTITY;
            }
            if let Ok(mut t) = transforms.get_mut(face.right_pupil) {
                t.translation.x = right_pupil_x + look_dx;
                t.translation.y = right_pupil_y + look_dy;
                t.scale         = Vec3::new(pupil_r, pupil_r, 1.0);
                t.rotation      = Quat::IDENTITY;
            }
            for &e in &[face.left_pupil2, face.right_pupil2] {
                if let Ok(mut t) = transforms.get_mut(e) {
                    t.scale = Vec3::ZERO;
                }
            }

            // Eyebrows: angle encodes expression
            // falling: outer-up fear (+0.30 left, -0.30 right)
            // grey:    inner-up angry (-0.18 left, +0.18 right)
            // yellow:  flat neutral (0.0)
            // green:   outer-up happy (+0.18 left, -0.18 right)
            let (left_brow_angle, right_brow_angle) = if falling {
                (0.30_f32, -0.30_f32)
            } else {
                match face.score_tier {
                    0 => (-0.18, 0.18),
                    1 => (0.0, 0.0),
                    _ => (0.18, -0.18),
                }
            };
            let eye_x = face.eye_x;
            let brow_y = face.brow_y;
            if let Ok(mut t) = transforms.get_mut(face.left_brow) {
                t.translation.x = -eye_x;
                t.translation.y = brow_y;
                t.scale    = Vec3::new(brow_w, brow_h, 1.0);
                t.rotation = Quat::from_rotation_z(left_brow_angle);
            }
            if let Ok(mut t) = transforms.get_mut(face.right_brow) {
                t.translation.x = eye_x;
                t.translation.y = brow_y;
                t.scale    = Vec3::new(brow_w, brow_h, 1.0);
                t.rotation = Quat::from_rotation_z(right_brow_angle);
            }
        }

        if let Ok(mut t) = transforms.get_mut(face.mouth) {
            t.scale = Vec3::new(mouth_sx, mouth_sy, 1.0);
        }

        // Crescent mask: slide it over the dark mouth to reveal only an arc.
        // Hidden while falling (O-mouth) or for yellow tier (no mask).
        if let Some(mask) = face.mouth_mask {
            if let Ok(mut t) = transforms.get_mut(mask) {
                if falling || face.score_tier == 1 {
                    t.scale = Vec3::ZERO;
                } else {
                    // Green: mask above mouth → bottom arc visible = smile
                    // Grey:  mask below mouth → top arc visible  = frown
                    let dy = if face.score_tier == 2 { mouth_sy * 0.75 } else { -mouth_sy * 0.75 };
                    t.translation.y = face.mouth_y + dy;
                    t.scale = Vec3::new(mouth_sx, mouth_sy, 1.0);
                }
            }
        }
    }
}
