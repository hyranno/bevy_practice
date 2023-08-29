use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
use seldom_state::prelude::*;

use crate::{
    util::state_machine::{insert_while_state, Timeout},
    cascade_input::button::{ButtonTrigger, ButtonJustPressedTrigger}, global_settings::NamedCollisionGroup,
};

use super::locomotion_system::{BasicLocomotion, AirborneLocomotion, JumpUp, CharacterRotation, HeadRotation};


#[derive(Bundle)]
pub struct GroundedStateMachineBundle {
    pub state_machine: StateMachine,
    pub sensor: Collider,
    pub transform: TransformBundle,
    pub label: GroundedStateMachine,
    pub sensor_label: Sensor,
    pub collision_groups: CollisionGroups,
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
            collision_groups: CollisionGroups::new(NamedCollisionGroup::PURE_SENSOR, NamedCollisionGroup::TERRAIN | NamedCollisionGroup::OBJECT),
            initial_state: Grounded,
        }
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

impl GroundedStateMachine {
    pub fn default_machine (
        jump_button: Entity,
    ) -> StateMachine {
        let ground_contact = GroundContact;
        StateMachine::default()
            .trans::<Grounded>(ground_contact.not(), Airborne)
            .trans::<Airborne>(ground_contact, Grounded)
            .trans::<Grounded>(ButtonJustPressedTrigger { button: jump_button }, JumpingUp)
            .trans::<JumpingUp>((ButtonTrigger { button: jump_button }).not(), Airborne)
            .trans::<JumpingUp>(DoneTrigger::Success, Airborne)
            .set_trans_logging(true)
    }
    pub fn set_state_components_sample (
        state_machine: StateMachine,
    ) -> StateMachine {
        let state_machine = insert_while_state::<Grounded, _>(state_machine, GroundedDefaultBundle::default());
        let state_machine = insert_while_state::<Airborne, _>(state_machine, AirborneDefaultBundle::default());
        let state_machine = insert_while_state::<JumpingUp, _>(state_machine, JumpingUpDefaultBundle::default());
        state_machine
    }
}
#[derive(Bundle, Default, Clone, Copy)]
pub struct GroundedDefaultBundle {
    pub locomotion: BasicLocomotion,
    pub rotation: CharacterRotation,
    pub head_rotation: HeadRotation,
}
#[derive(Bundle, Default, Clone, Copy)]
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
impl Default for JumpingUpDefaultBundle {
    fn default() -> Self {
        Self {
            timeout: Timeout::new(0.1),
            jump: JumpUp::default(),
            locomotion: BasicLocomotion::default(),
            rotation: CharacterRotation,
            head_rotation: HeadRotation,
        }
    }
}
