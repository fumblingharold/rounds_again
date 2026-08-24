use bevy_rapier2d::prelude::*;

pub const PLAYERS: Group = Group::from_bits(0b1).unwrap();
pub const BULLETS: Group = Group::from_bits(0b10).unwrap();
pub const PHYS_OBJECTS: Group = Group::from_bits(0b100).unwrap();
pub const WALLS: Group = Group::from_bits(0b1000).unwrap();
