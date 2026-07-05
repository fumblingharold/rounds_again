use bevy::prelude::*;

#[derive(Debug, Component, Default)]
pub struct Hp {
    pub hp: f32,
    pub max_hp: f32,
}

impl Hp {
    pub fn new(hp: f32) -> Self {
        Hp { hp, max_hp: hp }
    }

    pub fn decrement(&mut self, amount: f32) {
        self.hp -= amount;
    }
}
