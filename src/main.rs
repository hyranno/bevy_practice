use bevy::{
    pbr::{
        ScreenSpaceAmbientOcclusionBundle, ScreenSpaceAmbientOcclusionQualityLevel,
        ScreenSpaceAmbientOcclusionSettings,
    },
    core_pipeline::experimental::taa::{TemporalAntiAliasBundle, TemporalAntiAliasPlugin},
    prelude::*,
};
use bevy_rapier3d::prelude::*;
use cascade_input::{
    CascadeInputPlugin, CascadeInputSet,
    axis::{PositionalInput, RotationalInput}, button_like::ButtonTrigger,
};
use player_input::{PlayerInput, PlayerInputPlugin};
use seldom_state::{trigger::{Trigger, BoolTrigger, DoneTrigger, Done}, prelude::StateMachine, StateMachinePlugin};

mod util;
mod cascade_input;
mod player_input;

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, TemporalAntiAliasPlugin,))
        .add_plugins((RapierPhysicsPlugin::<NoUserData>::default(), StateMachinePlugin,))
        .add_plugins((CascadeInputPlugin, PlayerInputPlugin,))
        .insert_resource(Msaa::Off)
        .add_systems(Startup, setup)
        .add_systems(Update, jump_up.after(CascadeInputSet::Flush))
        .add_systems(Update, character_rotation.after(jump_up))
        .add_systems(Update, (grounded_locomotion, airborne_locomotion).after(character_rotation))
        .run();
}

#[derive(Component)]
struct Player;


/// set up a simple 3D scene
fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // plane
    commands
        .spawn(PbrBundle {
            mesh: meshes.add(shape::Plane::from_size(50.0).into()),
            material: materials.add(Color::rgb(0.3, 0.5, 0.3).into()),
            ..default()
        })
        .insert(Collider::cuboid(50.0, 0.001, 50.0));
    // cube
    commands
        .spawn(PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Cube { size: 1.0 })),
            material: materials.add(Color::rgb(0.8, 0.7, 0.6).into()),
            transform: Transform::from_xyz(0.0, 0.5, 0.0),
            ..default()
        })
        .insert(RigidBody::Dynamic)
        .insert(Collider::cuboid(0.5, 0.5, 0.5))
        .insert(Restitution::coefficient(0.1));
    // light
    commands
        .insert_resource(AmbientLight {
            brightness: 0.1,
            ..default()
        });
    commands
        .spawn(PointLightBundle {
            point_light: PointLight {
                intensity: 1500.0,
                shadows_enabled: true,
                ..default()
            },
            transform: Transform::from_xyz(4.0, 8.0, 4.0),
            ..default()
        });
    // camera
    let camera = commands
        .spawn(Camera3dBundle {
            transform: Transform::from_xyz(0.0, 2.5, 0.0),
            ..default()
        })
        .insert(ScreenSpaceAmbientOcclusionBundle {
            settings: ScreenSpaceAmbientOcclusionSettings {
                quality_level: ScreenSpaceAmbientOcclusionQualityLevel::Medium,
            },
            ..default()
        })
        .insert(TemporalAntiAliasBundle::default())
        .id();
    // player
    let mut player_builder = commands.spawn(Player);
    player_builder
        .insert(TransformBundle {
            local: Transform::from_xyz(-2.0, 0.0, 5.0),
            ..default()
        })
        .insert(Velocity::default())
        .insert(RigidBody::Dynamic)
        .insert(LockedAxes::ROTATION_LOCKED)
        .insert(Collider::capsule_y(1.5, 0.3))
        .insert(KinematicCharacterController {..default()})
        .add_child(camera);
    //controller
    let controller = PlayerInput::new_with_inputs(&mut player_builder);
    player_builder.insert(controller);
    player_builder.with_children(|parent| {
        parent.spawn(GroundedStateMachineBundle {
            state_machine: GroundedStateMachineBundle::set_default_transitions(StateMachine::default(), controller.jump),
            sensor: Collider::ball(0.2),
            transform: TransformBundle { local: Transform::from_xyz(0.0, -1.7, 0.0), ..default() },
            ..default()
        });
    });
}

fn jump_up (
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

fn character_rotation(
    mut players: Query<(&mut Transform, &PlayerInput, &Children), With<Player>>,
    mut cameras: Query<(Entity, &mut Transform), (With<Camera3d>, With<Parent>, Without<Player>)>,
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

fn grounded_locomotion (
    mut characters: Query<(&Transform, &mut Velocity, &PlayerInput)>,
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
fn airborne_locomotion (
    mut characters: Query<(&Transform, &mut Velocity, &PlayerInput)>,
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


#[derive(Bundle)]
struct GroundedStateMachineBundle {
    state_machine: StateMachine,
    sensor: Collider,
    transform: TransformBundle,
    label: GroundedStateMachine,
    sensor_label: Sensor,
    initial_state: Grounded,
}
impl Default for GroundedStateMachineBundle {
    fn default() -> Self {
        Self {
            state_machine: StateMachine::default(),
            sensor: Collider::ball(1.0),
            transform: TransformBundle::default(),
            label: GroundedStateMachine,
            sensor_label: Sensor,
            initial_state: Grounded,
        }
    }
}
impl GroundedStateMachineBundle {
    fn set_default_transitions(
        state_machine: StateMachine,
        jump_button: Entity,
    ) -> StateMachine {
        let ground_contact = GroundContact;
        let jump = JumpingUp {max_acceleration: 1.0, ..default()};
        let jump_trigger = ButtonTrigger {button: jump_button};
        state_machine
            .trans::<Grounded>(ground_contact.not(), Airborne)
            .trans::<Airborne>(ground_contact, Grounded)
            .trans::<Grounded>(jump_trigger, jump)
            .trans::<JumpingUp>(jump_trigger.not(), Airborne)
            .trans::<JumpingUp>(DoneTrigger::Success, Airborne)
            .set_trans_logging(true)
    }
}
#[derive(Component)]
struct GroundedStateMachine;
#[derive(Clone, Component, Reflect)]
#[component(storage = "SparseSet")]
struct Grounded;
#[derive(Clone, Component, Reflect)]
#[component(storage = "SparseSet")]
struct Airborne;
#[derive(Clone, Component, Reflect)]
#[component(storage = "SparseSet")]
struct Landing;
#[derive(Clone, Component, Reflect)]
#[component(storage = "SparseSet")]
struct JumpingUp {
    max_acceleration: f32,
    duration: f32,
    target_velocity: Vec3,
    elapsed_time: f32,
}
impl Default for JumpingUp {
    fn default() -> Self {
        Self {
            max_acceleration: 1.0,
            duration: 0.1,
            target_velocity: 30.0 * Vec3::Y,
            elapsed_time: 0.0,
        }
    }
}

#[derive(Copy, Clone)]
struct GroundContact;
impl BoolTrigger for GroundContact {
    type Param<'w, 's> = Res<'w, RapierContext>;
    fn trigger(
        &self,
        entity: Entity,
        rapier_context: Self::Param<'_, '_>,
    ) -> bool {
        let intersections = rapier_context.intersections_with(entity);
        0 < intersections.count()
    }
}
