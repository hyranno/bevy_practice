use bevy::{
    prelude::*, ecs::system::EntityCommands,
};
use crate::cascade_input::{
    CascadeInputSet,
    button_like::{ButtonInput, MappedKey, update_key_mapped_buttons, Toggle, update_toggle_buttons},
    axis::{StickInput, StickButtons, MappedMouse, MaxLength, DeadZone, update_four_button_axis, clamp_stick},
};


pub struct PlayerInputPlugin;
impl Plugin for PlayerInputPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(Update,
                (
                    update_four_button_axis,
                ).in_set(CascadeInputSet::Flush)
                .after(update_key_mapped_buttons)
            )
            .add_systems(Update,
                clamp_stick
                .in_set(CascadeInputSet::Flush)
                .after(update_four_button_axis)
            )
            .add_systems(Update,
                update_toggle_buttons::<WalkToggleLabel>
                .in_set(CascadeInputSet::Flush)
                .after(update_four_button_axis)
            )
            .add_systems(Update,
                update_walking
                .in_set(CascadeInputSet::Flush)
                .after(update_toggle_buttons::<WalkToggleLabel>)
            )
        ;
    }
}


#[derive(Component)]
pub struct PlayerInput {
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
            let walk_key = builder.spawn((
                ButtonInput::new(false),
                MappedKey::new(KeyCode::C),
            )).id();
            let walking = builder.spawn((
                ButtonInput::new(false),
                Toggle::<WalkToggleLabel>::new(walk_key),
            )).id();
            locomotion_stick = Some(builder.spawn((
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
            )).id());

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
        let Ok(walking) = buttons.get(walk_mode.walking) else {continue;};
        if !**walking {continue;};
        let value = **stick * walk_mode.amp;
        // check real change for component change detection
        if **stick != value {
            **stick = value;
        }
    }
}
