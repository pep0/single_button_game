use avian2d::prelude::{CollisionStart, LinearVelocity};
use bevy::audio::Volume;
use bevy::ecs::message::MessageReader;
use bevy::prelude::*;
use std::sync::Arc;
use std::f32::consts::PI;

use crate::constants::{MAX_HEIGHT, SLOT_MAX_WIDTH};
use super::components::*;
use super::resources::ProducedDimensions;

const NUM_THUDS: usize = 12;
const SAMPLE_RATE: u32 = 44100;

#[derive(Message)]
pub struct BlockLanded {
    pub area_ratio: f32,   // 0.0 (tiny) – 1.0 (max size)
    pub impact_speed: f32, // vel_drop for volume scaling
}

#[derive(Resource)]
pub struct TowerAudio {
    pub thuds: Vec<Handle<AudioSource>>,
}

fn build_wav(samples: &[i16]) -> Vec<u8> {
    let data_size = samples.len() * 2;
    let mut b = Vec::with_capacity(44 + data_size);
    b.extend_from_slice(b"RIFF");
    b.extend_from_slice(&((36 + data_size) as u32).to_le_bytes());
    b.extend_from_slice(b"WAVE");
    b.extend_from_slice(b"fmt ");
    b.extend_from_slice(&16u32.to_le_bytes());
    b.extend_from_slice(&1u16.to_le_bytes());  // PCM
    b.extend_from_slice(&1u16.to_le_bytes());  // mono
    b.extend_from_slice(&SAMPLE_RATE.to_le_bytes());
    b.extend_from_slice(&(SAMPLE_RATE * 2).to_le_bytes()); // byte rate
    b.extend_from_slice(&2u16.to_le_bytes());  // block align
    b.extend_from_slice(&16u16.to_le_bytes()); // bits/sample
    b.extend_from_slice(b"data");
    b.extend_from_slice(&(data_size as u32).to_le_bytes());
    for &s in samples {
        b.extend_from_slice(&s.to_le_bytes());
    }
    b
}

fn synth_thud(freq: f32) -> Vec<u8> {
    let n = (SAMPLE_RATE as f32 * 0.25) as usize; // 250 ms
    let samples: Vec<i16> = (0..n).map(|i| {
        let t = i as f32 / SAMPLE_RATE as f32;
        let env = (-t * 18.0).exp();
        let sig = (2.0 * PI * freq * t).sin()
                + 0.3 * (2.0 * PI * freq * 2.0 * t).sin();
        ((sig * env).clamp(-1.0, 1.0) * 32767.0) as i16
    }).collect();
    build_wav(&samples)
}

pub fn setup_audio(
    mut commands: Commands,
    mut audio_assets: ResMut<Assets<AudioSource>>,
) {
    let thuds = (0..NUM_THUDS).map(|i| {
        let t = i as f32 / (NUM_THUDS - 1) as f32;
        let freq = 110.0 - t * 70.0; // 110 Hz (small) → 40 Hz (large)
        let bytes = synth_thud(freq);
        audio_assets.add(AudioSource { bytes: Arc::from(bytes.as_slice()) })
    }).collect();
    commands.insert_resource(TowerAudio { thuds });
}

pub fn play_landing_audio(
    mut commands: Commands,
    audio: Option<Res<TowerAudio>>,
    mut events: MessageReader<BlockLanded>,
) {
    let Some(audio) = audio else { return; };
    for event in events.read() {
        let idx = ((event.area_ratio * (NUM_THUDS - 1) as f32).round() as usize)
            .min(NUM_THUDS - 1);
        let vol = (event.impact_speed / 300.0).clamp(0.3, 1.0);
        commands.spawn((
            PlayingEntity,
            AudioPlayer::new(audio.thuds[idx].clone()),
            PlaybackSettings { volume: Volume::Linear(vol), ..PlaybackSettings::ONCE },
        ));
    }
}

pub fn play_collision_audio(
    mut commands: Commands,
    time: Res<Time>,
    audio: Option<Res<TowerAudio>>,
    produced: Res<ProducedDimensions>,
    mut events: MessageReader<CollisionStart>,
    mut block_query: Query<(&TowerBlock, &TowerBlockDims, &LinearVelocity, &mut BlockSettleTimer)>,
) {
    let Some(audio) = audio else { return; };
    let dt = time.delta_secs();

    // Tick down cooldowns
    for (_, _, _, mut timer) in &mut block_query {
        timer.collision_cooldown = (timer.collision_cooldown - dt).max(0.0);
    }

    for event in events.read() {
        let bodies = [event.body1, event.body2];
        for &maybe_entity in &bodies {
            let Some(entity) = maybe_entity else { continue; };
            let Ok((block, dims, vel, mut timer)) = block_query.get_mut(entity) else { continue; };
            if timer.collision_cooldown > 0.0 {
                continue;
            }
            let speed = vel.0.length();
            if speed < 80.0 {
                continue; // ignore very slow contacts (sliding at rest)
            }

            let i = block.0;
            let width = if i < produced.widths.len() { produced.widths[i] } else { 80.0 };
            let area_ratio = (width * dims.height / (SLOT_MAX_WIDTH * MAX_HEIGHT)).sqrt();
            let idx = ((area_ratio * (NUM_THUDS - 1) as f32).round() as usize).min(NUM_THUDS - 1);
            let vol = (speed / 400.0).clamp(0.15, 0.6);

            commands.spawn((
                PlayingEntity,
                AudioPlayer::new(audio.thuds[idx].clone()),
                PlaybackSettings { volume: Volume::Linear(vol), ..PlaybackSettings::ONCE },
            ));
            timer.collision_cooldown = 0.15; // 150 ms cooldown per block
            break; // only play once per collision event
        }
    }
}

