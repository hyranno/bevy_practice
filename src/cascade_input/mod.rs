//! Helps you to make input virtual, cascaded.
//! 
//! Inputs can be cascaded, in other words, inputs can be used to update other inputs.
//! For example, WASD keys on device update each of virtual Move-Forward/Left/Back/Right buttons,
//! the Move buttons update virtual Move 2d-stick, then the Move 2d-stick update Locomotion 3d-input.
//!
//! Each input should be Entity.
//! Virtual gamepad will be set of reference to their inputs,
//! which may be child of the gamepad for recursive despawning.
//!
//! Remember cascade systems need to be explicit ordered.

use bevy::prelude::*;
use self::{
    button::ButtonInputPlugin,
    axis::AxisInputPlugin,
};

/// Deals with boolean inputs.
pub mod button;
/// Deals with vector inputs.
pub mod axis;

pub struct CascadeInputPlugin;
impl Plugin for CascadeInputPlugin {
    fn build(&self, app: &mut App) {
        app
            .configure_set(PreUpdate, CascadeInputSet::Flush.after(CascadeInputSet::Begin))
            .configure_set(PreUpdate, CascadeInputSet::DeviceMappedInputs.in_set(CascadeInputSet::Flush))
            .configure_set(PostUpdate, CascadeInputSet::Clear.after(seldom_state::set::StateSet::Transition))
            .add_plugins((ButtonInputPlugin, AxisInputPlugin, ))
        ;
    }
}

/// SystemSet which this module use.
/// Your systems to update inputs will be in set of Flush, on PreUpdate stage.
#[derive(Debug, Hash, PartialEq, Eq, Clone, SystemSet)]
pub enum CascadeInputSet {
    Begin,
    Flush,
    DeviceMappedInputs,
    Clear,
}
