
use std::{sync::{Arc, Mutex}, future::Future};
use bevy::prelude::*;
use genawaiter::{sync::{Co, Gen}, GeneratorState};

use self::nullable_access::{NullableWorldAccess, TemporalWorldSharing};

pub mod task;
pub mod sequencial;

mod nullable_access;


pub struct BehaviorTreePlugin;
impl Plugin for BehaviorTreePlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(PostUpdate, update)
        ;
    }
}

#[derive(Component)]
pub struct BehaviorTree {
    root: Arc<dyn Node>,
    gen: Option<Box<dyn NodeGen>>,
    result: Option<NodeResult>,
    world: Arc<Mutex<NullableWorldAccess>>,
}
#[derive(Component)]
pub struct Freeze;
#[derive(Component)]
pub struct Abort;

impl BehaviorTree {
    pub fn new(root: Arc<dyn Node>) -> Self {
        Self {
            root,
            gen: None,
            result: None,
            world: Arc::<Mutex::<NullableWorldAccess>>::default(),
        }
    }
    fn stub(&self) -> Self {
        Self {
            root: Arc::<StubNode>::default(),
            gen: None,
            result: None,
            world: Arc::default(),
        }
    }
}

fn update (
    world: &mut World,
    query: &mut QueryState<(Entity, &mut BehaviorTree, Option<&Abort>), Without<Freeze>>,
) {
    // Pull the trees out of the world so we can invoke mutable methods on them.
    let mut borrowed_trees: Vec<(Entity, BehaviorTree, bool)> = query.iter_mut(world)
        .map(|(entity, mut tree, abort)| {
            let stub = tree.stub();
            (entity, std::mem::replace(tree.as_mut(), stub), abort.is_some())
        })
        .collect()
    ;

    for (entity, tree, abort) in borrowed_trees.iter_mut() {
        if tree.result.is_some() { continue; }
        let _temporal_world = TemporalWorldSharing::new(tree.world.clone(), world);
        if tree.gen.is_none() {
            tree.gen = Some(tree.root.clone().run(tree.world.clone(), *entity));
        }
        let Some(gen) = tree.gen.as_mut() else {unreachable!()};
        if let GeneratorState::Complete(result) = if !*abort {gen.resume()} else {gen.abort()} {
            tree.result = Some(result);
            tree.gen = None;
        }
    }

    // put the borrowed trees back
    for (entity, tree, _) in borrowed_trees {
        *query.get_mut(world, entity).unwrap().1 = tree;
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum NodeResult {
    Success,
    Failure,
    Aborted,
}

#[derive(Debug, PartialEq, Eq)]
pub enum ResumeSignal {
    Resume,
    Abort,
}

type Y = ();
type R = ResumeSignal;
type C = NodeResult;

pub trait NodeGen: Send + Sync {
    fn resume(&mut self) -> GeneratorState<Y, C>;
    fn abort(&mut self) -> GeneratorState<Y, C>;
}
impl<F> NodeGen for Gen<Y, R, F> where
F: Future<Output = C> + Send + Sync + 'static
{
    fn resume(&mut self) -> GeneratorState<Y, C> {
        Gen::<Y, R, F>::resume_with(self, ResumeSignal::Resume)
    }
    fn abort(&mut self) -> GeneratorState<Y, C> {
        Gen::<Y, R, F>::resume_with(self, ResumeSignal::Abort)
    }
}

pub trait Node: Send + Sync {
    fn run(self: Arc<Self>, world: Arc<Mutex<NullableWorldAccess>>, entity: Entity) -> Box<dyn NodeGen>;
}

#[derive(Debug, Default)]
struct StubNode;
impl Node for StubNode {
    fn run(self: Arc<Self>, _world: Arc<Mutex<NullableWorldAccess>>, _entity: Entity) -> Box<dyn NodeGen> {
        let producer = |_co| async move {
            NodeResult::Failure
        };
        Box::new(Gen::new(producer))
    }
}


async fn complete_or_yield(co: &Co<(), ResumeSignal>, gen: &mut Box<dyn NodeGen>) -> NodeResult {
    let mut state = gen.resume();
    loop {
        match state {
            GeneratorState::Yielded(yielded_value) => {
                let signal = co.yield_(yielded_value).await;
                if signal == ResumeSignal::Abort {
                    gen.abort();
                    return NodeResult::Aborted;
                }
                state = gen.resume();
            },
            GeneratorState::Complete(result) => { return result; }
        }
    }
}
