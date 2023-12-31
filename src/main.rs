
use ai::AiPlugin;
use attack::{AttackPlugin, HitArea};
use bevior_tree::BehaviorTreePlugin;
use bevy::{prelude::*, input::keyboard::KeyboardInput};
#[cfg(not(target_family="wasm"))]
use bevy::{
    pbr::{
        ScreenSpaceAmbientOcclusionBundle, ScreenSpaceAmbientOcclusionQualityLevel,
        ScreenSpaceAmbientOcclusionSettings,
    },
    core_pipeline::experimental::taa::{TemporalAntiAliasBundle, TemporalAntiAliasPlugin},
};
use bevy_rapier3d::prelude::*;
use global_settings::NamedCollisionGroup;
use projectile_spawner::{
    simple_ball,
    ProjectileSpawnerPlugin,
};
use seldom_state::prelude::*;

use cascade_input::{CascadeInputPlugin, axis::{PositionalInput, RotationalInput}};
use character_control::{
    grounded_states::{GroundedStateMachineBundle, GroundedStateMachine, Grounded},
    CharacterControlPlugin, AttachedInput, Locomotion, HeadAttitude, Jump, Rotation, HeadBundle,
};
use player_input::{PlayerInputPlugin, create_player_inputs};
use util::{state_machine::StateMachineUtilPlugin, ecs::EcsUtilPlugin};
use ui::GameUiPlugin;

mod util;
mod global_settings;
mod cascade_input;
mod ui;
mod character_control;
mod player_input;
mod attack;
mod projectile_spawner;
mod ai;

fn main() {
    let mut app = App::new();
    setup_app(&mut app)
        .add_plugins((
            CascadeInputPlugin, EcsUtilPlugin, StateMachineUtilPlugin,
            CharacterControlPlugin, PlayerInputPlugin, AttackPlugin, ProjectileSpawnerPlugin,
            GameUiPlugin,
            AiPlugin,
        ))
        .insert_resource(Msaa::Off)
        .add_state::<GameStates>()
        .add_systems(Startup, setup)
        .add_systems(Last, pause.run_if(in_state(GameStates::MainGame)))
        .add_systems(Last, unpause.run_if(in_state(GameStates::Pause)))
    ;
    app
        .add_plugins(RapierPhysicsPlugin::<NoUserData>::default())
        .configure_sets(PostUpdate, (
            PhysicsSet::SyncBackend.run_if(in_state(GameStates::MainGame)),
            PhysicsSet::SyncBackendFlush.run_if(in_state(GameStates::MainGame)),
            PhysicsSet::StepSimulation.run_if(in_state(GameStates::MainGame)),
            PhysicsSet::Writeback.run_if(in_state(GameStates::MainGame)),
        ))
    ;
    app
        .add_plugins(StateMachinePlugin)
        .configure_sets(PostUpdate, (
            seldom_state::set::StateSet::Transition.run_if(in_state(GameStates::MainGame)),
            seldom_state::set::StateSet::RemoveDoneMarkers.run_if(in_state(GameStates::MainGame)),
        ))
    ;
    app
        .add_plugins(BehaviorTreePlugin::default())
        .configure_sets(PostUpdate,(
            bevior_tree::BehaviorTreeSystemSet::Update.run_if(in_state(GameStates::MainGame)),
        ))
    ;
    app.run();
}

#[cfg(not(target_family="wasm"))]
fn setup_app(app: &mut App) -> &mut App {
        app.add_plugins((DefaultPlugins, TemporalAntiAliasPlugin))
}

#[cfg(target_family="wasm")]
fn setup_app(app: &mut App) -> &mut App {
        app.add_plugins(DefaultPlugins)
}


#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash, Default, States)]
enum GameStates {
    #[default]
    MainGame,
    Pause,
}

fn pause (
    mut keyboard_input_events: EventReader<KeyboardInput>,
    mut state: ResMut<NextState<GameStates>>
) {
    if keyboard_input_events.iter().any(|event|
        (event.key_code, event.state) == (Some(KeyCode::Escape), bevy::input::ButtonState::Pressed)
    ) {
        state.set(GameStates::Pause);
    }
}
fn unpause (
    mut keyboard_input_events: EventReader<KeyboardInput>,
    mut state: ResMut<NextState<GameStates>>
) {
    if keyboard_input_events.iter().any(|event|
        (event.key_code, event.state) == (Some(KeyCode::Escape), bevy::input::ButtonState::Pressed)
    ) {
        state.set(GameStates::MainGame);
    }
}



#[derive(Component)]
struct Player;


/// set up scene
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
        .insert(Collider::cuboid(50.0, 0.01, 50.0))
        .insert(Friction::coefficient(0.8))
        .insert(CollisionGroups::new(NamedCollisionGroup::TERRAIN, NamedCollisionGroup::ALL))
    ;
    // cube
    let locomotion = commands.spawn(PositionalInput::default()).id();
    let rotation = commands.spawn(RotationalInput::default()).id();
    commands
        .spawn(PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Cube { size: 1.0 })),
            material: materials.add(Color::rgb(0.8, 0.7, 0.6).into()),
            transform: Transform::from_xyz(0.0, 0.5, 0.0),
            ..default()
        })
        .insert(RigidBody::Dynamic)
        .insert(Collider::cuboid(0.5, 0.5, 0.5))
        .insert(CollisionGroups::new(NamedCollisionGroup::OBJECT, NamedCollisionGroup::ALL))
        .insert(Restitution::coefficient(0.1))
        .insert(HitArea::default())
        .insert(Velocity::default())
        .insert(Grounded)
        .insert((AttachedInput::<Locomotion>::new(locomotion), AttachedInput::<Rotation>::new(rotation)))
        .add_child(locomotion)
        .insert(AttachedInput::<Locomotion>::new(locomotion))
        .add_child(rotation)
        .insert(AttachedInput::<Rotation>::new(rotation))
        .with_children(|parent| {
            parent.spawn(ai::behavior::sample_behavior());
        })
    ;
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
    let camera = spawn_camera(&mut commands);
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
        .insert(Friction::coefficient(0.98))
        .insert(CollisionGroups::new(NamedCollisionGroup::CHARACTER, NamedCollisionGroup::ALL))
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
                simple_ball::SpawnerBundle::new(controller.fire, controller.reload),
                Velocity::default(),
                TransformBundle {
                    local: Transform::from_xyz(0.0, 2.5, -1.0),
                    ..default()
                }
            ));
        });
        let grounded_state_machine = GroundedStateMachine::default_machine(controller.jump);
        let grounded_state_machine = GroundedStateMachine::set_state_components_sample(grounded_state_machine);
        player.spawn(GroundedStateMachineBundle {
            state_machine: grounded_state_machine,
            sensor: Collider::ball(0.2),
            transform: TransformBundle { local: Transform::from_xyz(0.0, -1.7, 0.0), ..default() },
            ..default()
        });
    });
}

#[cfg(not(target_family="wasm"))]
fn spawn_camera(commands: &mut Commands) -> Entity {
    commands
        .spawn(
            Camera3dBundle {
                transform: Transform::from_xyz(0.0, 2.5, 0.0),
                ..default()
            }
        )
        .insert(UiCameraConfig {
            show_ui: false,
        })
        .insert(
            ScreenSpaceAmbientOcclusionBundle {
                settings: ScreenSpaceAmbientOcclusionSettings {
                    quality_level: ScreenSpaceAmbientOcclusionQualityLevel::Medium,
                },
                ..default()
            }
        )
        .insert(
            TemporalAntiAliasBundle::default()
        )
        .id()
}

#[cfg(target_family="wasm")]
fn spawn_camera(commands: &mut Commands) -> Entity {
    commands.spawn(
        Camera3dBundle {
            transform: Transform::from_xyz(0.0, 2.5, 0.0),
            ..default()
        },    
    )
    .insert(    UiCameraConfig {
        show_ui: false,
    })
    .id()
}

