use super::bullet::Bullet;
use crate::shared::{Damage, Hp};
use bevy::prelude::*;
use bevy_rapier2d::prelude::*;

#[derive(Debug, Component, Default)]
pub struct PhysObject;

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
                hp.damage(damage.0);
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
