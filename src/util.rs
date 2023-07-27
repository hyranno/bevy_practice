use bevy::prelude::*;
use std::{
    ops::{Deref, DerefMut},
    marker::PhantomData,
};

#[derive(Component)]
pub struct ComponentWrapper<T, ComponentLabel>
    where ComponentLabel: Clone + Eq + Send + Sync + 'static
{
    value: T,
    _phantom: PhantomData<ComponentLabel>
}
impl<T, L> ComponentWrapper<T, L>
    where L: Clone + Eq + Send + Sync + 'static
{
    pub fn new(value: T) -> Self {
        Self {
            value: value,
            _phantom: PhantomData,
        }
    }
}
impl<T, L> Deref for ComponentWrapper<T, L>
    where L: Clone + Eq + Send + Sync + 'static
{
    type Target = T;
    fn deref(&self) -> &Self::Target {
        &self.value
    }
}
impl<T, L> DerefMut for ComponentWrapper<T, L>
    where L: Clone + Eq + Send + Sync + 'static
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.value
    }
}

pub trait SimpleDelegate {
    type Base;
    fn base(&self) -> &Self::Base;
}
pub trait SimpleDelegateMut: SimpleDelegate {
    fn base_mut(&mut self) -> &mut Self::Base;
}
