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
    axis::{PositionalInput, RotationalInput},
};
use player_input::{PlayerInput, PlayerInputPlugin};

mod util;
mod cascade_input;
mod player_input;

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, TemporalAntiAliasPlugin, CascadeInputPlugin, PlayerInputPlugin))
        .add_plugins(RapierPhysicsPlugin::<NoUserData>::default())
        .insert_resource(Msaa::Off)
        .add_systems(Startup, setup)
        .add_systems(Update, player_move.after(CascadeInputSet::Flush))
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
        .add_child(camera);
    //controller
    let controller = PlayerInput::new_with_inputs(&mut player_builder);
    player_builder.insert(controller);
}

fn player_move(
    mut players: Query<(&mut Transform, &mut Velocity, &PlayerInput, &Children), With<Player>>,
    mut cameras: Query<&mut Transform, (With<Camera3d>, With<Parent>, Without<Player>)>,
    positional_inputs: Query<&PositionalInput>,
    rotational_inputs: Query<&RotationalInput>,
) {
    for (mut transform, mut velocity, inputs, children) in players.iter_mut() {
        // rotation
        if let Ok(rotation) = rotational_inputs.get(inputs.rotation) {
            transform.rotate(**rotation);
        }
        // camera_rotation
        for &child in children {
            let Ok(mut camera_transform) = cameras.get_mut(child) else {continue;};
            let Ok(camera_attitude) = rotational_inputs.get(inputs.camera_attitude) else {continue;};
            camera_transform.rotation = **camera_attitude;
        }
        // translate
        if let Ok(locomotion) = positional_inputs.get(inputs.locomotion) {
            let movement_speed = 2.0;
            let linvel = movement_speed * transform.rotation.inverse().mul_vec3(**locomotion);
            velocity.linvel = linvel;
        }

    }
}