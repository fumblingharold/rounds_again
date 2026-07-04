//! The solution is using a fixed timestep. This means that we advance the physics simulation by a fixed amount
//! at a time. If the real time that passed between two frames is less than the fixed timestep, we simply
//! don't advance the physics simulation at all.
//! If it is more, we advance the physics simulation multiple times until we catch up.
//! You can read more about how Bevy implements this in the documentation for
//! [`bevy::time::Fixed`](https://docs.rs/bevy/latest/bevy/time/struct.Fixed.html).
//!
//! This leaves us with a last problem, however. If our physics simulation may advance zero or multiple times
//! per frame, there may be frames in which the player's position did not need to be updated at all,
//! and some where it is updated by a large amount that resulted from running the physics simulation multiple times.
//! This is physically correct, but visually jarring. Imagine a player moving in a straight line, but depending on the frame rate,
//! they may sometimes advance by a large amount and sometimes not at all. Visually, we want the player to move smoothly.
//! This is why we need to separate the player's position in the physics simulation from the player's position in the visual representation.
//! The visual representation can then be interpolated smoothly based on the previous and current actual player position in the physics simulation.
//!
//! This is a tradeoff: every visual frame is now slightly lagging behind the actual physical frame,
//! but in return, the player's movement will appear smooth.
//! There are other ways to compute the visual representation of the player, such as extrapolation.
//! See the [documentation of the lightyear crate](https://cbournhonesque.github.io/lightyear/book/concepts/advanced_replication/visual_interpolation.html)
//! for a nice overview of the different methods and their respective tradeoffs.

use bevy::window::PrimaryWindow;
use bevy::{color::palettes::tailwind, prelude::*};
use bevy_rapier2d::prelude::*;
use parry2d::shape::Cuboid;

/// How many pixels per Rapier meter.
const PIXELS_PER_METER: f32 = 200.;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(
            RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(PIXELS_PER_METER)
                .in_fixed_schedule(),
        )
        .add_plugins(RapierDebugRenderPlugin {
            default_collider_debug: ColliderDebug::AlwaysRender,
            enabled: true,
            style: DebugRenderStyle::default(),
            mode: DebugRenderMode::all(),
        })
        .add_plugins(bevy_mod_debugdump::CommandLineArgs)
        .init_resource::<DidFixedTimestepRunThisFrame>()
        .add_systems(
            Startup,
            (
                setup_walls,
                setup_phys_objects,
                spawn_text,
                setup_camera,
                setup_player,
                set_size_window,
            ),
        )
        // At the beginning of each frame, clear the flag that indicates whether the fixed timestep has run this frame.
        .add_systems(PreUpdate, clear_fixed_timestep_flag)
        // At the beginning of each fixed timestep, set the flag that indicates whether the fixed timestep has run this frame.
        .add_systems(FixedPreUpdate, set_fixed_time_step_flag)
        // Advance the physics simulation using a fixed timestep.
        .add_systems(
            FixedUpdate,
            (
                prepare_players.before(PhysicsSet::SyncBackend),
                update_players.after(PhysicsSet::Writeback),
            ),
        )
        .add_systems(
            FixedPostUpdate,
            (handle_player_hit, handle_bullet_collision).chain(),
        )
        .add_systems(
            // The `RunFixedMainLoop` schedule allows us to schedule systems to run before and after the fixed timestep loop.
            RunFixedMainLoop,
            (
                (
                    // Accumulate our input before the fixed timestep loop to tell the physics simulation what it should do during the fixed timestep.
                    update_input,
                )
                    .chain()
                    .in_set(RunFixedMainLoopSystems::BeforeFixedMainLoop),
                (
                    // Clear our accumulated input after it was processed during the fixed timestep.
                    // By clearing the input *after* the fixed timestep, we can still use `AccumulatedInput` inside `FixedUpdate` if we need it.
                    clear_input.run_if(did_fixed_timestep_run_this_frame),
                    // The player's visual representation needs to be updated after the physics simulation has been advanced.
                    // This could be run in `Update`, but if we run it here instead, the systems in `Update`
                    // will be working with the `Transform` that will actually be shown on screen.
                    //interpolate_rendered_transform,
                )
                    .chain()
                    .in_set(RunFixedMainLoopSystems::AfterFixedMainLoop),
            ),
        )
        .run();
}

/// Speed of the player.
const SPEED: f32 = 10.;

/// Sets the window size to 1920x1080. This seems to have trouble on Mac, but needs to be rewritten so it scales
/// properly, so I'm not fixing it for now.
fn set_size_window(mut window: Single<&mut Window>) {
    window.resolution.set(1920., 1080.);
}

/// Prints the position of every entity.
fn print_entity_positions(positions: Query<(Entity, &Transform), With<RigidBody>>) {
    for (entity, transform) in positions.iter() {
        println!("Entity: {} position: {}", entity, transform.translation);
    }
}

/// Prints the position and velocity of all players.
fn print_player_info(player: Query<(&Transform, &Velocity2), With<Player>>) {
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
struct AccumulatedInput {
    // The player's left-right movement input (AD).
    movement: f32,
    jump: bool,
    shoot: Option<Vec2>,
    block: bool,
}

/// Sets up the walls as Fixed RigidBodies.
fn setup_walls(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    commands
        .spawn((Name::new("Floor"), RigidBody::Fixed))
        .insert(Transform::from_xyz(0., 0., 0.))
        .insert(Visibility::default())
        .with_children(|parent| {
            let width = 1920.;
            let height = 1080.;
            let wall_width = 100.;
            let half_wall_width = wall_width / 2.;

            let horiz_wall_mesh = meshes.add(Rectangle::new(width, wall_width));
            let vert_wall_mesh = meshes.add(Rectangle::new(wall_width, height));
            let wall_material = materials.add(Color::from(tailwind::RED_600));

            parent
                .spawn(Collider::cuboid(width / 2.0, half_wall_width))
                .insert(Mesh2d(horiz_wall_mesh.clone()))
                .insert(MeshMaterial2d(wall_material.clone()))
                .insert(Transform::from_xyz(0., -height / 2. + half_wall_width, 0.));
            parent
                .spawn(Collider::cuboid(width / 2.0, half_wall_width))
                .insert(Mesh2d(horiz_wall_mesh.clone()))
                .insert(MeshMaterial2d(wall_material.clone()))
                .insert(Transform::from_xyz(0., height / 2. - half_wall_width, 0.));
            parent
                .spawn(Collider::cuboid(half_wall_width, height / 2.0))
                .insert(Mesh2d(vert_wall_mesh.clone()))
                .insert(MeshMaterial2d(wall_material.clone()))
                .insert(Transform::from_xyz(-width / 2. + half_wall_width, 0., 0.));
            parent
                .spawn(Collider::cuboid(half_wall_width, height / 2.0))
                .insert(Mesh2d(vert_wall_mesh.clone()))
                .insert(MeshMaterial2d(wall_material.clone()))
                .insert(Transform::from_xyz(width / 2. - half_wall_width, 0., 0.));
        });
}

/// Sets up all physics objects (Dynamic RigidBodies).
/// For now, this is just a few circles.
fn setup_phys_objects(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let ball_material = materials.add(Color::from(tailwind::AMBER_500));
    let ball_mesh = meshes.add(Circle::new(50.0));
    for _ in 0..3 {
        commands
            .spawn(Name::new("Ball"))
            .insert(Mesh2d(ball_mesh.clone()))
            .insert(MeshMaterial2d(ball_material.clone()))
            .insert(RigidBody::Dynamic)
            .insert(Collider::ball(50.))
            .insert(Restitution::coefficient(0.7))
            .insert(Transform::from_xyz(0., 400., 0.));
    }
}

/// Sets up the camera. Parameters are just left as default for now.
fn setup_camera(mut commands: Commands) {
    commands.spawn(Camera2d);
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
struct Abilities {
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
struct Hp(f32);

#[derive(Debug, Component, Default)]
struct Player;

#[derive(Debug, Component, Default)]
struct HpBarGreen;

#[derive(Debug, Component, Default)]
struct HpBarRed;

#[derive(Debug, Component, Default)]
struct Radius(f32);

/// Custom velocity struct. Need to keep track of player velocity but rapier's velocity doesn't seem
/// to play too nicely?
#[derive(Debug, Component, Default)]
struct Velocity2(Vect);

#[derive(Debug, Component, Default)]
struct Counter(u64);

const HP_BAR_SCALE: Vec2 = Vec2::new(0.9, 1. / 15.);

/// Sets up a player.
fn setup_player(
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
        .insert(Hp(55.))
        .insert(Transform::default())
        .insert(AccumulatedInput::default())
        .insert(Velocity2::default())
        .insert(Abilities::default())
        .insert(Radius(player_radius))
        .insert(Collider::ball(25.))
        .insert(Sleeping::disabled())
        .insert(LockedAxes::ROTATION_LOCKED)
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

#[derive(Debug, Component, Clone, Copy, PartialEq, Hash, Default)]
struct Bullet;

#[derive(Debug, Component, Clone, Copy, PartialEq, Hash, Default)]
struct Bounces(u8);

#[derive(Debug, Component, Clone, Copy, PartialEq, Default)]
struct Damage(f32);

/// Sets up a new bullet.
fn setup_bullet(
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
    let body_mesh = meshes.add(Capsule2d::new(5., 10.));

    commands
        .spawn(Bullet)
        .insert(Bounces(5))
        .insert(Damage(25.))
        .insert(GravityScale(0.5))
        .insert(Transform::from_translation(position))
        .insert(Mesh2d(body_mesh))
        .insert(MeshMaterial2d(body_material))
        .insert(RigidBody::Dynamic)
        .insert(Velocity::linear(velocity))
        .insert(Collider::capsule_y(5., 5.))
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

/// Spawn a bit of UI text to explain how to move the player.
fn spawn_text(mut commands: Commands) {
    let font = TextFont {
        // font_size: FontSize::Px(25.0),
        ..default()
    };
    commands.spawn((
        Node {
            position_type: PositionType::Absolute,
            bottom: px(12),
            left: px(12),
            flex_direction: FlexDirection::Column,
            ..default()
        },
        children![
            (Text::new("Move the player with AD and Space"), font.clone()),
            (Text::new("Left click to shoot"), font),
        ],
    ));
}

/// Handle keyboard input and accumulate it in the `AccumulatedInput` component.
///
/// There are many strategies for how to handle all the input that happened since the last fixed timestep.
/// This is a very simple one: we just use the last available input.
/// That strategy works fine for us since the user continuously presses the input keys in this example.
/// If we had some kind of instantaneous action like activating a boost ability, we would need to remember that that input
/// was pressed at some point since the last fixed timestep.
fn update_input(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mouse_input: Res<ButtonInput<MouseButton>>,
    window: Single<&Window, With<PrimaryWindow>>,
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
fn clear_fixed_timestep_flag(
    mut did_fixed_timestep_run_this_frame: ResMut<DidFixedTimestepRunThisFrame>,
) {
    did_fixed_timestep_run_this_frame.0 = false;
}

/// Set the flag during each fixed timestep.
fn set_fixed_time_step_flag(
    mut did_fixed_timestep_run_this_frame: ResMut<DidFixedTimestepRunThisFrame>,
) {
    did_fixed_timestep_run_this_frame.0 = true;
}

fn did_fixed_timestep_run_this_frame(
    did_fixed_timestep_run_this_frame: Res<DidFixedTimestepRunThisFrame>,
) -> bool {
    did_fixed_timestep_run_this_frame.0
}

// Clear the input after it was processed in the fixed timestep.
fn clear_input(mut input: Single<&mut AccumulatedInput>) {
    **input = AccumulatedInput::default();
}

/// Prepare all players for the physics update.
fn prepare_players(
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
fn update_players(
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

            println!("velocity: {}", distance);
        } else {
            velocity.0.y -= 9.8 * PIXELS_PER_METER * fixed_time.delta_secs();
        }
    }
}

fn handle_player_hit(
    mut commands: Commands,
    mut collision_events: MessageReader<CollisionEvent>,
    mut player_query: Query<(&mut Hp, &Radius, &Children), With<Player>>,
    bullet_query: Query<&Damage, With<Bullet>>,
    mut green_hp_bar_query: Query<&mut Transform, (With<HpBarGreen>, Without<HpBarRed>)>,
    mut red_hp_bar_query: Query<&mut Transform, (With<HpBarRed>, Without<HpBarGreen>)>,
) {
    for event in collision_events.read() {
        let (&left, &right, _flags) = match event {
            CollisionEvent::Started(left, right, flags) => (left, right, flags),
            CollisionEvent::Stopped(_, _, _) => break,
        };

        let mut handle_collision = |player, bullet| {
            if let Ok((mut hp, radius, children)) = player_query.get_mut(player)
                && let Ok(damage) = bullet_query.get(bullet)
            {
                hp.0 -= damage.0;
                for entity in children.iter() {
                    if let Ok(mut transform) = green_hp_bar_query.get_mut(entity) {
                        let scale = hp.0 / 100.;
                        transform.scale.x = scale;
                        transform.translation.x = (1. - scale) * radius.0 * -HP_BAR_SCALE.x;
                    } else if let Ok(mut transform) = red_hp_bar_query.get_mut(entity) {
                        let scale = 1. - hp.0 / 100.;
                        transform.scale.x = scale;
                        transform.translation.x = (1. - scale) * radius.0 * HP_BAR_SCALE.x;
                    }
                }
                if hp.0 <= 0. {
                    commands.entity(player).despawn();
                }
                commands.entity(bullet).despawn();
            }
        };

        handle_collision(left, right);
        handle_collision(right, left);
    }
}

fn handle_bullet_collision(
    mut commands: Commands,
    mut collision_events: MessageReader<CollisionEvent>,
    mut query: Query<&mut Bounces, With<Bullet>>,
) {
    for event in collision_events.read() {
        let (&left, &right, _flags) = match event {
            CollisionEvent::Started(left, right, flags) => (left, right, flags),
            CollisionEvent::Stopped(_, _, _) => break,
        };

        let mut handle_collision = |entity| {
            if let Ok(mut bounces) = query.get_mut(entity) {
                if bounces.0 == 0 {
                    commands.entity(entity).despawn();
                } else {
                    bounces.0 -= 1;
                }
            }
        };

        handle_collision(left);
        handle_collision(right);
    }
}
