use bevy::{color::palettes::tailwind, prelude::*};
use bevy_rapier2d::prelude::*;

#[derive(Debug, Component, Clone, Copy, PartialEq, Hash, Default)]
pub struct Bullet;

#[derive(Debug, Component, Clone, Copy, PartialEq, Hash, Default)]
pub struct Bounces(pub u8);

#[derive(Debug, Component, Clone, Copy, PartialEq, Default)]
pub struct Damage(pub f32);

#[derive(Debug, Component, Clone, Copy, PartialEq, Default)]
pub struct Source(u8);

/// Sets up a new bullet.
pub fn setup_bullet(
    commands: &mut Commands,
    player_position: Vec3,
    radius: f32,
    direction: Vec2,
    materials: &mut Assets<ColorMaterial>,
    meshes: &mut Assets<Mesh>,
) {
    let position = player_position + direction.extend(0.0) * (radius + 15.);
    let velocity = direction * 800.0;
    let body_material = materials.add(Color::from(tailwind::PINK_100));
    let body_mesh = meshes.add(Circle::new(5.));

    commands
        .spawn(Bullet)
        .insert(Bounces(5))
        .insert(Damage(25.))
        // TODO make this actually reflect the player shooting
        .insert(Source(0))
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
        .insert(ActiveEvents::COLLISION_EVENTS);
}

/// A Message indicating that a Bullet should be killed.
#[derive(Message)]
pub struct BulletKillMessage(pub Entity);

/// Updates all bullets based on their collisions.
/// Decrements bounces on hit and adds a BulletKillMessage when bounces run out.
pub fn handle_bullet_hit(
    mut collision_events: MessageReader<CollisionEvent>,
    mut kill_bullet_events: MessageWriter<BulletKillMessage>,
    mut query: Query<&mut Bounces, With<Bullet>>,
) {
    for event in collision_events.read() {
        let (&left, &right, _flags) = match event {
            CollisionEvent::Started(left, right, flags) => (left, right, flags),
            CollisionEvent::Stopped(_, _, _) => break,
        };

        let mut handle_collision = |entity| {
            if let Ok(mut bounces) = query.get_mut(entity) {
                if bounces.0 > 0 {
                    bounces.0 -= 1
                } else {
                    kill_bullet_events.write(BulletKillMessage(entity));
                }
            }
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
