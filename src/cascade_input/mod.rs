use bevy::prelude::*;
use self::{
    button_like::{update_key_mapped_buttons, clear_button_events},
    axis::update_mouse_mapped_sticks
};

pub mod button_like;
pub mod axis;

pub struct CascadeInputPlugin;
impl Plugin for CascadeInputPlugin {
    fn build(&self, app: &mut App) {
        app
            .configure_set(Update, CascadeInputSet::DeviceMappedInputs.in_set(CascadeInputSet::Flush))
            .add_systems(Update,update_key_mapped_buttons.in_set(CascadeInputSet::DeviceMappedInputs))
            .add_systems(Update,update_mouse_mapped_sticks.in_set(CascadeInputSet::DeviceMappedInputs))
            .add_systems(PostUpdate, clear_button_events)
        ;
    }
}

#[derive(Debug, Hash, PartialEq, Eq, Clone, SystemSet)]
pub enum CascadeInputSet {
    Flush,
    DeviceMappedInputs,
}
