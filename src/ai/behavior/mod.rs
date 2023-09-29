
use std::sync::Arc;

use bevy::{prelude::*, ecs::system::SystemParam};

use bevior_tree::{
    BehaviorTree,
    task::{TaskChecker, TaskState, TaskImpl, Task},
    sequential::variants::Sequence,
    conditional::{ConditionalLoop, variants::RepeatCount}, prelude::{Always, ForcedSequence},
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
            ]), RepeatCount {count: 3}),
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
        Always
    );
    BehaviorTree::new(root)
}


pub struct WaitTask {
    task: Arc<TaskImpl<TimeChecker>>,
}
impl WaitTask {
    pub fn new(
        duration: f32,
    ) -> Arc<Self> {
        let task = TaskImpl::new(TimeChecker)
            .insert_while_running(WrappedTimer { timer: Timer::from_seconds(duration, TimerMode::Once) })
        ;
        Arc::new(Self {
            task: Arc::new(task),
        })
    }
}
impl Task for WaitTask {
    type Checker = TimeChecker;
    fn task_impl(&self) -> Arc<TaskImpl<Self::Checker>> {
        self.task.clone()
    }
}

pub struct JumpTask {
    task: Arc<TaskImpl<TimeChecker>>,
}
impl JumpTask {
    pub fn new(
        jump: JumpUp,
        duration: f32,
    ) -> Arc<Self> {
        let task = TaskImpl::new(TimeChecker)
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
    type Checker = TimeChecker;
    fn task_impl(&self) -> Arc<TaskImpl<Self::Checker>> {
        self.task.clone()
    }
}


#[derive(Debug, Default)]
pub struct TimeChecker;
impl TaskChecker for TimeChecker {
    type Param<'w, 's> = Query<'w, 's, &'static WrappedTimer>;
    fn check (
        &self,
        entity: Entity,
        param: <<Self as TaskChecker>::Param<'_, '_> as SystemParam>::Item<'_, '_>,
    ) -> TaskState {
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
}


pub struct MoveToTask {
    task: Arc<TaskImpl<MoveToChecker>>,
}
impl Task for MoveToTask {
    type Checker = MoveToChecker;
    fn task_impl(&self) -> Arc<TaskImpl<Self::Checker>> {
        self.task.clone()
    }
}
impl MoveToTask {
    pub fn new(move_to: MoveTo, locomotion: BasicLocomotion, done_distance: f32, giveup_distance: Option<f32>) -> Arc<Self> {
        let task = TaskImpl::new(MoveToChecker { done_distance, giveup_distance })
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
impl TaskChecker for MoveToChecker {
    type Param<'w, 's> = (
        Query<'w, 's, (&'static MoveTo, &'static Parent)>,
        Query<'w, 's, &'static GlobalTransform, With<Children>>,
    );
    fn check (
        &self,
        entity: Entity,
        param: <<Self as TaskChecker>::Param<'_, '_> as SystemParam>::Item<'_, '_>,
    ) -> TaskState {
        let (query_move_to, characters) = param;
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
    }
}