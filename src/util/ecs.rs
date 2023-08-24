use bevy::prelude::*;
use std::{
    ops::{Deref, DerefMut},
    marker::PhantomData,
};

pub struct EcsUtilPlugin;
impl Plugin for EcsUtilPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(Update, (Lifetime::update, ))
        ;
    }
}

#[derive(Component, Clone, Copy)]
pub struct Lifetime {
    pub duration: f32,
    pub elapsed_time: f32,
}
impl Lifetime {
    pub fn new(duration: f32) -> Self {
        Self { duration, elapsed_time: 0.0 }
    }
    fn update (
        mut commands: Commands,
        mut query: Query<(Entity, &mut Lifetime)>,
        time: Res<Time>,
    ) {
        let delta = time.delta_seconds();
        for (entity, mut lifetime) in query.iter_mut() {
            lifetime.elapsed_time += delta;
            if lifetime.duration < lifetime.elapsed_time {
                commands.entity(entity).despawn();
            }
        }
    }
}

#[derive(Component, Default)]
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

