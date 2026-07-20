use super::AppState;
use crate::player::{Input, Player, PlayerColor, PlayerId, PlayerIdGen, setup_player};
use bevy::prelude::*;

pub struct LobbyPlugin;

impl Plugin for LobbyPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<PlayerIdGen>()
            .add_systems(OnEnter(AppState::Lobby), setup_lobby)
            .add_systems(Update, update_lobby.run_if(in_state(AppState::Lobby)))
            .add_systems(OnExit(AppState::Lobby), cleanup_lobby);
    }
}

/// All the data associated with the lobby.
#[derive(Resource)]
struct LobbyData {
    button_entity: Entity,
    player_colors: Vec<(Color, bool)>,
}

impl LobbyData {
    fn next_player_color(&mut self) -> Color {
        let (color, in_use) = self
            .player_colors
            .iter_mut()
            .find(|(_, in_use)| !*in_use)
            .expect("more players than colors");
        *in_use = true;
        *color
    }

    fn reinsert_player_color(&mut self, color: Color) {
        let (_, in_use) = self
            .player_colors
            .iter_mut()
            .find(|(other_color, _)| color == *other_color)
            .expect("color not a valid player color");
        *in_use = false;
    }
}

/// Setups up the lobby.
fn setup_lobby(mut commands: Commands) {
    let font = TextFont {
        font_size: 33.,
        ..default()
    };
    let color = TextColor(Color::srgb(0.9, 0.9, 0.9));
    let button_entity = commands
        .spawn((
            Node {
                width: percent(100),
                height: percent(100),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::End,
                ..default()
            },
            Visibility::Inherited,
            children![(
                Node {
                    width: px(1000),
                    height: px(65),
                    justify_content: JustifyContent::SpaceBetween,
                    align_items: AlignItems::Center,
                    ..default()
                },
                Visibility::Inherited,
                children![
                    (Text::new("Space or A to join"), font.clone(), color),
                    (Text::new("Esc or B or leave"), font.clone(), color),
                ],
            )],
        ))
        .id();
    let player_colors: Vec<_> = {
        use bevy::color::palettes::tailwind::*;
        vec![
            BLUE_700, GREEN_700, RED_700, CYAN_700, YELLOW_700, PURPLE_700,
        ]
    }
    .into_iter()
    .map(|color| (Color::from(color), false))
    .collect();
    commands.insert_resource(LobbyData {
        button_entity,
        player_colors,
    });
}

/// Adds/removes players in response to user input.
fn update_lobby(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut player_id_gen: ResMut<PlayerIdGen>,
    mut next_state: ResMut<NextState<AppState>>,
    mut lobby_data: ResMut<LobbyData>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    controllers: Query<(Entity, &Gamepad)>,
    players: Query<(Entity, &Input, &PlayerId, &PlayerColor), With<Player>>,
) {
    let get_player_from_input = |matching_input| {
        players
            .iter()
            .find_map(|(entity, input, player_id, player_color)| {
                if matching_input == *input {
                    Some((entity, player_id, player_color))
                } else {
                    None
                }
            })
    };
    let cleanup_matching = |matching_input: Input,
                            player_id_gen: &mut PlayerIdGen,
                            commands: &mut Commands,
                            lobby_data: &mut LobbyData| {
        if let Some((entity, player_id, player_color)) = get_player_from_input(matching_input) {
            player_id_gen.reinsert(*player_id);
            lobby_data.reinsert_player_color(player_color.0);
            commands.entity(entity).despawn();
        }
    };
    let mut try_setup_player = |input: Input,
                                player_id_gen: &mut PlayerIdGen,
                                commands: &mut Commands,
                                lobby_data: &mut LobbyData| {
        if get_player_from_input(input).is_none() {
            setup_player(
                commands,
                &mut meshes,
                &mut materials,
                input,
                lobby_data.next_player_color(),
                player_id_gen.next().expect("Too many players"),
            );
        }
    };

    if keyboard_input.just_pressed(KeyCode::Escape) {
        cleanup_matching(
            Input::Keyboard,
            &mut player_id_gen,
            &mut commands,
            lobby_data.reborrow().into_inner(),
        );
    }
    if keyboard_input.just_pressed(KeyCode::Space) {
        try_setup_player(
            Input::Keyboard,
            &mut player_id_gen,
            &mut commands,
            lobby_data.reborrow().into_inner(),
        );
    }
    for (controller_entity, controller) in controllers {
        if controller.just_pressed(GamepadButton::East) {
            cleanup_matching(
                Input::Controller(controller_entity),
                &mut player_id_gen,
                &mut commands,
                lobby_data.reborrow().into_inner(),
            );
        }
        if controller.just_pressed(GamepadButton::South) {
            try_setup_player(
                Input::Controller(controller_entity),
                &mut player_id_gen,
                &mut commands,
                lobby_data.reborrow().into_inner(),
            );
        }
    }

    if keyboard_input.just_pressed(KeyCode::Enter) {
        next_state.set(AppState::Game);
    }
}

/// Cleans up the lobby.
fn cleanup_lobby(mut commands: Commands, lobby_data: Res<LobbyData>) {
    // Should also clean up the resource
    commands.entity(lobby_data.button_entity).despawn();
}
