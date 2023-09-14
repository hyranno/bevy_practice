use bevy::prelude::*;
use std::{
    ops::{Deref, DerefMut},
    marker::PhantomData,
};

pub struct EcsUtilPlugin;
impl Plugin for EcsUtilPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(Update, (WrappedTimer::update, Lifetime::update, ))
        ;
    }
}

#[derive(Debug, Component, Clone)]
pub struct WrappedTimer {
    pub timer: Timer,
}
impl WrappedTimer {
    fn update (
        mut query: Query<&mut WrappedTimer>,
        time: Res<Time>,
    ) {
        let delta = time.delta();
        for mut timer in query.iter_mut() {
            timer.timer.tick(delta);
        }
    }
}

/// Despawns after lifetime.
#[derive(Component, Clone)]
pub struct Lifetime {
    pub timer: Timer,
}
impl Lifetime {
    pub fn new(duration: f32) -> Self {
        Self { timer: Timer::from_seconds(duration, TimerMode::Once) }
    }
    fn update (
        mut commands: Commands,
        mut query: Query<(Entity, &mut Lifetime)>,
        time: Res<Time>,
    ) {
        let delta = time.delta();
        for (entity, mut lifetime) in query.iter_mut() {
            lifetime.timer.tick(delta);
            if lifetime.timer.finished() {
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

