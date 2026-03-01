use avian2d::prelude::LinearVelocity;
use bevy::prelude::*;
use super::components::{BlockSettleTimer, PlayingEntity};

fn prand(seed: u32) -> f32 {
    let x = seed.wrapping_mul(1664525).wrapping_add(1013904223);
    let x = x.wrapping_mul(22695477).wrapping_add(1);
    ((x >> 8) & 0x00FF_FFFF) as f32 / 0x00FF_FFFFu32 as f32
}

#[derive(Component)]
pub struct BlockFace {
    pub score_tier: u8, // 0=grey, 1=yellow, 2=green
    eye_r: f32,
    pupil_r: f32,
    mouth_w: f32,
    mouth_h: f32,
    left_eye: Entity,
    right_eye: Entity,
    left_pupil: Entity,
    right_pupil: Entity,
    mouth: Entity,
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
    let eye_r   = face_unit * 0.115;
    let pupil_r = eye_r * 0.55;
    let mouth_w = face_unit * 0.22;
    let mouth_h = face_unit * 0.22;

    // Random variation in eye spacing (~24–32 % of face_unit from centre)
    let eye_x   = face_unit * (0.24 + prand(seed.wrapping_add(10)) * 0.08);
    let eye_y   =  ph * 0.10;
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

    let left_eye    = sp(white, -eye_x,           eye_y,   0.00);
    let right_eye   = sp(white,  eye_x,           eye_y,   0.00);
    let left_pupil  = sp(dark,  -eye_x + cross_x, eye_y,   0.01);
    let right_pupil = sp(dark,   eye_x - cross_x, eye_y,   0.01);
    let mouth       = sp(dark,   0.0,              mouth_y, 0.00);

    BlockFace {
        score_tier,
        eye_r,
        pupil_r,
        mouth_w,
        mouth_h,
        left_eye,
        right_eye,
        left_pupil,
        right_pupil,
        mouth,
    }
}

/// Updates eye and mouth scales every frame to show falling-panic or
/// settled-score expressions.
pub fn update_faces(
    block_query: Query<(&BlockFace, &BlockSettleTimer, &LinearVelocity)>,
    mut transforms: Query<&mut Transform>,
) {
    for (face, timer, vel) in &block_query {
        // Panic while block is still falling / bouncing
        let falling = timer.rest_secs < 0.35 || vel.0.length() > 60.0;

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

        for &e in &[face.left_eye, face.right_eye] {
            if let Ok(mut t) = transforms.get_mut(e) {
                t.scale = Vec3::new(eye_sx, eye_sy, 1.0);
            }
        }
        for &e in &[face.left_pupil, face.right_pupil] {
            if let Ok(mut t) = transforms.get_mut(e) {
                t.scale = Vec3::new(face.pupil_r, face.pupil_r, 1.0);
            }
        }
        if let Ok(mut t) = transforms.get_mut(face.mouth) {
            t.scale = Vec3::new(mouth_sx, mouth_sy, 1.0);
        }
    }
}
