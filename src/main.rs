use bevy::{
    input::mouse::MouseMotion,
    window::{Window, PrimaryWindow},
    pbr::{
        ScreenSpaceAmbientOcclusionBundle, ScreenSpaceAmbientOcclusionQualityLevel,
        ScreenSpaceAmbientOcclusionSettings,
    },
    core_pipeline::experimental::taa::{TemporalAntiAliasBundle, TemporalAntiAliasPlugin},
    prelude::*,
};
use bevy_rapier3d::prelude::*;
use util::ComponentWrapper;
use cascade_input::{
    button_like::{MappedKey, update_key_mapped_buttons},
    axis::update_four_button_axis,
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
        .add_systems(Update,
            (
                update_key_mapped_buttons::<NegativeX>,
                update_key_mapped_buttons::<PositiveX>,
                update_key_mapped_buttons::<NegativeY>,
                update_key_mapped_buttons::<PositiveY>,
            ).in_set(CascadingInputSet::KeyMappedButtons)
        )
        .add_systems(Update,
            (
                update_four_button_axis::<LocomotionAxis2D, NegativeX, PositiveX, NegativeY, PositiveY>,
            ).in_set(CascadingInputSet::Set)
            .after(CascadingInputSet::KeyMappedButtons)
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
#[derive(Clone, Copy, PartialEq, Eq)]
struct LocomotionButtonNegativeX;
type NegativeX = ComponentWrapper<bool, LocomotionButtonNegativeX>;
#[derive(Clone, Copy, PartialEq, Eq)]
struct LocomotionButtonPositiveX;
type PositiveX = ComponentWrapper<bool, LocomotionButtonPositiveX>;
#[derive(Clone, Copy, PartialEq, Eq)]
struct LocomotionButtonNegativeY;
type NegativeY = ComponentWrapper<bool, LocomotionButtonNegativeY>;
#[derive(Clone, Copy, PartialEq, Eq)]
struct LocomotionButtonPositiveY;
type PositiveY = ComponentWrapper<bool, LocomotionButtonPositiveY>;
#[derive(Clone, Copy, PartialEq, Eq)]
struct LocomotionAxis2DLabel;
type LocomotionAxis2D = ComponentWrapper<Vec2, LocomotionAxis2DLabel>;
#[derive(Bundle)]
struct PlayerInputBundle {
    button_negative_x: NegativeX,
    button_positive_x: PositiveX,
    button_negative_y: NegativeY,
    button_positive_y: PositiveY,
    key_negative_x: MappedKey<NegativeX>,  // MappedKey<TypeOf(button_negative_x)>
    key_positive_x: MappedKey<PositiveX>,
    key_negative_y: MappedKey<NegativeY>,
    key_positive_y: MappedKey<PositiveY>,
    stick_locomotion: LocomotionAxis2D,
}
impl Default for PlayerInputBundle {
    fn default() -> Self {
        Self {
            button_negative_x: ComponentWrapper::new(false),
            button_positive_x: ComponentWrapper::new(false),
            button_negative_y: ComponentWrapper::new(false),
            button_positive_y: ComponentWrapper::new(false),
            key_negative_x: MappedKey::new(KeyCode::A),
            key_positive_x: MappedKey::new(KeyCode::D),
            key_negative_y: MappedKey::new(KeyCode::S),
            key_positive_y: MappedKey::new(KeyCode::W),
            stick_locomotion: ComponentWrapper::new(Vec2::default()),
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
    commands
        .spawn(Player)
        .insert(TransformBundle {
            local: Transform::from_xyz(-2.0, 0.0, 5.0),
            ..default()
        })
        .insert(PlayerInputBundle::default())
        .insert(Velocity::default())
        .insert(RigidBody::Dynamic)
        .insert(LockedAxes::ROTATION_LOCKED)
        .insert(Collider::capsule_y(1.5, 0.3))
        .add_child(camera);
}

fn player_move(
    mut players: Query<(Entity, &mut Transform, &mut Velocity, &Children), (With<Player>, Without<Camera3d>)>,
    mut cameras: Query<(&mut Transform, &mut EulerAttitude), With<Camera3d>>,
    controller: Query<&LocomotionAxis2D>,
    windows: Query<&Window, &PrimaryWindow>,
    mut mouse_motion_events: EventReader<MouseMotion>,
) {
    let window = windows.get_single().unwrap();
    let camera_sensitivity = -Vec2::new(1.0, 1.0);
    for (entity, mut transform, mut velocity, children) in players.iter_mut() {
        // rotation
        for event in mouse_motion_events.iter() {
            transform.rotate_y(camera_sensitivity.x * event.delta.x / window.width());
            for &child in children {
                if let Ok((mut camera_transform, mut camera_attitude)) = cameras.get_mut(child) {
                    camera_attitude.0.x = (
                        camera_attitude.0.x + camera_sensitivity.y * event.delta.y / window.height()
                    ).clamp(-std::f32::consts::FRAC_PI_2, std::f32::consts::FRAC_PI_2);
                    camera_transform.rotation = Quat::from_euler(EulerRot::YXZ, camera_attitude.0.y, camera_attitude.0.x, camera_attitude.0.z);
                }
            }
        }

        if let Ok(locomotion_2d) = controller.get(entity) {
            // translate
            let movement_speed = 2.0;
            let z = transform.local_z();
            let x = transform.local_x();
            let mut linvel = Vec3::ZERO;
            linvel -= z * movement_speed * locomotion_2d.y;
            linvel += x * movement_speed * locomotion_2d.x;
            velocity.linvel = linvel;
        }
    }
}