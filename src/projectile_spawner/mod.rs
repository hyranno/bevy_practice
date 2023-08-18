use bevy::prelude::*;
use bevy_rapier3d::prelude::*;

use crate::cascade_input::{button_like::{ButtonInput, ButtonLike}, CascadeInputSet};

pub struct ProjectileSpawnerPlugin;
impl Plugin for ProjectileSpawnerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, fire_simple_ball.after(CascadeInputSet::Flush));
    }
}

fn fire_simple_ball(
    mut commands: Commands,
    inputs: Query<&ButtonInput, Changed<ButtonInput>>,
    spawners: Query<&SimpleBallProjectileSpawner>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    for spawner in spawners.iter() {
        let Ok(input) = inputs.get(spawner.trigger) else { continue; };
        if !input.is_pressed() { continue; }
        // TODO set transform, velocity
        // TODO set collider_group, filter
        // TODO use Res<SimpleBallProjectile> for bundle
        let mut projectile_builder = commands.spawn(());
        projectile_builder.insert(PbrBundle {
            mesh: meshes.add(Mesh::try_from(shape::Icosphere { radius: 0.2, ..default() }).unwrap()),
            material: materials.add(Color::rgb(0.2, 0.2, 0.2).into()),
            transform: Transform::from_xyz(0.0, 2.0, 0.0),
            ..default()
        });
        projectile_builder.insert(Velocity::linear(spawner.muzzle_speed * Vec3::Y));
        projectile_builder.insert(Collider::ball(0.1));
        projectile_builder.insert(RigidBody::Dynamic);

    }
}

#[derive(Bundle)]
struct SimpleBallProjectileBundle {
    velocity: Velocity,
    model: PbrBundle,
    collider: Collider,
    rigid_body: RigidBody,
}

#[derive(Component)]
pub struct SimpleBallProjectileSpawner {
    pub trigger: Entity,    // ButtonInput
    pub muzzle_speed: f32,
}
