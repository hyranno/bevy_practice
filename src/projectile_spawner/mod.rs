use bevy::prelude::*;
use bevy_rapier3d::prelude::*;

use crate::{util::ecs::Lifetime, global_settings::NamedCollisionGroup, };

use self::simple_ball::SimpleBallPlugin;


pub mod states;
pub mod simple_ball;


pub struct ProjectileSpawnerPlugin;
impl Plugin for ProjectileSpawnerPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(Update, (count_ricochet, ))
            .add_plugins((SimpleBallPlugin, ))
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
    restitution: Restitution,
    collision_group: CollisionGroups,
    ccd: Ccd,
}
impl Default for ProjectileTemplateBundle {
    fn default() -> Self {
        Self {
            velocity: Velocity::default(),
            collider: Collider::default(),
            rigid_body: RigidBody::Dynamic,
            lifetime: Lifetime::new(2.0),
            ricochet: RicochetCount::default(),
            restitution: Restitution::coefficient(0.4),
            collision_group: CollisionGroups::new(NamedCollisionGroup::PROJECTILE, NamedCollisionGroup::ALL - NamedCollisionGroup::PROJECTILE),
            ccd: Ccd::enabled(),
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

#[derive(Component, Clone, Debug)]
pub struct Magazine {
    pub capacity: u32,
    pub ammo_count: u32,
}
impl Magazine {
    pub fn is_full(&self) -> bool { self.ammo_count == self.capacity }
    pub fn is_empty(&self) -> bool { self.ammo_count == 0 }
}
