
use bevy::prelude::*;

use crate::util::ComponentWrapper;

use super::button_like::{ButtonLike, ButtonInput};

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct StickLabel;
pub type StickInput = ComponentWrapper<Vec2, StickLabel>;

#[derive(Component)]
pub struct StickButtons {
    pub negative_x: Entity,
    pub positive_x: Entity,
    pub negative_y: Entity,
    pub positive_y: Entity,
}

pub fn update_four_button_axis (
    mut sticks: Query<(&mut StickInput, &StickButtons)>,
    buttons: Query<&ButtonInput>,
) {
    for (mut stick, src) in sticks.iter_mut() {
        let Ok(negative_x) = buttons.get(src.negative_x) else {continue;};
        let Ok(positive_x) = buttons.get(src.positive_x) else {continue;};
        let Ok(negative_y) = buttons.get(src.negative_y) else {continue;};
        let Ok(positive_y) = buttons.get(src.positive_y) else {continue;};
        let value = Vec2::new(
            if negative_x.is_pressed() {-1.0} else {0.0} + if positive_x.is_pressed() {1.0} else {0.0},
            if negative_y.is_pressed() {-1.0} else {0.0} + if positive_y.is_pressed() {1.0} else {0.0},
        );
        // check real change for component change detection
        if **stick != value {
            **stick = value;
        }
    }
}

