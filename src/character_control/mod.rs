use bevy::prelude::*;

use locomotion_system::LocomotionSystemPlugin;

use crate::util::ComponentWrapper;

pub mod grounded_states;
pub mod locomotion_system;


pub type AttachedInput<Label> = ComponentWrapper<Entity, Label>;

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
