use super::{PIXELS_PER_METER, setup_bullet, shared::Hp};
use crate::game_state::bullet::{Bullet, Damage};
use bevy::{color::palettes::tailwind, prelude::*};
use bevy_rapier2d::prelude::*;
use parry2d::shape::Cuboid;

/// Speed of the player.
const SPEED: f32 = 10.;

/// Prints the position and velocity of all players.
pub fn print_player_info(player: Query<(&Transform, &Velocity2), With<Player>>) {
    for (transform, velocity) in player.iter() {
        println!(
            "Velocity: {} position: {}",
            velocity.0, transform.translation
        );
    }
}

/// Represents the player's input, accumulated over all frames that ran since the last time the
/// physics simulation was advanced.
/// Directionals are replaced with the most recent input while jump, shoot, and block are ANDed.
#[derive(Debug, Component, Clone, Copy, PartialEq, Default)]
pub struct AccumulatedInput {
    // The player's left-right movement input (AD).
    movement: f32,
    jump: bool,
    shoot: Option<Vec2>,
    block: bool,
}

/// The state of an ability.
#[derive(Debug, Clone, Copy, PartialEq)]
enum State {
    Cooldown(u16),
    Ready,
    InUse(u16),
}

/// Some ability.
#[derive(Debug, Clone, Copy, PartialEq)]
struct Ability {
    state: State,
    stock: u16,
    use_time: u16,
    cooldown_time: u16,
}

impl Ability {
    /// Updates the ability's cooldowns and such. Returns whether to apply the effect of the ability.
    fn tick(&mut self, use_ability: bool) -> bool {
        let (new_state, success) = match self.state {
            State::Ready => {
                if use_ability && self.stock > 0 {
                    self.stock -= 1;
                    if self.use_time > 0 {
                        (State::InUse(self.use_time - 1), true)
                    } else if self.cooldown_time > 0 {
                        (State::Cooldown(self.cooldown_time - 1), true)
                    } else {
                        (State::Ready, true)
                    }
                } else {
                    (State::Ready, false)
                }
            }
            State::InUse(timer) => {
                if timer > 0 && use_ability {
                    (State::InUse(timer - 1), true)
                } else {
                    if self.cooldown_time > 0 {
                        (State::Cooldown(self.cooldown_time - 1), use_ability)
                    } else {
                        (State::Ready, use_ability)
                    }
                }
            }
            State::Cooldown(timer) => (
                if timer > 0 {
                    State::Cooldown(timer - 1)
                } else {
                    State::Ready
                },
                false,
            ),
        };
        self.state = new_state;
        success
    }
}

/// All the abilities of a player.
#[derive(Debug, Component, Clone, Copy, PartialEq)]
pub struct Abilities {
    jump: Ability,
    shoot: Ability,
}

impl Abilities {
    /// Updates the all the abilities' cooldowns and such. Returns whether to apply the effect of
    /// each ability.
    fn tick(&mut self, jump: bool, shoot: bool) -> (bool, bool) {
        (self.jump.tick(jump), self.shoot.tick(shoot))
    }
}

impl Default for Abilities {
    fn default() -> Self {
        Self {
            jump: Ability {
                state: State::Ready,
                stock: 1,
                use_time: 5,
                cooldown_time: 0,
            },
            shoot: Ability {
                state: State::Ready,
                stock: 60,
                use_time: 0,
                cooldown_time: 60,
            },
        }
    }
}

#[derive(Debug, Component, Default)]
pub struct Player;

#[derive(Debug, Component, Default)]
pub struct HpBarGreen;

#[derive(Debug, Component, Default)]
pub struct HpBarRed;

#[derive(Debug, Component, Default)]
pub struct Radius(f32);

/// Custom velocity struct. Need to keep track of player velocity but rapier's velocity doesn't seem
/// to play too nicely?
#[derive(Debug, Component, Default)]
pub struct Velocity2(Vect);

#[derive(Debug, Component, Default)]
pub struct Counter(u64);

const HP_BAR_SCALE: Vec2 = Vec2::new(0.9, 1. / 15.);

/// Sets up a player.
pub fn setup_player(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let player_radius = 25.;
    let body_mesh = meshes.add(Circle::new(player_radius));
    let body_material = materials.add(Color::from(tailwind::PINK_100));
    let controller = KinematicCharacterController {
        //offset: CharacterLength::Relative(0.05),
        //filter_groups: Some(collision_groups),
        // normal_nudge_factor: 0.5,
        ..default()
    };
    let bar_mesh = meshes.add(Rectangle::new(
        player_radius * 2. * HP_BAR_SCALE.x,
        player_radius * 2. * HP_BAR_SCALE.y,
    ));

    commands
        .spawn(Player)
        .insert(Counter(0))
        .insert(Name::new("Player"))
        .insert(Hp::new(55.))
        .insert(Transform::default())
        .insert(AccumulatedInput::default())
        .insert(Velocity2::default())
        .insert(Abilities::default())
        .insert(Radius(player_radius))
        .insert(Collider::ball(25.))
        .insert(Sleeping::disabled())
        .insert(LockedAxes::ROTATION_LOCKED)
        .insert(Visibility::Visible)
        // .insert(CollidingEntities::default())
        // .insert(collision_groups)
        // .insert(friction)
        .insert(controller)
        .insert(KinematicCharacterControllerOutput::default())
        .with_children(|parent| {
            parent.spawn(RigidBody::KinematicPositionBased);
            parent
                .spawn(Mesh2d(body_mesh))
                .insert(MeshMaterial2d(body_material));
            let green_bar_material = materials.add(Color::from(tailwind::GREEN_500));
            parent
                .spawn(HpBarGreen)
                .insert(Transform::from_xyz(0., 30., 0.))
                .insert(Mesh2d(bar_mesh.clone()))
                .insert(MeshMaterial2d(green_bar_material));
            let red_bar_material = materials.add(Color::from(tailwind::RED_500));
            parent
                .spawn(HpBarRed)
                .insert(
                    Transform::from_xyz(player_radius * HP_BAR_SCALE.x, 30., 0.)
                        * Transform::from_scale(Vec3::new(0.0, 1.0, 0.0)),
                )
                .insert(Mesh2d(bar_mesh))
                .insert(MeshMaterial2d(red_bar_material));
        });
}

/// Handle keyboard input and accumulate it in the `AccumulatedInput` component.
///
/// There are many strategies for how to handle all the input that happened since the last fixed timestep.
/// This is a very simple one: we just use the last available input.
/// That strategy works fine for us since the user continuously presses the input keys in this example.
/// If we had some kind of instantaneous action like activating a boost ability, we would need to remember that that input
/// was pressed at some point since the last fixed timestep.
pub fn update_input(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mouse_input: Res<ButtonInput<MouseButton>>,
    window: Single<&Window, With<bevy::window::PrimaryWindow>>,
    camera: Single<(&Camera, &GlobalTransform)>,
    player: Query<&mut AccumulatedInput>,
) {
    let (camera, camera_transform) = camera.into_inner();
    for mut input in player {
        // Reset the input to zero before reading the new input. As mentioned above, we can only do this
        // because this is continuously pressed by the user. Do not reset e.g. whether the user wants to boost.
        input.movement = 0.;
        if keyboard_input.pressed(KeyCode::KeyA) {
            input.movement -= 1.0;
        }
        if keyboard_input.pressed(KeyCode::KeyD) {
            input.movement += 1.0;
        }
        if keyboard_input.pressed(KeyCode::Space) {
            input.jump = true;
        }
        if let Some(ray) = window
            .cursor_position()
            .and_then(|cursor| camera.viewport_to_world(camera_transform, cursor).ok())
            && mouse_input.pressed(MouseButton::Left)
        {
            let pos = ray.origin.truncate();
            input.shoot = Some(pos);
        }
        if mouse_input.pressed(MouseButton::Right) {
            input.block = true;
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
pub fn clear_input(mut input: Single<&mut AccumulatedInput>) {
    **input = AccumulatedInput::default();
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
    for (mut controller, input, mut velocity, mut abilities, position, radius) in players.iter_mut()
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
            let direction =
                (input.shoot.unwrap() - position.translation.truncate()).normalize_or_zero();
            setup_bullet(
                &mut commands,
                position.translation,
                radius.0,
                direction,
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
            abilities.jump.stock = 1;
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
/// For now, just updates Hp based on bullet damage. Also adjusts the HP bar.
pub fn handle_player_hit(
    mut collision_events: MessageReader<CollisionEvent>,
    mut player_query: Query<(&mut Hp, &Children, &Radius), With<Player>>,
    bullet_query: Query<&Damage, With<Bullet>>,
    mut hp_bar_query: Query<&mut Transform, With<HpBarGreen>>,
) {
    for event in collision_events.read() {
        let (&left, &right, _flags) = match event {
            CollisionEvent::Started(left, right, flags) => (left, right, flags),
            CollisionEvent::Stopped(_, _, _) => break,
        };

        let mut handle_collision = |player, bullet| {
            if let Ok((mut hp, children, radius)) = player_query.get_mut(player)
                && let Ok(damage) = bullet_query.get(bullet)
            {
                hp.decrement(damage.0);
                for entity in children.iter() {
                    if let Ok(mut transform) = hp_bar_query.get_mut(entity) {
                        let scale = hp.hp / 100.;
                        transform.scale.x = scale;
                        transform.translation.x = (1. - scale) * radius.0 * -HP_BAR_SCALE.x;
                    }
                }
            }
        };

        handle_collision(left, right);
        handle_collision(right, left);
    }
}

/// Kills all players with 0 health.
pub fn kill_players(mut commands: Commands, players: Query<(Entity, &Hp), With<Player>>) {
    for (entity, hp) in players {
        if hp.hp <= 0. {
            commands.entity(entity).despawn();
        }
    }
}
