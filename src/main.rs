
use bevy::{
    pbr::{
        ScreenSpaceAmbientOcclusionBundle, ScreenSpaceAmbientOcclusionQualityLevel,
        ScreenSpaceAmbientOcclusionSettings,
    },
    core_pipeline::experimental::taa::{TemporalAntiAliasBundle, TemporalAntiAliasPlugin},
    prelude::*,
};
use bevy_rapier3d::prelude::*;
use projectile_spawner::{ProjectileSpawnerPlugin, SimpleBallProjectileSpawner};
use seldom_state::prelude::*;

use cascade_input::CascadeInputPlugin;
use character_control::{
    grounded_states::GroundedStateMachineBundle,
    CharacterControlPlugin, AttachedInput, Locomotion, HeadAttitude, Jump, Rotation, HeadBundle,
};
use player_input::{PlayerInputPlugin, create_player_inputs};

mod util;
mod cascade_input;
mod character_control;
mod player_input;
mod projectile_spawner;

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, TemporalAntiAliasPlugin,))
        .add_plugins((RapierPhysicsPlugin::<NoUserData>::default(), StateMachinePlugin,))
        .add_plugins((CascadeInputPlugin, CharacterControlPlugin, PlayerInputPlugin, ProjectileSpawnerPlugin, ))
        .insert_resource(Msaa::Off)
        .add_systems(Startup, setup)
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
    ;
    //controller
    let controller = create_player_inputs(&mut player_builder);
    player_builder.insert((
        AttachedInput::<Locomotion>::new(controller.locomotion),
        AttachedInput::<Rotation>::new(controller.rotation),
        AttachedInput::<HeadAttitude>::new(controller.head_attitude),
        AttachedInput::<Jump>::new(controller.jump),
    ));
    player_builder.with_children(|player| {
        let mut head = player.spawn(HeadBundle::default());
        head.add_child(camera);
        head.with_children(|head| {
            head.spawn((
                SimpleBallProjectileSpawner {
                    trigger: controller.fire,
                    muzzle_speed: 10.0,
                },
                Velocity::default(),
                TransformBundle {
                    local: Transform::from_xyz(0.0, 2.5, -1.0),
                    ..default()
                }
            ));
        });
        player.spawn(GroundedStateMachineBundle {
            state_machine: GroundedStateMachineBundle::set_default_transitions(StateMachine::default(), controller.jump),
            sensor: Collider::ball(0.2),
            transform: TransformBundle { local: Transform::from_xyz(0.0, -1.7, 0.0), ..default() },
            ..default()
        });
    });
}

