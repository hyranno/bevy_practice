use bevy::prelude::*;
use self::{
    button_like::ButtonInputPlugin,
    axis::AxisInputPlugin,
};

pub mod button_like;
pub mod axis;

pub struct CascadeInputPlugin;
impl Plugin for CascadeInputPlugin {
    fn build(&self, app: &mut App) {
        app
            .configure_set(Update, CascadeInputSet::DeviceMappedInputs.in_set(CascadeInputSet::Flush))
            .add_plugins((ButtonInputPlugin, AxisInputPlugin, ))
        ;
    }
}

#[derive(Debug, Hash, PartialEq, Eq, Clone, SystemSet)]
pub enum CascadeInputSet {
    Flush,
    DeviceMappedInputs,
}
