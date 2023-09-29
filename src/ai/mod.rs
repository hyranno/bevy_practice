
use bevy::prelude::*;

use crate::{character_control::{AttachedInput, Locomotion, Rotation, grounded_states::Grounded}, cascade_input::{axis::{PositionalInput, RotationalInput}, CascadeInputSet}};

pub mod behavior;


pub struct AiPlugin;
impl Plugin for AiPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(PreUpdate, (update_inputs_from_move_to).before(CascadeInputSet::Begin))
        ;
    }
}


#[derive(Debug, Clone, Copy)]
pub enum AiTarget {
    Entity(Entity),
    Position(Vec3),
}

#[derive(Debug, Component, Clone, Copy)]
pub struct MoveTo {
    pub target: AiTarget,
    pub strafe: bool,
    pub speed_coef: f32,
}

pub fn update_inputs_from_move_to (
    params: Query<(&MoveTo, &Parent)>,
    characters: Query<(&GlobalTransform, &AttachedInput<Locomotion>, Option<&AttachedInput<Rotation>>, Option<&Grounded>), With<Children>>,
    mut locomotion_inputs: Query<&mut PositionalInput>,
    mut rotation_inputs: Query<&mut RotationalInput>,
    transforms: Query<&GlobalTransform>,
) {
    for (param, parent) in params.iter() {
        let Ok((transform, locomotion, rotation, grounded)) = characters.get(parent.get()) else {
            warn!("Invalid parent of MoveTo.");
            continue;
        };
        let target_position = match param.target {
            AiTarget::Position(position) => position,
            AiTarget::Entity(entity) => {
                let Ok(transform) = transforms.get(entity) else {
                    warn!("Targeting entity with no GlobalTransform.");
                    continue;
                };
                transform.translation()
            },
        };  // TODO pathfinding with navmesh
        let Ok(mut locomotion) = locomotion_inputs.get_mut(locomotion.entity) else {
            error!("Locomotion input entity not found.");
            continue;
        };
        let (_, entity_rotation, entity_translation) = transform.to_scale_rotation_translation();
        let mut target_translation = target_position - entity_translation;
        if grounded.is_some() {
            target_translation.y = 0.0;
        }
        if target_translation == Vec3::ZERO {
            continue;
        }
        let target_rotation = Quat::from_rotation_arc(transform.forward(), target_translation.normalize());
        if param.strafe {
            locomotion.value = entity_rotation.inverse().mul_vec3((param.speed_coef * target_translation).clamp_length_max(1.0));
        } else {
            locomotion.value = (param.speed_coef * Vec3::NEG_Z * target_translation.length()).clamp_length_max(1.0);
            let Ok(mut rotation) = rotation_inputs.get_mut(rotation.expect("Input not attached").entity) else {
                error!("Rotation input entity not found");
                continue;
            };
            rotation.value = target_rotation;
        }
    }
}
