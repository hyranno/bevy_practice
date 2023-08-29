use bevy::prelude::*;
use bevy_rapier3d::prelude::*;

use crate::{cascade_input::button::ButtonInput, global_settings::NamedCollisionGroup, attack::AttackArea};

use super::ProjectileTemplateBundle;


pub struct SimpleBallPlugin;
impl Plugin for SimpleBallPlugin {
    fn build(&self, app: &mut App) {
        app
            .init_resource::<SimpleBallProjectileBundle>()
            .add_systems(Update, fire_simple_ball)
        ;
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
        if !input.just_pressed() { continue; }
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
    attack: AttackArea,
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

#[derive(Component)]
pub struct SimpleBallProjectileSpawner {
    pub trigger: Entity,    // ButtonInput
    pub muzzle_speed: f32,
}
