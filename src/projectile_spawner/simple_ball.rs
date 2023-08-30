use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
use seldom_state::prelude::*;

use crate::{global_settings::NamedCollisionGroup, attack::AttackArea, util::state_machine::{timeout, Timeout}};

use super::{ProjectileTemplateBundle, Magazine, states::{Fire, Reload, SemiAutoStateMachine, Ready}};


pub struct SimpleBallPlugin;
impl Plugin for SimpleBallPlugin {
    fn build(&self, app: &mut App) {
        app
            .init_resource::<ProjectileBundle>()
            .add_systems(Update, (fire, reload.after(timeout)))
        ;
    }
}


#[derive(Component)]
pub struct ProjectileSpawner {
    pub muzzle_speed: f32,
}

#[derive(Bundle)]
pub struct SpawnerBundle {
    spawner: ProjectileSpawner,
    magazine: Magazine,
    state_machine: StateMachine,
    initial_state: Ready,
}
impl SpawnerBundle {
    // use Builder pattern to castomize
    pub fn new (
        fire_button: Entity,
        reload_button: Entity,
    ) -> Self {
        Self {
            spawner: ProjectileSpawner { muzzle_speed: 40.0 },
            magazine: Magazine {
                capacity: 12,
                ammo_count: 12,
            },
            state_machine: SemiAutoStateMachine::default_machine(fire_button, reload_button, 2.0, 1.3),
            initial_state: Ready,
        }
    }
}


#[derive(Resource, Bundle, Clone)]
struct ProjectileBundle {
    model: PbrBundle,
    projectile: ProjectileTemplateBundle,
    attack: AttackArea,
}
impl FromWorld for ProjectileBundle {
    fn from_world(world: &mut World) -> Self {
        let mut meshes = world.resource_mut::<Assets<Mesh>>();
        let mesh = meshes.add(Mesh::try_from(shape::Icosphere { radius: 0.1, ..default() }).unwrap());
        let mut materials = world.resource_mut::<Assets<StandardMaterial>>();
        let material = materials.add(Color::rgb(0.2, 0.2, 0.2).into());
        Self {
            model: PbrBundle {
                mesh: mesh,
                material: material,
                ..default()
            },
            projectile: ProjectileTemplateBundle {
                collider: Collider::ball(0.1),
                collision_group: CollisionGroups::new(
                    NamedCollisionGroup::PROJECTILE | NamedCollisionGroup::ATTACK,
                    NamedCollisionGroup::ALL - NamedCollisionGroup::PROJECTILE
                ),
                ..default()
            },
            attack: AttackArea::default(),
        }
    }
}


fn fire (
    mut commands: Commands,
    mut spawners: Query<(&ProjectileSpawner, &mut Magazine, &GlobalTransform), Added<Fire>>,
    bundle: Res<ProjectileBundle>,
) {
    for (spawner, mut magazine, transform) in spawners.iter_mut() {
        let local_linvel = spawner.muzzle_speed * transform.forward();
        let mut projectile_builder = commands.spawn(bundle.clone());
        projectile_builder.insert(Transform::from(*transform));
        magazine.ammo_count -= 1;
        // TODO add spawner global velocity to projectile velocity
        projectile_builder.insert(Velocity::linear(local_linvel));
        info!("Triggered fire. Ammo count {:?}", magazine.ammo_count);
    }
}

fn reload (
    mut spawners: Query<(&mut Magazine, &Timeout), (With<ProjectileSpawner>, With<Reload>)>,
) {
    for (mut magazine, timeout) in spawners.iter_mut() {
        if timeout.duration < timeout.elapsed_time {
            magazine.ammo_count = magazine.capacity;
        }
    }
}
