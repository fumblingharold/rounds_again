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
