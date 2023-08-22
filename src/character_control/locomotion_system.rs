use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
use seldom_state::prelude::*;

use crate::cascade_input::{
    CascadeInputSet,
    axis::{PositionalInput, RotationalInput}
};

use super::{Rotation, AttachedInput, HeadAttitude, Locomotion, Head};


pub struct LocomotionSystemPlugin;
impl Plugin for LocomotionSystemPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(Update, timeout)
            .add_systems(Update, (jump_up, character_rotation, head_rotation).after(CascadeInputSet::Flush))
            .add_systems(Update, (basic_locomotion, airborne_locomotion).after(character_rotation))
        ;
    }
}


#[derive(Component, Clone, Copy)]
pub struct Timeout {
    pub duration: f32,
    pub elapsed_time: f32,
}
pub fn timeout (
    mut commands: Commands,
    mut params: Query<(Entity, &mut Timeout)>,
    time: Res<Time>,
) {
    let delta = time.delta_seconds();
    for (state_machine, mut param) in params.iter_mut() {
        param.elapsed_time += delta;
        if param.duration < param.elapsed_time {
            commands.entity(state_machine).insert(Done::Success);
        }
    }
}

#[derive(Component, Clone, Copy)]
pub struct JumpUp {
    pub target_velocity: Vec3,
    pub max_acceleration: f32,
}
pub fn jump_up (
    mut params: Query<(&mut JumpUp, &Parent)>,
    mut velocities: Query<&mut Velocity>,
) {
    for (param, parent) in params.iter_mut() {
        let Ok(mut velocity) = velocities.get_mut(parent.get()) else {
            warn!("Parent does not have velocity!");
            continue;
        };
        let target_direction = param.target_velocity.normalize();
        let speed_diff = param.target_velocity.length() - velocity.linvel.dot(target_direction);
        let linvel = velocity.linvel + speed_diff.clamp(0.0, param.max_acceleration) * target_direction;
        // avoid false change detection
        if velocity.linvel != linvel {
            velocity.linvel = linvel;
        }
    }
}

#[derive(Component, Clone, Copy)]
pub struct CharacterRotation;
pub fn character_rotation(
    mut characters: Query<(&mut Transform, &AttachedInput<Rotation>)>,
    params: Query<(&CharacterRotation, &Parent)>,
    rotational_inputs: Query<&RotationalInput>,
) {
    for (_param, parent) in params.iter() {
        let Ok((mut transform, input)) = characters.get_mut(parent.get()) else {
            warn!("Entity not found!");
            continue;
        };
        // rotation
        if let Ok(rotation) = rotational_inputs.get(**input) {
            // avoid false change detection
            if **rotation != Quat::IDENTITY {
                transform.rotate(**rotation);
            }
        }
    }
}

#[derive(Component, Clone, Copy)]
pub struct HeadRotation;
pub fn head_rotation (
    characters: Query<&AttachedInput<HeadAttitude>, With<Children>>,
    mut heads: Query<(&mut Transform, &Parent), With<Head>>,
    params: Query<(&HeadRotation, &Parent)>,
    rotational_inputs: Query<&RotationalInput>,
) {
    for (_param, parent) in params.iter() {
        let Some((mut transform, parent)) = heads.iter_mut().filter(|(_, character)| parent.get() == character.get()).next() else {
            warn!("Head not found!");
            continue;
        };
        let Ok(inputs) = characters.get(parent.get()) else {
            warn!("Entity not found!");
            continue;
        };
        let Ok(head_attitude) = rotational_inputs.get(**inputs) else {
            warn!("Entity not found!");
            continue;
        };
        // avoid false change detection
        if transform.rotation != **head_attitude {
            transform.rotation = **head_attitude;
        }
    }
}

#[derive(Component, Clone, Copy)]
pub struct BasicLocomotion {
    pub speed: f32,
    pub max_acceleration: f32,
}
pub fn basic_locomotion (
    mut characters: Query<(&GlobalTransform, &mut Velocity, &AttachedInput<Locomotion>)>,
    params: Query<(&BasicLocomotion, &Parent)>,
    positional_inputs: Query<&PositionalInput>,
) {
    for (param, parent) in params.iter() {
        let Ok((transform, mut velocity, input)) = characters.get_mut(parent.get()) else {
            warn!("Entity not found!");
            continue;
        };
        let Ok(locomotion) = positional_inputs.get(**input) else {
            warn!("Entity not found!");
            continue;
        };
        let (_scale, rotation, _translation) = transform.to_scale_rotation_translation();
        let target_velocity = param.speed * rotation.mul_vec3(**locomotion);
        if 0.0 < target_velocity.length() {
            let target_direction = target_velocity.normalize();
            let speed_diff = target_velocity.length() - velocity.linvel.dot(target_direction);
            let linvel = velocity.linvel + speed_diff.clamp(0.0, param.max_acceleration) * target_direction;
            // avoid false change detection
            if velocity.linvel != linvel {
                velocity.linvel = linvel;
            }
        }
    }
}

#[derive(Component, Clone, Copy)]
pub struct AirborneLocomotion {
    pub speed: f32,
    pub max_acceleration: f32,
}
pub fn airborne_locomotion (
    mut characters: Query<(&GlobalTransform, &mut Velocity, &AttachedInput<Locomotion>)>,
    params: Query<(&AirborneLocomotion, &Parent)>,
    positional_inputs: Query<&PositionalInput>,
) {
    for (param, parent) in params.iter() {
        let Ok((transform, mut velocity, input)) = characters.get_mut(parent.get()) else {
            warn!("Entity not found!");
            continue;
        };
        let Ok(locomotion) = positional_inputs.get(**input) else {
            warn!("Entity not found!");
            continue;
        };
        // This formula intentionally enables circle-jump like infinite speed-up
        let (_scale, rotation, _translation) = transform.to_scale_rotation_translation();
        let locomotion_global = rotation.mul_vec3(**locomotion);
        let target = param.max_acceleration * Vec2::new(locomotion_global.x, locomotion_global.z);    // xz() swizzling not found in Bevy
        if 0.0 < target.length() {
            let horizontal_velocity = Vec2::new(velocity.linvel.x, velocity.linvel.z);
            let target_direction = target.normalize();
            let k = (horizontal_velocity.dot(target_direction) / param.speed).clamp(0.0, 1.0);
            let acceleration = target * if 0.0 < k {
                (1.0 - k) + k*horizontal_velocity.normalize().dot(target_direction)/2.0
            } else {
                1.0
            };
            // avoid false change detection
            if 0.0 < acceleration.length() {
                velocity.linvel += Vec3::new(acceleration.x, 0.0, acceleration.y);
            }
        }
    }
}
