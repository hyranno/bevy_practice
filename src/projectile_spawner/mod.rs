use bevy::prelude::*;
use bevy_rapier3d::prelude::*;

use crate::{cascade_input::{button_like::{ButtonInput, ButtonLike}, CascadeInputSet}, util::ecs::Lifetime, global_settings::NamedCollisionGroup};

pub struct ProjectileSpawnerPlugin;
impl Plugin for ProjectileSpawnerPlugin {
    fn build(&self, app: &mut App) {
        app
            .init_resource::<SimpleBallProjectileBundle>()
            .add_systems(Update, (count_ricochet, fire_simple_ball.after(CascadeInputSet::Flush)))
        ;
    }
}

#[derive(Bundle, Clone)]
struct ProjectileTemplateBundle {
    velocity: Velocity,
    collider: Collider,
    rigid_body: RigidBody,
    lifetime: Lifetime,
    ricochet: RicochetCount,
    collision_group: CollisionGroups,
}
impl Default for ProjectileTemplateBundle {
    fn default() -> Self {
        Self {
            velocity: Velocity::default(),
            collider: Collider::default(),
            rigid_body: RigidBody::Dynamic,
            lifetime: Lifetime::new(2.0),
            ricochet: RicochetCount::default(),
            collision_group: CollisionGroups::new(NamedCollisionGroup::PROJECTILE, NamedCollisionGroup::ALL - NamedCollisionGroup::PROJECTILE),
        }
    }
}

#[derive(Component, Clone, Copy)]
pub struct RicochetCount {
    pub remains: u32,
}
impl Default for RicochetCount {
    fn default() -> Self {
        Self {
            remains: 1,
        }
    }
}
fn count_ricochet (
    mut commands: Commands,
    mut projectiles: Query<(Entity, &mut RicochetCount)>,
    rapier_context: Res<RapierContext>,
) {
    for (projectile, mut count) in projectiles.iter_mut() {
        let contacts = rapier_context.contacts_with(projectile).filter(|pair| pair.has_any_active_contacts());
        if 0 < contacts.count() {
            count.remains -= 1;
        }
        if count.remains <= 0 {
            commands.entity(projectile).despawn();
        }
    }
}

fn fire_simple_ball(
    mut commands: Commands,
    inputs: Query<&ButtonInput, Changed<ButtonInput>>,
    spawners: Query<(&SimpleBallProjectileSpawner, &GlobalTransform)>,
    bundle: Res<SimpleBallProjectileBundle>,
) {
    for (spawner, transform) in spawners.iter() {
        let Ok(input) = inputs.get(spawner.trigger) else { continue; };
        if !input.pressed() { continue; }
        let local_linvel = spawner.muzzle_speed * transform.forward();
        let mut projectile_builder = commands.spawn(bundle.clone());
        projectile_builder.insert(Transform::from(*transform));
        // TODO add spawner global velocity to projectile velocity
        projectile_builder.insert(Velocity::linear(local_linvel));
    }
}

#[derive(Resource, Bundle, Clone)]
struct SimpleBallProjectileBundle {
    model: PbrBundle,
    projectile: ProjectileTemplateBundle,
}
impl FromWorld for SimpleBallProjectileBundle {
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
                ..default()
            }
        }
    }
}

#[derive(Component)]
pub struct SimpleBallProjectileSpawner {
    pub trigger: Entity,    // ButtonInput
    pub muzzle_speed: f32,
}
