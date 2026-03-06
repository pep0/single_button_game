use avian2d::prelude::*;
use bevy::prelude::*;

use crate::blueprint::Blueprint;
use crate::constants::*;
use super::components::*;
use super::faces;
use super::resources::*;

const BORDER_PX: f32 = 3.0;

/// Deterministic LCG hash returning a value in [0, 1).
fn lcg(seed: u32) -> f32 {
    let x = seed.wrapping_mul(1664525).wrapping_add(1013904223);
    let x = x.wrapping_mul(22695477).wrapping_add(1);
    ((x >> 8) & 0xFF_FFFF) as f32 / 0xFF_FFFFu32 as f32
}

/// Convert HSL (hue in degrees [0,360], saturation [0,1], lightness [0,1])
/// to linear sRGB.
fn hsl_to_rgb(h: f32, s: f32, l: f32) -> (f32, f32, f32) {
    let c = (1.0 - (2.0 * l - 1.0).abs()) * s;
    let h2 = h / 60.0;
    let x = c * (1.0 - (h2 % 2.0 - 1.0).abs());
    let (r1, g1, b1) = if h2 < 1.0 { (c, x, 0.0) }
        else if h2 < 2.0 { (x, c, 0.0) }
        else if h2 < 3.0 { (0.0, c, x) }
        else if h2 < 4.0 { (0.0, x, c) }
        else if h2 < 5.0 { (x, 0.0, c) }
        else              { (c, 0.0, x) };
    let m = l - c / 2.0;
    (r1 + m, g1 + m, b1 + m)
}

/// Generate a unique fill colour for a block at `idx` within the tier's HSL palette.
///
/// Green tier:  hue 100–160°, saturation 45–75%, lightness 40–65%
/// Yellow tier: hue  40– 65°, saturation 60–85%, lightness 45–65%
/// Grey tier:   hue 220–260°, saturation  0–15%, lightness 35–60%
fn block_fill_color(score_tier: u8, idx: usize) -> Color {
    let s = idx as u32 * 3779 + score_tier as u32 * 997;
    let t0 = lcg(s);
    let t1 = lcg(s.wrapping_add(1));
    let t2 = lcg(s.wrapping_add(2));
    let (h, sat, l) = match score_tier {
        2 => (100.0 + t0 * 60.0, 0.45 + t1 * 0.30, 0.40 + t2 * 0.25),
        1 => ( 40.0 + t0 * 25.0, 0.60 + t1 * 0.25, 0.45 + t2 * 0.20),
        _ => (220.0 + t0 * 40.0, 0.00 + t1 * 0.15, 0.35 + t2 * 0.25),
    };
    let (r, g, b) = hsl_to_rgb(h, sat, l);
    Color::srgb(r, g, b)
}

/// Derive a darker border colour from the fill colour by reducing lightness.
fn block_border_color(fill: Color) -> Color {
    let srgba = fill.to_srgba();
    // Darken each channel by ~35%
    Color::srgb(
        srgba.red   * 0.60,
        srgba.green * 0.60,
        srgba.blue  * 0.60,
    )
}

pub fn slot_oscillation(
    time: Res<Time>,
    mut slot_state: ResMut<SlotState>,
    build_state: Res<BuildState>,
    blueprint: Res<Blueprint>,
    mut slot_query: Query<&mut Transform, With<SlotIndicator>>,
) {
    if build_state.showing_level_complete {
        return;
    }
    if slot_state.locked_width.is_some() {
        return;
    }

    slot_state.phase += slot_state.speed * time.delta_secs();
    let t = (slot_state.phase.sin() + 1.0) / 2.0;
    slot_state.current_width = SLOT_MIN_WIDTH + t * (SLOT_MAX_WIDTH - SLOT_MIN_WIDTH);

    if let Ok(mut transform) = slot_query.single_mut() {
        transform.scale.x = slot_state.current_width;
        if build_state.current_index < blueprint.slots.len() {
            let target = &blueprint.slots[build_state.current_index];
            transform.translation.x = target.x;
            transform.translation.y = target.y + target.height / 2.0 + SPAWN_HEIGHT_ABOVE;
        }
    }
}

pub fn production_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    touches: Res<Touches>,
    time: Res<Time>,
    mut slot_state: ResMut<SlotState>,
    mut production: ResMut<ProductionState>,
    mut build_state: ResMut<BuildState>,
    mut produced: ResMut<ProducedDimensions>,
    blueprint: Res<Blueprint>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut prod_query: Query<(Entity, &mut Transform), With<ProductionRect>>,
    slot_query: Query<&Transform, (With<SlotIndicator>, Without<ProductionRect>)>,
) {
    if build_state.waiting_for_settle {
        return;
    }
    if build_state.current_index >= blueprint.slots.len() {
        return;
    }

    let slot_y = slot_query
        .single()
        .map(|t| t.translation.y)
        .unwrap_or(250.0);
    let target_slot = &blueprint.slots[build_state.current_index];
    let width = slot_state.locked_width.unwrap_or(slot_state.current_width);

    let just_pressed = keyboard.just_pressed(KeyCode::Space) || touches.any_just_pressed();
    let held = keyboard.pressed(KeyCode::Space) || touches.iter().next().is_some();
    let just_released = keyboard.just_released(KeyCode::Space) || touches.any_just_released();

    if just_pressed && !production.is_producing {
        slot_state.locked_width = Some(slot_state.current_width);
        production.is_producing = true;
        production.current_height = 2.0;

        let mesh = meshes.add(Rectangle::new(1.0, 1.0));
        let material = materials.add(ColorMaterial::from_color(PRODUCTION_COLOR));
        commands.spawn((
            PlayingEntity,
            ProductionRect,
            Mesh2d(mesh),
            MeshMaterial2d(material),
            Transform::from_xyz(target_slot.x, slot_y - production.current_height / 2.0, 0.5)
                .with_scale(Vec3::new(width, production.current_height, 1.0)),
        ));
    }

    if held && production.is_producing {
        let new_height = (production.current_height + GROW_SPEED * time.delta_secs()).min(MAX_HEIGHT);
        production.current_height = new_height;
        if new_height >= MAX_HEIGHT {
            production.auto_drop = true;
        }

        if let Ok((_entity, mut transform)) = prod_query.single_mut() {
            transform.scale.x = width;
            transform.scale.y = production.current_height;
            transform.translation.y = slot_y - production.current_height / 2.0;
        }
    }

    if (just_released || production.auto_drop) && production.is_producing {
        production.is_producing = false;
        let produced_width = width;
        let produced_height = production.current_height;

        produced.widths.push(produced_width);
        produced.heights.push(produced_height);

        if let Ok((entity, _)) = prod_query.single_mut() {
            commands.entity(entity).despawn();
        }

        let spawn_y = slot_y;
        let pw = produced_width;
        let ph = produced_height;
        let sw = target_slot.width;
        let sh = target_slot.height;
        let score = (pw / sw).min(sw / pw) * (ph / sh).min(sh / ph);
        let score_tier: u8 = if score >= 0.80 { 2 } else if score >= 0.60 { 1 } else { 0 };
        let idx = build_state.current_index;
        let fill_color = block_fill_color(score_tier, idx);
        let border_color = block_border_color(fill_color);

        let block_entity = commands.spawn((
            PlayingEntity,
            TowerBlock(build_state.current_index),
            TowerBlockDims { height: ph },
            BlockSettleTimer::default(),
            RigidBody::Dynamic,
            Collider::rectangle(pw, ph),
            CollisionEventsEnabled,
            Transform::from_xyz(target_slot.x, spawn_y, 0.5),
        )).id();

        // Border rectangle (slightly larger, drawn behind)
        commands.spawn((
            ChildOf(block_entity),
            Mesh2d(meshes.add(Rectangle::new(1.0, 1.0))),
            MeshMaterial2d(materials.add(ColorMaterial::from_color(border_color))),
            Transform::from_xyz(0.0, 0.0, -0.1)
                .with_scale(Vec3::new(pw, ph, 1.0)),
        ));
        // Fill rectangle
        commands.spawn((
            ChildOf(block_entity),
            Mesh2d(meshes.add(Rectangle::new(1.0, 1.0))),
            MeshMaterial2d(materials.add(ColorMaterial::from_color(fill_color))),
            Transform::from_xyz(0.0, 0.0, 0.0)
                .with_scale(Vec3::new(pw - BORDER_PX * 2.0, ph - BORDER_PX * 2.0, 1.0)),
        ));

        // Face — pass fill_color so the mouth mask matches the block body exactly.
        let face = faces::spawn_face(
            &mut commands, &mut meshes, &mut materials,
            block_entity, pw, ph, score_tier,
            build_state.current_index as u32,
            fill_color,
        );
        commands.entity(block_entity).insert(face);

        build_state.current_index += 1;
        if build_state.current_index >= blueprint.slots.len() {
            build_state.waiting_for_settle = true;
            build_state.settle_timer = 0.0;
            build_state.stability_window = 0.0;
        }

        slot_state.locked_width = None;
        production.current_height = 0.0;
        production.auto_drop = false;
    }
}
