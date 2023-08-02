
use bevy::{prelude::*, input::mouse::MouseMotion};

use crate::util::ComponentWrapper;

use super::button_like::{ButtonLike, ButtonInput};

#[derive(Clone, Copy, PartialEq, Eq, Default)]
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


#[derive(Component)]
pub struct MappedMouse {
    pub sensitivity: Vec2,
}

pub fn update_mouse_mapped_sticks(
    mut sticks: Query<(&mut StickInput, &MappedMouse)>,
    mut mouse_motion_events: EventReader<MouseMotion>,
) {
    let delta = mouse_motion_events.iter().map(|e| e.delta).reduce(|v1, v2| v1 + v2).unwrap_or_default();
    for (mut stick, &MappedMouse {sensitivity}) in sticks.iter_mut() {
        let value = delta * sensitivity;
        // check real change for component change detection
        if **stick != value {
            **stick = value;
        }
    }
}


#[derive(Clone, Copy, PartialEq, Eq)]
pub struct MaxLengthLabel;
pub type MaxLength = ComponentWrapper<f32, MaxLengthLabel>;

#[derive(Clone, Copy, PartialEq, Eq, Default)]
pub struct DeadZoneLabel;
pub type DeadZone = ComponentWrapper<f32, DeadZoneLabel>;

pub fn clamp_stick (
    mut sticks: Query<
        (&mut StickInput, Option<&MaxLength>, Option<&DeadZone>),
        (Or<(With<MaxLength>, With<DeadZone>)>, Changed<StickInput>)
    >,
) {
    for (mut stick, max_len, deadzone) in sticks.iter_mut() {
        let len = stick.length();
        if let Some(max_len) = max_len {
            if **max_len < len {
                **stick *= **max_len / len;
            }
        }
        if let Some(deadzone) = deadzone {
            if len < **deadzone {
                **stick = Vec2::ZERO;
            }
        }
    }
}
