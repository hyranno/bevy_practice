use bevy::{
    prelude::*, ecs::system::EntityCommands,
};
use crate::cascade_input::{
    CascadeInputSet,
    button_like::{ButtonInput, MappedKey, Toggle, update_toggle_buttons},
    axis::{StickInput, StickButtons, MappedMouse, MaxLength, DeadZone, update_four_button_axis, clamp_stick, PositionalInput, EulerAngleInput, update_rotation_from_euler, RotationalInput, MappedEulerAngle},
};

#[derive(Clone, Copy, PartialEq, Eq)]
struct DummyLabel;

pub struct PlayerInputPlugin;
impl Plugin for PlayerInputPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(Update,
                (
                    update_four_button_axis,
                ).in_set(CascadeInputSet::Flush)
                .after(CascadeInputSet::DeviceMappedInputs)
            )
            .add_systems(Update,
                clamp_stick
                .in_set(CascadeInputSet::Flush)
                .after(update_four_button_axis)
            )
            .add_systems(Update,
                update_toggle_buttons::<WalkToggleLabel>
                .in_set(CascadeInputSet::Flush)
                .after(CascadeInputSet::DeviceMappedInputs)
            )
            .add_systems(Update,
                update_walking
                .in_set(CascadeInputSet::Flush)
                .after(update_toggle_buttons::<WalkToggleLabel>)
            )
            .add_systems(Update,
                update_locomotion_from_stick
                .in_set(CascadeInputSet::Flush)
                .after(update_walking)
            )
            .add_systems(Update,
                update_rotation_from_stick
                .in_set(CascadeInputSet::Flush)
                .after(CascadeInputSet::DeviceMappedInputs)
            )
            .add_systems(Update,
                update_rotation_from_euler::<DummyLabel>
                .in_set(CascadeInputSet::Flush)
                .after(update_rotation_from_stick)
            )
        ;
    }
}


#[derive(Component)]
pub struct PlayerInput {
    pub locomotion: Entity,
    pub rotation: Entity,
    pub camera_attitude: Entity,
}
impl PlayerInput {
    pub fn new_with_inputs<'w, 's, 'a, 'b>(commands: &'b mut EntityCommands<'w, 's, 'a>) -> Self {
        let mut locomotion = None;
        let mut rotation = None;
        let mut camera_attitude = None;

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
            let walk_key = builder.spawn((
                ButtonInput::new(false),
                MappedKey::new(KeyCode::C),
            )).id();
            let walking = builder.spawn((
                ButtonInput::new(false),
                Toggle::<WalkToggleLabel>::new(walk_key),
            )).id();
            let locomotion_stick = builder.spawn((
                StickInput::new(Vec2::default()),
                StickButtons {
                    negative_x: negative_x,
                    positive_x: positive_x,
                    negative_y: negative_y,
                    positive_y: positive_y,
                },
                MaxLength::new(1.0),
                DeadZone::new(0.0),
                WalkMode {
                    walking: walking,
                    amp: 0.5
                },
            )).id();
            locomotion = Some(builder.spawn((
                PositionalInput::new(Vec3::default()),
                MappedStick {
                    stick: locomotion_stick,
                }
            )).id());

            let rotation_euler = builder.spawn((
                EulerAngleInput::new(Vec3::ZERO),
            )).id();
            let camera_attitude_euler = builder.spawn((
                EulerAngleInput::new(Vec3::ZERO),
            )).id();
            builder.spawn(( // rotation_stick
                StickInput::new(Vec2::default()),
                MappedMouse {
                    sensitivity: Vec2::new(0.0008, 0.0008),
                },
                TargetRotation {
                    sensitivity: Vec2::ONE,
                    rotation: rotation_euler,
                    camera_attitude: camera_attitude_euler,
                }
            ));
            rotation = Some(builder.spawn((
                RotationalInput::new(Quat::default()),
                MappedEulerAngle::<DummyLabel>::new(rotation_euler),
            )).id());
            camera_attitude = Some(builder.spawn((
                RotationalInput::new(Quat::default()),
                MappedEulerAngle::<DummyLabel>::new(camera_attitude_euler),
            )).id());
        });

        Self {
            locomotion: locomotion.unwrap(),
            rotation: rotation.unwrap(),
            camera_attitude: camera_attitude.unwrap(),
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Default)]
struct WalkToggleLabel;

#[derive(Component)]
struct WalkMode {
    walking: Entity,
    amp: f32,
}
fn update_walking(
    mut sticks: Query<(&mut StickInput, &WalkMode)>,
    buttons: Query<&ButtonInput>,
) {
    for (mut stick, walk_mode) in sticks.iter_mut() {
        let Ok(walking) = buttons.get(walk_mode.walking) else {
            warn!("Entity not found");
            continue;
        };
        if !**walking {continue;};
        let value = **stick * walk_mode.amp;
        // check real change for component change detection
        if **stick != value {
            **stick = value;
        }
    }
}


#[derive(Component)]
struct  MappedStick {
    stick: Entity,
}
fn update_locomotion_from_stick(
    mut locomotions: Query<(&mut PositionalInput, &MappedStick)>,
    sticks: Query<&StickInput>,
) {
    for (mut locomotion, mapped_stick) in locomotions.iter_mut() {
        let Ok(stick) = sticks.get(mapped_stick.stick) else {
            warn!("Entity not found");
            continue;
        };
        let value = Vec3::new(stick.x, 0.0, -stick.y);
        // check real change for component change detection
        if **locomotion != value {
            **locomotion = value;
        }
    }
}

#[derive(Component)]
struct  TargetRotation {    // attach this to stick
    sensitivity: Vec2,
    rotation: Entity,
    camera_attitude: Entity,
}
fn update_rotation_from_stick(
    mut angles: Query<&mut EulerAngleInput>,
    sticks: Query<(&StickInput, &TargetRotation)>,
) {
    for (stick, target) in sticks.iter() {
        let Ok(mut rotation) = angles.get_mut(target.rotation) else {
            warn!("Entity not found");
            continue;
        };
        let rotation_value = Vec3::new(0.0, -target.sensitivity.x * stick.x, 0.0);
        // avoid false change detection
        if **rotation != rotation_value {
            **rotation = rotation_value;
        }
        let Ok(mut camera_attitude) = angles.get_mut(target.camera_attitude) else {
            warn!("Entity not found");
            continue;
        };
        let attitude_x = (camera_attitude.x - target.sensitivity.y * stick.y).clamp(-std::f32::consts::FRAC_PI_2, std::f32::consts::FRAC_PI_2);
        // avoid false change detection
        if camera_attitude.x != attitude_x {
            camera_attitude.x = attitude_x;
        }
    }
}