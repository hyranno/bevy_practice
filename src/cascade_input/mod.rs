use bevy::prelude::*;
use self::{
    button::ButtonInputPlugin,
    axis::AxisInputPlugin,
};

pub mod button;
pub mod axis;

pub struct CascadeInputPlugin;
impl Plugin for CascadeInputPlugin {
    fn build(&self, app: &mut App) {
        app
            .configure_set(PreUpdate, CascadeInputSet::DeviceMappedInputs.in_set(CascadeInputSet::Flush))
            .configure_set(PostUpdate, CascadeInputSet::Clear.after(seldom_state::set::StateSet::Transition))
            .add_plugins((ButtonInputPlugin, AxisInputPlugin, ))
        ;
    }
}

#[derive(Debug, Hash, PartialEq, Eq, Clone, SystemSet)]
pub enum CascadeInputSet {
    Flush,
    DeviceMappedInputs,
    Clear,
}
