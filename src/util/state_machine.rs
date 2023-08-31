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
where State: Component + Clone, B: Bundle + Clone + Copy {
    state_machine
        .on_enter::<State>(move |commands| {commands.insert(bundle);})
        .on_exit::<State>(|commands| {commands.remove::<B>();})
}

#[derive(Component, Clone, Copy)]
pub struct Timeout {
    pub duration: f32,
    pub elapsed_time: f32,
}
impl Timeout {
    pub fn new(duration: f32) -> Self {
        Self {
            duration: duration,
            elapsed_time: 0.0,
        }
    }
    pub fn expired(&self) -> bool {
        self.duration < self.elapsed_time
    }
}
pub fn timeout (
    mut commands: Commands,
    mut params: Query<(Entity, &mut Timeout)>,
    time: Res<Time>,
) {
    let delta = time.delta_seconds();
    for (state_machine, mut param) in params.iter_mut() {
        param.elapsed_time += delta;
        if param.expired() {
            commands.entity(state_machine).insert(Done::Success);
        }
    }
}
