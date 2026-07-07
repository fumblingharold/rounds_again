use super::AppState;
use crate::player::{Input, Player, PlayerId, PlayerIdGen, setup_player};
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

#[derive(Resource)]
struct LobbyData {
    button_entity: Entity,
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
    commands.insert_resource(LobbyData { button_entity });
}

/// Adds/removes players in response to user input.
fn update_lobby(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut player_id_gen: ResMut<PlayerIdGen>,
    mut next_state: ResMut<NextState<AppState>>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    controllers: Query<(Entity, &Gamepad)>,
    players: Query<(Entity, &Input, &PlayerId), With<Player>>,
) {
    let get_player_from_input = |matching_input| {
        players.iter().find_map(|(entity, input, player_id)| {
            if matching_input == *input {
                Some((entity, player_id))
            } else {
                None
            }
        })
    };
    let cleanup_matching =
        |matching_input: Input, player_id_gen: &mut PlayerIdGen, commands: &mut Commands| {
            if let Some((entity, player_id)) = get_player_from_input(matching_input) {
                player_id_gen.reinsert(*player_id);
                commands.entity(entity).despawn();
            }
        };
    let mut try_setup_player =
        |input: Input, player_id_gen: &mut PlayerIdGen, commands: &mut Commands| {
            if get_player_from_input(input).is_none() {
                setup_player(
                    commands,
                    &mut meshes,
                    &mut materials,
                    input,
                    player_id_gen.next().expect("Too many players"),
                );
            }
        };

    if keyboard_input.just_pressed(KeyCode::Escape) {
        cleanup_matching(Input::Keyboard, &mut player_id_gen, &mut commands);
    }
    if keyboard_input.just_pressed(KeyCode::Space) {
        try_setup_player(Input::Keyboard, &mut player_id_gen, &mut commands);
    }
    for (controller_entity, controller) in controllers {
        if controller.just_pressed(GamepadButton::East) {
            cleanup_matching(
                Input::Controller(controller_entity),
                &mut player_id_gen,
                &mut commands,
            );
        }
        if controller.just_pressed(GamepadButton::South) {
            try_setup_player(
                Input::Controller(controller_entity),
                &mut player_id_gen,
                &mut commands,
            );
        }
    }

    if keyboard_input.just_pressed(KeyCode::Enter) {
        next_state.set(AppState::Game);
    }
}

/// Cleans up the lobby.
fn cleanup_lobby(mut commands: Commands, lobby_data: Res<LobbyData>) {
    commands.entity(lobby_data.button_entity).despawn();
}
