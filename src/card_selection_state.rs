use super::AppState;
use crate::card::{
    BouncesUp, BulletSpeedUp, Card, DamageUp, HealthUp, JumpsUp, SELECTED_CARD_BORDER,
    UNSELECTED_CARD_BORDER,
};
use crate::player::{Input, Player};
use crate::shared::Hp;
use bevy::prelude::*;

pub struct CardSelectionPlugin;

impl Plugin for CardSelectionPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(AppState::CardSelection), setup_card_selection)
            .add_systems(
                Update,
                update_card_selection.run_if(in_state(AppState::CardSelection)),
            )
            .add_systems(
                OnExit(AppState::CardSelection),
                (cleanup_card_selection, reset_players),
            );
    }
}

/// All the data associated with card selection.
#[derive(Resource)]
struct CardSelectionData {
    ui: Entity,
    card_buttons: Vec<Entity>,
    cards: Vec<Box<dyn Card>>,
    players_left: Vec<Entity>,
    selected_card: Option<u8>,
}

/// Draws five cards.
///
/// Returns the UI entity, a vector for the card buttons, and a vector of the cards.
fn draw_5(mut commands: Commands) -> (Entity, Vec<Entity>, Vec<Box<dyn Card>>) {
    // TODO draw cards from a deck randomly
    let cards: Vec<Box<dyn Card>> = vec![
        Box::new(HealthUp),
        Box::new(DamageUp),
        Box::new(BulletSpeedUp),
        Box::new(BouncesUp),
        Box::new(JumpsUp),
    ];

    let buttons: Vec<_> = cards
        .iter()
        .map(|card| card.to_button(commands.reborrow()))
        .collect();

    let mut sub_ui = commands.spawn((Node {
        width: px(1600),
        height: px(65),
        justify_content: JustifyContent::SpaceBetween,
        align_items: AlignItems::Center,
        ..default()
    },));

    sub_ui.add_children(&buttons);
    let sub_ui = sub_ui.id();

    let mut ui = commands.spawn((Node {
        width: percent(100),
        height: percent(100),
        justify_content: JustifyContent::Center,
        align_items: AlignItems::Center,
        ..default()
    },));

    ui.add_child(sub_ui);

    (ui.id(), buttons, cards)
}

/// Setups up the card selection.
fn setup_card_selection(mut commands: Commands, players: Query<Entity, With<Player>>) {
    // TODO don't add all players to the queue
    let players_left = players.iter().collect();
    let (ui, card_buttons, cards) = draw_5(commands.reborrow());
    commands.insert_resource(CardSelectionData {
        ui,
        card_buttons,
        cards,
        players_left,
        selected_card: None,
    });
}

/// Adds/removes players in response to user input.
fn update_card_selection(
    mut commands: Commands,
    mut next_state: ResMut<NextState<AppState>>,
    mut card_selection_data: ResMut<CardSelectionData>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    controllers: Query<&Gamepad>,
    mut players: Query<&Input, With<Player>>,
    mut card_buttons: Query<&mut Outline>,
) {
    let player = card_selection_data.players_left.last().unwrap();
    let input = players.get_mut(*player).unwrap();

    let mut process_input = |select, left, right| {
        if select {
            // Take card
            commands.entity(card_selection_data.ui).despawn();
            let player = card_selection_data.players_left.pop().unwrap();
            card_selection_data.cards[card_selection_data.selected_card.unwrap() as usize]
                .update_player(commands.reborrow(), player);
            if card_selection_data.players_left.is_empty() {
                next_state.set(AppState::Game);
            } else {
                let (ui, card_buttons, cards) = draw_5(commands.reborrow());
                card_selection_data.ui = ui;
                card_selection_data.cards = cards;
                card_selection_data.card_buttons = card_buttons;
                card_selection_data.selected_card = None;
            }
        } else {
            if left {
                card_selection_data.selected_card = Some(match card_selection_data.selected_card {
                    None => (card_selection_data.cards.len() - 1) as u8,
                    Some(0) => (card_selection_data.cards.len() - 1) as u8,
                    Some(n) => n - 1,
                })
            }
            if right {
                card_selection_data.selected_card = Some(match card_selection_data.selected_card {
                    None => 0,
                    Some(n) => (n + 1) % card_selection_data.cards.len() as u8,
                })
            }

            for card_button in card_selection_data.card_buttons.iter() {
                card_buttons.get_mut(*card_button).unwrap().color = UNSELECTED_CARD_BORDER;
            }
            if let Some(selected_card) = card_selection_data.selected_card {
                card_buttons
                    .get_mut(card_selection_data.card_buttons[selected_card as usize])
                    .unwrap()
                    .color = SELECTED_CARD_BORDER;
            }
        }
    };

    match input {
        Input::Controller(controller) => {
            let controller = controllers.get(*controller).unwrap();
            process_input(
                controller.just_pressed(GamepadButton::South),
                controller.just_pressed(GamepadButton::DPadLeft),
                controller.just_pressed(GamepadButton::DPadRight),
            );
        }
        Input::Keyboard => {
            process_input(
                keyboard_input.just_pressed(KeyCode::Space),
                keyboard_input.just_pressed(KeyCode::KeyA),
                keyboard_input.just_pressed(KeyCode::KeyD),
            );
        }
    }
}

/// Cleans up the lobby.
fn cleanup_card_selection(mut commands: Commands) {
    commands.remove_resource::<CardSelectionData>();
}

/// Resets the players to prepare for the next match.
///
/// TODO move this into the match load state
fn reset_players(mut players: Query<&mut Hp, With<Player>>) {
    for mut hp in players.iter_mut() {
        hp.reset();
    }
}
