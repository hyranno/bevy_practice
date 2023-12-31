use bevy::prelude::*;
use bevy_rapier3d::prelude::*;

use crate::global_settings::NamedCollisionGroup;


pub struct AttackPlugin;
impl Plugin for AttackPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(Update, hit)
            .add_systems(PostUpdate, clear_hit)
            .add_systems(PostUpdate, trace_hit.before(clear_hit))
        ;
    }
}


#[derive(Component, Default, Clone)]
pub struct AttackArea {
    pub events: Vec<HitEvent>,
}
#[derive(Component, Default, Clone)]
pub struct HitArea {
    pub events: Vec<HitEvent>,
}
#[derive(Clone, Copy, Debug)]
pub struct HitEvent {
    pub hit_on: Entity,
    pub attack: Entity,
}

#[derive(Bundle, Clone)]
pub struct AttackAreaBundle {
    collider: Collider,
    collision_group: CollisionGroups,
    label: AttackArea,
    sensor_label: Sensor,
}
impl Default for AttackAreaBundle {
    fn default() -> Self {
        Self {
            collider: Collider::ball(1.0),
            collision_group: CollisionGroups::new(NamedCollisionGroup::ATTACK, NamedCollisionGroup::ALL),
            label: AttackArea::default(),
            sensor_label: Sensor,
        }
    }
}

#[derive(Bundle, Clone)]
pub struct HitAreaBundle {
    collider: Collider,
    collision_group: CollisionGroups,
    label: HitArea,
    sensor_label: Sensor,
}
impl Default for HitAreaBundle {
    fn default() -> Self {
        Self {
            collider: Collider::ball(1.0),
            collision_group: CollisionGroups::new(NamedCollisionGroup::PURE_SENSOR, NamedCollisionGroup::ATTACK),
            label: HitArea::default(),
            sensor_label: Sensor,
        }
    }
}


fn hit (
    mut hit_areas: Query<(Entity, &mut HitArea, Option<&Sensor>), With<Collider>>,
    mut attack_areas: Query<(Entity, &mut AttackArea), With<Collider>>,
    rapier_context: Res<RapierContext>,
) {
    for (hit_entity, mut hit_area, sensor) in hit_areas.iter_mut() {
        let collided_entities: Vec<Entity> = if sensor.is_some() {
            rapier_context.intersections_with(hit_entity).filter_map(
                |(e1, e2, intersect)| if intersect {
                    Some(if hit_entity == e1 {e2} else {e1})
                } else {
                    None
                }
            ).collect()
        } else {
            rapier_context.contacts_with(hit_entity).filter_map(
                |contact| if contact.has_any_active_contacts() {
                    let e1 = contact.collider1();
                    let e2 = contact.collider2();
                    Some(if hit_entity == e1 {e2} else {e1})
                } else {
                    None
                }
            ).collect()
        };
        for collided_entity in collided_entities {
            let Ok((attack_entity, mut attack_area)) = attack_areas.get_mut(collided_entity) else {
                // not Attack
                continue;
            };
            let event = HitEvent {hit_on: hit_entity, attack: attack_entity};
            hit_area.events.push(event);
            attack_area.events.push(event);
        }
    }
}

fn clear_hit (
    mut hit_areas: Query<&mut HitArea>,
    mut attack_areas: Query<&mut AttackArea>,
) {
    for mut hit_area in hit_areas.iter_mut() {
        hit_area.events.clear();
    }
    for mut attack_area in attack_areas.iter_mut() {
        attack_area.events.clear();
    }
}

fn trace_hit (
    hit_areas: Query<&HitArea>,
) {
    for hit_area in hit_areas.iter() {
        if 0 < hit_area.events.len() {
            info!("{:?}", hit_area.events);
        }
    }
}
