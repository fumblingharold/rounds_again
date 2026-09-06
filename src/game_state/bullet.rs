use crate::collision_groups;
use crate::shared::{Bounces, Damage, Source};
use bevy::{color::palettes::tailwind, prelude::*};
use bevy_rapier2d::prelude::*;

#[derive(Debug, Component, Clone, Copy, PartialEq, Hash, Default)]
pub struct Bullet;

/// Sets up a new bullet.
#[allow(clippy::too_many_arguments)]
pub fn setup_bullet(
    commands: &mut Commands,
    player_position: Vec3,
    radius: f32,
    bounces: Bounces,
    damage: Damage,
    source: Source,
    velocity: Vec2,
    materials: &mut Assets<ColorMaterial>,
    meshes: &mut Assets<Mesh>,
) {
    let position = player_position + velocity.normalize_or_zero().extend(0.0) * (radius + 15.);
    let body_material = materials.add(Color::from(tailwind::PINK_100));
    let body_mesh = meshes.add(Circle::new(5.));

    commands
        .spawn(Bullet)
        .insert(bounces)
        .insert(damage)
        .insert(source)
        .insert(GravityScale(0.5))
        .insert(Transform::from_translation(position))
        .insert(Mesh2d(body_mesh))
        .insert(MeshMaterial2d(body_material))
        .insert(RigidBody::Dynamic)
        .insert(Velocity::linear(velocity))
        .insert(Collider::ball(5.))
        .insert(LockedAxes::ROTATION_LOCKED)
        .insert(Friction {
            coefficient: 0.0,
            combine_rule: CoefficientCombineRule::Min,
        })
        .insert(Restitution {
            coefficient: 0.9,
            combine_rule: CoefficientCombineRule::Max,
        })
        .insert(CollisionGroups::new(
            collision_groups::BULLETS,
            Group::default(),
        ))
        .insert(SolverGroups::new(Group::empty(), Group::empty()))
        .insert(ActiveEvents::COLLISION_EVENTS);
}

/// A Message indicating that a Bullet should be killed.
#[derive(Message)]
pub struct BulletKillMessage(pub Entity);

/// Updates all bullets based on their collisions and applies impulses to the objects hit.
/// TODO: move the non-bullet part somewhere else
/// Decrements bounces on hit and adds a BulletKillMessage when bounces run out.
pub fn handle_bullet_hit(
    mut collision_events: MessageReader<CollisionEvent>,
    mut kill_bullet_events: MessageWriter<BulletKillMessage>,
    rapier_context: ReadRapierContext,
    mut bullet_query: Query<(&mut Bounces, &mut Velocity), With<Bullet>>,
    damage_query: Query<&Damage, With<Bullet>>,
    mut other_query: Query<&mut ExternalImpulse, Without<Bullet>>,
) {
    let rapier_context = rapier_context.single().unwrap();
    for event in collision_events.read() {
        let (&left, &right, _flags) = match event {
            CollisionEvent::Started(left, right, flags) => (left, right, flags),
            CollisionEvent::Stopped(_, _, _) => break,
        };

        let normal = rapier_context
            .contact_pair(left, right)
            .unwrap()
            .find_deepest_contact()
            .unwrap()
            .0
            .normal();

        let damage = damage_query
            .get(left)
            .or_else(|_| damage_query.get(right))
            .unwrap()
            .0;

        let mut handle_collision = |entity| {
            if let Ok((mut bounces, mut velocity)) = bullet_query.get_mut(entity) {
                if bounces.0 > 0 {
                    bounces.0 -= 1
                } else {
                    kill_bullet_events.write(BulletKillMessage(entity));
                }
                velocity.linear = velocity.linear.reflect(normal);
            } else if let Ok(mut impulse) = other_query.get_mut(entity) {
                // TODO have a reason for this magic value
                impulse.impulse -= normal * damage * 100000.;
            };
        };

        handle_collision(left);
        handle_collision(right);
    }
}

/// Processes all bullet kill events.
pub fn kill_bullets(
    mut commands: Commands,
    mut bullet_kill_events: MessageReader<BulletKillMessage>,
) {
    for BulletKillMessage(bullet) in bullet_kill_events.read() {
        if let Ok(mut entity) = commands.get_entity(*bullet) {
            entity.despawn();
        }
    }
}
