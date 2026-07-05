use super::bullet::{Bullet, Damage};
use super::shared::Hp;
use bevy::{color::palettes::tailwind, prelude::*};
use bevy_rapier2d::prelude::*;

#[derive(Debug, Component, Default)]
pub struct PhysObject;

/// Sets up all physics objects (Dynamic RigidBodies).
/// For now, this is just a few circles.
pub fn setup_phys_objects(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let ball_material = materials.add(Color::from(tailwind::AMBER_500));
    let ball_mesh = meshes.add(Circle::new(50.0));
    for _ in 0..2 {
        commands
            .spawn(PhysObject)
            .insert(Name::new("Ball"))
            .insert(Hp::new(f32::INFINITY))
            .insert(Mesh2d(ball_mesh.clone()))
            .insert(MeshMaterial2d(ball_material.clone()))
            .insert(RigidBody::Dynamic)
            .insert(Collider::ball(50.))
            .insert(Restitution::coefficient(0.7))
            .insert(Transform::from_xyz(0., 400., 0.));
    }
    let breakable_ball_material = materials.add(Color::from(tailwind::AMBER_800));
    commands
        .spawn(PhysObject)
        .insert(Name::new("Ball"))
        .insert(Hp::new(55.))
        .insert(Mesh2d(ball_mesh.clone()))
        .insert(MeshMaterial2d(breakable_ball_material.clone()))
        .insert(RigidBody::Dynamic)
        .insert(Collider::ball(50.))
        .insert(Restitution::coefficient(0.7))
        .insert(Transform::from_xyz(0., 400., 0.));
}

/// Updates physics objects based on bullet collisions.
/// For now, just updates Hp based on bullet damage.
pub fn handle_phys_object_hit(
    mut collision_events: MessageReader<CollisionEvent>,
    mut phys_object_query: Query<&mut Hp, With<PhysObject>>,
    bullet_query: Query<&Damage, With<Bullet>>,
) {
    for event in collision_events.read() {
        let (&left, &right, _flags) = match event {
            CollisionEvent::Started(left, right, flags) => (left, right, flags),
            CollisionEvent::Stopped(_, _, _) => break,
        };

        let mut handle_collision = |phys_object, bullet| {
            if let Ok(mut hp) = phys_object_query.get_mut(phys_object)
                && let Ok(damage) = bullet_query.get(bullet)
            {
                hp.decrement(damage.0);
            }
        };

        handle_collision(left, right);
        handle_collision(right, left);
    }
}

/// Kills all physics objects with 0 health.
pub fn kill_phys_objects(
    mut commands: Commands,
    phys_objects: Query<(Entity, &Hp), With<PhysObject>>,
) {
    for (entity, hp) in phys_objects {
        if hp.hp <= 0. {
            commands.entity(entity).despawn();
        }
    }
}
