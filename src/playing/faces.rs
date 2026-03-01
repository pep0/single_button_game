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
    let face_unit = ph.min(pw).min(52.0).max(12.0);

    // Random eye size: ±~25 % band around baseline
    let eye_r   = face_unit * (0.09 + prand(seed.wrapping_add(20)) * 0.05);
    let pupil_r = eye_r * 0.55;

    // Random mouth dimensions: ±~20 % width variation
    let mouth_w = face_unit * (0.18 + prand(seed.wrapping_add(21)) * 0.08);
    let mouth_h = face_unit * 0.22;

    // Eye spacing scales with block width so wide blocks have wider-set eyes
    // (24–32 % of pw from centre instead of face_unit, which was capped at 52)
    let eye_x = pw * (0.24 + prand(seed.wrapping_add(10)) * 0.08);

    // Small random vertical jitter (±5 % of face_unit) for high/low brow variety
    let eye_y_jitter = face_unit * (prand(seed.wrapping_add(22)) - 0.5) * 0.10;
    let eye_y   =  ph * 0.10 + eye_y_jitter;
    let mouth_y = -ph * 0.12;

    // 20 % chance of crossed eyes
    let crossed = prand(seed.wrapping_add(7)) < 0.20;
    let cross_x = if crossed { eye_r * 0.48 } else { 0.0 };

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

    let left_eye     = sp(white, -eye_x,           eye_y,   0.00);
    let right_eye    = sp(white,  eye_x,           eye_y,   0.00);
    let left_pupil   = sp(dark,  -eye_x + cross_x, eye_y,   0.01);
    let right_pupil  = sp(dark,   eye_x - cross_x, eye_y,   0.01);
    // Second bar of the X-eye (hidden unless block is tilted)
    let left_pupil2  = sp(dark,  -eye_x + cross_x, eye_y,   0.01);
    let right_pupil2 = sp(dark,   eye_x - cross_x, eye_y,   0.01);
    let mouth        = sp(dark,   0.0,              mouth_y, 0.00);

    // Crescent mask: a fill-coloured circle that slides over the dark mouth to
    // leave only a bottom arc (smile) or top arc (frown) visible.
    let mouth_mask = match score_tier {
        2 => Some(sp(BLOCK_GREEN, 0.0, mouth_y, 0.02)),
        0 => Some(sp(BLOCK_GREY,  0.0, mouth_y, 0.02)),
        _ => None,
    };

    BlockFace {
        score_tier,
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
    }
}

/// Updates eye and mouth scales every frame to show falling-panic or
/// settled-score expressions.
pub fn update_faces(
    // With<BlockFace> + Without<BlockFace> make the two Transform accesses disjoint.
    block_query: Query<(&BlockFace, &BlockSettleTimer, &LinearVelocity, &Transform), With<BlockFace>>,
    mut transforms: Query<&mut Transform, Without<BlockFace>>,
) {
    for (face, timer, vel, block_tf) in &block_query {
        // Panic while block is still falling / bouncing
        let falling = timer.rest_secs < 0.35 || vel.0.length() > 60.0;

        // Tilted > 25° (0.436 rad) → show X-eyes instead of normal pupils
        let tilt = block_tf.rotation.to_euler(EulerRot::XYZ).2.abs();
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
        } else {
            for &e in &[face.left_eye, face.right_eye] {
                if let Ok(mut t) = transforms.get_mut(e) {
                    t.scale = Vec3::new(eye_sx, eye_sy, 1.0);
                }
            }
            for &e in &[face.left_pupil, face.right_pupil] {
                if let Ok(mut t) = transforms.get_mut(e) {
                    t.scale    = Vec3::new(face.pupil_r, face.pupil_r, 1.0);
                    t.rotation = Quat::IDENTITY;
                }
            }
            for &e in &[face.left_pupil2, face.right_pupil2] {
                if let Ok(mut t) = transforms.get_mut(e) {
                    t.scale = Vec3::ZERO;
                }
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
