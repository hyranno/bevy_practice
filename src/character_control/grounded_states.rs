use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
use seldom_state::prelude::*;

use crate::{
    util::state_machine::{insert_while_state, Timeout},
    cascade_input::button_like::ButtonTrigger,
};

use super::locomotion_system::{BasicLocomotion, AirborneLocomotion, JumpUp, CharacterRotation, HeadRotation};


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
        let jump_trigger = ButtonTrigger {button: jump_button};
        state_machine
            .trans::<Grounded>(ground_contact.not(), Airborne)
            .trans::<Airborne>(ground_contact, Grounded)
            .trans::<Grounded>(jump_trigger, JumpingUp)
            .trans::<JumpingUp>(jump_trigger.not(), Airborne)
            .trans::<JumpingUp>(DoneTrigger::Success, Airborne)
            .set_trans_logging(true)
    }
    pub fn set_default_state_components (
        state_machine: StateMachine,
    ) -> StateMachine {
        let state_machine = insert_while_state::<Grounded, _>(state_machine, GroundedDefaultBundle {
            locomotion: BasicLocomotion { speed: 4.0, max_acceleration: 2.0 },
            rotation: CharacterRotation,
            head_rotation: HeadRotation,
        });
        let state_machine = insert_while_state::<Airborne, _>(state_machine, AirborneDefaultBundle {
            locomotion: AirborneLocomotion { speed: 1.0, max_acceleration: 0.2 },
            rotation: CharacterRotation,
            head_rotation: HeadRotation,
        });
        let state_machine = insert_while_state::<JumpingUp, _>(state_machine, JumpingUpDefaultBundle {
            timeout: Timeout { duration: 0.1, elapsed_time: 0.0 },
            jump: JumpUp {target_velocity: 20.0 * Vec3::Y, max_acceleration: 1.0},
            locomotion: BasicLocomotion { speed: 4.0, max_acceleration: 2.0 },
            rotation: CharacterRotation,
            head_rotation: HeadRotation,
        });
        state_machine
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
pub struct JumpingUp;

#[derive(Bundle, Clone, Copy)]
pub struct GroundedDefaultBundle {
    pub locomotion: BasicLocomotion,
    pub rotation: CharacterRotation,
    pub head_rotation: HeadRotation,
}
#[derive(Bundle, Clone, Copy)]
pub struct AirborneDefaultBundle {
    pub locomotion: AirborneLocomotion,
    pub rotation: CharacterRotation,
    pub head_rotation: HeadRotation,
}
#[derive(Bundle, Clone, Copy)]
pub struct JumpingUpDefaultBundle {
    pub timeout: Timeout,
    pub jump: JumpUp,
    pub locomotion: BasicLocomotion,
    pub rotation: CharacterRotation,
    pub head_rotation: HeadRotation,
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
