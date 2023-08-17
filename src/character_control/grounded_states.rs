use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
use seldom_state::prelude::*;

use crate::cascade_input::button_like::ButtonTrigger;



#[derive(Bundle)]
pub struct GroundedStateMachineBundle {
    pub state_machine: StateMachine,
    pub sensor: Collider,
    pub transform: TransformBundle,
    pub label: GroundedStateMachine,
    pub sensor_label: Sensor,
    pub initial_state: Grounded,
}
impl Default for GroundedStateMachineBundle {
    fn default() -> Self {
        Self {
            state_machine: StateMachine::default(),
            sensor: Collider::ball(1.0),
            transform: TransformBundle::default(),
            label: GroundedStateMachine,
            sensor_label: Sensor,
            initial_state: Grounded,
        }
    }
}
impl GroundedStateMachineBundle {
    pub fn set_default_transitions(
        state_machine: StateMachine,
        jump_button: Entity,
    ) -> StateMachine {
        let ground_contact = GroundContact;
        let jump = JumpingUp {max_acceleration: 1.0, ..default()};
        let jump_trigger = ButtonTrigger {button: jump_button};
        state_machine
            .trans::<Grounded>(ground_contact.not(), Airborne)
            .trans::<Airborne>(ground_contact, Grounded)
            .trans::<Grounded>(jump_trigger, jump)
            .trans::<JumpingUp>(jump_trigger.not(), Airborne)
            .trans::<JumpingUp>(DoneTrigger::Success, Airborne)
            .set_trans_logging(true)
    }
}
#[derive(Component)]
pub struct GroundedStateMachine;
#[derive(Clone, Component, Reflect)]
#[component(storage = "SparseSet")]
pub struct Grounded;
#[derive(Clone, Component, Reflect)]
#[component(storage = "SparseSet")]
pub struct Airborne;
#[derive(Clone, Component, Reflect)]
#[component(storage = "SparseSet")]
pub struct Landing;
#[derive(Clone, Component, Reflect)]
#[component(storage = "SparseSet")]
pub struct JumpingUp {
    pub max_acceleration: f32,
    pub duration: f32,
    pub target_velocity: Vec3,
    pub elapsed_time: f32,
}
impl Default for JumpingUp {
    fn default() -> Self {
        Self {
            max_acceleration: 1.0,
            duration: 0.1,
            target_velocity: 30.0 * Vec3::Y,
            elapsed_time: 0.0,
        }
    }
}

#[derive(Copy, Clone)]
pub struct GroundContact;
impl BoolTrigger for GroundContact {
    type Param<'w, 's> = Res<'w, RapierContext>;
    fn trigger(
        &self,
        entity: Entity,
        rapier_context: Self::Param<'_, '_>,
    ) -> bool {
        let intersections = rapier_context.intersections_with(entity);
        0 < intersections.count()
    }
}
