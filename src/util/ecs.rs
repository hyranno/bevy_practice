use bevy::prelude::*;


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

