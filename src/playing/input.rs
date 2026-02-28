use avian2d::prelude::*;
use bevy::prelude::*;

use crate::blueprint::Blueprint;
use crate::constants::*;
use crate::state::TowerModeActive;
use super::components::*;
use super::resources::*;

const BLOCK_GREEN:  Color = Color::srgb(0.38, 0.72, 0.45);
const BLOCK_YELLOW: Color = Color::srgb(0.82, 0.70, 0.30);
const BLOCK_GREY:   Color = Color::srgb(0.48, 0.46, 0.52);
const BLOCK_BORDER: Color = Color::srgb(0.10, 0.10, 0.15);
const BORDER_PX: f32 = 3.0;

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
        // Position slot above current target block
        if build_state.current_index < blueprint.slots.len() {
            let target = &blueprint.slots[build_state.current_index];
            transform.translation.x = target.x;
            transform.translation.y = target.y + target.height / 2.0 + SPAWN_HEIGHT_ABOVE;
        }
    }
}

pub fn production_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    mut slot_state: ResMut<SlotState>,
    mut production: ResMut<ProductionState>,
    mut build_state: ResMut<BuildState>,
    mut produced: ResMut<ProducedDimensions>,
    blueprint: Res<Blueprint>,
    tower_mode: Option<Res<TowerModeActive>>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut prod_query: Query<(Entity, &mut Transform), With<ProductionRect>>,
    slot_query: Query<&Transform, (With<SlotIndicator>, Without<ProductionRect>)>,
) {
    // Don't allow input during final settle phase
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

    if keyboard.just_pressed(KeyCode::Space) && !production.is_producing {
        // Lock the slot width
        slot_state.locked_width = Some(slot_state.current_width);
        production.is_producing = true;
        production.current_height = 2.0;

        // Spawn production rectangle
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

    if keyboard.pressed(KeyCode::Space) && production.is_producing {
        production.current_height =
            (production.current_height + GROW_SPEED * time.delta_secs()).min(MAX_HEIGHT);

        if let Ok((_entity, mut transform)) = prod_query.single_mut() {
            transform.scale.x = width;
            transform.scale.y = production.current_height;
            transform.translation.y = slot_y - production.current_height / 2.0;
        }
    }

    if keyboard.just_released(KeyCode::Space) && production.is_producing {
        production.is_producing = false;
        let produced_width = width;
        let produced_height = production.current_height;

        produced.widths.push(produced_width);
        produced.heights.push(produced_height);

        // Remove the production rect, spawn a dynamic rigid body
        if let Ok((entity, _)) = prod_query.single_mut() {
            commands.entity(entity).despawn();
        }

        // Spawn as dynamic rigid body at slot_y, it will fall with gravity
        let spawn_y = slot_y;
        let pw = produced_width;
        let ph = produced_height;
        let sw = target_slot.width;
        let sh = target_slot.height;
        let score = (pw / sw).min(sw / pw) * (ph / sh).min(sh / ph);
        let fill_color = if score >= 0.80 {
            BLOCK_GREEN
        } else if score >= 0.60 {
            BLOCK_YELLOW
        } else {
            BLOCK_GREY
        };

        let mut entity_cmd = commands.spawn((
            TowerBlock(build_state.current_index),
            TowerBlockDims { height: ph },
            BlockSettleTimer::default(),
            RigidBody::Dynamic,
            Collider::rectangle(pw, ph),
            CollisionEventsEnabled,
            Transform::from_xyz(target_slot.x, spawn_y, 0.5),
        ));
        if tower_mode.is_none() {
            entity_cmd.insert(PlayingEntity);
        }
        let block_entity = entity_cmd.id();

        // Border rectangle (slightly larger, drawn behind)
        commands.spawn((
            ChildOf(block_entity),
            Mesh2d(meshes.add(Rectangle::new(1.0, 1.0))),
            MeshMaterial2d(materials.add(ColorMaterial::from_color(BLOCK_BORDER))),
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

        // Advance to next block immediately; only enter settle phase after last block
        build_state.current_index += 1;
        if build_state.current_index >= blueprint.slots.len() {
            build_state.waiting_for_settle = true;
            build_state.settle_timer = 0.0;
            build_state.stability_window = 0.0;
        }

        // Unlock the slot
        slot_state.locked_width = None;
        production.current_height = 0.0;
    }
}
