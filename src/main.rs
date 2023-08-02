use bevy::{
    pbr::{
        ScreenSpaceAmbientOcclusionBundle, ScreenSpaceAmbientOcclusionQualityLevel,
        ScreenSpaceAmbientOcclusionSettings,
    },
    core_pipeline::experimental::taa::{TemporalAntiAliasBundle, TemporalAntiAliasPlugin},
    prelude::*, ecs::system::EntityCommands,
};
use bevy_rapier3d::prelude::*;
use cascade_input::{
    button_like::{MappedKey, update_key_mapped_buttons, ButtonInput},
    axis::{update_four_button_axis, StickInput, StickButtons, MappedMouse, update_mouse_mapped_sticks, MaxLength, DeadZone, clamp_stick},
};

mod util;
mod cascade_input;

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, TemporalAntiAliasPlugin))
        .add_plugins(RapierPhysicsPlugin::<NoUserData>::default())
        .insert_resource(Msaa::Off)
        .add_systems(Startup, setup)
        .configure_set(Update, CascadingInputSet::KeyMappedButtons.in_set(CascadingInputSet::Set))
        .add_systems(Update,update_key_mapped_buttons.in_set(CascadingInputSet::Set))
        .add_systems(Update,update_mouse_mapped_sticks.in_set(CascadingInputSet::Set))
        .add_systems(Update,
            (
                update_four_button_axis,
            ).in_set(CascadingInputSet::Set)
            .after(update_key_mapped_buttons)
        )
        .add_systems(Update,
            clamp_stick
            .in_set(CascadingInputSet::Set)
            .after(update_four_button_axis)
        )
        .add_systems(Update, player_move.after(CascadingInputSet::Set))
        .run();
}

#[derive(Component)]
struct Player;

#[derive(Component)]
struct EulerAttitude(Vec3);

#[derive(Debug, Hash, PartialEq, Eq, Clone, SystemSet)]
enum CascadingInputSet {
    Set,
    KeyMappedButtons,
}
#[derive(Component)]
struct PlayerInput {
    pub locomotion_stick: Entity,
    pub rotation_stick: Entity,
}
impl PlayerInput {
    pub fn new_with_inputs<'w, 's, 'a, 'b>(commands: &'b mut EntityCommands<'w, 's, 'a>) -> Self {
        let mut locomotion_stick = None;
        let mut rotation_stick = None;
        commands.with_children(|builder| {
            let negative_x = builder.spawn((
                ButtonInput::new(false),
                MappedKey::new(KeyCode::A),
            )).id();
            let positive_x = builder.spawn((
                ButtonInput::new(false),
                MappedKey::new(KeyCode::D),
            )).id();
            let negative_y = builder.spawn((
                ButtonInput::new(false),
                MappedKey::new(KeyCode::S),
            )).id();
            let positive_y = builder.spawn((
                ButtonInput::new(false),
                MappedKey::new(KeyCode::W),
            )).id();
            locomotion_stick = Some(builder.spawn((
                StickInput::new(Vec2::default()),
                StickButtons {
                    negative_x: negative_x,
                    positive_x: positive_x,
                    negative_y: negative_y,
                    positive_y: positive_y,
                }
            ))
            .insert(MaxLength::new(1.0))
            .insert(DeadZone::new(0.0))
            .id());
            rotation_stick = Some(builder.spawn((
                StickInput::new(Vec2::default()),
                MappedMouse {
                    sensitivity: Vec2::new(0.0008, 0.0008),
                }
            )).id());
        });
        Self {
            locomotion_stick: locomotion_stick.unwrap(),
            rotation_stick: rotation_stick.unwrap(),
        }
    }
}




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
        .insert(EulerAttitude {0: Vec3 { x: 0.0, y: 0.0, z: 0.0 }})
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
    mut cameras: Query<(&mut Transform, &mut EulerAttitude), (With<Camera3d>, With<Parent>, Without<Player>)>,
    stick_inputs: Query<&StickInput>,
) {
    for (mut transform, mut velocity, inputs, children) in players.iter_mut() {
        // rotation
        if let Ok(stick) = stick_inputs.get(inputs.rotation_stick) {
            let camera_sensitivity = Vec2::new(1.0, 1.0);
            transform.rotate_y(-camera_sensitivity.x * stick.x);
            for &child in children {
                let Ok((mut camera_transform, mut camera_attitude)) = cameras.get_mut(child) else {continue;};
                camera_attitude.0.x = (
                    camera_attitude.0.x - camera_sensitivity.y * stick.y
                ).clamp(-std::f32::consts::FRAC_PI_2, std::f32::consts::FRAC_PI_2);
                camera_transform.rotation = Quat::from_euler(EulerRot::YXZ, camera_attitude.0.y, camera_attitude.0.x, camera_attitude.0.z);
            }
        }
        // translate
        if let Ok(stick) = stick_inputs.get(inputs.locomotion_stick) {
            let movement_speed = 2.0;
            let z = transform.local_z();
            let x = transform.local_x();
            let mut linvel = Vec3::ZERO;
            linvel -= z * movement_speed * stick.y;
            linvel += x * movement_speed * stick.x;
            velocity.linvel = linvel;
        }

    }
}