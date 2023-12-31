use bevy::{
    prelude::*,
    core_pipeline::clear_color::ClearColorConfig,
    sprite::MaterialMesh2dBundle,
};

use crate::Player;
use crate::global_settings::CameraOrder;
use crate::projectile_spawner::Magazine;


pub struct GameUiPlugin;
impl Plugin for GameUiPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(Startup, spawn_ui)
            .add_systems(PostUpdate, (update_magazine_ui, ))
        ;
    }
}


pub fn spawn_ui (
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    commands.spawn(
        Camera2dBundle {
            camera_2d: Camera2d {
                clear_color: ClearColorConfig::None,
                ..default()
            },
            camera: Camera {
                order: CameraOrder::Ui2d as isize,
                ..default()
            },
            ..default()
        }
    );
    commands.spawn((
        MaterialMesh2dBundle {
            mesh: meshes.add(shape::Circle::new(3.).into()).into(),
            material: materials.add(ColorMaterial::from(Color::WHITE)),
            transform: Transform::from_translation(Vec3::new(0., 0., 0.)),
            ..default()
        },
        Crosshair
    ));
    commands.spawn((
        TextBundle::from_sections([
            TextSection::from_style(TextStyle {
                font_size: 60.0,
                color: Color::WHITE,
                ..default()
            })
            ]).with_style(Style {
                position_type: PositionType::Absolute,
                bottom: Val::Px(15.0),
                right: Val::Px(25.0),
            ..default()
        }),
        UiMagazine
    ));
}

#[derive(Component, Debug, Clone, Copy)]
struct Crosshair;

#[derive(Component, Debug, Clone, Copy)]
struct UiMagazine;
fn update_magazine_ui (
    player_query: Query<Entity, With<Player>>,
    descend_query: Query<&Children>,
    magazines: Query<(Entity, &Magazine)>,
    mut texts: Query<&mut Text, With<UiMagazine>>,
) {
    let Ok(player) = player_query.get_single() else {
        warn!("No player found!");
        return;
    };
    let Some((_, magazine)) = descend_query.iter_descendants(player).find_map(|descendant| magazines.get(descendant).ok()) else {
        // Player has no magazine
        for mut text in texts.iter_mut() {
            text.sections[0].value = format!("");
        }
        return;
    };
    for mut text in texts.iter_mut() {
        text.sections[0].value = format!("{}", magazine.ammo_count);
    }
}
