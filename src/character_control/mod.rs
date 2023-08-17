use bevy::prelude::*;

use locomotion_system::LocomotionSystemPlugin;

pub mod grounded_states;
pub mod locomotion_system;

#[derive(Component, Clone, Copy)]
pub struct CharacterInput {
    pub locomotion: Entity,
    pub rotation: Entity,
    pub camera_attitude: Entity,
    pub jump: Entity,
}

pub struct CharacterControlPlugin;
impl Plugin for CharacterControlPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((LocomotionSystemPlugin, ));
    }
}
