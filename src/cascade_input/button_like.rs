use std::marker::PhantomData;

use bevy::{
    prelude::*,
    input::{keyboard::KeyboardInput, ButtonState},
};
use seldom_state::trigger::BoolTrigger;
use crate::util::ComponentWrapper;

pub trait ButtonLike {
    fn is(&self, state: ButtonState) -> bool;
    fn pressed(&self) -> bool {
        self.is(ButtonState::Pressed)
    }
}
pub trait ButtonLikeMut: ButtonLike {
    fn press(&mut self);
    fn release(&mut self);
    fn set_state(&mut self, state: ButtonState) {
        if state == ButtonState::Pressed {
            self.press()
        } else {
            self.release()
        }
    }
}


#[derive(Clone, Copy, PartialEq, Eq, Default)]
pub struct ButtonLabel;
pub type ButtonInput = ComponentWrapper<bool, ButtonLabel>;

impl ButtonLike for bool {
    fn is(&self, state: ButtonState) -> bool {
        *self == (state == ButtonState::Pressed)
    }
}
impl ButtonLikeMut for bool {
    fn press(&mut self) {*self = true}
    fn release(&mut self) {*self = false}
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

#[derive(Component)]
pub struct MappedKey {
    pub key_code: KeyCode,
}
impl MappedKey {
    pub fn new(key_code: KeyCode) -> Self {
        Self {
            key_code: key_code,
        }
    }
}

pub fn update_key_mapped_buttons (
    mut buttons: Query<(&mut ButtonInput, &MappedKey)>,
    mut keyboard_input_events: EventReader<KeyboardInput>,
) {
    for event in keyboard_input_events.iter() {
        let Some(key_code) = event.key_code else {continue;};
        for (mut button, mapped_key) in buttons.iter_mut() {
            if key_code == mapped_key.key_code {
                button.set_state(event.state)
            }
        }
    }
}



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
        if **source {
            **button = !**button;
        }
    }
}
