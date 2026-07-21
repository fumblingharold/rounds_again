use super::{Bullet, PIXELS_PER_METER, setup_bullet};
use crate::AppState;
use crate::player::{
    Abilities, AccumulatedInput, BulletSpeed, Counter, DamageTakenThisTick, HpBarGreen, Input,
    LastHit, Player, Radius, SPEED, Velocity2,
};
use crate::shared::{Bounces, Damage, Hp, Source};
use bevy::prelude::*;
use bevy_rapier2d::prelude::*;
use parry2d::shape::Cuboid;

/// Handle keyboard input and accumulate it in the `AccumulatedInput` component.
///
/// There are many strategies for how to handle all the input that happened since the last fixed timestep.
/// This is a very simple one: we just use the last available input.
/// That strategy works fine for us since the user continuously presses the input keys in this example.
/// If we had some kind of instantaneous action like activating a boost ability, we would need to remember that that input
/// was pressed at some point since the last fixed timestep.
pub fn update_input(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    controllers: Query<&Gamepad>,
    mouse_input: Res<ButtonInput<MouseButton>>,
    window: Single<&Window, With<bevy::window::PrimaryWindow>>,
    camera: Single<(&Camera, &GlobalTransform)>,
    players: Query<(&mut AccumulatedInput, &Input, &Transform), With<Player>>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    let (camera, camera_transform) = camera.into_inner();
    for (mut accumulated_input, input, transform) in players {
        // Reset the input to zero before reading the new input. As mentioned above, we can only do this
        // because this is continuously pressed by the user. Do not reset e.g. whether the user wants to boost.
        accumulated_input.movement = 0.;
        match input {
            Input::Keyboard => {
                if keyboard_input.pressed(KeyCode::KeyA) {
                    accumulated_input.movement -= 1.0;
                }
                if keyboard_input.pressed(KeyCode::KeyD) {
                    accumulated_input.movement += 1.0;
                }
                if keyboard_input.pressed(KeyCode::Space) {
                    accumulated_input.jump = true;
                }
                if keyboard_input.pressed(KeyCode::Escape) {
                    next_state.set(AppState::Pause)
                }
                if let Some(ray) = window
                    .cursor_position()
                    .and_then(|cursor| camera.viewport_to_world(camera_transform, cursor).ok())
                    && mouse_input.pressed(MouseButton::Left)
                {
                    let pos = ray.origin.truncate();
                    let direction = (pos - transform.translation.truncate()).normalize_or_zero();
                    accumulated_input.shoot = Some(direction);
                }
                if mouse_input.pressed(MouseButton::Right) {
                    accumulated_input.block = true;
                }
            }
            Input::Controller(controller_entity) => {
                if let Ok(controller) = controllers.get(*controller_entity) {
                    let movement = controller.left_stick().x;
                    // Apply movement if outside deadzone
                    // Deadzone is huge since my controller is garbage
                    // TODO also allow dpad movement
                    if movement.abs() > 0.15 {
                        accumulated_input.movement = movement;
                    }
                    if controller.pressed(GamepadButton::South) {
                        accumulated_input.jump = true;
                    }
                    if controller.pressed(GamepadButton::Start) {
                        next_state.set(AppState::Pause)
                    }

                    // Need to detect digital and analog triggers
                    if controller.pressed(GamepadButton::RightTrigger2)
                        || controller
                            .get(GamepadAxis::RightZ)
                            .map(|val| val > 0.)
                            .unwrap_or(false)
                    {
                        accumulated_input.shoot =
                            Some(controller.right_stick().normalize_or_zero());
                    }
                    if controller.pressed(GamepadButton::LeftTrigger2)
                        || controller
                            .get(GamepadAxis::LeftZ)
                            .map(|val| val > 0.)
                            .unwrap_or(false)
                    {
                        accumulated_input.block = true;
                    }
                }
            }
        }
    }
}

/// A simple resource that tells us whether the fixed timestep ran this frame.
#[derive(Resource, Debug, Deref, DerefMut, Default)]
pub struct DidFixedTimestepRunThisFrame(bool);

/// Reset the flag at the start of every frame.
pub fn clear_fixed_timestep_flag(
    mut did_fixed_timestep_run_this_frame: ResMut<DidFixedTimestepRunThisFrame>,
) {
    did_fixed_timestep_run_this_frame.0 = false;
}

/// Set the flag during each fixed timestep.
pub fn set_fixed_time_step_flag(
    mut did_fixed_timestep_run_this_frame: ResMut<DidFixedTimestepRunThisFrame>,
) {
    did_fixed_timestep_run_this_frame.0 = true;
}

pub fn did_fixed_timestep_run_this_frame(
    did_fixed_timestep_run_this_frame: Res<DidFixedTimestepRunThisFrame>,
) -> bool {
    did_fixed_timestep_run_this_frame.0
}

// Clear the input after it was processed in the fixed timestep.
pub fn clear_input(mut inputs: Query<&mut AccumulatedInput>) {
    for mut input in inputs.iter_mut() {
        *input = AccumulatedInput::default();
    }
}

/// Prepare all players for the physics update.
pub fn prepare_players(
    mut commands: Commands,
    materials: ResMut<Assets<ColorMaterial>>,
    meshes: ResMut<Assets<Mesh>>,
    fixed_time: Res<Time<Fixed>>,
    mut players: Query<
        (
            &mut KinematicCharacterController,
            &AccumulatedInput,
            &Damage,
            &Bounces,
            &BulletSpeed,
            &mut Velocity2,
            &mut Abilities,
            &Transform,
            &Radius,
        ),
        With<Player>,
    >,
) {
    let materials = materials.into_inner();
    let meshes = meshes.into_inner();
    for (
        mut controller,
        input,
        damage,
        bounces,
        bullet_speed,
        mut velocity,
        mut abilities,
        position,
        radius,
    ) in players.iter_mut()
    {
        if input.movement.abs() == 0.0 || input.movement.signum() != velocity.0.x.signum() {
            velocity.0.x *= 0.8;
        }
        if input.movement.abs() != 0.0 {
            velocity.0.x += input.movement * SPEED;
            velocity.0.x = velocity.0.x.clamp(-500., 500.);
        }
        let (jump, shoot) = abilities.tick(input.jump, input.shoot.is_some());
        if jump {
            velocity.0.y = 500.;
        }
        if shoot {
            setup_bullet(
                &mut commands,
                position.translation,
                radius.0,
                *bounces,
                *damage,
                input.shoot.unwrap() * bullet_speed.0,
                materials,
                meshes,
            );
        }

        controller.translation = Some(velocity.0 * fixed_time.delta_secs());
    }
}

/// Update the player after the physics update.
pub fn update_players(
    fixed_time: Res<Time<Fixed>>,
    rapier_context: ReadRapierContext,
    mut players: Query<
        (
            Entity,
            &KinematicCharacterControllerOutput,
            &Radius,
            &mut Counter,
            &mut Velocity2,
            &mut Abilities,
            &Transform,
        ),
        With<Player>,
    >,
) {
    let rapier_context = rapier_context.single().unwrap();
    for (entity, controller_output, radius, mut counter, mut velocity, mut abilities, position) in
        players.iter_mut()
    {
        let cast_distance = radius.0;
        let cast_half_width = radius.0 * f32::sqrt(2.) / 2.;
        let sitting_distance = radius.0 * 4. / 5.;
        counter.0 += 1;
        velocity.0.y *= 0.995;
        velocity.0 = controller_output.effective_translation / fixed_time.delta_secs();

        let hit = rapier_context.cast_shape(
            position.translation.truncate(),
            0.,
            Vect::new(0., -cast_distance),
            &Cuboid::new([cast_half_width, cast_half_width].into()),
            ShapeCastOptions::with_max_time_of_impact(1.),
            QueryFilter {
                groups: None,
                exclude_collider: Some(entity),
                ..default()
            },
        );

        const LEG_STRENGTH: f32 = 5.;

        if let Some((_entity, shape_cast)) = hit
            && velocity.0.y < 20.
        {
            abilities.refill_jump();
            velocity.0.y *= 0.8;

            let distance = shape_cast.time_of_impact * cast_distance;
            let difference = sitting_distance - distance;
            velocity.0.y += difference
                * LEG_STRENGTH
                * if f32::is_sign_negative(difference) {
                    1.
                } else {
                    2.
                };
        } else {
            velocity.0.y -= 9.8 * PIXELS_PER_METER * fixed_time.delta_secs();
        }
    }
}

/// Updates players based on bullet collisions.
/// Adds damage to the player in [`DamageTakenThisTick`].
pub fn handle_player_hit(
    mut collision_events: MessageReader<CollisionEvent>,
    mut player_query: Query<&mut DamageTakenThisTick, With<Player>>,
    bullet_query: Query<(&Damage, &Source), With<Bullet>>,
) {
    for event in collision_events.read() {
        let (&left, &right, _flags) = match event {
            CollisionEvent::Started(left, right, flags) => (left, right, flags),
            CollisionEvent::Stopped(_, _, _) => break,
        };

        let mut handle_collision = |player, bullet| {
            if let Ok(mut damage_taken_this_tick) = player_query.get_mut(player)
                && let Ok((damage, source)) = bullet_query.get(bullet)
            {
                // TODO apply damage where it should actually be applied
                damage_taken_this_tick.0[0] += damage.0;
            }
        };

        handle_collision(left, right);
        handle_collision(right, left);
    }
}

/// Applies accumulated damage from the tick.
///
/// Also updates the health bar.
pub fn handle_player_damage(
    mut player_query: Query<
        (
            Entity,
            &mut DamageTakenThisTick,
            &mut Hp,
            &mut LastHit,
            &Children,
            &Radius,
        ),
        With<Player>,
    >,
    mut hp_bar_query: Query<&mut Transform, With<HpBarGreen>>,
) {
    // TODO need to remove damage from this player from max calculation
    for (entity, mut damage_this_tick, mut hp, mut last_hit, children, radius) in
        player_query.iter_mut()
    {
        let damage_from_player0 = damage_this_tick.0[0];
        hp.damage(damage_from_player0);
        let (max_idx, max) = damage_this_tick.0.iter_mut().enumerate().skip(1).fold(
            (0, damage_from_player0),
            |(max_idx, max), (val_idx, val)| {
                let saved_val = *val;
                hp.damage(saved_val);
                if saved_val > max {
                    (val_idx, saved_val)
                } else {
                    (max_idx, max)
                }
            },
        );
        damage_this_tick.0.fill(0.);
        // Update last_hit if a player got a hit
        if max > 0. {
            last_hit.0 = max_idx as u8;
        }

        for entity in children.iter() {
            if let Ok(mut transform) = hp_bar_query.get_mut(entity) {
                let scale = hp.hp / hp.max_hp;
                transform.scale.x = scale;
                transform.translation.x = (1. - scale) * radius.0 * -crate::player::HP_BAR_SCALE.x;
            }
        }
    }
}

/// Kills all players with 0 health.
pub fn kill_players(mut commands: Commands, players: Query<(Entity, &Hp), With<Player>>) {
    for (entity, hp) in players {
        if hp.hp <= 0. {
            // commands.entity(entity).despawn();
        }
    }
}
