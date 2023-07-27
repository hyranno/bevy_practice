use std::{
    ops::{Deref, DerefMut},
    marker::PhantomData
};

use bevy::{
    prelude::*,
    input::{keyboard::KeyboardInput, ButtonState},
};
use crate::util::{
    SimpleDelegate, SimpleDelegateMut, ComponentWrapper,
};

pub trait ButtonLike {
    fn is(&self, state: ButtonState) -> bool;
    fn is_pressed(&self) -> bool {
        self.is(ButtonState::Pressed)
    }
}
pub trait ButtonLikeMut: ButtonLike {
    fn press(&mut self);
    fn release(&mut self);
    fn set(&mut self, state: ButtonState) {
        if state == ButtonState::Pressed {
            self.press()
        } else {
            self.release()
        }
    }
}

/*
TODO: remove this when Deref<Target=ButtonLike> can be T:ButtonLike
 */
impl<T: ButtonLike, L> ButtonLike for ComponentWrapper<T, L>
    where L: Clone + Eq + Send + Sync + 'static
{
    fn is(&self, state: ButtonState) -> bool {self.deref().is(state)}
}
impl<T: ButtonLikeMut, L> ButtonLikeMut for ComponentWrapper<T, L>
    where L: Clone + Eq + Send + Sync + 'static
{
    fn press(&mut self) {self.deref_mut().press()}
    fn release(&mut self) {self.deref_mut().release()}
}


impl ButtonLike for bool {
    fn is(&self, state: ButtonState) -> bool {
        *self == (state == ButtonState::Pressed)
    }
}
impl ButtonLikeMut for bool {
    fn press(&mut self) {*self = true}
    fn release(&mut self) {*self = false}
}

impl<Delegatee, Delegator> ButtonLike for Delegator
where
    Delegatee: ButtonLike,
    Delegator: SimpleDelegate<Base = Delegatee>
{
    fn is(&self, state: ButtonState) -> bool {
        self.base().is(state)
    }
}
impl<Delegatee, Delegator> ButtonLikeMut for Delegator
where
    Delegatee: ButtonLikeMut,
    Delegator: SimpleDelegateMut<Base = Delegatee>
{
    fn press(&mut self) {
        self.base_mut().press()
    }
    fn release(&mut self) {
        self.base_mut().release()
    }
}



#[derive(Component)]
pub struct MappedKey<Button: Component + ButtonLikeMut> {
    pub key_code: KeyCode,
    _phantom: PhantomData<Button>,
}
impl<B: Component + ButtonLikeMut> MappedKey<B> {
    pub fn new(key_code: KeyCode) -> Self {
        Self {
            key_code: key_code,
            _phantom: PhantomData,
        }
    }
}

pub fn update_key_mapped_buttons<Button> (
    mut buttons: Query<(&mut Button, &MappedKey<Button>)>,
    mut keyboard_input_events: EventReader<KeyboardInput>,
)
    where Button: Component + ButtonLikeMut
{
    for event in keyboard_input_events.iter() {
        let Some(key_code) = event.key_code else {continue;};
        for (mut button, mapped_key) in buttons.iter_mut() {
            if key_code == mapped_key.key_code {
                button.set(event.state)
            }
        }
    }
}



