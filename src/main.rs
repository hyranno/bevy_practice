use bevy::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, setup)
        .add_systems(Update, player_move)
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
    commands.spawn(PbrBundle {
        mesh: meshes.add(shape::Plane::from_size(5.0).into()),
        material: materials.add(Color::rgb(0.3, 0.5, 0.3).into()),
        ..default()
    });
    // cube
    commands.spawn(PbrBundle {
        mesh: meshes.add(Mesh::from(shape::Cube { size: 1.0 })),
        material: materials.add(Color::rgb(0.8, 0.7, 0.6).into()),
        transform: Transform::from_xyz(0.0, 0.5, 0.0),
        ..default()
    });
    // light
    commands.spawn(PointLightBundle {
        point_light: PointLight {
            intensity: 1500.0,
            shadows_enabled: true,
            ..default()
        },
        transform: Transform::from_xyz(4.0, 8.0, 4.0),
        ..default()
    });
    // player
    commands.spawn(
        Player
    ).insert(
        TransformBundle {
            local: Transform::from_xyz(-2.0, 0.0, 5.0),
            ..default()
        }
    ).with_children(|parent| {
        parent.spawn(Camera3dBundle {
            transform: Transform::from_xyz(0.0, 2.5, 0.0).looking_at(Vec3::new(2.0, 0.0, -5.0), Vec3::Y),
            ..default()
        });
    });
}

fn player_move(
    mut query: Query<&mut Transform, With<Player>>,
    keyboard_input: Res<Input<KeyCode>>,
) {
    for mut transform in query.iter_mut() {
        let z = transform.local_z();
        let x = transform.local_x();
        if keyboard_input.pressed(KeyCode::W) {
            transform.translation -= z * 0.1;
        }
        if keyboard_input.pressed(KeyCode::S) {
            transform.translation += z * 0.1;
        }
        if keyboard_input.pressed(KeyCode::A) {
            transform.translation -= x * 0.1;
        }
        if keyboard_input.pressed(KeyCode::D) {
            transform.translation += x * 0.1;
        }
    }
}