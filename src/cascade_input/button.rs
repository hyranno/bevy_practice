use std::marker::PhantomData;

use bevy::{
    prelude::*,
    input::{keyboard::KeyboardInput, ButtonState, mouse::MouseButtonInput},
};
use seldom_state::trigger::BoolTrigger;

use super::CascadeInputSet;


pub struct ButtonInputPlugin;
impl Plugin for ButtonInputPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(Update, update_key_mapped_buttons.in_set(CascadeInputSet::DeviceMappedInputs))
            .add_systems(PostUpdate, clear_button_events)
        ;
    }
}

#[derive(Component, Clone)]
pub struct ButtonInput {
    state: ButtonState,
    events: Vec<ButtonState>,
}
impl Default for ButtonInput {
    fn default() -> Self {
        Self {
            state: ButtonState::Released,
            events: Vec::default(),
        }
    }
}
impl ButtonInput {
    pub fn new(state: ButtonState) -> Self {
        Self {
            state: state,
            ..default()
        }
    }
    pub fn is(&self, state: ButtonState) -> bool { self.state == state }
    pub fn set(&mut self, state: ButtonState) {
        if !self.is(state) {
            self.state = state;
            self.events.push(state);
        }
    }
    pub fn press(&mut self) { self.set(ButtonState::Pressed); }
    pub fn release(&mut self) { self.set(ButtonState::Released); }
    pub fn pressed(&self) -> bool { self.is(ButtonState::Pressed) }
    pub fn released(&self) -> bool { self.is(ButtonState::Released) }
    pub fn just_pressed(&self) -> bool { self.events.contains(&ButtonState::Pressed) }
    pub fn just_released(&self) -> bool { self.events.contains(&ButtonState::Released) }
    pub fn events(&self) -> Vec<ButtonState> { self.events.clone() }
}

fn clear_button_events (
    mut buttons: Query<&mut ButtonInput>,
) {
    for mut button in buttons.iter_mut() {
        if !button.events.is_empty() {
            button.events.clear();
        }
    }
}

#[derive(Clone, Copy)]
pub struct ButtonTrigger {
    pub button: Entity,
}
impl BoolTrigger for ButtonTrigger {
    type Param<'w, 's> = Query<'w, 's, &'static ButtonInput>;
    fn trigger(
        &self,
        _entity: Entity,
        buttons: Self::Param<'_, '_>,
    ) -> bool {
        let Ok(button) = buttons.get(self.button) else {
            warn!("Entity not found!");
            return false;
        };
        button.pressed()
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum DeviceButtonCode {
    Key(KeyCode),
    Mouse(MouseButton),
    // Gamepad(GamepadButtonType)
}
#[derive(Component)]
pub struct MappedDeviceButton {
    pub code: DeviceButtonCode,
}
impl MappedDeviceButton {
    pub fn new(code: DeviceButtonCode) -> Self {
        Self {
            code,
        }
    }
}

fn update_key_mapped_buttons (
    mut buttons: Query<(&mut ButtonInput, &MappedDeviceButton)>,
    mut keyboard_input_events: EventReader<KeyboardInput>,
    mut mouse_button_input_events: EventReader<MouseButtonInput>,
) {
    for event in keyboard_input_events.iter() {
        let Some(key_code) = event.key_code else {continue;};
        for (mut button, mapped_button) in buttons.iter_mut() {
            if DeviceButtonCode::Key(key_code) == mapped_button.code {
                // avoid false change detection
                if !button.is(event.state) {
                    button.set(event.state)
                }
            }
        }
    }
    for event in mouse_button_input_events.iter() {
        let button_code = event.button;
        for (mut button, mapped_button) in buttons.iter_mut() {
            if DeviceButtonCode::Mouse(button_code) == mapped_button.code {
                // avoid false change detection
                if !button.is(event.state) {
                    button.set(event.state)
                }
            }
        }
    }}



#[derive(Component)]
pub struct Toggle<SystemLabel> where
    SystemLabel: Clone + Eq + Send + Sync + 'static
{
    pub source: Entity,
    _phantom: PhantomData<SystemLabel>
}
impl<S> Toggle<S>
    where S: Clone + Eq + Send + Sync + 'static
{
    pub fn new(source: Entity) -> Self {
        Self {
            source: source,
            _phantom: PhantomData,
        }
    }
}
pub fn update_toggle_buttons<SystemLabel> (
    mut buttons: Query<(&mut ButtonInput, &Toggle<SystemLabel>)>,
    source: Query<&ButtonInput, (Changed<ButtonInput>, Without<Toggle<SystemLabel>>)>,
) where
    SystemLabel: Clone + Eq + Send + Sync + 'static
{
    for (mut button, toggle) in buttons.iter_mut() {
        let Ok(source) = source.get(toggle.source) else {continue;};
        if source.just_pressed() {
            if button.pressed() {
                button.release();
            } else {
                button.press();
            }
        }
    }
}
