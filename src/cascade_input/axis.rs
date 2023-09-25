
use std::marker::PhantomData;

use bevy::{prelude::*, input::mouse::MouseMotion};

use super::{button::ButtonInput, CascadeInputSet};


pub struct AxisInputPlugin;
impl Plugin for AxisInputPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(PreUpdate, update_mouse_mapped_sticks.in_set(CascadeInputSet::DeviceMappedInputs))
        ;
    }
}


#[derive(Debug, Component, Clone, Copy, Default, PartialEq)]
pub struct StickInput {
    pub value: Vec2,
}


#[derive(Debug, Component, Clone, Copy, Default, PartialEq)]
pub struct PositionalInput {
    pub value: Vec3,
}

#[derive(Debug, Component, Clone, Copy, Default, PartialEq)]
pub struct EulerAngleInput {
    pub value: Vec3,
}

#[derive(Debug, Component, Clone, Copy, Default, PartialEq)]
pub struct RotationalInput {
    pub value: Quat,
}

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
        let (
            Ok(negative_x), Ok(positive_x), Ok(negative_y), Ok(positive_y)
        ) = (
            buttons.get(src.negative_x), buttons.get(src.positive_x), buttons.get(src.negative_y), buttons.get(src.positive_y),
        ) else {
            warn!("Buttons not found");
            continue;
        };
        let value = Vec2::new(
            if negative_x.pressed() {-1.0} else {0.0} + if positive_x.pressed() {1.0} else {0.0},
            if negative_y.pressed() {-1.0} else {0.0} + if positive_y.pressed() {1.0} else {0.0},
        );
        // check real change for component change detection
        if stick.value != value {
            stick.value = value;
        }
    }
}


#[derive(Debug, Component, Clone, Copy, Default, PartialEq)]
pub struct MappedMouse {
    pub sensitivity: Vec2,
}

fn update_mouse_mapped_sticks(
    mut sticks: Query<(&mut StickInput, &MappedMouse)>,
    mut mouse_motion_events: EventReader<MouseMotion>,
) {
    let delta = mouse_motion_events.iter().map(|e| e.delta).reduce(|v1, v2| v1 + v2).unwrap_or_default();
    for (mut stick, &MappedMouse {sensitivity}) in sticks.iter_mut() {
        let value = delta * sensitivity;
        // check real change for component change detection
        if stick.value != value {
            stick.value = value;
        }
    }
}


#[derive(Debug, Component, Clone, Copy, Default, PartialEq)]
pub struct MaxLength {
    pub value: f32,
}

#[derive(Debug, Component, Clone, Copy, Default, PartialEq)]
pub struct DeadZone {
    pub value: f32,
}

pub fn clamp_stick (
    mut sticks: Query<
        (&mut StickInput, Option<&MaxLength>, Option<&DeadZone>),
        (Or<(With<MaxLength>, With<DeadZone>)>, Changed<StickInput>)
    >,
) {
    for (mut stick, max_len, deadzone) in sticks.iter_mut() {
        let len = stick.value.length();
        if let Some(max_len) = max_len {
            if max_len.value < len {
                stick.value *= max_len.value / len;
            }
        }
        if let Some(deadzone) = deadzone {
            if len < deadzone.value {
                stick.value = Vec2::ZERO;
            }
        }
    }
}


#[derive(Component)]
pub struct MappedEulerAngle<SystemLabel> where
    SystemLabel: Clone + Eq + Send + Sync + 'static
{
    pub source: Entity,
    _phantom: PhantomData<SystemLabel>,
}
impl<S> MappedEulerAngle<S> where 
    S: Clone + Eq + Send + Sync + 'static
{
    pub fn new(source: Entity) -> Self {
        Self {
            source: source,
            _phantom: PhantomData,
        }
    }
}
pub fn update_rotation_from_euler<SystemLabel> (
    mut dests: Query<(&mut RotationalInput, &MappedEulerAngle<SystemLabel>)>,
    source: Query<&EulerAngleInput>,
) where
    SystemLabel: Clone + Eq + Send + Sync + 'static
{
    for (mut rotation, mapping) in dests.iter_mut() {
        let Ok(source) = source.get(mapping.source) else {
            warn!("Entity not found");
            continue;
        };
        let value = Quat::from_euler(EulerRot::YXZ, source.value.y, source.value.x, source.value.z);
        // avoid false change detection
        if rotation.value != value {
            rotation.value = value;
        }
    }
}

