use crate::{
    collision_groups,
    shared::{Bounces, Damage, Hp, Source},
};
use arrayvec::ArrayVec;
use bevy::{color::palettes::tailwind, prelude::*};
use bevy_rapier2d::prelude::*;
use std::ops::{Index, IndexMut};

/// The maximum number of players in a match.
const MAX_PLAYERS: u8 = 255;

#[derive(Debug, Component, Clone, Copy, PartialEq)]
pub enum Input {
    Keyboard,
    Gamepad(Entity),
}

/// A unique id for a player. There are max 255 ids. Id 0 is reserved for non-player objects.
#[derive(Debug, Component, Clone, Copy, PartialEq)]
pub struct PlayerId(u8);

impl PlayerId {
    /// Converts the PlayerId into a Source component.
    pub fn into_source(self) -> Source {
        Source(self.0)
    }
}

/// Generates unique ids for players.
#[derive(Debug, Clone, Eq, PartialEq, Hash, Resource)]
pub struct PlayerIdGen(u8, Vec<u8>);

impl PlayerIdGen {
    pub fn new() -> Self {
        Self(0, Vec::new())
    }

    /// Adds the PlayerId back to the generator. It will be added at the front of the queue.
    pub fn reinsert(&mut self, value: PlayerId) {
        let idx = self.1.binary_search(&value.0).err().unwrap();
        self.1.insert(idx, value.0);
    }
}

impl Default for PlayerIdGen {
    fn default() -> Self {
        Self::new()
    }
}

impl Iterator for PlayerIdGen {
    type Item = PlayerId;
    fn next(&mut self) -> Option<Self::Item> {
        if let Some(val) = self.1.pop() {
            Some(PlayerId(val))
        } else {
            match &mut self.0 {
                &mut MAX_PLAYERS => None,
                val => {
                    *val += 1;
                    Some(PlayerId(*val))
                }
            }
        }
    }
}

/// Speed of the player.
pub const SPEED: f32 = 10.;

/// Prints the position and velocity of all players.
pub fn print_player_info(player: Query<(&Transform, &Velocity), With<Player>>) {
    for (transform, velocity) in player.iter() {
        println!(
            "Velocity: {} position: {}",
            velocity.linear, transform.translation
        );
    }
}

/// Represents the player's input, accumulated over all frames that ran since the last time the
/// physics simulation was advanced.
/// Directionals are replaced with the most recent input while jump, shoot, and block are ANDed.
#[derive(Debug, Component, Clone, Copy, PartialEq, Default)]
pub struct AccumulatedInput {
    // The player's left-right movement input (AD).
    pub movement: f32,
    pub down: bool,
    pub jump: bool,
    pub shoot: Option<Vec2>,
    pub block: bool,
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
    max_stock: u16,
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
    pub fn tick(&mut self, jump: bool, shoot: bool) -> (bool, bool) {
        (self.jump.tick(jump), self.shoot.tick(shoot))
    }

    /// Refills jump stock.
    pub fn refill_jump(&mut self) {
        self.jump.stock = self.jump.max_stock;
    }

    /// Increases max jump stock by 1.
    pub fn add_jump(&mut self, additional_jumps: u16) {
        self.jump.max_stock += additional_jumps;
    }
}

// TODO need to make jumps not trigger when holding the button
impl Default for Abilities {
    fn default() -> Self {
        Self {
            jump: Ability {
                state: State::Ready,
                stock: 1,
                max_stock: 1,
                use_time: 5,
                cooldown_time: 20,
            },
            shoot: Ability {
                state: State::Ready,
                stock: 60,
                max_stock: 60,
                use_time: 0,
                cooldown_time: 60,
            },
        }
    }
}

#[derive(Debug, Component, Default)]
pub struct Player;

/// The last player to hit this player.
#[derive(Debug, Component, Default)]
pub struct LastHit(pub u8);

#[derive(Debug, Component)]
pub struct DamageTakenThisTick(ArrayVec<f32, { MAX_PLAYERS as usize + 1 }>);

impl DamageTakenThisTick {
    /// Creates a new `DamageTakenThisTick` for the given number of players.
    pub fn new(num_players: u8) -> Self {
        Self(ArrayVec::from_iter(std::iter::repeat_n(
            0.,
            (num_players + 1) as usize,
        )))
    }

    /// Resets the damage to 0 for all sources.
    pub fn zero(&mut self) {
        self.0.fill(0.);
    }
}

impl Index<Source> for DamageTakenThisTick {
    type Output = f32;

    fn index(&self, index: Source) -> &Self::Output {
        &self.0[index.0 as usize]
    }
}

impl IndexMut<Source> for DamageTakenThisTick {
    fn index_mut(&mut self, index: Source) -> &mut Self::Output {
        &mut self.0[index.0 as usize]
    }
}

impl<'a> IntoIterator for &'a DamageTakenThisTick {
    type Item = &'a f32;
    type IntoIter = std::slice::Iter<'a, f32>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.iter()
    }
}

#[derive(Debug, Component, Default)]
pub struct HpBarGreen;

#[derive(Debug, Component, Default)]
pub struct HpBarRed;

#[derive(Debug, Component, Default)]
pub struct Radius(pub f32);

#[derive(Debug, Component, Default)]
pub struct Counter(pub u64);

#[derive(Debug, Component, Default)]
pub struct BulletSpeed(pub f32);

#[derive(Debug, Component, Default)]
pub struct PlayerColor(pub Color);

pub const HP_BAR_SCALE: Vec2 = Vec2::new(0.9, 1. / 15.);

/// Sets up a player.
pub fn setup_player(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<ColorMaterial>>,
    input: Input,
    color: Color,
    player_id: PlayerId,
) {
    let player_radius = 25.;
    let body_mesh = meshes.add(Circle::new(player_radius));
    let body_material = materials.add(color);
    let bar_mesh = meshes.add(Rectangle::new(
        player_radius * 2. * HP_BAR_SCALE.x,
        player_radius * 2. * HP_BAR_SCALE.y,
    ));

    commands
        .spawn(Player)
        .insert(Counter(0))
        .insert(Name::new("Player"))
        .insert(input)
        .insert(player_id)
        .insert(PlayerColor(color))
        .insert(Hp::new(55.))
        .insert(Damage(25.))
        .insert(BulletSpeed(800.))
        .insert(Bounces(0))
        .insert(Transform::default())
        .insert(LastHit(0))
        .insert(AccumulatedInput::default())
        .insert(Abilities::default())
        .insert(Radius(player_radius))
        .insert(Collider::ball(25.))
        .insert(Sleeping::disabled())
        .insert(LockedAxes::ROTATION_LOCKED)
        .insert(Visibility::Visible)
        .insert(ActiveEvents::CONTACT_FORCE_EVENTS)
        // TODO have a reason for this value
        .insert(ContactForceEventThreshold(0.1))
        .insert(Velocity::default())
        // TODO have a reason for this value
        .insert(GravityScale(1.))
        .insert(CollisionGroups::new(
            collision_groups::PLAYERS,
            Group::default(),
        ))
        .insert(RigidBody::Dynamic)
        .with_children(|parent| {
            // TODO clean up the health bar code a bit
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
