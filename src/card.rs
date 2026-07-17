use crate::player::{Abilities, BulletSpeed};
use crate::shared::{Bounces, Damage, Hp};
use bevy::prelude::*;
use std::fmt::Debug;
use std::sync::LazyLock;

static CARD_FONT: LazyLock<TextFont> = LazyLock::new(|| TextFont {
    font_size: 33.,
    ..default()
});
const CARD_TEXT_COLOR: TextColor = TextColor(Color::srgb(0.9, 0.9, 0.9));
pub const UNSELECTED_CARD_BORDER: Color = Color::srgb(0., 0.5, 0.5);
pub const SELECTED_CARD_BORDER: Color = Color::srgb(0.5, 0.5, 0.);

#[derive(Component)]
pub struct Deck(pub Vec<Box<dyn Card>>);

/// Creates a card button with the following text. Returns the id of this entity.
fn make_button(mut commands: Commands, text: &str) -> Entity {
    commands
        .spawn((
            Node {
                width: px(200),
                height: px(400),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(Color::srgb(0., 0., 0.)),
            Outline::new(px(5), px(0), UNSELECTED_CARD_BORDER),
            Visibility::Inherited,
            children![(Text::new(text), CARD_FONT.clone(), CARD_TEXT_COLOR),],
        ))
        .id()
}

/// A game Card.
pub trait Card: Debug + Send + Sync {
    /// Updates the player with the card effects.
    fn update_player(&self, commands: Commands, player: Entity);

    /// Spawns a card button. Returns the id of this entity.
    fn to_button(&self, commands: Commands) -> Entity;
}

/// Increases health by 10%.
#[derive(Component, Debug, Clone)]
pub struct HealthUp;

impl Card for HealthUp {
    fn update_player(&self, mut commands: Commands, player: Entity) {
        let mut player = commands.get_entity(player).unwrap();
        player
            .entry::<Hp>()
            .and_modify(|mut hp| hp.true_max_hp *= 1.1);
    }

    fn to_button(&self, commands: Commands) -> Entity {
        make_button(commands, "Health Up")
    }
}

/// Increases damage by 10%.
#[derive(Component, Debug, Clone)]
pub struct DamageUp;

impl Card for DamageUp {
    fn update_player(&self, mut commands: Commands, player: Entity) {
        let mut player = commands.get_entity(player).unwrap();
        player
            .entry::<Damage>()
            .and_modify(|mut damage| damage.0 *= 1.1);
    }

    fn to_button(&self, commands: Commands) -> Entity {
        make_button(commands, "Damage Up")
    }
}

/// Increases bullet speed by 10%.
#[derive(Component, Debug, Clone)]
pub struct BulletSpeedUp;

impl Card for BulletSpeedUp {
    fn update_player(&self, mut commands: Commands, player: Entity) {
        let mut player = commands.get_entity(player).unwrap();
        player
            .entry::<BulletSpeed>()
            .and_modify(|mut bullet_speed| bullet_speed.0 *= 1.1);
    }

    fn to_button(&self, commands: Commands) -> Entity {
        make_button(commands, "Bullet Speed Up")
    }
}

/// Increases bullet bounces by 1.
#[derive(Component, Debug, Clone)]
pub struct BouncesUp;

impl Card for BouncesUp {
    fn update_player(&self, mut commands: Commands, player: Entity) {
        let mut player = commands.get_entity(player).unwrap();
        player
            .entry::<Bounces>()
            .and_modify(|mut bounces| bounces.0 += 1);
    }

    fn to_button(&self, commands: Commands) -> Entity {
        make_button(commands, "Bounces Up")
    }
}

/// Increases jumps by 1.
#[derive(Component, Debug, Clone)]
pub struct JumpsUp;

impl Card for JumpsUp {
    fn update_player(&self, mut commands: Commands, player: Entity) {
        let mut player = commands.get_entity(player).unwrap();
        player
            .entry::<Abilities>()
            .and_modify(|mut abilities| abilities.add_jump(1));
    }

    fn to_button(&self, commands: Commands) -> Entity {
        make_button(commands, "Jumps Up")
    }
}
