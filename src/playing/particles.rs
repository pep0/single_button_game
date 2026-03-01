use bevy::prelude::*;
use super::components::PlayingEntity;

#[derive(Component)]
pub struct Particle {
    pub velocity: Vec2,
    pub lifetime: f32,
    pub age:      f32,
}

/// Deterministic pseudo-random in 0.0..1.0 from a u32 seed.
fn prand(seed: u32) -> f32 {
    let x = seed.wrapping_mul(1664525).wrapping_add(1013904223);
    let x = x.wrapping_mul(22695477).wrapping_add(1);
    ((x >> 8) & 0x00FF_FFFF) as f32 / 0x00FF_FFFFu32 as f32
}

/// Burst of smoke particles from the bottom edge of a settled block.
pub fn spawn_smoke_burst(
    commands:  &mut Commands,
    meshes:    &mut Assets<Mesh>,
    materials: &mut Assets<ColorMaterial>,
    center_x:  f32,
    bottom_y:  f32,
    block_w:   f32,
    seed_base: u32,
) {
    const COUNT: u32 = 8;
    for i in 0..COUNT {
        let s = seed_base.wrapping_mul(131).wrapping_add(i * 1337);

        // Spread spawn position along the bottom edge
        let x_off   = (prand(s)            - 0.5) * block_w * 0.9;
        // Velocity: fan outward from centre (left half go left, right half go right)
        let side    = if x_off < 0.0 { -1.0 } else { 1.0 };
        let hspeed  = (20.0 + prand(s + 1) * 55.0) * side;
        let vspeed  = 15.0 + prand(s + 2) * 35.0; // upward
        let lifetime = 0.45 + prand(s + 3) * 0.45;
        let radius   = 2.5 + prand(s + 4) * 4.5;
        // Slight grey variation
        let grey     = 0.62 + prand(s + 5) * 0.18;

        let mesh = meshes.add(Circle::new(radius));
        let mat  = materials.add(ColorMaterial::from_color(
            Color::srgba(grey, grey, grey + 0.04, 0.85),
        ));

        commands.spawn((
            PlayingEntity,
            Particle {
                velocity: Vec2::new(hspeed, vspeed),
                lifetime,
                age: 0.0,
            },
            Mesh2d(mesh),
            MeshMaterial2d(mat),
            Transform::from_xyz(center_x + x_off, bottom_y + 2.0, 1.5),
        ));
    }
}

/// Moves particles, fades them out, and despawns expired ones.
pub fn tick_particles(
    mut commands:  Commands,
    time:          Res<Time>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut query:     Query<(Entity, &mut Particle, &mut Transform, &MeshMaterial2d<ColorMaterial>)>,
) {
    let dt = time.delta_secs();
    for (entity, mut p, mut transform, mat_handle) in &mut query {
        p.age += dt;
        if p.age >= p.lifetime {
            commands.entity(entity).despawn();
            continue;
        }

        let t = p.age / p.lifetime; // 0 → 1

        // Fade alpha
        if let Some(mat) = materials.get_mut(mat_handle.id()) {
            let c = mat.color.to_srgba();
            mat.color = Color::srgba(c.red, c.green, c.blue, (1.0 - t) * 0.85);
        }

        // Move
        transform.translation.x += p.velocity.x * dt;
        transform.translation.y += p.velocity.y * dt;

        // Gentle upward float + horizontal drag
        p.velocity.y  += 18.0 * dt;
        p.velocity.x  *= 1.0 - dt * 2.2;
    }
}
