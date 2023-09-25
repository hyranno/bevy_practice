use std::marker::PhantomData;

use bevy::prelude::*;

use locomotion_system::LocomotionSystemPlugin;

pub mod grounded_states;
pub mod locomotion_system;


#[derive(Debug, Component)]
pub struct AttachedInput<Label> {
    entity: Entity,
    _phantom: PhantomData<Label>,
}
impl<Label> AttachedInput<Label> {
    pub fn new(entity: Entity) -> Self {
        Self {
            entity,
            _phantom: PhantomData::<Label>::default()
        }
    }
}

#[derive(Component, Default)]
pub struct Head;
#[derive(Bundle, Default)]
pub struct HeadBundle {
    head: Head,
    transform: TransformBundle,
}

#[derive(Clone, Copy, PartialEq, Eq, Default)]
pub struct Locomotion;
#[derive(Clone, Copy, PartialEq, Eq, Default)]
pub struct Rotation;
#[derive(Clone, Copy, PartialEq, Eq, Default)]
pub struct  HeadAttitude;
#[derive(Clone, Copy, PartialEq, Eq, Default)]
pub struct Jump;


pub struct CharacterControlPlugin;
impl Plugin for CharacterControlPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((LocomotionSystemPlugin, ));
    }
}
