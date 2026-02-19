use bevy::prelude::*;

#[derive(States, Debug, Clone, PartialEq, Eq, Hash, Default)]
pub enum GameState {
    #[default]
    Menu,
    Playing,
    Scoring,
    Failed,
    Stats,
    Editor,
}

#[derive(Resource, Default)]
pub struct LevelSequence {
    pub entries: Vec<String>,
}

#[derive(Resource, Default)]
pub struct Score {
    pub round: usize,
    pub total_score: f32,
    pub rounds_played: usize,
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
