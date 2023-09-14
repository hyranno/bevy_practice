
use std::sync::Arc;

use bevy::{prelude::*, ecs::system::SystemParam};

use crate::{
    behavior_tree::{task::{TaskChecker, TaskState, TaskImpl, Task}, Node, sequencial::Sequence, BehaviorTree},
    util::ecs::WrappedTimer,
    character_control::locomotion_system::JumpUp,
};


pub fn jump10() -> BehaviorTree {
    let waiter = Arc::new(WaitTask::new(2.0));
    let jumper = Arc::new(JumpTask::new(
        JumpUp { target_velocity: Vec3::Y, max_acceleration: 0.4 },
        0.2
    ));
    let tasks: Vec<Arc<dyn Node>> = vec![
        waiter.clone(), jumper.clone(),
        waiter.clone(), jumper.clone(),
        waiter.clone(), jumper.clone(),
        waiter.clone(), jumper.clone(),
        waiter.clone(), jumper.clone(),
        waiter.clone(), jumper.clone(),
        waiter.clone(), jumper.clone(),
        waiter.clone(), jumper.clone(),
        waiter.clone(), jumper.clone(),
        waiter.clone(), jumper.clone(),
    ];
    let sequence = Arc::new(Sequence::new(tasks));
    BehaviorTree::new(sequence)
}


pub struct WaitTask {
    task: Arc<TaskImpl<TimeChecker>>,
}
impl WaitTask {
    pub fn new(
        duration: f32,
    ) -> Self {
        let task = TaskImpl::new(TimeChecker)
            .on_enter(move |entity, mut commands| {
                commands.entity(entity).insert(WrappedTimer { timer: Timer::from_seconds(duration, TimerMode::Once) });
            })
            .on_exit(|entity, mut commands| {
                commands.entity(entity).remove::<WrappedTimer>();
            })
        ;
        Self {
            task: Arc::new(task),
        }
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
    ) -> Self {
        let task = TaskImpl::new(TimeChecker)
            .on_enter(move |entity, mut commands| {
                commands.entity(entity)
                    .insert(WrappedTimer { timer: Timer::from_seconds(duration, TimerMode::Once) })
                    .insert(jump)
                ;
            })
            .on_exit(|entity, mut commands| {
                commands.entity(entity)
                    .remove::<WrappedTimer>()
                    .remove::<JumpUp>()
                ;
            })
        ;
        Self {
            task: Arc::new(task),
        }
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
