use bevy::prelude::*;

#[derive(Debug, Component, Default)]
pub struct Hp {
    pub hp: f32,
    pub max_hp: f32,
    pub true_max_hp: f32,
}

impl Hp {
    pub fn new(hp: f32) -> Self {
        Hp {
            hp,
            max_hp: hp,
            true_max_hp: hp,
        }
    }

    pub fn damage(&mut self, amount: f32) {
        self.hp -= amount;
    }

    pub fn heal(&mut self, amount: f32) {
        self.hp = f32::max(self.hp + amount, self.max_hp);
    }

    pub fn reset(&mut self) {
        self.hp = self.true_max_hp;
        self.max_hp = self.true_max_hp
    }
}

#[derive(Debug, Component, Clone, Copy, PartialEq, Default)]
pub struct Damage(pub f32);

#[derive(Debug, Component, Clone, Copy, PartialEq, Default)]
pub struct Source(pub u8);

impl Source {
    /// The source of world damage (physics objects and walls).
    pub const WORLD: Source = Source(0);
}

#[derive(Debug, Component, Clone, Copy, PartialEq, Hash, Default)]
pub struct Bounces(pub u8);
