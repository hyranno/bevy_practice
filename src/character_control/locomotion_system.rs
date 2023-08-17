use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
use seldom_state::prelude::*;

use crate::cascade_input::{
    CascadeInputSet,
    axis::{PositionalInput, RotationalInput}
};
use super::CharacterInput;

use super::grounded_states::*;


pub struct LocomotionSystemPlugin;
impl Plugin for LocomotionSystemPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(Update, jump_up.after(CascadeInputSet::Flush))
            .add_systems(Update, character_rotation.after(jump_up))
            .add_systems(Update, (grounded_locomotion, airborne_locomotion).after(character_rotation))
        ;
    }
}


pub fn jump_up (
    mut commands: Commands,
    mut states: Query<(Entity, &mut JumpingUp, &Parent), With<GroundedStateMachine>>,
    mut velocities: Query<&mut Velocity>,
    time: Res<Time>,
) {
    for (state_machine, mut state, parent) in states.iter_mut() {
        let Ok(mut velocity) = velocities.get_mut(parent.get()) else {
            warn!("Parent does not have velocity!");
            continue;
        };
        let target_direction = state.target_velocity.normalize();
        let speed_diff = state.target_velocity.length() - velocity.linvel.dot(target_direction);
        let linvel = velocity.linvel + speed_diff.clamp(0.0, state.max_acceleration) * target_direction;
        // avoid false change detection
        if velocity.linvel != linvel {
            velocity.linvel = linvel;
        }
        // done after duration
        state.elapsed_time += time.delta_seconds();
        if state.duration < state.elapsed_time {
            commands.entity(state_machine).insert(Done::Success);
        }
    }
}

pub fn character_rotation(
    mut players: Query<(&mut Transform, &CharacterInput, &Children)>,
    mut cameras: Query<(Entity, &mut Transform), (With<Camera3d>, With<Parent>, Without<CharacterInput>)>,
    rotational_inputs: Query<&RotationalInput>,
) {
    for (mut transform, inputs, children) in players.iter_mut() {
        // rotation
        if let Ok(rotation) = rotational_inputs.get(inputs.rotation) {
            // avoid false change detection
            if **rotation != Quat::IDENTITY {
                transform.rotate(**rotation);
            }
        }
        // camera_rotation
        let child_cameras = cameras.iter_mut().filter(|(entity, _)| children.contains(entity));
        for (_, mut camera_transform) in child_cameras {
            let Ok(camera_attitude) = rotational_inputs.get(inputs.camera_attitude) else {continue;};
            // avoid false change detection
            if camera_transform.rotation != **camera_attitude {
                camera_transform.rotation = **camera_attitude;
            }
        }
    }
}

pub fn grounded_locomotion (
    mut characters: Query<(&Transform, &mut Velocity, &CharacterInput)>,
    states: Query<&Parent, Or<(With<Grounded>, With<JumpingUp>)>>,
    positional_inputs: Query<&PositionalInput>,
) {
    for character in states.iter() {
        let Ok((transform, mut velocity, inputs)) = characters.get_mut(character.get()) else {
            warn!("Entity not found!");
            continue;
        };
        let Ok(locomotion) = positional_inputs.get(inputs.locomotion) else {
            warn!("Entity not found!");
            continue;
        };
        // TODO: parameterize
        let movement_speed = 2.0;
        let max_acceleration = 2.0;
        let target_velocity = movement_speed * transform.rotation.inverse().mul_vec3(**locomotion);
        if 0.0 < target_velocity.length() {
            let target_direction = target_velocity.normalize();
            let speed_diff = target_velocity.length() - velocity.linvel.dot(target_direction);
            let linvel = velocity.linvel + speed_diff.clamp(0.0, max_acceleration) * target_direction;
            // avoid false change detection
            if velocity.linvel != linvel {
                velocity.linvel = linvel;
            }
        }
    }
}

pub fn airborne_locomotion (
    mut characters: Query<(&Transform, &mut Velocity, &CharacterInput)>,
    states: Query<&Parent, With<Airborne>>,
    positional_inputs: Query<&PositionalInput>,
) {
    for character in states.iter() {
        let Ok((transform, mut velocity, inputs)) = characters.get_mut(character.get()) else {
            warn!("Entity not found!");
            continue;
        };
        let Ok(locomotion) = positional_inputs.get(inputs.locomotion) else {
            warn!("Entity not found!");
            continue;
        };
        // This formula intentionally enables circle-jump like infinite speed-up
        // TODO: parameterize
        let max_acceleration = 0.02;
        let max_speed = 0.6;
        let locomotion_global = transform.rotation.inverse().mul_vec3(**locomotion);
        let target = max_acceleration * Vec2::new(locomotion_global.x, locomotion_global.z);    // xz() swizzling not found in Bevy
        if 0.0 < target.length() {
            let horizontal_velocity = Vec2::new(velocity.linvel.x, velocity.linvel.z);
            let target_direction = target.normalize();
            let k = (horizontal_velocity.dot(target_direction) / max_speed).clamp(0.0, 1.0);
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
