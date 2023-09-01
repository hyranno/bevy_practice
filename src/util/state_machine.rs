use bevy::prelude::*;
use seldom_state::prelude::*;

pub struct StateMachineUtilPlugin;
impl Plugin for StateMachineUtilPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(Update, timeout)
        ;
    }
}

pub fn insert_while_state<State, B> (
    state_machine: StateMachine,
    bundle: B,
) -> StateMachine
where State: Component + Clone, B: Bundle + Clone {
    state_machine
        .on_enter::<State>(move |commands| {commands.insert(bundle.clone());})
        .on_exit::<State>(|commands| {commands.remove::<B>();})
}

/// Done after duration.
#[derive(Component, Clone)]
pub struct Timeout {
    pub timer: Timer,
}
impl Timeout {
    pub fn new(duration: f32) -> Self {
        Self { timer: Timer::from_seconds(duration, TimerMode::Once) }
    }
}
pub fn timeout (
    mut commands: Commands,
    mut state_machines: Query<(Entity, &mut Timeout)>,
    time: Res<Time>,
) {
    let delta = time.delta();
    for (state_machine, mut timeout) in state_machines.iter_mut() {
        timeout.timer.tick(delta);
        if timeout.timer.finished() {
            commands.entity(state_machine).insert(Done::Success);
        }
    }
}
