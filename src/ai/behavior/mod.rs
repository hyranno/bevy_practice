
use std::sync::Arc;

use bevy::prelude::*;

use bevior_tree::{
    BehaviorTree,
    task::{TaskState, TaskImpl, Task},
    sequential::variants::{Sequence, ForcedSequence},
    conditional::ConditionalLoop,
};
use crate::{
    util::ecs::WrappedTimer,
    character_control::locomotion_system::{JumpUp, CharacterRotation, BasicLocomotion},
};

use super::{AiTarget, MoveTo};


pub fn sample_behavior() -> BehaviorTree {
    let locomotion = BasicLocomotion {speed: 1.0, max_acceleration: 0.4};
    let root = ConditionalLoop::new(ForcedSequence::new(vec![
            ConditionalLoop::new(Sequence::new(vec![
                JumpTask::new(
                    JumpUp { target_velocity: Vec3::Y, max_acceleration: 0.4 },
                    0.2
                ),
                WaitTask::new(1.0),
            ]), |In((_, count, _))| count < 3),
            MoveToTask::new(
                MoveTo {
                    target: AiTarget::Position(Vec3::new(10.0, 0.0, 0.0)), strafe: false, speed_coef: 1.0
                }, locomotion, 1.0, None
            ),
            MoveToTask::new(
                MoveTo {
                    target: AiTarget::Position(Vec3::ZERO), strafe: true, speed_coef: 1.0
                }, locomotion, 1.0, None
            ),
        ]),
        |In(_)| true
    );
    BehaviorTree::new(root)
}


pub struct WaitTask {
    task: Arc<TaskImpl>,
}
impl WaitTask {
    pub fn new(
        duration: f32,
    ) -> Arc<Self> {
        let task = TaskImpl::new(check_time)
            .insert_while_running(WrappedTimer { timer: Timer::from_seconds(duration, TimerMode::Once) })
        ;
        Arc::new(Self {
            task: Arc::new(task),
        })
    }
}
impl Task for WaitTask {
    fn task_impl(&self) -> Arc<TaskImpl> {
        self.task.clone()
    }
}

pub struct JumpTask {
    task: Arc<TaskImpl>,
}
impl JumpTask {
    pub fn new(
        jump: JumpUp,
        duration: f32,
    ) -> Arc<Self> {
        let task = TaskImpl::new(check_time)
            .insert_while_running((
                WrappedTimer { timer: Timer::from_seconds(duration, TimerMode::Once) },
                jump,
            ))
        ;
        Arc::new(Self {
            task: Arc::new(task),
        })
    }
}
impl Task for JumpTask {
    fn task_impl(&self) -> Arc<TaskImpl> {
        self.task.clone()
    }
}

fn check_time(In(entity): In<Entity>, param: Query<&WrappedTimer>) -> TaskState {
    if let Ok(timer) = param.get(entity) {
        if timer.timer.finished() {
            TaskState::Success
        } else {
            TaskState::Running
        }
    } else {
        warn!("Entity does not have WrappwdTimer!");
        TaskState::Failure
    }
}


pub struct MoveToTask {
    task: Arc<TaskImpl>,
}
impl Task for MoveToTask {
    fn task_impl(&self) -> Arc<TaskImpl> {
        self.task.clone()
    }
}
impl MoveToTask {
    pub fn new(move_to: MoveTo, locomotion: BasicLocomotion, done_distance: f32, giveup_distance: Option<f32>) -> Arc<Self> {
        let task = TaskImpl::new(MoveToChecker { done_distance, giveup_distance }.into_system())
            .insert_while_running((
                move_to, locomotion, CharacterRotation,
            ))
        ;
        Arc::new(Self { task: Arc::new(task) })
    }
}
#[derive(Debug, Default)]
pub struct MoveToChecker {
    pub done_distance: f32,
    pub giveup_distance: Option<f32>,
}
impl MoveToChecker {
    pub fn into_system(self) -> impl ReadOnlySystem<In=Entity, Out=TaskState> {
        let func = move |In(entity): In<Entity>, query_move_to: Query<(&MoveTo, &Parent)>, characters: Query<&GlobalTransform, With<Children>>| {
            let Ok((move_to, character)) = query_move_to.get(entity) else {
                warn!("Checking entity without MoveTo");
                return TaskState::Failure;
            };
            let Ok(transform) = characters.get(character.get()) else {
                warn!("Invalid parent of MoveTo.");
                return TaskState::Failure;
            };
            let target_position = match move_to.target {
                AiTarget::Position(position) => position,
                AiTarget::Entity(entity) => {
                    let Ok(transform) = characters.get(entity) else {
                        warn!("Targeting entity with no GlobalTransform.");
                        return TaskState::Failure;
                    };
                    transform.translation()
                },
            };
            // navmesh?
            let len = (target_position - transform.translation()).length();
            if len < self.done_distance {
                TaskState::Success
            } else if self.giveup_distance.is_some() && self.giveup_distance.unwrap() < len {
                TaskState::Failure
            } else {
                TaskState::Running
            }
        };
        IntoSystem::into_system(func)
    }
}
