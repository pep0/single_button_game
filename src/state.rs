use bevy::prelude::*;

#[derive(States, Debug, Clone, PartialEq, Eq, Hash, Default)]
pub enum GameState {
    #[default]
    Menu,
    Playing,
    Failed,
    Stats,
    Editor,
}

#[derive(Resource, Default)]
pub struct LevelSequence {
    pub entries: Vec<String>,
}

#[derive(Resource)]
pub struct Score {
    pub round: usize,
    pub total_score: f32,
    pub rounds_played: usize,
    pub lives: usize,
}

impl Default for Score {
    fn default() -> Self {
        Self { round: 0, total_score: 0.0, rounds_played: 0, lives: 3 }
    }
}

#[derive(Resource, Default)]
pub struct FailureReason {
    pub message: String,
}

pub fn cleanup<T: Component>(mut commands: Commands, query: Query<Entity, With<T>>) {
    for entity in &query {
        commands.entity(entity).despawn();
    }
}

pub fn cleanup_shared_resources(mut commands: Commands) {
    commands.remove_resource::<crate::blueprint::Blueprint>();
    commands.remove_resource::<crate::playing::ProducedDimensions>();
}
